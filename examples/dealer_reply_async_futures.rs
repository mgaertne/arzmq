#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, DealerSocket, ReplySocket, ZmqResult};
use futures::{executor::ThreadPool, join, task::SpawnExt};

mod common;

use common::ITERATIONS;

async fn run_replier(reply: ReplySocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 1 {
        common::run_multipart_recv_reply_async(&reply, msg).await;
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

        let port = 5563;

        let context = Context::new()?;

        let reply = ReplySocket::from_context(&context)?;
        reply.bind(format!("tcp://*:{port}"))?;

        let dealer = DealerSocket::from_context(&context)?;
        dealer.connect(format!("tcp://localhost:{port}"))?;

        let dealer_handle = executor
            .spawn_with_handle(run_dealer(dealer, "Hello"))
            .unwrap();
        let reply_handle = executor
            .spawn_with_handle(run_replier(reply, "World"))
            .unwrap();

        let _ = join!(reply_handle, dealer_handle);

        Ok(())
    })
}
