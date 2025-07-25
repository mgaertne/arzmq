#![cfg(feature = "examples-futures")]
use core::{error::Error, sync::atomic::Ordering};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

use arzmq::prelude::{Context, MultipartReceiver, StreamSocket};
use futures::{executor::ThreadPool, join, task::SpawnExt};

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
        tcp_stream.write_all(msg.as_bytes()).unwrap();

        let mut buffer = [0; 256];
        if let Ok(length) = tcp_stream.read(&mut buffer)
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

#[cfg(feature = "examples-futures")]
fn main() -> Result<(), Box<dyn Error>> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let port = 5558;

        let context = Context::new()?;

        let zmq_stream = StreamSocket::from_context(&context)?;

        let stream_endpoint = format!("tcp://127.0.0.1:{port}");
        zmq_stream.bind(&stream_endpoint)?;

        let tcp_endpoint = format!("127.0.0.1:{port}");
        let tcp_stream = TcpStream::connect(tcp_endpoint)?;

        let tcp_handle = executor.spawn_with_handle(run_tcp_client(tcp_stream, "Hello"))?;
        let zmq_stream_handle =
            executor.spawn_with_handle(run_stream_server(zmq_stream, "World"))?;

        let _ = join!(zmq_stream_handle, tcp_handle);

        Ok(())
    })
}
