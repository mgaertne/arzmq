use std::thread;

use arzmq::{ZmqResult, context::Context, socket::ChannelSocket};

mod common;

fn main() -> ZmqResult<()> {
    let endpoint = "inproc://arzmq-example-channel";
    let iterations = 10;

    let context = Context::new()?;

    let channel_server = ChannelSocket::from_context(&context)?;
    channel_server.bind(endpoint)?;

    thread::spawn(move || {
        (1..=iterations)
            .try_for_each(|_| common::run_recv_send(&channel_server, "World"))
            .unwrap();
    });

    let channel_client = ChannelSocket::from_context(&context)?;
    channel_client.connect(endpoint)?;

    (0..iterations).try_for_each(|_| common::run_send_recv(&channel_client, "Hello"))
}
