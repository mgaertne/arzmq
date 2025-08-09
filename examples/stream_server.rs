use core::error::Error;
#[rustversion::since(1.87)]
use core::str;
#[rustversion::before(1.87)]
use std::str;
use std::{io::prelude::*, net::TcpStream, thread};

use arzmq::prelude::{
    Context, MultipartReceiver, MultipartSender, RecvFlags, SendFlags, StreamSocket, ZmqResult,
};

fn run_stream_socket(zmq_stream: &StreamSocket, msg: &str) -> ZmqResult<()> {
    let mut message = zmq_stream.recv_multipart(RecvFlags::empty())?;
    println!("Received request: {:?}", message.pop_back().unwrap());

    message.push_back(msg.into());
    zmq_stream.send_multipart(message, SendFlags::empty())
}

#[rustversion::attr(all(nightly, since(1.88)), allow(clippy::collapsible_if))]
fn run_tcp_client(endpoint: &str, iterations: i32) -> Result<(), Box<dyn Error>> {
    let mut tcp_stream = TcpStream::connect(endpoint)?;
    (0..iterations).try_for_each(|request_no| {
        println!("Sending requrst {request_no}");
        tcp_stream.write_all("Hello".as_bytes()).unwrap();

        let mut buffer = [0; 256];
        if let Ok(length) = tcp_stream.read(&mut buffer) {
            if length != 0 {
                let recevied_msg = &buffer[..length];
                println!(
                    "Received reply {request_no:2} {}",
                    str::from_utf8(recevied_msg).unwrap()
                );
            }
        }

        Ok::<(), Box<dyn Error>>(())
    })?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let iterations = 10;

    let context = Context::new()?;

    let zmq_stream = StreamSocket::from_context(&context)?;

    zmq_stream.bind("tcp://127.0.0.1:*")?;
    let tcp_endpoint = zmq_stream.last_endpoint()?;

    thread::spawn(move || {
        let mut connect_msg = zmq_stream.recv_multipart(RecvFlags::empty()).unwrap();
        let _routing_id = connect_msg.pop_front().unwrap();

        loop {
            run_stream_socket(&zmq_stream, "World").unwrap();
        }
    });

    run_tcp_client(tcp_endpoint.strip_prefix("tcp://").unwrap(), iterations)?;

    Ok(())
}
