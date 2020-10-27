//! Contains code for abstracting multiple possible network implementations.

use std::io;
use std::time::Duration;

use crate::protocol::PaxosMsg;

pub trait NetworkNode: Sized {
    type Addr;

    /// Creates a new network node.
    fn new() -> Self;

    fn discover(&mut self, other_node: usize);

    /// Receives a message from any of this node's peers.
    /// Returns `io::Error` if no message is received within timeout.
    fn recv(&self, timeout: Duration) -> io::Result<(usize, PaxosMsg)>;

    fn broadcast(&self, msg: PaxosMsg);

    /// Tries to send the message to the peer with ID dst.
    /// Returns `true` on success `false` on failure.
    fn send(&self, dst: usize, msg: PaxosMsg) -> bool;

    fn id(&self) -> usize;

    fn addr_to_node_id(addr: Self::Addr) -> Option<usize>;
    fn node_id_to_addr(node_id: usize) -> Self::Addr;
}
