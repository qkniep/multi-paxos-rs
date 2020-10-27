// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! Contains the network-message-types for the Paxos consensus protocol.

use std::fmt::Debug;
use std::time::Duration;

use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, trace_span, warn};

use crate::network::NetworkNode;

type Value = Vec<u8>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PaxosMsg {
    Prepare {
        id: ProposalID,
        holes: Vec<usize>,
    },
    Promise {
        id: ProposalID,
        accepted: Vec<(usize, ProposalID, Value)>,
    },

    Propose {
        id: ProposalID,
        instance: usize,
        value: Value,
    },
    Accept {
        id: ProposalID,
        instance: usize,
    },

    Learn {
        id: ProposalID,
        instance: usize,
        value: Value,
    },

    /// Currently only used for rejecting Proposals.
    Nack {
        id: ProposalID,
        instance: usize,
    },

    Relay(Value),

    // TODO: Configuration changes:
    // Join,
    // Leave,
}

/// Unique monotonically-increasing ID
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct ProposalID(usize, usize);

impl ProposalID {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

type Promise<V> = Vec<(usize, ProposalID, V)>;

/// Handles all Paxos related state for a single node,
/// acting as proposer, acceptor and learner.
#[derive(Debug)]
pub struct PaxosServer<N> {
    node_id: usize,
    node: N,
    client_cmd_queue: Vec<Value>,
    log: Vec<PaxosInstance>,
    majority: usize,
    current_leader: usize,
    current_id: ProposalID,
    highest_promised: ProposalID,
    promises: Vec<(ProposalID, Promise<Value>)>,
}

/// Holds the state representing a single slot in the log.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct PaxosInstance {
    value: Option<Value>,
    acceptances: usize,
    accepted_id: ProposalID,
    chosen: bool,
}

impl<N: NetworkNode> PaxosServer<N> {
    /// Creates a new Paxos server.
    ///
    /// # Arguments
    ///
    /// * `node` - The network node used for sending messages to other Paxos servers.
    /// * `node_id` - A unique number identifying this Paxos server.
    /// * `node_count` - The number of Paxos servers operating in this network.
    ///
    /// # Remarks
    ///
    /// At the time of creation, this server has an empty log and doesn't know who the leader is.
    pub fn new(node: N, node_id: usize, node_count: usize) -> Self {
        Self {
            node_id,
            node,
            client_cmd_queue: Vec::new(),
            log: Vec::new(),
            majority: node_count / 2 + 1,
            current_leader: 0,
            current_id: ProposalID(1, node_id),
            highest_promised: ProposalID(0, 0),
            promises: Vec::new(),
        }
    }

    /// Runs this Paxos server's main loop.
    pub fn run(&mut self) {
        // configure a span to associate log-entries with this network node
        let tracing_span = trace_span!("NetworkNode", self.node_id);
        let _guard = tracing_span.enter();
        info!("Starting Paxos Node with ID {}", self.node_id);

        loop {
            // event loop for incoming messages
            while let Ok(msg) = self.node.recv(Duration::from_millis(1000)) {
                let (src, cmd) = msg;
                self.handle_paxos_message(src, cmd);
            }

            warn!("Long time w/o receiving anything!");
            self.start_election();
        }
    }

    /// Parses the message and calls the method corresponding to the message type.
    fn handle_paxos_message(&mut self, src: usize, cmd: PaxosMsg) {
        match cmd {
            PaxosMsg::Prepare { id, holes } => self.handle_prepare(src, id, holes),
            PaxosMsg::Promise { id, accepted } => self.handle_promise(id, accepted),
            PaxosMsg::Propose {
                id,
                instance,
                value,
            } => self.handle_propose(src, id, instance, value),
            PaxosMsg::Accept { id, instance } => self.handle_accept(id, instance),
            PaxosMsg::Learn {
                id,
                instance,
                value,
            } => self.handle_learn(id, instance, value),
            PaxosMsg::Nack { id, instance } => self.handle_nack(id, instance),
            PaxosMsg::Relay(value) => self.handle_client_request(value),
        }
    }

    /// Responds to a Paxos Prepare (1a) message.
    /// Sends a Promise back to the sender iff this node has not made Promise for a higher id yet.
    fn handle_prepare(&mut self, src: usize, id: ProposalID, holes: Vec<usize>) {
        if id < self.highest_promised {
            warn!("Prepare rejected: {:?}<{:?}", id, self.highest_promised);
            return;
        }

        debug!("Promise vote: {:?}", id);
        self.highest_promised = id;
        self.current_leader = src;

        // Fills accepted_values with all values this node has accepted and
        // the sender of the Prepare has marked as not yet known to be chosen (in holes).
        let mut accepted_values = Vec::new();
        for (index, id, value) in self.get_accepted_values_iter() {
            for hole in holes.iter().copied().chain(holes.last().unwrap() + 1..) {
                if index < hole {
                    break;
                } else if hole < index {
                    continue;
                }
                accepted_values.push((index, id, value.clone()));
            }
        }

        let promise = PaxosMsg::Promise {
            id,
            accepted: accepted_values,
        };
        self.node.send(src, promise);
    }

    /// Responds to a Paxos Promise (1b) message.
    fn handle_promise(&mut self, id: ProposalID, accepted: Vec<(usize, ProposalID, Value)>) {
        if id != self.current_id {
            warn!("Promise rejected: {:?}!={:?}", id, self.current_id);
            return;
        }

        debug!("Got a promise: {:?}, {:?}", id, accepted);
        self.promises.push((id, accepted)); // TODO: Remove promises w/ old id before counting.

        if self.promises.len() == self.majority {
            info!("Got elected.");
            self.current_leader = self.current_id.1;
            for (i, instance) in (&self.log)
                .iter()
                .enumerate()
                .filter(|(_, instance)| !instance.chosen)
            {
                self.node.broadcast(PaxosMsg::Propose {
                    instance: i,
                    id,
                    value: instance.value.as_ref().unwrap().clone(),
                });
            }
        }
    }

