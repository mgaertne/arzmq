use core::sync::atomic::Ordering;
use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{PublishSocket, XSubscribeSocket},
};

mod common;

use common::KEEP_RUNNING;

const SUBSCRIBED_TOPIC: &str = "arzmq-example";

fn main() -> ZmqResult<()> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let publish = PublishSocket::from_context(&context)?;

    let publish_endpoint = format!("tcp://*:{port}");
    publish.bind(&publish_endpoint)?;

    thread::spawn(move || {
        let published_msg = format!("{SUBSCRIBED_TOPIC} important update");
        common::run_publisher(&publish, &published_msg).unwrap();
    });

    let xsubscribe = XSubscribeSocket::from_context(&context)?;

    let xsubscribe_endpoint = format!("tcp://localhost:{port}");
    xsubscribe.connect(&xsubscribe_endpoint)?;

    xsubscribe.subscribe(SUBSCRIBED_TOPIC)?;

    (0..iterations).try_for_each(|number| {
        common::run_subscribe_client(&xsubscribe, SUBSCRIBED_TOPIC)?;
        xsubscribe.subscribe(format!("topic-{number}"))
    })?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
