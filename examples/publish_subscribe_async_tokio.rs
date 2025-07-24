#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, PublishSocket, SendFlags, Sender, SubscribeSocket, ZmqResult};
use tokio::{join, task::spawn};

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
        publisher.send_msg_async(msg, SendFlags::empty()).await;
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let port = 5556;

    let context = Context::new()?;

    let publisher = PublishSocket::from_context(&context)?;
    publisher.bind(format!("tcp://*:{port}"))?;

    let subscriber = SubscribeSocket::from_context(&context)?;
    subscriber.subscribe("arzmq-example")?;
    subscriber.connect(format!("tcp://localhost:{port}"))?;

    let publish_handle = spawn(run_publisher(publisher, "arzmq-example important update"));
    let subscribe_handle = spawn(run_subscriber(subscriber, "arzmq-example"));

    let _ = join!(publish_handle, subscribe_handle);

    Ok(())
}
