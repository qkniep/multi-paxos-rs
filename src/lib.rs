// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! Implementation of a replicated log using the Multi-Paxos consensus protocol.

use std::thread;

use bincode::serialize;
use serde::Serialize;

use network::NetworkNode;
use protocol::{PaxosMsg, PaxosServer};
use udp_network::UdpNetworkNode;

mod network;
mod protocol;
mod udp_network;

pub fn start_replicas(group_size: usize) -> Vec<usize> {
    let mut nodes = Vec::new();
    for _ in 0..group_size {
        nodes.push(UdpNetworkNode::new());
    }
    for node_a in 0..group_size {
        for node_b in 0..group_size {
            if node_a == node_b { continue; }
            let node_b_id = nodes[node_b].id();
            nodes[node_a].discover(node_b_id);
        }
    }
    let mut node_ids = Vec::new();
    for node in nodes {
        let node_id = node.id();
        node_ids.push(node_id);
        thread::spawn(move || PaxosServer::new(node, node_id, group_size).run());
    }
    node_ids
}

pub fn start_replica(group_size: usize) -> usize {
    let node = UdpNetworkNode::new();
    let node_id = node.id();
    thread::spawn(move || PaxosServer::new(node, node_id, group_size).run());
    node_id
}

pub fn submit_value<T: Serialize>(node_id: usize, value: &T) {
    let node = UdpNetworkNode::new();
    let serialized_value: Vec<u8> = serialize(value).unwrap();
    node.send(node_id, PaxosMsg::Relay(serialized_value));
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
                start_replica(group_size);
            }
        }
        #[test]
        fn submit_random_value_test(s in "\\PC*{1,128}") {
            let nodes = start_replicas(3);
            submit_value(nodes[0], &s);
        }
    }

    #[test]
    fn start_replicas_test() {
        start_replicas(3);
    }

    #[test]
    fn submit_value_test() {
        use tracing::Level;
        use tracing_subscriber::{FmtSubscriber, fmt::time::ChronoLocal};

        // initialize the tracer
        FmtSubscriber::builder()
            .with_timer(ChronoLocal::with_format("[%Mm %Ss]".to_string()))
            .with_max_level(Level::TRACE)
            .init();

        let nodes = start_replicas(2);
        thread::sleep(std::time::Duration::new(3, 0));
        submit_value(nodes[0], &"Hello");
        submit_value(nodes[1], &"aAௗ0㌀0");
        thread::sleep(std::time::Duration::new(3, 0));
    }
}
