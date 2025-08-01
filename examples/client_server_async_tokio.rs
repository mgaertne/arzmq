#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{
    ClientSocket, Context, Message, Receiver, SendFlags, Sender, ServerSocket, ZmqResult,
};
use tokio::{join, task};

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_server(server: ServerSocket, msg: &str) -> ZmqResult<()> {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        if let Some(message) = server.recv_msg_async().await {
            println!("Received request: {message:?}");

            let response: Message = msg.into();
            response.set_routing_id(message.routing_id().unwrap())?;
            server.send_msg_async(response, SendFlags::empty()).await;
        }
    }

    Ok(())
}

async fn run_client(peer: ClientSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        println!("Sending message: {msg:?}");
        let _ = peer.send_msg_async(msg, SendFlags::empty()).await;

        loop {
            if let Some(message) = peer.recv_msg_async().await {
                println!("Received mesaage: {message:?}");

                ITERATIONS.fetch_sub(1, Ordering::Release);

                break;
            }
        }
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let context = Context::new()?;

    let server = ServerSocket::from_context(&context)?;
    server.bind("tcp://127.0.0.1:*")?;
    let client_endpoint = server.last_endpoint()?;

    let client = ClientSocket::from_context(&context)?;
    client.connect(client_endpoint)?;

    let client_handle = task::spawn(run_client(client, "Hello"));
    let server_handle = task::spawn(run_server(server, "World"));

    let _ = join!(server_handle, client_handle);

    Ok(())
}
