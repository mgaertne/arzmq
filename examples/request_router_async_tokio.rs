#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, RequestSocket, RouterSocket, ZmqResult};
use tokio::{join, task};

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_router(router: RouterSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 1 {
        common::run_multipart_recv_reply_async(&router, msg).await;
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

    let port = 5561;

    let context = Context::new()?;

    let router = RouterSocket::from_context(&context)?;
    router.bind(format!("tcp://*:{port}"))?;

    let request = RequestSocket::from_context(&context)?;
    request.connect(format!("tcp://localhost:{port}"))?;

    let request_handle = task::spawn(run_requester(request, "Hello"));
    let router_handle = task::spawn(run_router(router, "World"));

    let _ = join!(router_handle, request_handle);

    Ok(())
}
