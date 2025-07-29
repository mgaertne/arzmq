#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, ReplySocket, RequestSocket, ZmqResult};
use futures::{executor::ThreadPool, join, task::SpawnExt};

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_replier(reply: ReplySocket, msg: &str) {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        common::run_recv_send_async(&reply, msg).await;
    }
}

async fn run_requester(request: RequestSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        common::run_send_recv_async(&request, msg).await;
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let context = Context::new()?;

        let reply = ReplySocket::from_context(&context)?;
        reply.bind("tcp://127.0.0.1:*")?;
        let request_endpoint = reply.last_endpoint()?;

        let request = RequestSocket::from_context(&context)?;
        request.connect(request_endpoint)?;

        let request_handle = executor
            .spawn_with_handle(run_requester(request, "Hello"))
            .unwrap();
        let reply_handle = executor
            .spawn_with_handle(run_replier(reply, "World"))
            .unwrap();

        let _ = join!(reply_handle, request_handle);

        Ok(())
    })
}
