// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

use std::{
    env::args,
    fs, io,
    thread::sleep,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tracing::Level;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientCommand {
    Open {
        account: String,
    },
    Deposit {
        account: String,
        amount: usize,
    },
    Withdraw {
        account: String,
        amount: usize,
    },
    Transfer {
        src: String,
        dst: String,
        amount: usize,
    },
}

/// Creates and connects a number of branch offices for the bank.
pub fn setup_offices(office_count: usize, log_path: &str) -> io::Result<()> {
    fs::create_dir_all(log_path)?;

    // create various network nodes and start them
    for address in 0..office_count {
        paxos::start_replica(address, office_count)?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    use tracing_subscriber::{fmt::time::ChronoLocal, FmtSubscriber};
    let log_path = args().nth(1).unwrap_or_else(|| "logs".to_string());

    // initialize the tracer
    FmtSubscriber::builder()
        .with_timer(ChronoLocal::with_format("[%Mm %Ss]".to_string()))
        .with_max_level(Level::TRACE)
        .init();

    // create and connect a number of offices
    let _channels = setup_offices(6, &log_path)?;

    paxos::submit_value(&ClientCommand::Open {
        account: "Peter".to_string(),
    });

    sleep(Duration::new(2, 0));

    Ok(())
}
