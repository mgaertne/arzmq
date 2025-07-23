use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{DealerSocket, RouterSocket},
};

mod common;

fn main() -> ZmqResult<()> {
    let port = 5564;
    let iterations = 10;

    let context = Context::new()?;

    let router = RouterSocket::from_context(&context)?;
    let router_endpoint = format!("tcp://*:{port}");
    router.bind(&router_endpoint)?;

    thread::spawn(move || {
        (0..iterations)
            .try_for_each(|_| common::run_multipart_recv_reply(&router, "World"))
            .unwrap();
    });

    let dealer = DealerSocket::from_context(&context)?;

    let dealer_endpoint = format!("tcp://localhost:{port}");
    dealer.connect(&dealer_endpoint)?;

    (0..iterations).try_for_each(|_| common::run_multipart_send_recv(&dealer, "Hello"))
}
