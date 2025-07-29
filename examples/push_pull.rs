use core::sync::atomic::Ordering;
use std::thread;

use arzmq::prelude::{Context, PullSocket, PushSocket, Receiver, RecvFlags, ZmqError, ZmqResult};

mod common;

use common::KEEP_RUNNING;

fn main() -> ZmqResult<()> {
    let iterations = 10;

    let context = Context::new()?;

    let push = PushSocket::from_context(&context)?;
    push.bind("tcp://127.0.0.1:*")?;
    let pull_endpoint = push.last_endpoint()?;

    thread::spawn(move || common::run_publisher(&push, "important update").unwrap());

    let pull = PullSocket::from_context(&context)?;
    pull.connect(pull_endpoint)?;

    (0..iterations).try_for_each(|i| {
        let msg = pull.recv_msg(RecvFlags::empty())?;
        println!("Received message {i:2}: {msg}");

        Ok::<(), ZmqError>(())
    })?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
