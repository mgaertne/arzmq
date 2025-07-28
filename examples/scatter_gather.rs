use core::sync::atomic::Ordering;
use std::thread;

use arzmq::prelude::{
    Context, GatherSocket, Receiver, RecvFlags, ScatterSocket, ZmqError, ZmqResult,
};

mod common;

use common::KEEP_RUNNING;

fn main() -> ZmqResult<()> {
    let iterations = 10;

    let context = Context::new()?;

    let scatter = ScatterSocket::from_context(&context)?;
    scatter.bind("tcp://127.0.0.1:*")?;
    let gather_endpoint = scatter.last_endpoint()?;

    thread::spawn(move || {
        common::run_publisher(&scatter, "important update").unwrap();
    });

    let gather = GatherSocket::from_context(&context)?;
    gather.connect(&gather_endpoint)?;

    (0..iterations).try_for_each(|i| {
        let msg = gather.recv_msg(RecvFlags::empty())?;
        println!("Received message {i:2}: {msg:?}");

        Ok::<(), ZmqError>(())
    })?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
