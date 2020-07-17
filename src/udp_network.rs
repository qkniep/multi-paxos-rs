// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

use std::{io, net::UdpSocket, time::Duration};

use bincode::{deserialize, serialize};
// use crc32fast::Hasher;

use crate::network::NetworkNode;
use crate::protocol::Command;

const MAX_SZ: usize = 64 * 1024;

/// A network node that uses UDP and bincode for sending messages.
pub struct UdpNetworkNode {
    pub socket: UdpSocket,
    pub peers: Vec<usize>,
}

impl NetworkNode for UdpNetworkNode {
    /// Creates a new UdpTransport.
    fn new(addr: usize) -> io::Result<Self> {
        let socket = UdpSocket::bind(("127.0.0.1", 64000 + addr as u16))?;
        Ok(Self {
            socket,
            peers: Vec::new(),
        })
    }

    /// Blocks until the next message is received.
    fn recv(&self, timeout: Duration) -> io::Result<(usize, Command)> {
        self.socket
            .set_read_timeout(Some(timeout))
            .expect("set_read_timeout call failed");

        let mut buf = [0; MAX_SZ];
        let (n, from) = self.socket.recv_from(&mut buf)?;

        let cmd: Command = deserialize(&buf[..n]).unwrap();
        Ok(((from.port() - 64000) as usize, cmd))
    }

    fn broadcast(&self, cmd: Command) {
        for &addr in &self.peers {
            self.send(addr, cmd.clone());
        }
    }

    /// Enqueues the message to be sent. May be sent 0-N times with no ordering guarantees.
    fn send(&self, dst: usize, cmd: Command) -> bool {
        let serialized = serialize(&cmd).unwrap();
        assert!(serialized.len() <= MAX_SZ);
        let _n = self
            .socket
            .send_to(&serialized, ("127.0.0.1", 64000 + dst as u16))
            .unwrap();
        return true;
    }
}
