#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::{
    ZmqResult,
    prelude::{Context, PullSocket, PushSocket, Receiver, SendFlags, Sender},
};
use futures::{executor::ThreadPool, join, task::SpawnExt};

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
        push.send_msg_async(msg, SendFlags::empty()).await;
    }
}

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let context = Context::new()?;

        let push = PushSocket::from_context(&context)?;
        push.bind("tcp://127.0.0.1:*")?;
        let pull_endpoint = push.last_endpoint()?;

        let pull = PullSocket::from_context(&context)?;
        pull.connect(pull_endpoint)?;

        let push_handle = executor
            .spawn_with_handle(run_publisher(push, "important update"))
            .unwrap();
        let pull_handle = executor.spawn_with_handle(run_subscriber(pull)).unwrap();

        let _ = join!(push_handle, pull_handle);

        Ok(())
    })
}
