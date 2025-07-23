use std::thread;

use arzmq::{ZmqResult, context::Context, socket::DealerSocket};

mod common;

fn main() -> ZmqResult<()> {
    let port = 5562;
    let iterations = 10;

    let context = Context::new()?;

    let dealer_server = DealerSocket::from_context(&context)?;

    let server_endpoint = format!("tcp://*:{port}");
    dealer_server.bind(&server_endpoint)?;

    thread::spawn(move || {
        (0..iterations)
            .try_for_each(|_| common::run_multipart_recv_reply(&dealer_server, "World"))
            .unwrap();
    });

    let dealer_client = DealerSocket::from_context(&context)?;

    let client_endpoint = format!("tcp://localhost:{port}");
    dealer_client.connect(&client_endpoint)?;

    (0..iterations).try_for_each(|_| common::run_multipart_send_recv(&dealer_client, "Hello"))
}
