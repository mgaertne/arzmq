#![cfg(feature = "examples-async-std")]
use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{Receiver, ReplySocket, RequestSocket, SendFlags, Sender},
};
use async_std::task;
use futures::join;

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
static ITERATIONS: AtomicI32 = AtomicI32::new(0);

async fn run_replier(reply: ReplySocket) -> ZmqResult<()> {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        if let Some(message) = reply.recv_msg_async().await {
            println!("Received request: {message}");
            reply.send_msg_async("World", SendFlags::empty()).await;
        }
    }

    Ok(())
}

async fn run_requester(request: RequestSocket) -> ZmqResult<()> {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        let request_no = ITERATIONS.load(Ordering::Acquire);
        println!("Sending request {request_no}");
        let _ = request.send_msg_async("Hello", SendFlags::empty()).await;

        loop {
            if let Some(message) = request.recv_msg_async().await {
                println!("Received reply {request_no}: {message}");

                ITERATIONS.fetch_sub(1, Ordering::Release);

                break;
            }
        }
    }

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}

#[async_std::main]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let port = 5556;

    let context = Context::new()?;

    let reply = ReplySocket::from_context(&context)?;
    reply.bind(format!("tcp://*:{port}"))?;

    let request = RequestSocket::from_context(&context)?;
    request.connect(format!("tcp://localhost:{port}"))?;

    let request_handle = task::spawn(run_requester(request));
    let reply_handle = task::spawn(run_replier(reply));

    let _ = join!(reply_handle, request_handle);

    Ok(())
}
