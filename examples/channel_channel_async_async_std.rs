#![cfg(feature = "examples-async-std")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{ChannelSocket, Context, ZmqResult};
use async_std::task;
use futures::join;

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_channel_server(channel: ChannelSocket, msg: &str) {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        common::run_recv_send_async(&channel, msg).await;
    }
}

async fn run_channel_client(channel: ChannelSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        common::run_send_recv_async(&channel, msg).await;
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

#[async_std::main]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let endpoint = "inproc://arzmq-example-channel";

    let context = Context::new()?;

    let channel_server = ChannelSocket::from_context(&context)?;
    channel_server.bind(endpoint)?;

    let channel_client = ChannelSocket::from_context(&context)?;
    channel_client.connect(endpoint)?;

    let channel_client_handle = task::spawn(run_channel_client(channel_client, "Hello"));
    let channel_server_handle = task::spawn(run_channel_server(channel_server, "World"));

    let _ = join!(channel_server_handle, channel_client_handle);

    Ok(())
}
