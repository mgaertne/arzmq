#![cfg(feature = "examples-tokio")]
use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{PublishSocket, Receiver, SendFlags, Sender, SubscribeSocket},
};
use tokio::{join, task::spawn};

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
static ITERATIONS: AtomicI32 = AtomicI32::new(0);

async fn run_subscriber(subscribe: SubscribeSocket) -> ZmqResult<()> {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        if let Some(zmq_msg) = subscribe.recv_msg_async().await {
            let zmq_str = zmq_msg.to_string();
            let pubsub_item = zmq_str.split_once(" ");
            assert_eq!(Some(("arzmq-example", "important update")), pubsub_item);

            let (topic, item) = pubsub_item.unwrap();
            println!("Received msg for topic {topic:?}: {item}",);

            ITERATIONS.fetch_sub(1, Ordering::Release);
        };
    }

    KEEP_RUNNING.store(false, Ordering::Release);
    Ok(())
}

async fn run_publisher(publisher: PublishSocket) -> ZmqResult<()> {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        publisher
            .send_msg_async("arzmq-example important update", SendFlags::empty())
            .await;
    }

    Ok(())
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

    let publish_handle = spawn(run_publisher(publisher));
    let subscribe_handle = spawn(run_subscriber(subscriber));

    let _ = join!(publish_handle, subscribe_handle);

    Ok(())
}
