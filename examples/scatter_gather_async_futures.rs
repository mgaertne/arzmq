#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::{
    ZmqResult,
    prelude::{Context, GatherSocket, Receiver, ScatterSocket, SendFlags, Sender},
};
use futures::{executor::ThreadPool, join, task::SpawnExt};

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_gather(gather: GatherSocket) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        if let Some(message) = gather.recv_msg_async().await {
            println!("Received message: {message:?}");
            ITERATIONS.fetch_sub(1, Ordering::Release);
        }
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

async fn run_scatter(scatter: ScatterSocket, msg: &str) {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        scatter.send_msg_async(msg, SendFlags::empty()).await;
    }
}

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let context = Context::new()?;

        let scatter = ScatterSocket::from_context(&context)?;
        scatter.bind("tcp://127.0.0.1:*")?;
        let gather_endpoint = scatter.last_endpoint()?;

        let gather = GatherSocket::from_context(&context)?;
        gather.connect(gather_endpoint)?;

        let scatter_handle = executor
            .spawn_with_handle(run_scatter(scatter, "important update"))
            .unwrap();
        let gather_handle = executor.spawn_with_handle(run_gather(gather)).unwrap();

        let _ = join!(scatter_handle, gather_handle);

        Ok(())
    })
}
