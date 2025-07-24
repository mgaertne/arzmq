#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, PublishSocket, SendFlags, Sender, XSubscribeSocket, ZmqResult};
use futures::{executor::ThreadPool, join, task::SpawnExt};

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_subscriber(subscribe: XSubscribeSocket, subscribed_topic: &str) {
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

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let port = 5556;

        let context = Context::new()?;

        let publisher = PublishSocket::from_context(&context)?;
        publisher.bind(format!("tcp://*:{port}"))?;

        let xsubscribe = XSubscribeSocket::from_context(&context)?;
        xsubscribe.subscribe("arzmq-example")?;
        xsubscribe.connect(format!("tcp://localhost:{port}"))?;

        let publish_handle = executor
            .spawn_with_handle(run_publisher(publisher, "arzmq-example important update"))
            .unwrap();
        let subscribe_handle = executor
            .spawn_with_handle(run_subscriber(xsubscribe, "arzmq-example"))
            .unwrap();

        let _ = join!(publish_handle, subscribe_handle);

        Ok(())
    })
}
