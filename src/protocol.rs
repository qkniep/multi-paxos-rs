// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! Contains the main algorithm for the Paxos consensus protocol.

use std::fmt::Debug;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, trace_span, warn};

use crate::udp_network::UdpNetworkNode;

/// Duration
static LEASE_DURATION: u64 = 2;

/// Unique monotonically-increasing ID.
#[derive(Serialize, Deserialize)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Ballot(usize, usize);

impl Ballot {
    fn increment_for(&mut self, node_id: usize) {
        if self.1 > node_id {
            self.0 += 1;
        }
        self.1 = node_id;
    }
}

type PValue<V> = (usize, Ballot, V);
type Promise<V> = Vec<PValue<V>>;

/// Internal messages for the Paxos protocol.
#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub enum PaxosMsg<V: Debug> {
    Prepare {
        id: Ballot,
        holes: Vec<usize>,
    },
    Promise {
        id: Ballot,
        accepted: Promise<V>,
    },

    Propose {
        id: Ballot,
        instance: usize,
        value: V,
    },
    Accept {
        id: Ballot,
        instance: usize,
    },

    Learn {
        id: Ballot,
        instance: usize,
        value: V,
    },

    /// Currently only used for rejecting Proposals.
    Nack {
        id: Ballot,
        instance: usize,
    },

    ClientRequest(V),
}

/// Handles all Paxos related state for a single node,
/// acting as proposer, acceptor and learner.
#[derive(Debug)]
pub struct PaxosServer<V: Debug> {
    node_id: usize,
    node: UdpNetworkNode<V>,
    client_cmd_queue: Vec<V>,
    log: Vec<LogEntry<V>>,
    quorum: usize,
    // TODO: replace with Option<usize>
    current_leader: usize,
    leader_lease_start: Instant,
    /// Always holds the highest
    highest_promised: Ballot,
    promises: Vec<(Ballot, Promise<V>)>,
}

/// Holds the state representing a single slot in the log.
#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, Default)]
struct LogEntry<V> {
    value: Option<V>,
    acceptances: usize,
    accepted_id: Ballot,
    chosen: bool,
}

impl<V: crate::AppCommand> PaxosServer<V> {
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
    pub fn new(node: UdpNetworkNode<V>, node_id: usize, node_count: usize) -> Self {
        Self {
            node_id,
            node,
            client_cmd_queue: Vec::new(),
            log: Vec::new(),
            quorum: node_count / 2 + 1,
            current_leader: 0,
            leader_lease_start: Instant::now(),
            highest_promised: Ballot(0, 0),
            promises: Vec::new(),
        }
    }

    /// Runs this Paxos server's main loop.
    pub fn run(&mut self) {
        // configure a span to associate tracing output with this network node
        let tracing_span = trace_span!("Server", id = self.node_id);
        let _guard = tracing_span.enter();
        info!("Starting Paxos Node with ID {}", self.node_id);

        loop {
            // event loop for incoming messages
            while let Ok(msg) = self.node.recv(Duration::from_millis(100)) {
                let (src, cmd) = msg;
                self.handle_paxos_message(src, cmd);
            }

            //warn!("Long time w/o receiving anything!");
            if self.leader_lease_start.elapsed().as_secs() >= LEASE_DURATION {
                warn!("Leader's lease timed out: Starting election.");
                self.start_election();
            } else if self.leader_lease_start.elapsed().as_secs() >= LEASE_DURATION / 2 &&
                self.node_id == self.current_leader {
                    info!("Extending my lease: Starting election.");
                    self.start_election();
            }
        }
    }

