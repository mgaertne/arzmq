# Asynchronous Rust bindings for 0MQ (arzmq)

[![Apache 2.0 licensed](https://img.shields.io/badge/license-Apache2.0-blue.svg)](./LICENSE-APACHE)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)
![Crates.io Version](https://img.shields.io/crates/v/arzmq)
[![docs](https://docs.rs/zmq/badge.svg)](https://docs.rs/arzmq)

[Documentation](https://docs.rs/crate/arzmq/)

# About

The `arzmq` crate provides bindings for the `libzmq` library from the
[ZeroMQ](https://zeromq.org/) project. The API exposed by `arzmq` should
be safe (in the usual Rust sense), but it follows the C API closely,
so it is not very idiomatic.

There are feature flags for enabling a `builder` interface for contexts and 
sockets as well as a feature flag for enabling 0MQ's `draft-api` available.

# Compatibility

The aim of this project is to track latest zmq releases as close as possible.

# Usage

`arzmq` is a pretty straight forward port of the C API into Rust:

```rust
use std::thread;

use arzmq::{context::Context, socket::{RequestSocket, ReplySocket, SendFlags, Receiver, RecvFlags, Sender}};

fn run_reply(context: &Context, endpoint: &str) {
    let socket = ReplySocket::from_context(context).unwrap();
    socket.bind(endpoint).unwrap();
    
    thread::spawn(move || {
       while socket.recv_msg(RecvFlags::DONT_WAIT).is_err() {}
    });
}

fn main() {
    let ctx = Context::new().unwrap();
    
    run_reply(&ctx, "tcp://127.0.0.1:1234");

    let socket = RequestSocket::from_context(&ctx).unwrap();
    socket.connect("tcp://127.0.0.1:1234").unwrap();
    let _ = socket.send_msg("hello world!", SendFlags::DONT_WAIT);
}
```

You can find more usage examples in
<https://github.com/mgaertne/arzmq/tree/master/examples>.
