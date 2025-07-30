#![cfg(feature = "examples-async-std")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, PublishSocket, SendFlags, Sender, SubscribeSocket, ZmqResult};
use async_std::task::spawn;
use futures::join;

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_subscriber(subscribe: SubscribeSocket, subscribed_topic: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        common::run_subscribe_client_async(&subscribe, subscribed_topic).await;
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

async fn run_publisher(publisher: PublishSocket, msg: &str) {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        publisher.send_msg_async(msg, SendFlags::DONT_WAIT).await;
    }
}

#[async_std::main]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let context = Context::new()?;

    let publish = PublishSocket::from_context(&context)?;
    publish.bind("tcp://127.0.0.1:*")?;
    let subscribe_endpoint = publish.last_endpoint()?;

    let subscrib = SubscribeSocket::from_context(&context)?;
    subscrib.subscribe("arzmq-example")?;
    subscrib.connect(subscribe_endpoint)?;

    let publish_handle = spawn(run_publisher(publish, "arzmq-example important update"));
    let subscribe_handle = spawn(run_subscriber(subscrib, "arzmq-example"));

    let _ = join!(publish_handle, subscribe_handle);

    Ok(())
}
