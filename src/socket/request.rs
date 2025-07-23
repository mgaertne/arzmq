use crate::{
    ZmqResult, sealed,
    socket::{MultipartReceiver, MultipartSender, Socket, SocketOption, SocketType},
};

/// # A Requester socket `ZMQ_REQ`
///
/// A socket of type [`Request`] is used by a client to send requests to and receive replies from
/// a service. This socket type allows only an alternating sequence of [`send_msg()`] and
/// subsequent [`recv_msg()`] calls. Each request sent is round-robined among all services, and
/// each reply received is matched with the last issued request.
///
/// For connection-oriented transports, If the [`immediate()`] option is set and there is no
/// service available, then any send operation on the socket shall block until at least one service
/// becomes available. The [`Request`] socket shall not discard messages.
///
/// [`Request`]: RequestSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #impl-Sender-for-Socket<T>
/// [`recv_msg()`]: #impl-Receiver-for-Socket<T>
pub type RequestSocket = Socket<Request>;

pub struct Request {}

impl sealed::SenderFlag for Request {}
impl sealed::ReceiverFlag for Request {}

impl sealed::SocketType for Request {
    fn raw_socket_type() -> SocketType {
        SocketType::Request
    }
}

unsafe impl Sync for Socket<Request> {}
unsafe impl Send for Socket<Request> {}

impl MultipartSender for Socket<Request> {}
impl MultipartReceiver for Socket<Request> {}

impl Socket<Request> {
    /// # match replies with requests `ZMQ_REQ_CORRELATE`
    ///
    /// The default behaviour of [`Request`] sockets is to rely on the ordering of messages to
    /// match requests and responses and that is usually sufficient. When this option is set to
    /// `true`, the [`Request`] socket will prefix outgoing messages with an extra frame containing
    /// a request id. That means the full message is `(request id, 0, user frames...)`. The
    /// [`Request`] socket will discard all incoming messages that don’t begin with these two
    /// frames.
    ///
    /// [`Request`]: RequestSocket
    pub fn set_correlate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::RequestCorrelate, value)
    }

    /// # relax strict alternation between request and reply `ZMQ_REQ_RELAXED`
    ///
    /// By default, a [`Request`] socket does not allow initiating a new request with
    /// [`send_msg()`] until the reply to the previous one has been received. When set to `true`,
    /// sending another message is allowed and previous replies will be discarded if any. The
    /// request-reply state machine is reset and a new request is sent to the next available peer.
    ///
    /// If set to `true`, also enable [`set_correlate()`] to ensure correct matching of requests
    /// and replies. Otherwise a late reply to an aborted request can be reported as the reply to
    /// the superseding request.
    ///
    /// [`Request`]: RequestSocket
    /// [`send_msg()`]: #method.send_msg
    /// [`set_correlate()`]: #method.set_correlate
    pub fn set_relaxed(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::RequestRelaxed, value)
    }

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

    /// # bootstrap connections to ROUTER sockets `ZMQ_PROBE_ROUTER`
    ///
    /// When set to `true`, the socket will automatically send an empty message when a new
    /// connection is made or accepted. You may set this on [`Request`], [`Dealer`], or [`Router`]
    /// sockets connected to a [`Router`] socket. The application must filter such empty messages.
    /// The [`ProbeRouter`] option in effect provides the [`Router`] application with an event
    /// signaling the arrival of a new peer.
    ///
    /// | Default value | Applicable socket types             |
    /// | :-----------: | :---------------------------------: |
    /// | false         | [`Router`], [`Dealer`], [`Request`] |
    ///
    /// [`ProbeRouter`]: SocketOption::ProbeRouter
    /// [`Router`]: super::RouterSocket
    /// [`Dealer`]: super::DealerSocket
    /// [`Request`]: RequestSocket
    pub fn set_probe_router(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::ProbeRouter, value)
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::RequestSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "RequestBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`RequestSocket`].\n\n")]
    #[allow(dead_code)]
    pub struct RequestConfig {
        socket_config: SocketBuilder,
        #[builder(default = false)]
        correlate: bool,
        #[builder(default = false)]
        relaxed: bool,
        #[builder(setter(into), default = "Default::default()")]
        routing_id: String,
    }

    impl RequestBuilder {
        pub fn apply(self, socket: &RequestSocket) -> ZmqResult<()> {
            if let Some(socket_config) = self.socket_config {
                socket_config.apply(socket)?;
            }

            if let Some(correlate) = self.correlate {
                socket.set_correlate(correlate)?;
            }

            if let Some(relaxed) = self.relaxed {
                socket.set_relaxed(relaxed)?;
            }

            if let Some(routing_id) = self.routing_id {
                socket.set_routing_id(&routing_id)?;
            }

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<RequestSocket> {
            let socket = RequestSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
}
