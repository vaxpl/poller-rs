poller-rs
=========

File I/O events library for Rust.

Examples
--------

```rs
use poller::{Events, Poller};

fn main() {
    let mut poller = Poller::new();
    poller.add(0, Events::new().with_read());
    for (fd, events) in poller.pull_events(1000).unwrap().iter() {
        println!("Fd={}, Events={}", fd, events);
    }
}
```
