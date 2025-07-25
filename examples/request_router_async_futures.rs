#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, RequestSocket, RouterSocket, ZmqResult};
use futures::{executor::ThreadPool, join, task::SpawnExt};

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

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let port = 5561;

        let context = Context::new()?;

        let router = RouterSocket::from_context(&context)?;
        router.bind(format!("tcp://*:{port}"))?;

        let request = RequestSocket::from_context(&context)?;
        request.connect(format!("tcp://localhost:{port}"))?;

        let request_handle = executor
            .spawn_with_handle(run_requester(request, "Hello"))
            .unwrap();
        let router_handle = executor
            .spawn_with_handle(run_router(router, "World"))
            .unwrap();

        let _ = join!(router_handle, request_handle);

        Ok(())
    })
}
