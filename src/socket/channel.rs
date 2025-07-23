use crate::{
    sealed,
    socket::{Socket, SocketType},
};

/// # A channel socket `ZMQ_CHANNEL`
///
/// A socket of type [`Channel`] can only be connected to a single peer at any one time. No
/// message routing or filtering is performed on messages sent over a [`Channel`] socket.
///
/// When a [`Channel`] socket enters the 'mute' state due to having reached the high water mark
/// for the connected peer, or, for connection-oriented transports, if the [`immediate()`]
/// option is set and there is no connected peer, then any [`send_msg()`] operation on the socket
/// shall block until the peer becomes available for sending; messages are not discarded.
///
/// While [`Channel`] sockets can be used over transports other than `inproc`, their inability to
/// auto-reconnect coupled with the fact new incoming connections will be terminated while any
/// previous connections (including ones in a closing state) exist makes them unsuitable for TCP
/// in most cases.
///
/// [`Channel`]: ChannelSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #method.send_msg
pub type ChannelSocket = Socket<Channel>;

pub struct Channel {}

impl sealed::SenderFlag for Channel {}
impl sealed::ReceiverFlag for Channel {}

impl sealed::SocketType for Channel {
    fn raw_socket_type() -> SocketType {
        SocketType::Channel
    }
}

unsafe impl Sync for Socket<Channel> {}
unsafe impl Send for Socket<Channel> {}

impl Socket<Channel> {}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use crate::socket::SocketBuilder;

    /// Builder for [`ChannelSocket`](super::ChannelSocket)
    pub type ChannelBuilder = SocketBuilder;
}
