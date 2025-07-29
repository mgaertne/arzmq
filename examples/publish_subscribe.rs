use core::sync::atomic::Ordering;
use std::thread;

use arzmq::prelude::{Context, PublishSocket, SubscribeSocket, ZmqResult};

mod common;

use common::KEEP_RUNNING;

const SUBSCRIBED_TOPIC: &str = "arzmq-example";

fn main() -> ZmqResult<()> {
    let iterations = 10;

    let context = Context::new()?;

    let publish = PublishSocket::from_context(&context)?;
    publish.bind("tcp://127.0.0.1:*")?;
    let subscribe_endpoint = publish.last_endpoint()?;

    thread::spawn(move || {
        let published_msg = format!("{SUBSCRIBED_TOPIC} important update");
        common::run_publisher(&publish, &published_msg).unwrap();
    });

    let subscribe = SubscribeSocket::from_context(&context)?;
    subscribe.connect(subscribe_endpoint)?;
    subscribe.subscribe(SUBSCRIBED_TOPIC)?;

    (0..iterations).try_for_each(|number| {
        common::run_subscribe_client(&subscribe, SUBSCRIBED_TOPIC)?;

        subscribe.subscribe(format!("topic-{number}"))
    })?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
