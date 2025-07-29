use core::sync::atomic::Ordering;
use std::thread;

use arzmq::prelude::{Context, PublishSocket, XSubscribeSocket, ZmqResult};

mod common;

use common::KEEP_RUNNING;

const SUBSCRIBED_TOPIC: &str = "arzmq-example";

fn main() -> ZmqResult<()> {
    let iterations = 10;

    let context = Context::new()?;

    let publish = PublishSocket::from_context(&context)?;
    publish.bind("tcp://127.0.0.1:*")?;
    let xsubscribe_endpoint = publish.last_endpoint()?;

    thread::spawn(move || {
        let published_msg = format!("{SUBSCRIBED_TOPIC} important update");
        common::run_publisher(&publish, &published_msg).unwrap();
    });

    let xsubscribe = XSubscribeSocket::from_context(&context)?;
    xsubscribe.connect(xsubscribe_endpoint)?;
    xsubscribe.subscribe(SUBSCRIBED_TOPIC)?;

    (0..iterations).try_for_each(|number| {
        common::run_subscribe_client(&xsubscribe, SUBSCRIBED_TOPIC)?;
        xsubscribe.subscribe(format!("topic-{number}"))
    })?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
