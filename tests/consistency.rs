// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn random_behaviour(_a in 0..1) {
            paxos::start_replica(2);
            paxos::start_replica(2);
        }
    }
}
