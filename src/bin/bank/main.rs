// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

use std::{io, thread::sleep, time::Duration};

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
pub fn setup_offices(office_count: usize) -> io::Result<()> {
    // create various network nodes and start them
    for _address in 0..office_count {
        paxos::start_replica(office_count);
    }

    Ok(())
}

fn main() -> io::Result<()> {
    use tracing_subscriber::{fmt::time::ChronoLocal, FmtSubscriber};

    // initialize the tracer
    FmtSubscriber::builder()
        .with_timer(ChronoLocal::with_format("[%Mm %Ss]".to_string()))
        .with_max_level(Level::TRACE)
        .init();

    // create and connect a number of offices
    let _channels = setup_offices(6)?;

    paxos::submit_value(&ClientCommand::Open {
        account: "Peter".to_string(),
    });

    sleep(Duration::new(2, 0));

    Ok(())
}
