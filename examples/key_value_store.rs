// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

use std::collections::HashMap;
use std::{io, thread, time::Duration};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tracing::{info, info_span, Level};

use paxos::{PaxosReplica, ReplicatedStateMachine, UdpNetworkNode};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Operation {
    Put { key: String, value: String },
    Get { key: String },
}

impl paxos::AppCommand for Operation {}

#[derive(Default)]
struct KeyValueStore {
    store: HashMap<String, String>,
}

impl ReplicatedStateMachine for KeyValueStore {
    type Command = Operation;

    fn execute(&mut self, action: Self::Command) -> Result<String, ()> {
        match action {
            Operation::Put { key, value } => self.store.insert(key, value).ok_or(()),
            Operation::Get { key } => self.store.get(&key).map(|v| v.clone()).ok_or(()),
        }
    }
}

pub fn start_kv_stores(group_size: usize) {
    // create the network nodes
    let mut nodes = Vec::new();
    for _ in 0..group_size {
        nodes.push(UdpNetworkNode::<Operation>::new());
    }

    // start the replicas and make them know about everyone else
    let node_ids = nodes.iter().map(|n| n.id()).collect();
    for mut node in nodes {
        node.discover(&node_ids);
        let node_id = node.id();
        let mut replica = PaxosReplica::<Operation>::new(node, node_id, group_size);
        thread::spawn(move || {
            // configure a span to associate tracing output with this replica
            let tracing_span = info_span!("Replica", id = node_id);
            let _guard = tracing_span.enter();
            info!("Starting Paxos Replica with ID {}", node_id);

            // main loop
            loop {
                replica.tick();
                if thread_rng().gen_range(0, 500) == 0 {
                    replica.submit_value(Operation::Put {
                        key: "Hello".to_string(),
                        value: "World".to_string(),
                    });
                }
            }
        });
    }
}

fn main() -> io::Result<()> {
    use tracing_subscriber::{fmt::time::ChronoLocal, FmtSubscriber};

    // initialize the tracer
    FmtSubscriber::builder()
        .with_timer(ChronoLocal::with_format("[%Mm %Ss]".to_string()))
        .with_max_level(Level::DEBUG)
        .init();

    // create and connect a number of Paxos replicas maintaining the key value store
    start_kv_stores(5);
    thread::sleep(Duration::new(6, 0));

    Ok(())
}
