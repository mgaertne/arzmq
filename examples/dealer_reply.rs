use std::thread;

use arzmq::prelude::{Context, DealerSocket, ReplySocket, ZmqResult};

mod common;

fn main() -> ZmqResult<()> {
    let iterations = 10;

    let context = Context::new()?;

    let reply = ReplySocket::from_context(&context)?;
    reply.bind("tcp://127.0.0.1:*")?;
    let dealer_endpoint = reply.last_endpoint()?;

    thread::spawn(move || {
        (0..iterations)
            .try_for_each(|_| common::run_multipart_recv_reply(&reply, "World"))
            .unwrap();
    });

    let dealer = DealerSocket::from_context(&context)?;
    dealer.connect(&dealer_endpoint)?;

    (0..iterations).try_for_each(|_| common::run_multipart_send_recv(&dealer, "Hello"))
}
