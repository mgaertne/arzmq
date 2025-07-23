use crate::{
    ZmqResult, sealed,
    socket::{MultipartReceiver, MultipartSender, Socket, SocketOption, SocketType},
};

/// # A Reply socket `ZMQ_REP`
///
/// A socket of type [`Reply`] is used by a service to receive requests from and send replies to a
/// client. This socket type allows only an alternating sequence of [`recv_msg()`] and subsequent
/// [`send_msg()`] calls. Each request received is fair-queued from among all clients, and each
/// reply sent is routed to the client that issued the last request. If the original requester does
/// not exist any more the reply is silently discarded.
///
/// [`Reply`]: ReplySocket
/// [`send_msg()`]: #impl-Sender-for-Socket<T>
/// [`recv_msg()`]: #impl-Receiver-for-Socket<T>
pub type ReplySocket = Socket<Reply>;

pub struct Reply {}

impl sealed::SenderFlag for Reply {}
impl sealed::ReceiverFlag for Reply {}

impl sealed::SocketType for Reply {
    fn raw_socket_type() -> SocketType {
        SocketType::Reply
    }
}

unsafe impl Sync for Socket<Reply> {}
unsafe impl Send for Socket<Reply> {}

impl MultipartSender for Socket<Reply> {}
impl MultipartReceiver for Socket<Reply> {}

impl Socket<Reply> {
    /// # Set socket routing id `ZMQ_ROUTING_ID`
    ///
    /// The [`set_routing_id()`] option shall set the routing id of the specified 'socket' when
    /// connecting to a [`Router`] socket.
    ///
    /// A routing id must be at least one byte and at most 255 bytes long. Identities starting with
    /// a zero byte are reserved for use by the 0MQ infrastructure.
    ///
    /// If two clients use the same routing id when connecting to a [`Router`], the results shall
    /// depend on the [`set_router_handover()`] option setting. If that is not set (or set to the
    /// default of zero), the [`Router`] socket shall reject clients trying to connect with an
    /// already-used routing id. If that option is set to `true`, the [`Router`]socket shall
    /// hand-over the connection to the new client and disconnect the existing one.
    ///
    /// [`set_routing_id()`]: #method.set_routing_id
    /// [`Router`]: super::RouterSocket
    /// [`set_router_handover()`]: super::RouterSocket::set_router_handover
    pub fn set_routing_id<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::RoutingId, value)
    }

    /// # Retrieve socket routing id `ZMQ_ROUTING_ID`
    ///
    /// The [`routing_id()`] option shall retrieve the routing id of the specified 'socket'.
    /// Routing ids are used only by the request/reply pattern. Specifically, it can be used in
    /// tandem with [`Router`] socket to route messages to the peer with a specific routing id.
    ///
    /// A routing id must be at least one byte and at most 255 bytes long. Identities starting
    /// with a zero byte are reserved for use by the 0MQ infrastructure.
    ///
    /// [`routing_id()`]: #method.routing_id
    /// [`Router`]: super::RouterSocket
    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOption::RoutingId)
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::ReplySocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "ReplyBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`ReplySocket`].\n\n")]
    #[allow(dead_code)]
    struct ReplyConfig {
        socket_config: SocketBuilder,
        #[builder(setter(into), default = "Default::default()")]
        routing_id: String,
    }

    impl ReplyBuilder {
        pub fn apply(self, socket: &ReplySocket) -> ZmqResult<()> {
            if let Some(socket_config) = self.socket_config {
                socket_config.apply(socket)?;
            }

            if let Some(routing_id) = self.routing_id {
                socket.set_routing_id(routing_id)?;
            }

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<ReplySocket> {
            let socket = ReplySocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
}
