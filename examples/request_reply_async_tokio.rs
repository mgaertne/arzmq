#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, ReplySocket, RequestSocket, ZmqResult};
use tokio::{join, task};

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

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let port = 5560;

    let context = Context::new()?;

    let reply = ReplySocket::from_context(&context)?;
    reply.bind(format!("tcp://*:{port}"))?;

    let request = RequestSocket::from_context(&context)?;
    request.connect(format!("tcp://localhost:{port}"))?;

    let request_handle = task::spawn(run_requester(request, "Hello"));
    let reply_handle = task::spawn(run_replier(reply, "World"));

    let _ = join!(reply_handle, request_handle);

    Ok(())
}
