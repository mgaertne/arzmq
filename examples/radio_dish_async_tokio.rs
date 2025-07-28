#![cfg(feature = "examples-tokio")]
use core::sync::atomic::Ordering;

use arzmq::prelude::{
    Context, DishSocket, Message, RadioSocket, Receiver, SendFlags, Sender, ZmqResult,
};
use tokio::{join, spawn};

mod common;

use common::{ITERATIONS, KEEP_RUNNING};

static GROUP: &str = "radio-dish-ex";

async fn run_dish(dish: DishSocket) {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        if let Some(message) = dish.recv_msg_async().await {
            println!("Received message: {message:?}");
            ITERATIONS.fetch_sub(1, Ordering::Release);
        }
    }
    KEEP_RUNNING.store(false, Ordering::Release);
}

async fn run_radio(radio: RadioSocket, msg: &str) -> ZmqResult<()> {
    while KEEP_RUNNING.load(Ordering::Acquire) {
        let message: Message = msg.into();
        message.set_group(GROUP)?;
        radio.send_msg_async(message, SendFlags::empty()).await;
    }

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let context = Context::new()?;

    let radio = RadioSocket::from_context(&context)?;
    radio.bind("tcp://127.0.0.1:*")?;
    let dish_endpoint = radio.last_endpoint()?;

    let dish = DishSocket::from_context(&context)?;
    dish.connect(dish_endpoint)?;
    dish.join(GROUP)?;

    let radio_handle = spawn(run_radio(radio, "important update"));
    let dish_handle = spawn(run_dish(dish));

    let _ = join!(radio_handle, dish_handle);

    Ok(())
}
