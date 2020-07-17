# Multi-Paxos implementation in Rust

[![Build Status](https://img.shields.io/travis/qkniep/multi-paxos-rs?logo=travis)](https://travis-ci.org/qkniep/multi-paxos-rs) ![LoC](https://tokei.rs/b1/github/qkniep/multi-paxos-rs?category=code) ![License](https://img.shields.io/github/license/qkniep/multi-paxos-rs)

This is a very basic Multi-Paxos implementation in Rust.
I do the project for fun, it is not intended for production use!

# Dependencies

# Using Library

# Roadmap

- [ ] random failure testing
- [ ] liveness recovery after failures testing
- [ ] handle NACKs
- [ ] runtime consistency checks (checksum pushed into the log by leader)
- [ ] persistent storage of log
- [ ] checksums on stored data
- [ ] group membership changes
- [ ] snapshot support

# References

- Paxos Made Simple
- Paxos Made Live
