#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, DealerSocket, RouterSocket, ZmqResult};
use futures::{executor::ThreadPool, join, task::SpawnExt};

mod common;

use common::ITERATIONS;

async fn run_router(router: RouterSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 1 {
        common::run_multipart_recv_reply_async(&router, msg).await;
    }
}

async fn run_dealer(dealer: DealerSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        common::run_multipart_send_recv_async(&dealer, msg).await;
    }
}

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let context = Context::new()?;

        let router = RouterSocket::from_context(&context)?;
        router.bind("tcp://127.0.0.1:*")?;
        let dealer_endpoint = router.last_endpoint()?;

        let dealer = DealerSocket::from_context(&context)?;
        dealer.connect(dealer_endpoint)?;

        let dealer_handle = executor
            .spawn_with_handle(run_dealer(dealer, "Hello"))
            .unwrap();
        let reply_handle = executor
            .spawn_with_handle(run_router(router, "World"))
            .unwrap();

        let _ = join!(reply_handle, dealer_handle);

        Ok(())
    })
}
