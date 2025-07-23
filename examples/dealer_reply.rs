use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{DealerSocket, ReplySocket},
};

mod common;

fn main() -> ZmqResult<()> {
    let port = 5563;
    let iterations = 10;

    let context = Context::new()?;

    let reply = ReplySocket::from_context(&context)?;

    let reply_endpoint = format!("tcp://*:{port}");
    reply.bind(&reply_endpoint)?;

    thread::spawn(move || {
        (0..iterations)
            .try_for_each(|_| common::run_multipart_recv_reply(&reply, "World"))
            .unwrap();
    });

    let dealer = DealerSocket::from_context(&context)?;

    let dealer_endpoint = format!("tcp://localhost:{port}");
    dealer.connect(&dealer_endpoint)?;

    (0..iterations).try_for_each(|_| common::run_multipart_send_recv(&dealer, "Hello"))
}
