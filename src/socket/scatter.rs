use crate::{
    sealed,
    socket::{Socket, SocketType},
};

/// # A radio socket `ZMQ_SCATTER`
///
/// A socket of type [`Scatter`] is used by a scatter-gather node to send messages to downstream
/// scatter-gather nodes. Messages are round-robined to all connected downstream nodes.
///
/// When a [`Scatter`] socket enters the 'mute' state due to having reached the high water mark
/// for all downstream nodes, or, for connection-oriented transports, if the [`immediate()`]
/// option is set and there are no downstream nodes at all, then any [`send_msg()`] operations on
/// the socket shall block until the mute state ends or at least one downstream node becomes
/// available for sending; messages are not discarded.
///
/// [`Scatter`]: ScatterSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #method.send_msg
pub type ScatterSocket = Socket<Scatter>;

pub struct Scatter {}

impl sealed::SenderFlag for Scatter {}
impl sealed::SocketType for Scatter {
    fn raw_socket_type() -> SocketType {
        SocketType::Scatter
    }
}

unsafe impl Sync for Socket<Scatter> {}
unsafe impl Send for Socket<Scatter> {}

impl Socket<Scatter> {}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use crate::socket::SocketBuilder;

    /// Builder for [`ScatterSocket`](super::ScatterSocket)
    pub type ScatterBuilder = SocketBuilder;
}
