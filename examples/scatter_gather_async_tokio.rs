#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::{
    ZmqResult,
    prelude::{Context, GatherSocket, Receiver, ScatterSocket, SendFlags, Sender},
};
use tokio::{join, spawn};

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
        scatter.send_msg_async(msg, SendFlags::DONT_WAIT).await;
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let context = Context::new()?;

    let scatter = ScatterSocket::from_context(&context)?;
    scatter.bind("tcp://127.0.0.1:*")?;
    let gather_endpoint = scatter.last_endpoint()?;

    let gather = GatherSocket::from_context(&context)?;
    gather.connect(gather_endpoint)?;

    let scatter_handle = spawn(run_scatter(scatter, "important update"));
    let gather_handle = spawn(run_gather(gather));

    let _ = join!(scatter_handle, gather_handle);

    Ok(())
}
