// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! Implementation of a replicated log using the Multi-Paxos consensus protocol.

use std::thread;

use bincode::serialize;
use serde::Serialize;

use network::NetworkNode;
use protocol::{Command, PaxosServer};
use udp_network::UdpNetworkNode;

mod network;
mod protocol;
mod udp_network;

pub fn start_replica(group_size: usize) {
    let node = UdpNetworkNode::new();
    let node_id = node.id();
    thread::spawn(move || PaxosServer::new(node, node_id, group_size).run());
}

pub fn submit_value<T: Serialize>(value: &T) {
    let node = UdpNetworkNode::new();
    let serialized_value: Vec<u8> = serialize(value).unwrap();
    node.send(0, Command::Relay(serialized_value));
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
            start_replica(2);
            submit_value(&s);
        }
    }

    #[test]
    fn start_replica_test() {
        start_replica(3);
        start_replica(3);
        start_replica(3);
    }

    #[test]
    fn submit_value_test() {
        start_replica(2);
        start_replica(2);
        submit_value(&"Hello");
        submit_value(&"aAௗ0㌀0");
    }
}
