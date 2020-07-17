# Multi-Paxos implementation in Rust

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
