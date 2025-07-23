use core::sync::atomic::Ordering;
use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{PublishSocket, SubscribeSocket},
};

mod common;

use common::KEEP_RUNNING;

const SUBSCRIBED_TOPIC: &str = "arzmq-example";

fn main() -> ZmqResult<()> {
    let port = 5555;
    let iterations = 10;

    let context = Context::new()?;

    let publish = PublishSocket::from_context(&context)?;

    let publish_endpoint = format!("tcp://*:{port}");
    publish.bind(&publish_endpoint)?;

    thread::spawn(move || {
        let published_msg = format!("{SUBSCRIBED_TOPIC} important update");
        common::run_publisher(&publish, &published_msg).unwrap();
    });

    let subscribe = SubscribeSocket::from_context(&context)?;

    let subscribe_endpoint = format!("tcp://localhost:{port}");
    subscribe.connect(&subscribe_endpoint)?;

    subscribe.subscribe(SUBSCRIBED_TOPIC)?;

    (0..iterations).try_for_each(|number| {
        common::run_subscribe_client(&subscribe, SUBSCRIBED_TOPIC)?;

        subscribe.subscribe(format!("topic-{number}"))
    })?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
