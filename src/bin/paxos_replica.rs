// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! A simple binary for launching a single Paxos replica.

use clap::*;

fn main() {
    let matches = clap_app!(MutliPaxos_CLI_Test =>
        (version: "0.1")
        (author: "Quentin M. Kniep <hello@quentinkniep.com>")
        (about: "Launch a single Paxos replica.")
        (@arg GROUP_SIZE: +required "Sets the current Paxos group size")
        (@arg host: -h --host +takes_value "Sets the network interface")
        (@arg port: -p --port +takes_value "Sets the port")
    )
    .get_matches();

    let host = matches.value_of("host").unwrap_or("127.0.0.1");
    let port = value_t!(matches, "port", u16).unwrap_or(0);
    let group_size = value_t!(matches, "GROUP_SIZE", usize).unwrap();

    println!("Running Paxos replica on {}:{}.", host, port);

    paxos::start_replica::<String>(group_size);

    loop {}
}
