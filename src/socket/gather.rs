use crate::{
    sealed,
    socket::{Socket, SocketType},
};

/// # A gather socket `ZMQ_GATHER`
///
/// A socket of type [`Gather`] is used by a scatter-gather node to receive messages from upstream
/// scatter-gather nodes. Messages are fair-queued from among all connected upstream nodes.
///
/// [`Gather`]: GatherSocket
pub type GatherSocket = Socket<Gather>;

pub struct Gather {}

impl sealed::ReceiverFlag for Gather {}
impl sealed::SocketType for Gather {
    fn raw_socket_type() -> SocketType {
        SocketType::Gather
    }
}

unsafe impl Sync for Socket<Gather> {}
unsafe impl Send for Socket<Gather> {}

impl Socket<Gather> {}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use crate::socket::SocketBuilder;

    /// Builder for [`GatherSocket`](super::GatherSocket)
    pub type GatherBuilder = SocketBuilder;
}
