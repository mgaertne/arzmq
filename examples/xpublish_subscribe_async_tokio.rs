#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, SendFlags, Sender, SubscribeSocket, XPublishSocket, ZmqResult};
use tokio::{join, task::spawn};

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_subscriber(subscribe: SubscribeSocket, subscribed_topic: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        common::run_subscribe_client_async(&subscribe, subscribed_topic).await;
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

async fn run_publisher(publisher: XPublishSocket, msg: &str) {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        publisher.send_msg_async(msg, SendFlags::DONT_WAIT).await;
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let context = Context::new()?;

    let publish = XPublishSocket::from_context(&context)?;
    publish.bind("tcp://127.0.0.1:*")?;
    let xsubscribe_endpoint = publish.last_endpoint()?;

    let xsubscribe = SubscribeSocket::from_context(&context)?;
    xsubscribe.subscribe("arzmq-example")?;
    xsubscribe.connect(xsubscribe_endpoint)?;

    let publish_handle = spawn(run_publisher(publish, "arzmq-example important update"));
    let subscribe_handle = spawn(run_subscriber(xsubscribe, "arzmq-example"));

    let _ = join!(publish_handle, subscribe_handle);

    Ok(())
}
