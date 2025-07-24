#![cfg(feature = "examples-async-std")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, SendFlags, Sender, XPublishSocket, XSubscribeSocket, ZmqResult};
use async_std::task::spawn;
use futures::join;

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_subscriber(subscribe: XSubscribeSocket, subscribed_topic: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        common::run_subscribe_client_async(&subscribe, subscribed_topic).await;
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

async fn run_publisher(publisher: XPublishSocket, msg: &str) {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        publisher.send_msg_async(msg, SendFlags::empty()).await;
    }
}

#[async_std::main]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let port = 5556;

    let context = Context::new()?;

    let publisher = XPublishSocket::from_context(&context)?;
    publisher.bind(format!("tcp://*:{port}"))?;

    let xsubscriber = XSubscribeSocket::from_context(&context)?;
    xsubscriber.subscribe("arzmq-example")?;
    xsubscriber.connect(format!("tcp://localhost:{port}"))?;

    let publish_handle = spawn(run_publisher(publisher, "arzmq-example important update"));
    let subscribe_handle = spawn(run_subscriber(xsubscriber, "arzmq-example"));

    let _ = join!(publish_handle, subscribe_handle);

    Ok(())
}
