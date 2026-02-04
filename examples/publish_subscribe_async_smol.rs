#![cfg(feature = "examples-smol")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, PublishSocket, SendFlags, Sender, SubscribeSocket, ZmqResult};
use futures::join;
use macro_rules_attribute::apply;
use smol_macros::{Executor, main};

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

#[apply(main!)]
async fn main(executor: &Executor<'_>) -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let context = Context::new()?;

    let publish = PublishSocket::from_context(&context)?;
    publish.bind("tcp://127.0.0.1:*")?;
    let subscribe_endpoint = publish.last_endpoint()?;

    let subscrib = SubscribeSocket::from_context(&context)?;
    subscrib.subscribe("arzmq-example")?;
    subscrib.connect(subscribe_endpoint)?;

    let publish_handle = executor.spawn(run_publisher(publish, "arzmq-example important update"));
    let subscribe_handle = executor.spawn(run_subscriber(subscrib, "arzmq-example"));

    let _ = join!(publish_handle, subscribe_handle);

    Ok(())
}
