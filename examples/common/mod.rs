use core::sync::atomic::{AtomicBool, Ordering};

use arzmq::{
    ZmqResult,
    message::Message,
    socket::{MultipartReceiver, MultipartSender, Receiver, RecvFlags, SendFlags, Sender},
};

#[allow(dead_code)]
pub static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

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
    println!("Sending message {msg:?}");
    send_recv.send_msg(msg, SendFlags::empty())?;

    let message = send_recv.recv_msg(RecvFlags::empty())?;
    println!("Recevied message {message:?}");

    Ok(())
}

#[allow(dead_code)]
pub fn run_recv_send<S>(recv_send: &S, msg: &str) -> ZmqResult<()>
where
    S: Receiver + Sender,
{
    let message = recv_send.recv_msg(RecvFlags::empty())?;
    println!("Received message {message:?}");

    recv_send.send_msg(msg, SendFlags::empty())
}

#[allow(dead_code)]
pub fn run_multipart_send_recv<S>(send_recv: &S, msg: &str) -> ZmqResult<()>
where
    S: MultipartReceiver + MultipartSender,
{
    println!("Sending message {msg:?}");
    let multipart: Vec<Message> = vec![vec![].into(), msg.into()];
    send_recv.send_multipart(multipart, SendFlags::empty())?;

    let mut multipart = send_recv.recv_multipart(RecvFlags::empty())?;
    let content = multipart.pop_back().unwrap();
    if !content.is_empty() {
        println!("Received reply {content:?}");
    }

    Ok(())
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