    /// Parses the message and calls the method corresponding to the message type.
    fn handle_paxos_message(&mut self, src: usize, cmd: PaxosMsg<V>) {
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
            PaxosMsg::ClientRequest(value) => self.handle_client_request(value),
        }
    }

    /// Responds to a Paxos Prepare (1a) message.
    /// Sends a Promise back to the sender iff this node has not made Promise for a higher id yet.
    fn handle_prepare(&mut self, src: usize, id: Ballot, holes: Vec<usize>) {
        if id < self.highest_promised {
            warn!("Prepare rejected: {:?}<{:?}", id, self.highest_promised);
            return;
        } else if self.leader_lease_start.elapsed().as_secs() < LEASE_DURATION && src !=
            self.current_leader {
            warn!("Prepare rejected: {:?} currently holds the lease", self.current_leader);
            return;
        }

        debug!("Promise vote: {:?}", id);
        self.highest_promised = id;
        self.current_leader = src;
        self.leader_lease_start = Instant::now();
        // TODO: flush to disk

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
        self.node.send(src, &promise);
    }

    /// Responds to a Paxos Promise (1b) message.
    fn handle_promise(&mut self, id: Ballot, accepted: Promise<V>) {
        if id != self.highest_promised {
            warn!("Promise rejected: {:?}!={:?}", id, self.highest_promised);
            return;
        }

        debug!("Got a promise: {:?}, {:?}", id, accepted);
        self.promises.push((id, accepted)); // TODO: Remove promises w/ old id before counting.

        if self.promises.len() == self.quorum {
            info!("Got elected.");
            self.current_leader = self.node_id;
            self.leader_lease_start = Instant::now();
            for (i, instance) in (&self.log)
                .iter()
                .enumerate()
                .filter(|(_, instance)| !instance.chosen)
            {
                // TODO: do not chose own value if any is present in promise
                self.node.broadcast(PaxosMsg::Propose {
                    instance: i,
                    id,
                    value: instance.value.as_ref().unwrap().clone(),
                });
            }
        }
    }

    /// Responds to a Paxos Propose (2a) message.
    fn handle_propose(&mut self, src: usize, id: Ballot, instance: usize, value: V) {
        if id < self.highest_promised {
            warn!("Proposal rejected: {:?}", value);
            self.node.send(src, &PaxosMsg::Nack { instance, id });
            return;
        }

        trace!("Proposal accepted: {:?}", value);
        self.current_leader = src;
        while instance >= self.log.len() {
            self.log.push(LogEntry::new());
        }
        self.node.send(src, &PaxosMsg::Accept { instance, id });
        self.log[instance].value = Some(value);
        self.log[instance].accepted_id = id;
    }

    /// Responds to a Paxos Accept (2b) message.
    fn handle_accept(&mut self, id: Ballot, instance: usize) {
        if id != self.highest_promised {
            warn!(
                "Accept rejected: [{}]: {:?}!={:?}",
                instance, id, self.highest_promised
            );
            return;
        }
        self.log[instance].acceptances += 1;
        if self.log[instance].acceptances == self.quorum {
            debug!(
                "Sending learn with {}/{} acceptances.",
                self.log[instance].acceptances, self.quorum
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
    fn handle_learn(&mut self, id: Ballot, instance: usize, value: V) {
        info!("Learned: [{}] {:?},{:?}", instance, id, value);
        while instance >= self.log.len() {
            self.log.push(LogEntry::new());
        }
        self.log[instance].value = Some(value);
        self.log[instance].accepted_id = id;
        self.log[instance].chosen = true;
        // TODO: flush to disk
    }

    /// Handles a negative acknowledgement message.
    fn handle_nack(&mut self, _id: Ballot, _instance: usize) {
        warn!("Received a Nack!");
        // TODO: clean state for request #id
    }

    /// Handles a client request directly if this server believes itself ot be the leader.
    /// Relays the request to the current leader otherwise.
    fn handle_client_request(&mut self, cmd: V) {
        if self.node_id == self.current_leader {
            debug!("Handling client request: {:?}", cmd);
            let value = cmd;
            self.log.push(LogEntry::new_with_value(value.clone()));
            self.node.broadcast(PaxosMsg::Propose {
                instance: self.log.len() - 1,
                id: self.highest_promised,
                value,
            });
        } else {
            trace!("Received a client request. Relaying to leader.");
            if !self
                .node
                .send(self.current_leader, &PaxosMsg::ClientRequest(cmd.clone()))
            {
                error!("Relaying command to leader failed!");
                self.client_cmd_queue.push(cmd);
            }
        }
    }

    /// Initiates a new election (Prepare/Promise sequence).
    fn start_election(&mut self) {
        self.highest_promised.increment_for(self.node_id);
        let accepted_values = self
            .get_accepted_values_iter()
            .map(|(id, i, v)| (id, i, v.clone()))
            .collect();
        self.promises = vec![(self.highest_promised, accepted_values)];

        let mut holes: Vec<usize> = (&self.log)
            .iter()
            .enumerate()
            .filter(|(_, i)| i.chosen == false)
            .map(|(index, _)| index)
            .collect();
        holes.push(self.log.len());
        debug!("Missing values: {:?}", holes);

        self.node.broadcast(PaxosMsg::Prepare {
            id: self.highest_promised,
            holes,
        });
    }

    fn get_accepted_values_iter(&self) -> impl Iterator<Item = (usize, Ballot, &V)> {
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

impl<V> LogEntry<V> {
    fn new() -> Self {
        Self {
            value: None,
            acceptances: 0,
            accepted_id: Ballot(0, 0),
            chosen: false,
        }
    }

    fn new_with_value(value: V) -> Self {
        Self {
            value: Some(value),
            acceptances: 1,
            accepted_id: Ballot(0, 0),
            chosen: false,
        }
    }
}

/*#[cfg(test)]
mod tests {
    use super::*;

    fn accepted_values() {
        let node = NetworkNodeStub::new();
        let paxos = PaxosServer::new(node, 0, 1);
        assert_eq!(paxos.get_accepted_values_iter().next(), None);
    }
}*/
