#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::{
    ZmqResult,
    prelude::{Context, PullSocket, PushSocket, Receiver, SendFlags, Sender},
};
use tokio::{join, spawn};

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_subscriber(pull: PullSocket) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        if let Some(message) = pull.recv_msg_async().await {
            println!("Received message: {message:?}");
            ITERATIONS.fetch_sub(1, Ordering::Release);
        }
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

async fn run_publisher(push: PushSocket, msg: &str) {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        push.send_msg_async(msg, SendFlags::DONT_WAIT).await;
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let context = Context::new()?;

    let push = PushSocket::from_context(&context)?;
    push.bind("tcp://127.0.0.1:*")?;
    let pull_endpoint = push.last_endpoint()?;

    let pull = PullSocket::from_context(&context)?;
    pull.connect(pull_endpoint)?;

    let push_handle = spawn(run_publisher(push, "important update"));
    let pull_handle = spawn(run_subscriber(pull));

    let _ = join!(push_handle, pull_handle);

    Ok(())
}
