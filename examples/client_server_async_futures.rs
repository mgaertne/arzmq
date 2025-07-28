#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::{
    message::Message,
    prelude::{ClientSocket, Context, Receiver, SendFlags, Sender, ServerSocket, ZmqResult},
};
use futures::{executor::ThreadPool, join, task::SpawnExt};

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

async fn run_client(client: ClientSocket, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        println!("Sending message: {msg:?}");
        let _ = client.send_msg_async(msg, SendFlags::empty()).await;

        loop {
            if let Some(message) = client.recv_msg_async().await {
                println!("Received mesaage: {message:?}");

                ITERATIONS.fetch_sub(1, Ordering::Release);

                break;
            }
        }
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let context = Context::new()?;

        let server = ServerSocket::from_context(&context)?;
        server.bind("tcp://127.0.0.1:*")?;
        let client_endpoint = server.last_endpoint()?;

        let client = ClientSocket::from_context(&context)?;
        client.connect(&client_endpoint)?;

        let client_handle = executor
            .spawn_with_handle(run_client(client, "Hello"))
            .unwrap();
        let server_handle = executor
            .spawn_with_handle(run_server(server, "World"))
            .unwrap();

        let _ = join!(server_handle, client_handle);

        Ok(())
    })
}
