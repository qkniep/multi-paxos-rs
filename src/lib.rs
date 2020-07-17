// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! Implementation of the Multi-Paxos consensus protocol for an example banking application.

use std::io;
use std::thread;

use bincode::serialize;
use serde::Serialize;

use network::NetworkNode;
use protocol::{Command, PaxosServer};
use udp_network::UdpNetworkNode;

mod network;
mod protocol;
mod udp_network;

pub fn start_replica(address: usize, group_size: usize) -> io::Result<()> {
    let mut node = UdpNetworkNode::new(address)?;
    node.peers = (0..group_size).collect();
    thread::spawn(move || PaxosServer::new(node, address, group_size).run());

    Ok(())
}

pub fn submit_value<T: Serialize>(value: &T) {
    let node = UdpNetworkNode::new(99).unwrap();
    let serialized_value: Vec<u8> = serialize(value).unwrap();
    node.send(0, Command::Relay(serialized_value));
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_replica_test() {
        assert!(start_replica(0, 2).is_ok());
        assert!(start_replica(0, 2).is_err());
        assert!(start_replica(1, 2).is_ok());
    }

    #[test]
    fn submit_value_test() {
        start_replica(3, 2).ok();
        start_replica(4, 2).ok();
        submit_value(&"Hello".to_string());
    }
}
