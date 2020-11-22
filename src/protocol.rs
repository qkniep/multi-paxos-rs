// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! Contains the main algorithm for the Paxos consensus protocol.

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// Duration until the leader's lease expires after election.
pub static LEASE_DURATION: u128 = 2000; //2000 ms (= 2 seconds)

/// Unique monotonic increasing ID.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Ballot(usize, usize);

impl Ballot {
    pub fn increment_for(&mut self, node_id: usize) {
        if self.1 > node_id {
            self.0 += 1;
        }
        self.1 = node_id;
    }
}

type PValue<V> = (usize, Ballot, V);
pub type Promise<V> = Vec<PValue<V>>;

/// Internal messages for the Paxos protocol.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PaxosMsg<V: Debug> {
    Prepare {
        id: Ballot,
        holes: Vec<usize>,
    },
    Promise {
        id: Ballot,
        accepted: Promise<V>,
    },

    Propose {
        index: usize,
        id: Ballot,
        value: V,
    },
    Accept {
        index: usize,
        id: Ballot,
    },

    Learn {
        index: usize,
        id: Ballot,
        value: V,
    },

    /// Currently only used for rejecting Proposals.
    Nack {
        index: usize,
        id: Ballot,
    },

    ClientRequest(V),
}

/// Holds the state representing a single slot in the log.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry<V> {
    pub value: Option<V>,
    pub acceptances: Vec<usize>,
    pub accepted_id: Ballot,
    pub chosen: bool,
}

impl<V> LogEntry<V> {
    pub fn new(value: V) -> Self {
        Self {
            value: Some(value),
            acceptances: vec![0], // TODO: actually place own ID here, or don't count it alltogether
            accepted_id: Ballot(0, 0),
            chosen: false,
        }
    }
}

impl<V> Default for LogEntry<V> {
    fn default() -> Self {
        Self {
            value: None,
            acceptances: Vec::new(),
            accepted_id: Ballot(0, 0),
            chosen: false,
        }
    }
}
