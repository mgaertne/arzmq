use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use arzmq::prelude::{
    Message, MultipartReceiver, MultipartSender, Receiver, RecvFlags, SendFlags, Sender, ZmqResult,
};

#[allow(dead_code)]
pub static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

#[allow(dead_code)]
pub static ITERATIONS: AtomicI32 = AtomicI32::new(10);

#[allow(dead_code)]
pub fn run_publisher<S>(socket: &S, msg: &str) -> ZmqResult<()>
where
    S: Sender,
{
    while KEEP_RUNNING.load(Ordering::Acquire) {
        socket.send_msg(msg, SendFlags::empty())?;
    }

    Ok(())
}

#[allow(dead_code)]
pub fn run_send_recv<S>(send_recv: &S, msg: &str) -> ZmqResult<()>
where
    S: Sender + Receiver,
{
    println!("Sending message: {msg:?}");
    send_recv.send_msg(msg, SendFlags::empty())?;

    let message = send_recv.recv_msg(RecvFlags::empty())?;
    println!("Recevied message: {message:?}");

    Ok(())
}

#[allow(dead_code)]
#[cfg(feature = "futures")]
pub async fn run_send_recv_async<S>(send_recv: &S, msg: &str)
where
    S: Sender + Receiver + Sync,
{
    println!("Sending message: {msg:?}");
    let _ = send_recv.send_msg_async(msg, SendFlags::empty()).await;

    loop {
        if let Some(message) = send_recv.recv_msg_async().await {
            println!("Received mesaage: {message:?}");

            ITERATIONS.fetch_sub(1, Ordering::Release);

            break;
        }
    }
}

#[allow(dead_code)]
pub fn run_recv_send<S>(recv_send: &S, msg: &str) -> ZmqResult<()>
where
    S: Receiver + Sender,
{
    let message = recv_send.recv_msg(RecvFlags::empty())?;
    println!("Received message: {message:?}");

    recv_send.send_msg(msg, SendFlags::empty())
}

#[allow(dead_code)]
#[cfg(feature = "futures")]
pub async fn run_recv_send_async<S>(send_recv: &S, msg: &str)
where
    S: Sender + Receiver + Sync,
{
    if let Some(message) = send_recv.recv_msg_async().await {
        println!("Received request: {message:?}");
        send_recv.send_msg_async(msg, SendFlags::empty()).await;
    }
}

#[allow(dead_code)]
pub fn run_multipart_send_recv<S>(send_recv: &S, msg: &str) -> ZmqResult<()>
where
    S: MultipartReceiver + MultipartSender,
{
    println!("Sending message: {msg:?}");
    let multipart: Vec<Message> = vec![vec![].into(), msg.into()];
    send_recv.send_multipart(multipart, SendFlags::empty())?;

    let mut multipart = send_recv.recv_multipart(RecvFlags::empty())?;
    let content = multipart.pop_back().unwrap();
    if !content.is_empty() {
        println!("Received reply: {content:?}");
    }

    Ok(())
}

#[allow(dead_code)]
#[cfg(feature = "futures")]
pub async fn run_multipart_send_recv_async<S>(send_recv: &S, msg: &str)
where
    S: MultipartReceiver + MultipartSender + Sync,
{
    println!("Sending message {msg:?}");
    let multipart: Vec<Message> = vec![vec![].into(), msg.into()];
    let _ = send_recv
        .send_multipart_async(multipart, SendFlags::empty())
        .await;

    let mut message = send_recv.recv_multipart_async().await;
    let content = message.pop_back().unwrap();
    if !content.is_empty() {
        println!("Received reply: {content:?}",);

        ITERATIONS.fetch_sub(1, Ordering::Release);
    }
}

#[allow(dead_code)]
pub fn run_multipart_recv_reply<S>(recv_send: &S, msg: &str) -> ZmqResult<()>
where
    S: MultipartSender + MultipartReceiver,
{
    let mut multipart = recv_send.recv_multipart(RecvFlags::empty())?;

    let content = multipart.pop_back().unwrap();
    if !content.is_empty() {
        println!("Received multipart: {content:?}");
    }

    multipart.push_back(msg.into());
    recv_send.send_multipart(multipart, SendFlags::empty())
}

#[allow(dead_code)]
#[cfg(feature = "futures")]
pub async fn run_multipart_recv_reply_async<S>(recv_send: &S, msg: &str)
where
    S: MultipartSender + MultipartReceiver + Sync,
{
    let mut multipart = recv_send.recv_multipart_async().await;
    let content = multipart.pop_back().unwrap();
    if !content.is_empty() {
        println!("Received request: {content:?}");
    }
    multipart.push_back(msg.into());
    recv_send
        .send_multipart_async(multipart, SendFlags::empty())
        .await;
}

#[allow(dead_code)]
pub fn run_subscribe_client<S>(socket: &S, subscribed_topic: &str) -> ZmqResult<()>
where
    S: Receiver,
{
    let zmq_msg = socket.recv_msg(RecvFlags::empty())?;
    let zmq_str = zmq_msg.to_string();
    let pubsub_item = zmq_str.split_once(" ");
    assert_eq!(Some((subscribed_topic, "important update")), pubsub_item);

    let (topic, item) = pubsub_item.unwrap();
    println!("Received msg for topic {topic:?}: {item}",);

    Ok(())
}

#[allow(dead_code)]
#[cfg(feature = "futures")]
pub async fn run_subscribe_client_async<S>(socket: &S, subscribed_topic: &str)
where
    S: Receiver + Sync,
{
    if let Some(zmq_msg) = socket.recv_msg_async().await {
        let zmq_str = zmq_msg.to_string();
        let pubsub_item = zmq_str.split_once(" ");
        assert_eq!(Some((subscribed_topic, "important update")), pubsub_item);

        let (topic, item) = pubsub_item.unwrap();
        println!("Received msg for topic {topic:?}: {item}",);

        ITERATIONS.fetch_sub(1, Ordering::Release);
    }
}
