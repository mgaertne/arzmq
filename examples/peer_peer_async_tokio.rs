#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{Context, Message, PeerSocket, Receiver, SendFlags, Sender, ZmqResult};
use tokio::{join, task};

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_peer_server(peer: PeerSocket, msg: &str) -> ZmqResult<()> {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        if let Some(message) = peer.recv_msg_async().await {
            println!("Received request: {message:?}");

            let response: Message = msg.into();
            response.set_routing_id(message.routing_id().unwrap())?;
            peer.send_msg_async(response, SendFlags::empty()).await;
        }
    }

    Ok(())
}

async fn run_peer_client(peer: PeerSocket, routing_id: u32, msg: &str) -> ZmqResult<()> {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        println!("Sending message: {msg:?}");
        let message: Message = msg.into();
        message.set_routing_id(routing_id)?;
        let _ = peer.send_msg_async(message, SendFlags::empty()).await;

        loop {
            if let Some(message) = peer.recv_msg_async().await {
                println!("Received mesaage: {message:?}");

                ITERATIONS.fetch_sub(1, Ordering::Release);

                break;
            }
        }
    }
    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let endpoint = "inproc://arzmq-example-peer";

    let context = Context::new()?;

    let peer_server = PeerSocket::from_context(&context)?;
    peer_server.bind(endpoint)?;

    let peer_client = PeerSocket::from_context(&context)?;
    let routing_id = peer_client.connect_peer(endpoint)?;

    let peer_client_handle = task::spawn(run_peer_client(peer_client, routing_id, "Hello"));
    let peer_server_handle = task::spawn(run_peer_server(peer_server, "World"));

    let _ = join!(peer_server_handle, peer_client_handle);

    Ok(())
}
