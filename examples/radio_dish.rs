use core::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use arzmq::prelude::{
    Context, DishSocket, Message, RadioSocket, Receiver, RecvFlags, SendFlags, Sender, ZmqError,
    ZmqResult,
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
    let iterations = 10;

    let context = Context::new()?;

    let radio = RadioSocket::from_context(&context)?;
    radio.bind("tcp://127.0.0.1:*")?;
    let dish_endpoint = radio.last_endpoint()?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            run_radio_socket(&radio, "radio msg").unwrap();
        }
    });

    let dish = DishSocket::from_context(&context)?;
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
