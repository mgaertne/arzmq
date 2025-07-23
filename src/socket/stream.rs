use crate::{
    ZmqResult, sealed,
    socket::{MultipartReceiver, MultipartSender, Socket, SocketOption, SocketType},
};

/// # A stream socket `ZMQ_STREAM`
///
/// A socket of type [`Stream`] is used to send and receive TCP data from a non-0MQ peer, when
/// using the tcp:// transport. A [`Stream`] socket can act as client and/or server, sending
/// and/or receiving TCP data asynchronously.
///
/// When receiving TCP data, a [`Stream`] socket shall prepend a message part containing the
/// routing id of the originating peer to the message before passing it to the application.
/// Messages received are fair-queued from among all connected peers.
///
/// When sending TCP data, a [`Stream`] socket shall remove the first part of the message and use
/// it to determine the routing id of the peer the message shall be routed to, and unroutable
/// messages shall cause an `Err(`[`HostUnreachable`]`)` or `Err(`[`Again`]`)` error.
///
/// To open a connection to a server, use the [`connect()`] call, and then fetch the socket routing
/// id using [`routing_id()`] option.
///
/// To close a specific connection, send the routing id frame followed by a zero-length message.
///
/// When a connection is made, a zero-length message will be received by the application.
/// Similarly, when the peer disconnects (or the connection is lost), a zero-length message will
/// be received by the application.
///
/// You must send one routing id frame followed by one data frame. The [`SEND_MORE`] flag is
/// required for routing id frames but is ignored on data frames.
///
/// [`Stream`]: StreamSocket
/// [`HostUnreachable`]: crate::ZmqError::HostUnreachable
/// [`Again`]: crate::ZmqError::Again
/// [`connect()`]: #method.connect
/// [`routing_id()`]: #method.routing_id
/// [`SEND_MORE`]: super::SendFlags::SEND_MORE
pub type StreamSocket = Socket<Stream>;

pub struct Stream {}

impl sealed::SenderFlag for Stream {}
impl sealed::ReceiverFlag for Stream {}

impl sealed::SocketType for Stream {
    fn raw_socket_type() -> SocketType {
        SocketType::Stream
    }
}

unsafe impl Sync for Socket<Stream> {}
unsafe impl Send for Socket<Stream> {}

impl MultipartSender for Socket<Stream> {}
impl MultipartReceiver for Socket<Stream> {}

impl Socket<Stream> {
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

    /// # Assign the next outbound routing id `ZMQ_CONNECT_ROUTING_ID`
    ///
    /// The [`set_connect_routing_id()`] option sets the peer id of the peer connected via the next
    /// [`connect()`] call, such that that connection is immediately ready for data transfer with
    /// the given routing id. This option applies only to the first subsequent call to
    /// [`connect()`], [`connect()`] calls thereafter use the default connection behaviour.
    ///
    /// Typical use is to set this socket option ahead of each [`connect()`] call. Each connection
    /// MUST be assigned a unique routing id. Assigning a routing id that is already in use is not
    /// allowed.
    ///
    /// Useful when connecting [`Router`] to [`Router`], or [`Stream`] to [`Stream`], as it allows
    /// for immediate sending to peers. Outbound routing id framing requirements for [`Router`] and
    /// [`Stream`] sockets apply.
    ///
    /// The routing id must be from 1 to 255 bytes long and MAY NOT start with a zero byte (such
    /// routing ids are reserved for internal use by the 0MQ infrastructure).
    ///
    /// [`Stream`]: StreamSocket
    /// [`Router`]: super::RouterSocket
    /// [`connect()`]: #method.connect
    /// [`set_connect_routing_id()`]: #method.set_connect_routing_id
    pub fn set_connect_routing_id<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::ConnectRoutingId, value)
    }

    /// # send connect and disconnect notifications `ZMQ_STREAM_NOTIFY`
    ///
    /// Enables connect and disconnect notifications on a [`Stream`] socket, when set to `true`.
    /// When notifications are enabled, the socket delivers a zero-length message when a peer
    /// connects or disconnects.
    ///
    /// [`Stream`]: StreamSocket
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_stream_notify(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::StreamNotify, value)
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::StreamSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "StreamBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`StreamSocket`].\n\n")]
    #[allow(dead_code)]
    struct StreamConfig {
        socket_config: SocketBuilder,
        #[builder(setter(into), default = "Default::default()")]
        routing_id: String,
        #[builder(setter(into), default = "Default::default()")]
        connect_routing_id: String,
        #[cfg(feature = "draft-api")]
        #[doc(cfg(feature = "draft-api"))]
        #[builder(default = false)]
        stream_notify: bool,
    }

    impl StreamBuilder {
        pub fn apply(self, socket: &StreamSocket) -> ZmqResult<()> {
            if let Some(socket_config) = self.socket_config {
                socket_config.apply(socket)?;
            }

            if let Some(routing_id) = self.routing_id {
                socket.set_routing_id(&routing_id)?;
            }

            if let Some(connect_routing_id) = self.connect_routing_id {
                socket.set_connect_routing_id(&connect_routing_id)?;
            }

            #[cfg(feature = "draft-api")]
            if let Some(stream_notify) = self.stream_notify {
                socket.set_stream_notify(stream_notify)?;
            }

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<StreamSocket> {
            let socket = StreamSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
}
