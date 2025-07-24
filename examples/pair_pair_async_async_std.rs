#![cfg(feature = "examples-async-std")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, PairSocket, ZmqResult};
use async_std::task;
use futures::join;

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_pair_server(pair: PairSocket, msg: &str) {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        common::run_recv_send_async(&pair, msg).await;
    }
}

async fn run_pair_client(pair: PairSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        common::run_send_recv_async(&pair, msg).await;
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

#[async_std::main]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let endpoint = "inproc://arzmq-example-pair";

    let context = Context::new()?;

    let pair_server = PairSocket::from_context(&context)?;
    pair_server.bind(endpoint)?;

    let pair_client = PairSocket::from_context(&context)?;
    pair_client.connect(endpoint)?;

    let pair_client_handle = task::spawn(run_pair_client(pair_client, "Hello"));
    let pair_server_handle = task::spawn(run_pair_server(pair_server, "World"));

    let _ = join!(pair_server_handle, pair_client_handle);

    Ok(())
}
