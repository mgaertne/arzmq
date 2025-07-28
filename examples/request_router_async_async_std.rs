#![cfg(feature = "examples-async-std")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, RequestSocket, RouterSocket, ZmqResult};
use async_std::task;
use futures::join;

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_router(router: RouterSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 1 {
        common::run_multipart_recv_reply_async(&router, msg).await;
    }
}

async fn run_requester(request: RequestSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        common::run_multipart_send_recv_async(&request, msg).await;
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

#[async_std::main]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let context = Context::new()?;

    let router = RouterSocket::from_context(&context)?;
    router.bind("tcp://127.0.0.1:*")?;
    let request_endpoint = router.last_endpoint()?;

    let request = RequestSocket::from_context(&context)?;
    request.connect(request_endpoint)?;

    let request_handle = task::spawn(run_requester(request, "Hello"));
    let reply_handle = task::spawn(run_router(router, "World"));

    let _ = join!(reply_handle, request_handle);

    Ok(())
}
