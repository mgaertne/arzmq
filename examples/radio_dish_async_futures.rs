#![cfg(feature = "examples-futures")]
use core::sync::atomic::Ordering;

use arzmq::{
    ZmqResult,
    prelude::{Context, DishSocket, Message, RadioSocket, Receiver, SendFlags, Sender},
};
use futures::{executor::ThreadPool, join, task::SpawnExt};

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
        radio.send_msg_async(message, SendFlags::DONT_WAIT).await;
    }

    Ok(())
}

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let context = Context::new()?;

        let radio = RadioSocket::from_context(&context)?;
        radio.bind("tcp://127.0.0.1:*")?;
        let dish_endpoint = radio.last_endpoint()?;

        let dish = DishSocket::from_context(&context)?;
        dish.connect(dish_endpoint)?;
        dish.join(GROUP)?;

        let radio_handle = executor
            .spawn_with_handle(run_radio(radio, "important update"))
            .unwrap();
        let dish_handle = executor.spawn_with_handle(run_dish(dish)).unwrap();

        let _ = join!(radio_handle, dish_handle);

        Ok(())
    })
}
