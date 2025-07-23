use crate::{
    ZmqResult, sealed,
    socket::{MultipartSender, Socket, SocketOption, SocketType},
};

/// # A push socket `ZMQ_PUSH`
///
/// A socket of type [`Push`] is used by a pipeline node to send messages to downstream pipeline
/// nodes. Messages are round-robined to all connected downstream nodes. The `recv_msg()` function
/// is not implemented for this socket type.
///
/// When a [`Push`] socket enters the 'mute' state due to having reached the high water mark for
/// all downstream nodes, or, for connection-oriented transports, if the [`immediate()`] option is
/// set and there are no downstream nodes at all, then any [`send_msg()`] operations on the socket
/// shall block until the mute state ends or at least one downstream node becomes available for
/// sending; messages are not discarded.
///
/// [`Push`]: PushSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #impl-Sender-for-Socket<T>
pub type PushSocket = Socket<Push>;

pub struct Push {}

impl sealed::SenderFlag for Push {}

impl sealed::SocketType for Push {
    fn raw_socket_type() -> SocketType {
        SocketType::Push
    }
}

unsafe impl Sync for Socket<Push> {}
unsafe impl Send for Socket<Push> {}

impl MultipartSender for Socket<Push> {}

impl Socket<Push> {
    /// # Keep only last message `ZMQ_CONFLATE`
    ///
    /// If set, a socket shall keep only one message in its inbound/outbound queue, this message
    /// being the last message received/the last message to be sent. Ignores
    /// [`receive_highwater_mark()`] and [`send_highwater_mark()`] options. Does not support
    /// multi-part messages, in particular, only one part of it is kept in the socket internal
    /// queue.
    ///
    /// # Note
    ///
    /// If [`recv_msg()`] is not called on the inbound socket, the queue and memory will grow with
    /// each message received. Use [`events()`] to trigger the conflation of the messages.
    ///
    /// [`receive_highwater_mark()`]: #method.receive_highwater_mark
    /// [`send_highwater_mark()`]: #method.send_highwater_mark
    /// [`recv_msg()`]: #method.recv_msg
    /// [`events()`]: #method.events
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::Conflate, value)
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::PushSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "PushBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`PushSocket`].\n\n")]
    #[allow(dead_code)]
    struct PushConfig {
        socket_config: SocketBuilder,
        #[builder(default = false)]
        conflate: bool,
    }

    impl PushBuilder {
        pub fn apply(self, socket: &PushSocket) -> ZmqResult<()> {
            if let Some(socket_config) = self.socket_config {
                socket_config.apply(socket)?;
            }

            if let Some(conflate) = self.conflate {
                socket.set_conflate(conflate)?;
            }

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<PushSocket> {
            let socket = PushSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
}
