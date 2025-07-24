#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, DealerSocket, ZmqResult};
use futures::{executor::ThreadPool, join, task::SpawnExt};

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

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let port = 5556;

        let context = Context::new()?;

        let dealer_server = DealerSocket::from_context(&context)?;
        dealer_server.bind(format!("tcp://*:{port}"))?;

        let dealer_client = DealerSocket::from_context(&context)?;
        dealer_client.connect(format!("tcp://localhost:{port}"))?;

        let dealer_handle = executor
            .spawn_with_handle(run_dealer_client(dealer_client, "Hello"))
            .unwrap();
        let reply_handle = executor
            .spawn_with_handle(run_dealer_server(dealer_server, "World"))
            .unwrap();

        let _ = join!(reply_handle, dealer_handle);

        Ok(())
    })
}
