use core::{error::Error, sync::atomic::Ordering};
use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{Receiver, RecvFlags, SendFlags, Sender, SubscribeSocket, XPublishSocket},
};

mod common;

use common::KEEP_RUNNING;

const SUBSCRIBED_TOPIC: &str = "arzmq-example";

fn run_xpublish_socket(context: &Context, endpoint: &str) -> ZmqResult<()> {
    let xpublish = XPublishSocket::from_context(context)?;
    xpublish.bind(endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            let subscription = xpublish.recv_msg(RecvFlags::empty()).unwrap();
            let subscription_bytes = subscription.bytes();
            let (first_byte, subscription_topic) = (
                subscription_bytes[0],
                str::from_utf8(&subscription_bytes[1..]).unwrap(),
            );
            assert_eq!(first_byte, 1);
            println!("{first_byte} {subscription_topic}");

            let published_msg = format!("{SUBSCRIBED_TOPIC} important update");
            xpublish
                .send_msg(&published_msg, SendFlags::empty())
                .unwrap();
        }
    });

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let xpublish_endpoint = format!("tcp://*:{port}");
    run_xpublish_socket(&context, &xpublish_endpoint)?;

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
