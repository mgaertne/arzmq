#![cfg(feature = "examples-async-std")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, DealerSocket, ZmqResult};
use async_std::task;
use futures::join;

mod common;

use common::ITERATIONS;

async fn run_dealer_server(dealer: DealerSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 1 {
        common::run_multipart_recv_reply_async(&dealer, msg).await;
    }
}

async fn run_dealer_client(dealer: DealerSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        common::run_multipart_send_recv_async(&dealer, msg).await;
    }
}

#[async_std::main]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let context = Context::new()?;

    let dealer_server = DealerSocket::from_context(&context)?;
    dealer_server.bind("tcp://127.0.0.1:*")?;
    let client_endpoint = dealer_server.last_endpoint()?;

    let dealer_client = DealerSocket::from_context(&context)?;
    dealer_client.connect(&client_endpoint)?;

    let dealer_handle = task::spawn(run_dealer_client(dealer_client, "Hello"));
    let reply_handle = task::spawn(run_dealer_server(dealer_server, "World"));

    let _ = join!(reply_handle, dealer_handle);

    Ok(())
}
