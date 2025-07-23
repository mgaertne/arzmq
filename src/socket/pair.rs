use crate::{
    sealed,
    socket::{MultipartReceiver, MultipartSender, Socket, SocketType},
};

/// # A pair socket `ZMQ_PAIR`
///
/// A socket of type [`Pair`] can only be connected to a single peer at any one time. No message
/// routing or filtering is performed on messages sent over a [`Pair`] socket.
///
/// When a [`Pair`] socket enters the 'mute' state due to having reached the high water mark for
/// the connected peer, or, for connection-oriented transports, if the
/// [`immediate()`] option is set and there is no connected peer, then any
/// [`send_msg()`] operations on the socket shall block until the peer becomes available for
/// sending; messages are not discarded.
///
/// While [`Pair`] sockets can be used over transports other than `inproc`, their inability to
/// auto-reconnect coupled with the fact new incoming connections will be terminated while any
/// previous connections (including ones in a closing state) exist makes them unsuitable for TCP
/// in most cases.
///
/// <div class="warning">
///
/// [`Pair`] sockets are designed for inter-thread communication across the `inproc` transport
/// and do not implement functionality such as auto-reconnection.
///
/// </div>
///
/// [`Pair`]: PairSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #impl-Sender-for-Socket<T>
pub type PairSocket = Socket<Pair>;

pub struct Pair {}

impl sealed::SenderFlag for Pair {}
impl sealed::ReceiverFlag for Pair {}

impl sealed::SocketType for Pair {
    fn raw_socket_type() -> SocketType {
        SocketType::Pair
    }
}

unsafe impl Sync for Socket<Pair> {}
unsafe impl Send for Socket<Pair> {}

impl MultipartSender for Socket<Pair> {}
impl MultipartReceiver for Socket<Pair> {}

impl Socket<Pair> {}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use crate::socket::SocketBuilder;

    /// Builder for [`PairSocket`](super::PairSocket)
    pub type PairBuilder = SocketBuilder;
}
