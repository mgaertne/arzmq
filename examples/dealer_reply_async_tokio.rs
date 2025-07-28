#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, DealerSocket, ReplySocket, ZmqResult};
use tokio::{join, task};

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

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let context = Context::new()?;

    let reply = ReplySocket::from_context(&context)?;
    reply.bind("tcp://127.0.0.1:*")?;
    let dealer_endpoint = reply.last_endpoint()?;

    let dealer = DealerSocket::from_context(&context)?;
    dealer.connect(dealer_endpoint)?;

    let dealer_handle = task::spawn(run_dealer(dealer, "Hello"));
    let reply_handle = task::spawn(run_replier(reply, "World"));

    let _ = join!(reply_handle, dealer_handle);

    Ok(())
}
