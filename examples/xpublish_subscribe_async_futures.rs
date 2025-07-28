#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, SendFlags, Sender, SubscribeSocket, XPublishSocket, ZmqResult};
use futures::{executor::ThreadPool, join, task::SpawnExt};

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
        publisher.send_msg_async(msg, SendFlags::empty()).await;
    }
}

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let context = Context::new()?;

        let publish = XPublishSocket::from_context(&context)?;
        publish.bind("tcp://127.0.0.1:*")?;
        let xsubscribe_endpoint = publish.last_endpoint()?;

        let xsubscribe = SubscribeSocket::from_context(&context)?;
        xsubscribe.subscribe("arzmq-example")?;
        xsubscribe.connect(xsubscribe_endpoint)?;

        let publish_handle = executor
            .spawn_with_handle(run_publisher(publish, "arzmq-example important update"))
            .unwrap();
        let subscribe_handle = executor
            .spawn_with_handle(run_subscriber(xsubscribe, "arzmq-example"))
            .unwrap();

        let _ = join!(publish_handle, subscribe_handle);

        Ok(())
    })
}
