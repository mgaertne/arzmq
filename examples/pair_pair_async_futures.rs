#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, PairSocket, ZmqResult};
use futures::{executor::ThreadPool, join, task::SpawnExt};

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

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let endpoint = "inproc://arzmq-example-pair";

        let context = Context::new()?;

        let pair_server = PairSocket::from_context(&context)?;
        pair_server.bind(endpoint)?;

        let pair_client = PairSocket::from_context(&context)?;
        pair_client.connect(endpoint)?;

        let pair_client_handle = executor
            .spawn_with_handle(run_pair_client(pair_client, "Hello"))
            .unwrap();
        let pair_server_handle = executor
            .spawn_with_handle(run_pair_server(pair_server, "World"))
            .unwrap();

        let _ = join!(pair_server_handle, pair_client_handle);

        Ok(())
    })
}
