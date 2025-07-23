use core::sync::atomic::Ordering;
use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{Receiver, RecvFlags, SendFlags, Sender, XPublishSocket, XSubscribeSocket},
};

mod common;

use common::KEEP_RUNNING;

const SUBSCRIBED_TOPIC: &str = "arzmq-example";

fn run_xpublish_socket(xpublish: &XPublishSocket, msg: &str) -> ZmqResult<()> {
    let subscription = xpublish.recv_msg(RecvFlags::empty())?;
    let subscription_bytes = subscription.bytes();
    let (first_byte, subscription_topic) = (
        subscription_bytes[0],
        str::from_utf8(&subscription_bytes[1..]).unwrap(),
    );
    println!("{first_byte} {subscription_topic}");

    let published_msg = format!("{SUBSCRIBED_TOPIC} {msg}");
    xpublish.send_msg(&published_msg, SendFlags::empty())
}

fn main() -> ZmqResult<()> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let xpublish = XPublishSocket::from_context(&context)?;

    let xpublish_endpoint = format!("tcp://*:{port}");
    xpublish.bind(&xpublish_endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            run_xpublish_socket(&xpublish, "important update").unwrap();
        }
    });

    let xsubscribe = XSubscribeSocket::from_context(&context)?;

    let xsubscribe_endpoint = format!("tcp://localhost:{port}");
    xsubscribe.connect(&xsubscribe_endpoint)?;

    xsubscribe.subscribe(SUBSCRIBED_TOPIC)?;

    (0..iterations).try_for_each(|number| {
        common::run_subscribe_client(&xsubscribe, SUBSCRIBED_TOPIC)?;
        if number % 2 == 0 {
            xsubscribe.subscribe(format!("topic-{number}"))
        } else {
            xsubscribe.unsubscribe(format!("topic-{}", number - 1))
        }
    })?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
