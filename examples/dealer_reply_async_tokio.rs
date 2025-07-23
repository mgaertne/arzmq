#![cfg(feature = "examples-tokio")]
use core::sync::atomic::{AtomicI32, Ordering};

use arzmq::{
    ZmqResult,
    context::Context,
    message::Message,
    socket::{DealerSocket, MultipartReceiver, MultipartSender, ReplySocket, SendFlags},
};
use tokio::{join, task};

static ITERATIONS: AtomicI32 = AtomicI32::new(0);

async fn run_replier(reply: ReplySocket) -> ZmqResult<()> {
    while ITERATIONS.load(Ordering::Acquire) > 1 {
        let mut multipart = reply.recv_multipart_async().await;
        let content = multipart.pop_back().unwrap();
        if !content.is_empty() {
            println!("Received request: {content}");
        }
        multipart.push_back("World".into());
        reply
            .send_multipart_async(multipart, SendFlags::empty())
            .await;
    }

    Ok(())
}

async fn run_dealer(dealer: DealerSocket) -> ZmqResult<()> {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        let request_no = ITERATIONS.load(Ordering::Acquire);
        println!("Sending request {request_no}");
        let multipart: Vec<Message> = vec![vec![].into(), "Hello".into()];
        let _ = dealer
            .send_multipart_async(multipart, SendFlags::empty())
            .await;

        let mut message = dealer.recv_multipart_async().await;
        let content = message.pop_back().unwrap();
        if !content.is_empty() {
            println!("Received reply {request_no}: {content}",);

            ITERATIONS.fetch_sub(1, Ordering::Release);
        }
    }

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let port = 5556;

    let context = Context::new()?;

    let reply = ReplySocket::from_context(&context)?;
    reply.bind(format!("tcp://*:{port}"))?;

    let dealer = DealerSocket::from_context(&context)?;
    dealer.connect(format!("tcp://localhost:{port}"))?;

    let dealer_handle = task::spawn(run_dealer(dealer));
    let reply_handle = task::spawn(run_replier(reply));

    let _ = join!(reply_handle, dealer_handle);

    Ok(())
}
