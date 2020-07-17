// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

use std::io;

fn main() -> io::Result<()> {
    paxos::start_replica(0, 6)?;

    Ok(())
}
