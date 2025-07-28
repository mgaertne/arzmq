#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, DealerSocket, RouterSocket, ZmqResult};
use tokio::{join, task};

mod common;

use common::ITERATIONS;

async fn run_router(router: RouterSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 1 {
        common::run_multipart_recv_reply_async(&router, msg).await;
    }
}

async fn run_dealer_client(dealer: DealerSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        common::run_multipart_send_recv_async(&dealer, msg).await;
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let context = Context::new()?;

    let router = RouterSocket::from_context(&context)?;
    router.bind("tcp://127.0.0.1:*")?;
    let dealer_endpoint = router.last_endpoint()?;

    let dealer = DealerSocket::from_context(&context)?;
    dealer.connect(dealer_endpoint)?;

    let client_handle = task::spawn(run_dealer_client(dealer, "Hello"));
    let server_handle = task::spawn(run_router(router, "World"));

    let _ = join!(server_handle, client_handle);

    Ok(())
}
