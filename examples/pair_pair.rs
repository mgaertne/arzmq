use std::thread;

use arzmq::{ZmqResult, context::Context, socket::PairSocket};

mod common;

fn main() -> ZmqResult<()> {
    let endpoint = "inproc://arzmq-example-pair";
    let iterations = 10;

    let context = Context::new()?;

    let pair_server = PairSocket::from_context(&context)?;
    pair_server.bind(endpoint)?;

    thread::spawn(move || {
        (0..iterations)
            .try_for_each(|_| common::run_recv_send(&pair_server, "World"))
            .unwrap();
    });

    let pair_client = PairSocket::from_context(&context)?;
    pair_client.connect(endpoint)?;

    (0..iterations).try_for_each(|_| common::run_send_recv(&pair_client, "Hello"))
}
