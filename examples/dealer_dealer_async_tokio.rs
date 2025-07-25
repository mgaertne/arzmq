#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, DealerSocket, ZmqResult};
use tokio::{join, task};

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

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let port = 5562;

    let context = Context::new()?;

    let dealer_server = DealerSocket::from_context(&context)?;
    dealer_server.bind(format!("tcp://*:{port}"))?;

    let dealer_client = DealerSocket::from_context(&context)?;
    dealer_client.connect(format!("tcp://localhost:{port}"))?;

    let client_handle = task::spawn(run_dealer_client(dealer_client, "Hello"));
    let server_handle = task::spawn(run_dealer_server(dealer_server, "World"));

    let _ = join!(server_handle, client_handle);

    Ok(())
}
