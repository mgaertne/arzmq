use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{ReplySocket, RequestSocket},
};

mod common;

fn main() -> ZmqResult<()> {
    let port = 5560;
    let iterations = 10;

    let context = Context::new()?;

    let reply = ReplySocket::from_context(&context)?;

    let reply_endpoint = format!("tcp://*:{port}");
    reply.bind(&reply_endpoint)?;

    thread::spawn(move || {
        (1..=iterations)
            .try_for_each(|_| common::run_recv_send(&reply, "World"))
            .unwrap();
    });

    let request = RequestSocket::from_context(&context)?;

    let request_endpoint = format!("tcp://localhost:{port}");
    request.connect(&request_endpoint)?;

    (0..iterations).try_for_each(|_| common::run_send_recv(&request, "Hello"))
}
