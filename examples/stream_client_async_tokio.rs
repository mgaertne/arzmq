#![cfg(feature = "examples-tokio")]
#[rustversion::since(1.87)]
use core::str;
use core::{error::Error, sync::atomic::Ordering};
#[rustversion::before(1.87)]
use std::str;

use arzmq::prelude::{
    Context, MultipartMessage, MultipartReceiver, MultipartSender, SendFlags, StreamSocket,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    join,
    net::TcpListener,
    spawn,
};

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

async fn run_tcp_server(listener: TcpListener, msg: &str) {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        let (mut tcp_stream, _socket_addr) = listener.accept().await.unwrap();
        tcp_stream.write_all("".as_bytes()).await.unwrap();
        loop {
            let mut buffer = [0; 256];
            if let Ok(length) = tcp_stream.read(&mut buffer).await {
                if length == 0 {
                    break;
                }
                let recevied_msg = &buffer[..length];
                println!(
                    "Received request: {}",
                    str::from_utf8(recevied_msg).unwrap()
                );
                tcp_stream.write_all(msg.as_bytes()).await.unwrap();
            }
        }
    }
}

async fn run_stream_socket(zmq_stream: StreamSocket, routing_id: Vec<u8>, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        let mut multipart = MultipartMessage::new();
        multipart.push_back(routing_id.clone().into());
        multipart.push_back(msg.into());
        zmq_stream
            .send_multipart_async(multipart, SendFlags::empty())
            .await;

        let mut message = zmq_stream.recv_multipart_async().await;
        println!("Received reply {:?}", message.pop_back().unwrap());

        ITERATIONS.fetch_sub(1, Ordering::Release);
    }

    KEEP_RUNNING.store(false, Ordering::Release);
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn Error>> {
    ITERATIONS.store(10, Ordering::Release);

    let port = 5558;

    let tcp_endpoint = format!("127.0.0.1:{port}");
    let tcp_listener = TcpListener::bind(tcp_endpoint).await?;

    let context = Context::new()?;

    let zmq_stream = StreamSocket::from_context(&context)?;

    let stream_endpoint = format!("tcp://127.0.0.1:{port}");
    zmq_stream.connect(stream_endpoint)?;

    let mut connect_msg = zmq_stream.recv_multipart_async().await;
    let routing_id = connect_msg.pop_front().unwrap();

    let zmq_stream_handle = spawn(run_stream_socket(zmq_stream, routing_id.bytes(), "Hello"));
    let tcp_handle = spawn(run_tcp_server(tcp_listener, "World"));

    let _ = join!(zmq_stream_handle, tcp_handle);

    Ok(())
}
