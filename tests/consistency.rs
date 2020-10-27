// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(25))]

        #[test]
        fn random_behaviour(group_size in 3..20usize) {
            for _ in 0..group_size {
                paxos::start_replica(group_size);
            }
        }
    }
}
