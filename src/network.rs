//! Contains code for abstracting multiple possible network implementations.

use std::io;
use std::time::Duration;

use crate::protocol::Command;

pub trait NetworkNode: Sized {
    /// Creates a new network node.
    fn new(address: usize) -> io::Result<Self>;

    /// Receives a message from any of this node's peers.
    /// Returns `io::Error` if no message is received within timeout.
    fn recv(&self, timeout: Duration) -> io::Result<(usize, Command)>;

    fn broadcast(&self, msg: Command);

    /// Tries to send the message to the peer with ID dst.
    /// Returns `true` on success `false` on failure.
    fn send(&self, dst: usize, msg: Command) -> bool;
}
