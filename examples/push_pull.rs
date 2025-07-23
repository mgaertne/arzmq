use core::sync::atomic::Ordering;
use std::thread;

use arzmq::{
    ZmqError, ZmqResult,
    context::Context,
    socket::{PullSocket, PushSocket, Receiver, RecvFlags},
};

mod common;

use common::KEEP_RUNNING;

fn main() -> ZmqResult<()> {
    let port = 5557;
    let iterations = 10;

    let context = Context::new()?;

    let push = PushSocket::from_context(&context)?;

    let push_endpoint = format!("tcp://*:{port}");
    push.bind(&push_endpoint)?;

    thread::spawn(move || common::run_publisher(&push, "important update").unwrap());

    let pull = PullSocket::from_context(&context)?;

    let pull_endpoint = format!("tcp://localhost:{port}");
    pull.connect(&pull_endpoint)?;

    (0..iterations).try_for_each(|i| {
        let msg = pull.recv_msg(RecvFlags::empty())?;
        println!("Received message {i:2}: {msg}");

        Ok::<(), ZmqError>(())
    })?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
