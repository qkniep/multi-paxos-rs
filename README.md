# Multi-Paxos replicated log in Rust

[![Build Status](https://img.shields.io/travis/qkniep/multi-paxos-rs?logo=travis)](https://travis-ci.org/qkniep/multi-paxos-rs)
[![codecov](https://codecov.io/gh/qkniep/multi-paxos-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/qkniep/multi-paxos-rs)
![LoC](https://tokei.rs/b1/github/qkniep/multi-paxos-rs?category=code)
![License](https://img.shields.io/github/license/qkniep/multi-paxos-rs)

This is a very basic replicated log using Multi-Paxos implemented in Rust.
I do the project for fun, it is not intended for production use!

# Using the Library

TBA, the API is not fixed yet.

# Roadmap

- [ ] random failure testing
- [ ] liveness recovery after failures testing
- [ ] persistent storage of log
- [ ] handle NACKs
- [ ] specify library API
- [ ] benchmarking
- [ ] group membership changes
- [ ] snapshot support
- [ ] runtime consistency checks (checksum pushed into the log by leader)
- [ ] checksums on stored data

# References

- L. Lamport 2001 ["Paxos Made Simple"](http://lamport.azurewebsites.net/pubs/paxos-simple.pdf)
- T. Chandra et al. 2007 ["Paxos Made Live"](http://www.read.seas.harvard.edu/~kohler/class/08w-dsi/chandra07paxos.pdf)
