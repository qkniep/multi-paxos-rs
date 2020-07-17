// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

#[cfg(test)]
mod tests {
    #[test]
    fn random_behaviour() {
        paxos::start_replica(0, 2).ok();
        paxos::start_replica(1, 2).ok();
    }
}
