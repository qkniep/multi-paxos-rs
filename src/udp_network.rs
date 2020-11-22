// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;
use std::{fmt::Debug, io};

use bincode::{deserialize, serialize};
use rand::prelude::*;

use crate::protocol::PaxosMsg;

const MAX_MSG_SIZE: usize = 64 * 1024;

/// A network node that uses UDP and bincode for sending messages.
#[derive(Debug)]
pub struct UdpNetworkNode<V> {
    pub socket: UdpSocket,
    pub peers: HashSet<usize>,
    _marker: std::marker::PhantomData<V>,
}

impl<V: crate::AppCommand> UdpNetworkNode<V> {
    /// Creates a new network node on localhost with random port.
    pub fn new() -> Self {
        // loop until we find an unused port
        loop {
            let port = rand::thread_rng().gen_range(1024, 65535);
            if let Ok(socket) = UdpSocket::bind(("127.0.0.1", port)) {
                return Self {
                    socket,
                    peers: HashSet::new(),
                    _marker: Default::default(),
                };
            }
        }
    }

    /// Adds another peer's ID to this node's list of known peers.
    pub fn discover(&mut self, other_node: usize) {
        self.peers.insert(other_node);
    }

    /// Blocks until the next message is received.
    /// If this takes longer than timeout an io::Error is returned instead.
    pub fn recv(&self, timeout: Duration) -> io::Result<(usize, PaxosMsg<V>)> {
        self.socket
            .set_read_timeout(Some(timeout))
            .expect("set_read_timeout call failed");

        let mut buf = [0; MAX_MSG_SIZE];
        let (n, from) = self.socket.recv_from(&mut buf)?;

        let cmd: PaxosMsg<V> = deserialize(&buf[..n]).unwrap();
        Ok((Self::addr_to_node_id(from).unwrap(), cmd))
    }

    /// Sends the Paxos message to all other replicas.
    pub fn broadcast(&self, cmd: PaxosMsg<V>) {
        for addr in self.peers.clone() {
            self.send(addr, &cmd);
        }
    }

    /// Sends the Paxos message to another replica.
    pub fn send(&self, dst: usize, cmd: &PaxosMsg<V>) -> bool {
        let serialized = serialize(cmd).unwrap();
        assert!(serialized.len() <= MAX_MSG_SIZE);
        self.socket
            .send_to(&serialized, Self::node_id_to_addr(dst))
            .is_ok()
    }

    pub fn id(&self) -> usize {
        Self::addr_to_node_id(self.socket.local_addr().unwrap()).unwrap()
    }

    /// Convert a socket address (IP + port) into a usize node ID.
    /// This transformation can be reversed.
    fn addr_to_node_id(addr: SocketAddr) -> Option<usize> {
        let port = addr.port();
        if let IpAddr::V4(ip) = addr.ip() {
            let ipv4: u32 = ip.into();
            Some(ipv4 as usize * 65536 + port as usize)
        } else {
            None
        }
    }

    /// Convert a usize node ID into a socket address (IP + port).
    /// This transformation can be reversed.
    fn node_id_to_addr(node_id: usize) -> SocketAddr {
        let port = (node_id % 65536) as u16;
        let ip = (node_id / 65536) as u32;
        SocketAddr::from((Ipv4Addr::from(ip), port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn node_id_addr_conversion(ip: u32, port: u16) {
            let addr = SocketAddr::from((Ipv4Addr::from(ip), port));
            assert_eq!(UdpNetworkNode::<u32>::node_id_to_addr(UdpNetworkNode::<u32>::addr_to_node_id(addr).unwrap()), addr);
        }
    }
}
