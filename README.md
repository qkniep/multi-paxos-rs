# Multi-Paxos replicated log in Rust

[![GH Build Status](https://img.shields.io/github/workflow/status/qkniep/multi-paxos-rs/Rust/main?logo=GitHub&style=for-the-badge&labelColor=222)](https://github.com/qkniep/multi-paxos-rs/actions)
[![Travis Build Status](https://img.shields.io/travis/qkniep/multi-paxos-rs?logo=Travis&style=for-the-badge&labelColor=222)](https://travis-ci.org/qkniep/multi-paxos-rs)
[![Test Coverage](https://img.shields.io/codecov/c/github/qkniep/multi-paxos-rs?logo=codecov&style=for-the-badge&labelColor=222)](https://codecov.io/gh/qkniep/multi-paxos-rs)
![Lines of Code](https://img.shields.io/tokei/lines/github/qkniep/multi-paxos-rs?style=for-the-badge&labelColor=222)
![License](https://img.shields.io/github/license/qkniep/multi-paxos-rs?style=for-the-badge&labelColor=222)

This is a very basic replicated log using Multi-Paxos implemented in Rust.
I do the project for fun, it is not intended for production use!

# Using the Library

TBA, the API is not fixed yet.

# Roadmap

- [x] master leases
- [ ] persistent storage of log
- [ ] full disclosure
- [ ] specify library API
- [ ] random failure testing
- [ ] liveness recovery after failures testing
- [ ] handle NACKs
- [ ] group membership changes
- [ ] snapshots
- [ ] benchmarking
- [ ] runtime consistency checks (checksum pushed into the log by leader)
- [ ] checksums on stored data

# References

- L. Lamport 2001 ["Paxos Made Simple"](http://lamport.azurewebsites.net/pubs/paxos-simple.pdf)
- T. Chandra et al. 2007 ["Paxos Made Live"](http://www.read.seas.harvard.edu/~kohler/class/08w-dsi/chandra07paxos.pdf)
- B. Lampson 1996 ["How to Build a Highly Available System Using Consensus"](https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.61.8330&rep=rep1&type=pdf)
- [Understanding Paxos](https://understandingpaxos.wordpress.com/)