    /// Responds to a Paxos Propose (2a) message.
    fn handle_propose(&mut self, src: usize, id: ProposalID, instance: usize, value: Value) {
        if id < self.highest_promised {
            warn!("Proposal rejected: {:?}", value);
            self.node.send(src, PaxosMsg::Nack { instance, id });
            return;
        }

        trace!("Proposal accepted: {:?}", value);
        self.current_leader = src;
        while instance >= self.log.len() {
            self.log.push(PaxosInstance::new());
        }
        self.node.send(src, PaxosMsg::Accept { instance, id });
        self.log[instance].value = Some(value);
        self.log[instance].accepted_id = id;
    }

    /// Responds to a Paxos Accept (2b) message.
    fn handle_accept(&mut self, id: ProposalID, instance: usize) {
        if id != self.current_id {
            warn!(
                "Accept rejected: [{}]: {:?}!={:?}",
                instance, id, self.current_id
            );
            return;
        }
        self.log[instance].acceptances += 1;
        if self.log[instance].acceptances == self.majority {
            debug!(
                "Sending learn with {}/{} acceptances.",
                self.log[instance].acceptances, self.majority
            );
            let value = self.log[instance].value.clone().unwrap();
            info!("Value was chosen: [{}]: {:?}, {:?}", instance, id, value);
            self.node.broadcast(PaxosMsg::Learn {
                instance,
                id,
                value,
            });
            self.log[instance].chosen = true;
        }
    }

    /// Handles a Learn message.
    fn handle_learn(&mut self, id: ProposalID, instance: usize, value: Value) {
        info!("Learned: [{}] {:?},{:?}", instance, id, value);
        while instance >= self.log.len() {
            self.log.push(PaxosInstance::new());
        }
        self.log[instance].value = Some(value);
        self.log[instance].accepted_id = id;
        self.log[instance].chosen = true;
    }

    /// Handles a negative acknowledgement message.
    fn handle_nack(&mut self, _id: ProposalID, _instance: usize) {
        warn!("Received a Nack!");
        // TODO: clean state for request #id
    }

    /// Handles a client request directly if this server believes itself ot be the leader.
    /// Relays the request to the current leader otherwise.
    fn handle_client_request(&mut self, cmd: Value) {
        if self.node_id == self.current_leader {
            debug!("Handling client request: {:?}", cmd);
            let value = cmd;
            self.log.push(PaxosInstance::new_with_value(value.clone()));
            self.node.broadcast(PaxosMsg::Propose {
                instance: self.log.len() - 1,
                id: self.current_id,
                value,
            });
        } else {
            trace!("Received a client request. Relaying to leader.");
            if !self
                .node
                .send(self.current_leader, PaxosMsg::Relay(cmd.clone()))
            {
                error!("Relaying command to leader failed!");
                self.client_cmd_queue.push(cmd);
            }
        }
    }

    /// Initiates a new election (Prepare/Promise sequence).
    fn start_election(&mut self) {
        info!("Starting election.");
        self.current_id.increment();
        self.highest_promised = self.current_id;
        let accepted_values = self
            .get_accepted_values_iter()
            .map(|(id, i, v)| (id, i, v.clone()))
            .collect();
        self.promises = vec![(self.current_id, accepted_values)];

        let mut holes: Vec<usize> = (&self.log)
            .iter()
            .enumerate()
            .filter(|(_, i)| i.chosen == false)
            .map(|(index, _)| index)
            .collect();
        holes.push(self.log.len());
        debug!("Missing values: {:?}", holes);

        self.node.broadcast(PaxosMsg::Prepare {
            id: self.current_id,
            holes,
        });
    }

    fn get_accepted_values_iter(&self) -> impl Iterator<Item = (usize, ProposalID, &Value)> {
        (&self.log)
            .iter()
            .enumerate()
            .filter(|(_, i)| i.value.is_some())
            .map(|(index, instance)| {
                (
                    index,
                    instance.accepted_id,
                    instance.value.as_ref().unwrap(),
                )
            })
    }
}

impl PaxosInstance {
    fn new() -> Self {
        Self {
            value: None,
            acceptances: 0,
            accepted_id: ProposalID(0, 0),
            chosen: false,
        }
    }

    fn new_with_value(value: Value) -> Self {
        Self {
            value: Some(value),
            acceptances: 1,
            accepted_id: ProposalID(0, 0),
            chosen: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io, time::Duration};

    struct NetworkNodeStub;

    impl NetworkNode for NetworkNodeStub {
        type Addr = usize;

        fn new() -> Self {
            NetworkNodeStub
        }

        fn discover(&mut self, _other_node: usize) {}

        fn recv(&self, _timeout: Duration) -> io::Result<(usize, PaxosMsg)> {
            Ok((0, PaxosMsg::Relay(Vec::new())))
        }

        fn broadcast(&self, _msg: PaxosMsg) {}
        fn send(&self, _dst: usize, _msg: PaxosMsg) -> bool {
            false
        }

        fn id(&self) -> usize {
            0
        }
        fn addr_to_node_id(addr: Self::Addr) -> Option<usize> {
            Some(addr)
        }
        fn node_id_to_addr(node_id: usize) -> Self::Addr {
            node_id
        }
    }

    #[test]
    fn accepted_values() {
        let node = NetworkNodeStub::new();
        let paxos = PaxosServer::new(node, 0, 1);
        assert_eq!(paxos.get_accepted_values_iter().next(), None);
    }
}
