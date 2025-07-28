#![cfg(feature = "examples-futures")]
use core::{error::Error, sync::atomic::Ordering};
use std::net::TcpStream;

use arzmq::prelude::{Context, MultipartReceiver, StreamSocket};
use futures::{
    executor::ThreadPool,
    io::{self, AllowStdIo, AsyncReadExt, AsyncWriteExt},
    join,
    task::SpawnExt,
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

async fn run_tcp_client(mut tcp_stream: AllowStdIo<TcpStream>, msg: &str) -> io::Result<()> {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        tcp_stream.write_all(msg.as_bytes()).await?;

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

    Ok(())
}

#[cfg(feature = "examples-futures")]
fn main() -> Result<(), Box<dyn Error>> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let context = Context::new()?;

        let zmq_stream = StreamSocket::from_context(&context)?;
        zmq_stream.bind("tcp://127.0.0.1:*")?;
        let tcp_endpoint = zmq_stream.last_endpoint()?;

        let tcp_stream = TcpStream::connect(tcp_endpoint.strip_prefix("tcp://").unwrap())?;

        let tcp_handle =
            executor.spawn_with_handle(run_tcp_client(AllowStdIo::new(tcp_stream), "Hello"))?;
        let zmq_stream_handle =
            executor.spawn_with_handle(run_stream_server(zmq_stream, "World"))?;

        let _ = join!(zmq_stream_handle, tcp_handle);

        Ok(())
    })
}
