// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! Implementation of a replicated log using the Multi-Paxos consensus protocol.

use std::fmt::Debug;
use std::thread;

use serde::{Serialize, de::DeserializeOwned};

use protocol::{PaxosMsg, PaxosServer};
use udp_network::UdpNetworkNode;

mod protocol;
mod udp_network;

pub trait AppCommand: Clone + Debug + Serialize + DeserializeOwned + Send + 'static {}
impl AppCommand for String {}

pub trait ReplicatedStateMachine {
    type Command: AppCommand;

    fn execute(&mut self, v: Self::Command) -> bool;
}

pub fn start_replicas<V: AppCommand>(group_size: usize) -> Vec<usize> {
    // create the network nodes
    let mut nodes = Vec::new();
    for _ in 0..group_size {
        nodes.push(UdpNetworkNode::<V>::new());
    }
    // make every node know about everyone else
    for node_a in 0..group_size {
        for node_b in 0..group_size {
            if node_a == node_b {
                continue;
            }
            let node_b_id = nodes[node_b].id();
            nodes[node_a].discover(node_b_id);
        }
    }
    // return a Vec of all Paxos IDs
    let mut node_ids = Vec::new();
    for node in nodes {
        let node_id = node.id();
        node_ids.push(node_id);
        thread::spawn(move || PaxosServer::<V>::new(node, node_id, group_size).run());
    }
    node_ids
}

pub fn start_replica<V: AppCommand>(group_size: usize) -> usize {
    let node = UdpNetworkNode::new();
    let node_id = node.id();
    thread::spawn(move || PaxosServer::<V>::new(node, node_id, group_size).run());
    node_id
}

pub fn submit_value<T: AppCommand>(node_id: usize, value: T) {
    let node = UdpNetworkNode::new();
    node.send(node_id, &PaxosMsg::ClientRequest(value));
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(25))]

        #[test]
        fn random_start_replica_test(group_size in 1..50usize) {
            for _ in 0..group_size {
                start_replica::<String>(group_size);
            }
        }
        #[test]
        fn submit_random_value_test(s in "\\PC*{1,128}") {
            let nodes = start_replicas::<String>(3);
            submit_value(nodes[0], s);
        }
    }

    #[test]
    fn start_replicas_test() {
        start_replicas::<String>(3);
    }

    #[test]
    fn submit_value_test() {
        use tracing::Level;
        use tracing_subscriber::{fmt::time::ChronoLocal, FmtSubscriber};

        // initialize the tracer
        FmtSubscriber::builder()
            .with_timer(ChronoLocal::with_format("[%Mm %Ss]".to_string()))
            .with_max_level(Level::TRACE)
            .init();

        let nodes = start_replicas::<String>(2);
        thread::sleep(std::time::Duration::new(3, 0));
        submit_value(nodes[0], "Hello".to_owned());
        submit_value(nodes[1], "World".to_owned());
        thread::sleep(std::time::Duration::new(3, 0));
    }
}
