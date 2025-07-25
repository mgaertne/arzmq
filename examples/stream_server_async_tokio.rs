#![cfg(feature = "examples-tokio")]
use core::{error::Error, sync::atomic::Ordering};

use arzmq::prelude::{Context, MultipartReceiver, StreamSocket};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    join,
    net::TcpStream,
    spawn,
};

mod common;

use common::ITERATIONS;

async fn run_stream_server(zmq_stream: StreamSocket, msg: &str) {
    let mut connect_msg = zmq_stream.recv_multipart_async().await;
    let _routing_id = connect_msg.pop_front().unwrap();

    while ITERATIONS.load(Ordering::Acquire) > 1 {
        common::run_multipart_recv_reply_async(&zmq_stream, msg).await;
    }
}

async fn run_tcp_client(mut tcp_stream: TcpStream, msg: &str) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        tcp_stream.write_all(msg.as_bytes()).await.unwrap();

        let mut buffer = [0; 256];
        if let Ok(length) = tcp_stream.read(&mut buffer).await
            && length != 0
        {
            let recevied_msg = &buffer[..length];
            println!(
                "Received reply. {:?}",
                str::from_utf8(recevied_msg).unwrap()
            );

            ITERATIONS.fetch_sub(1, Ordering::Release);
        }
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn Error>> {
    ITERATIONS.store(10, Ordering::Release);

    let port = 5559;

    let context = Context::new()?;

    let zmq_stream = StreamSocket::from_context(&context)?;

    let stream_endpoint = format!("tcp://127.0.0.1:{port}");
    zmq_stream.bind(&stream_endpoint)?;

    let tcp_endpoint = format!("127.0.0.1:{port}");
    let tcp_stream = TcpStream::connect(tcp_endpoint).await?;

    let tcp_handle = spawn(run_tcp_client(tcp_stream, "Hello"));
    let zmq_stream_handle = spawn(run_stream_server(zmq_stream, "World"));

    let _ = join!(zmq_stream_handle, tcp_handle);

    Ok(())
}
