use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    message::Message,
    socket::{PeerSocket, Receiver, RecvFlags, SendFlags, Sender},
};

fn run_peer_server(peer: &PeerSocket, msg: &str) -> ZmqResult<()> {
    let message = peer.recv_msg(RecvFlags::empty())?;
    println!("Received request: {message:?}");

    let response: Message = msg.into();
    response.set_routing_id(message.routing_id().unwrap())?;
    peer.send_msg(response, SendFlags::empty())
}

fn run_peer_client(peer: &PeerSocket, routing_id: u32, msg: &str) -> ZmqResult<()> {
    let request: Message = msg.into();
    request.set_routing_id(routing_id)?;
    peer.send_msg(request, SendFlags::empty())?;

    let message = peer.recv_msg(RecvFlags::empty())?;
    println!("Received reply: {message:?}");

    Ok(())
}

fn main() -> ZmqResult<()> {
    let endpoint = "inproc://arzmq-example-peer";
    let iterations = 10;

    let context = Context::new()?;

    let peer_server = PeerSocket::from_context(&context)?;
    peer_server.bind(endpoint)?;

    thread::spawn(move || {
        (0..iterations)
            .try_for_each(|_| run_peer_server(&peer_server, "World"))
            .unwrap();
    });

    let peer_client = PeerSocket::from_context(&context)?;
    let routing_id = peer_client.connect_peer(endpoint)?;

    (0..iterations).try_for_each(|_| run_peer_client(&peer_client, routing_id, "Hello"))
}
