# Collaborative Terminal Editor Written in Rust
## Abstract
Optimistic approaches to share data is gaining attention with database solutions like Riak and collaborative editors. And it has its properties to back it with immediately response of the local data type and it being conflict-free. This project implements a Conflictfree Replicated Data Type (CRDT), a network stack and a terminal to make a collaborative editor. The projects show that is possible to integrate and have functional editor written in Rust using the Ditto library.

## Compile and run the editors in seperate shells
```
cargo build
cargo run -p coeditor node_a.toml
cargo run -p coeditor node_b.toml
cargo run -p coeditor node_c.toml
```

![](crdt_demo.gif)

## Testing
```
cargo test
```
