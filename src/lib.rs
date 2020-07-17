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
