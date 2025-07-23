use core::sync::atomic::Ordering;
use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    message::Message,
    socket::{ClientSocket, Receiver, RecvFlags, SendFlags, Sender, ServerSocket},
};

mod common;

use common::KEEP_RUNNING;

fn run_server_socket(server: &ServerSocket, reply: &str) -> ZmqResult<()> {
    let message = server.recv_msg(RecvFlags::empty())?;
    println!("Received message: \"{message:?}\"");

    let returned: Message = reply.into();
    returned.set_routing_id(message.routing_id().unwrap())?;
    server.send_msg(returned, SendFlags::empty())
}

fn main() -> ZmqResult<()> {
    let port = 5678;
    let iterations = 10;

    let context = Context::new()?;

    let server = ServerSocket::from_context(&context)?;

    let server_endpoint = format!("tcp://*:{port}");
    server.bind(&server_endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            run_server_socket(&server, "World").unwrap();
        }
    });

    let client = ClientSocket::from_context(&context)?;

    let client_endpoint = format!("tcp://localhost:{port}");
    client.connect(&client_endpoint)?;

    (0..iterations).try_for_each(|_| common::run_send_recv(&client, "Hello"))?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
