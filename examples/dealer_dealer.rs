use std::thread;

use arzmq::prelude::{Context, DealerSocket, ZmqResult};

mod common;

fn main() -> ZmqResult<()> {
    let iterations = 10;

    let context = Context::new()?;

    let dealer_server = DealerSocket::from_context(&context)?;
    dealer_server.bind("tcp://127.0.0.1:*")?;
    let client_endpoint = dealer_server.last_endpoint()?;

    thread::spawn(move || {
        (0..iterations)
            .try_for_each(|_| common::run_multipart_recv_reply(&dealer_server, "World"))
            .unwrap();
    });

    let dealer_client = DealerSocket::from_context(&context)?;
    dealer_client.connect(client_endpoint)?;

    (0..iterations).try_for_each(|_| common::run_multipart_send_recv(&dealer_client, "Hello"))
}
