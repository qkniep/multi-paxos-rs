// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! Contains structures, types and constants used by the rest of the Paxos implementation.

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// Duration until the leader's lease expires after election.
pub static LEASE_DURATION: u128 = 2000; //2000 ms (= 2 seconds)

/// Unique monotonic increasing ID.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Ballot(usize, usize);

impl Ballot {
    /// Changes this Ballot number to be a higher number than before.
    /// The resulting Ballot number is again in the space of numbers for this peer,
    /// i.e. no other peer could ever generate the same number.
    pub fn increment_for(&mut self, node_id: usize) {
        if self.1 > node_id {
            self.0 += 1;
        }
        self.1 = node_id;
    }
}

/// Represents a preliminary log entry as (index, ballot, value).
type PValue<V> = (usize, Ballot, V);
pub type Promise<V> = Vec<PValue<V>>;

/// Internal messages for the Paxos protocol.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PaxosMsg<V: Debug> {
    /// Paxos phase 1a message
    Prepare {
        ballot: Ballot,
        holes: Vec<usize>,
    },
    /// Paxos phase 1b message
    Promise {
        ballot: Ballot,
        accepted: Promise<V>,
    },

    /// Paxos phase 2a message
    Propose {
        index: usize,
        ballot: Ballot,
        value: V,
    },
    /// Paxos phase 2b message
    Accept {
        index: usize,
        ballot: Ballot,
    },

    Learn {
        index: usize,
        ballot: Ballot,
        value: V,
    },

    /// This message is sent when a Prepare/Propose request is rejected due to a higher Ballot.
    Nack {
        ballot: Ballot,
    },

    ClientRequest(V),
}

/// Holds the state representing a single slot in the log.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry<V> {
    /// The value this replica currently believes to be the value for this entry.
    pub value: Option<V>,
    /// The `node_id`s of the replicas that have accepted this entry.
    pub acceptances: Vec<usize>,
    pub accepted_ballot: Ballot,
    pub chosen: bool, // TODO: replace with accepted_id==Ballot(INFINITY, INFINITY)?
}

impl<V> LogEntry<V> {
    pub fn new(value: V) -> Self {
        Self {
            value: Some(value),
            acceptances: vec![0], // TODO: actually place own ID here, or don't count it alltogether
            accepted_ballot: Ballot(0, 0),
            chosen: false,
        }
    }
}

impl<V> Default for LogEntry<V> {
    /// Create a completely empty entry for the log.
    /// This can be used at indices for which we have received no Propose message yet.
    fn default() -> Self {
        Self {
            value: None,
            acceptances: Vec::new(),
            accepted_ballot: Ballot(0, 0),
            chosen: false,
        }
    }
}
