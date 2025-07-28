use std::thread;

use arzmq::prelude::{Context, RequestSocket, RouterSocket, ZmqResult};

mod common;

fn main() -> ZmqResult<()> {
    let iterations = 10;

    let context = Context::new()?;

    let router = RouterSocket::from_context(&context)?;
    router.bind("tcp://127.0.0.1:*")?;
    let request_endpoint = router.last_endpoint()?;

    thread::spawn(move || {
        (0..iterations)
            .try_for_each(|_| common::run_multipart_recv_reply(&router, "World"))
            .unwrap();
    });

    let request = RequestSocket::from_context(&context)?;
    request.connect(&request_endpoint)?;

    (0..iterations).try_for_each(|_| common::run_send_recv(&request, "Hello"))
}
