use core::sync::atomic::Ordering;
use std::thread;

use arzmq::{
    ZmqError, ZmqResult,
    context::Context,
    socket::{GatherSocket, Receiver, RecvFlags, ScatterSocket},
};

mod common;

use common::KEEP_RUNNING;

fn main() -> ZmqResult<()> {
    let port = 5680;
    let iterations = 10;

    let context = Context::new()?;

    let scatter = ScatterSocket::from_context(&context)?;

    let scatter_endpoint = format!("tcp://*:{port}");
    scatter.bind(&scatter_endpoint)?;

    thread::spawn(move || {
        common::run_publisher(&scatter, "important update").unwrap();
    });

    let gather = GatherSocket::from_context(&context)?;

    let gather_endpoint = format!("tcp://localhost:{port}");
    gather.connect(&gather_endpoint)?;

    (0..iterations).try_for_each(|i| {
        let msg = gather.recv_msg(RecvFlags::empty())?;
        println!("Received message {i:2}: {msg:?}");

        Ok::<(), ZmqError>(())
    })?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
