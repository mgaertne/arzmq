use std::thread;

use arzmq::prelude::{Context, ReplySocket, RequestSocket, ZmqResult};

mod common;

fn main() -> ZmqResult<()> {
    let iterations = 10;

    let context = Context::new()?;

    let reply = ReplySocket::from_context(&context)?;
    reply.bind("tcp://127.0.0.1:*")?;
    let request_endpoint = reply.last_endpoint()?;

    thread::spawn(move || {
        (1..=iterations)
            .try_for_each(|_| common::run_recv_send(&reply, "World"))
            .unwrap();
    });

    let request = RequestSocket::from_context(&context)?;
    request.connect(request_endpoint)?;

    (0..iterations).try_for_each(|_| common::run_send_recv(&request, "Hello"))
}
