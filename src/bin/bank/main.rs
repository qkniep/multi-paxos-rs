// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

use std::collections::HashMap;
use std::{io, thread::sleep, time::Duration};

use serde::{Deserialize, Serialize};
use tracing::Level;

use paxos::ReplicatedStateMachine;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CustomerAction {
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

impl paxos::AppCommand for CustomerAction {}

#[derive(Default)]
struct Bank {
    balances: HashMap<String, usize>,
}

impl ReplicatedStateMachine for Bank {
    type Command = CustomerAction;

    fn execute(&mut self, action: Self::Command) -> bool {
        match action {
            CustomerAction::Open { account } => {
                self.balances.insert(account, 0).is_some()
            }
            CustomerAction::Deposit { account, amount } => {
                if let Some(balance) = self.balances.get_mut(&account) {
                    *balance += amount;
                    true
                } else {
                    false
                }
            }
            CustomerAction::Withdraw { account, amount } => {
                if let Some(balance) = self.balances.get_mut(&account) {
                    *balance -= amount;
                    true
                } else {
                    false
                }
            }
            CustomerAction::Transfer { src, dst, amount } => {
                if self.balances.contains_key(&src) && self.balances.contains_key(&dst) {
                    *self.balances.get_mut(&src).unwrap() -= amount;
                    *self.balances.get_mut(&dst).unwrap() += amount;
                    true
                } else {
                    false
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    use tracing_subscriber::{fmt::time::ChronoLocal, FmtSubscriber};

    // initialize the tracer
    FmtSubscriber::builder()
        .with_timer(ChronoLocal::with_format("[%Mm %Ss]".to_string()))
        .with_max_level(Level::INFO)
        .init();

    // create and connect a number of bank branches
    let branches = paxos::start_replicas::<CustomerAction>(5);
    sleep(Duration::new(3, 0));

    paxos::submit_value(
        branches[0],
        CustomerAction::Open {
            account: "Peter".to_string(),
        },
    );
    paxos::submit_value(
        branches[3],
        CustomerAction::Open {
            account: "Dieter".to_string(),
        },
    );

    paxos::submit_value(
        branches[1],
        CustomerAction::Deposit {
            account: "Peter".to_string(),
            amount: 100,
        },
    );
    paxos::submit_value(
        branches[2],
        CustomerAction::Withdraw {
            account: "Dieter".to_string(),
            amount: 50,
        },
    );

    paxos::submit_value(
        branches[4],
        CustomerAction::Deposit {
            account: "Dieter".to_string(),
            amount: 60,
        },
    );
    paxos::submit_value(
        branches[0],
        CustomerAction::Withdraw {
            account: "Peter".to_string(),
            amount: 80,
        },
    );

    sleep(Duration::new(3, 0));

    Ok(())
}
