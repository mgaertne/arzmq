use core::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use arzmq::{
    ZmqError, ZmqResult,
    context::Context,
    message::Message,
    socket::{DishSocket, RadioSocket, Receiver, RecvFlags, SendFlags, Sender},
};

static GROUP: &str = "radio-dish-ex";
static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

fn run_radio_socket(radio: &RadioSocket, message: &str) -> ZmqResult<()> {
    let msg: Message = message.into();
    msg.set_group(GROUP).unwrap();

    radio.send_msg(msg, SendFlags::empty()).unwrap();

    Ok(())
}

fn main() -> ZmqResult<()> {
    let port = 5679;
    let iterations = 10;

    let context = Context::new()?;

    let radio = RadioSocket::from_context(&context)?;

    let radio_endpoint = format!("tcp://*:{port}");
    radio.bind(&radio_endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            run_radio_socket(&radio, "radio msg").unwrap();
        }
    });

    let dish = DishSocket::from_context(&context)?;

    let dish_endpoint = format!("tcp://localhost:{port}");
    dish.connect(&dish_endpoint)?;
    dish.join(GROUP)?;

    (0..iterations).try_for_each(|i| {
        let msg = dish.recv_msg(RecvFlags::empty())?;
        println!("Received message {i:2}: {msg:?}");
        Ok::<(), ZmqError>(())
    })?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
