use super::{MultipartReceiver, MultipartSender, Socket, SocketOption, SocketType};
use crate::{ZmqResult, sealed};

/// # A dealer socket `ZMQ_DEALER`
///
/// A socket of type [`Dealer`] is an advanced pattern used for extending request/reply sockets.
/// Each message sent is round-robined among all connected peers, and each message received is
/// fair-queued from all connected peers.
///
/// When a [`Dealer`] socket enters the 'mute' state due to having reached the high water mark for
/// all peers, or, for connection-oriented transports, if the [`immediate()`] option is set and
/// there are no peers at /// all, then any [`send_msg()`] operations on the socket shall block
/// until the mute state ends or at least one peer becomes available for sending; messages are not
/// discarded.
///
/// When a [`Dealer`] socket is connected to a [`Reply`](type@super::ReplySocket) socket each
/// message sent must consist of an empty message part, the delimiter, followed by one or more body
/// parts.
///
/// [`Dealer`]: DealerSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #impl-Sender-for-Socket<T>
pub type DealerSocket = Socket<Dealer>;

pub struct Dealer {}

impl sealed::SenderFlag for Dealer {}
impl sealed::ReceiverFlag for Dealer {}

impl sealed::SocketType for Dealer {
    fn raw_socket_type() -> SocketType {
        SocketType::Dealer
    }
}

unsafe impl Sync for Socket<Dealer> {}
unsafe impl Send for Socket<Dealer> {}

impl MultipartSender for Socket<Dealer> {}
impl MultipartReceiver for Socket<Dealer> {}

impl Socket<Dealer> {
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
    /// If [`receive_highwater_mark()`] is not called on the inbound socket, the queue and memory
    /// will grow with each message received. Use [`events()`] to trigger the conflation of the
    /// messages.
    ///
    /// [`receive_highwater_mark()`]: #method.receive_highwater_mark
    /// [`send_highwater_mark()`]: #method.send_highwater_mark
    /// [`recv_msg()`]: #method.recv_msg
    /// [`events()`]: #method.events
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::Conflate, value)
    }

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
    /// If [`receive_highwater_mark()`] is not called on the inbound socket, the queue and memory
    /// will grow with each message received. Use [`events()`] to trigger the conflation of the
    /// messages.
    ///
    /// [`receive_highwater_mark()`]: #method.receive_highwater_mark
    /// [`send_highwater_mark()`]: #method.send_highwater_mark
    /// [`recv_msg()`]: #method.recv_msg
    /// [`events()`]: #method.events
    pub fn conflate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::Conflate)
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
    /// [`set_router_handover()`]: #method.set_router_handover
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
    /// [`Dealer`]: DealerSocket
    /// [`Request`]: super::RequestSocket
    pub fn set_probe_router(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::ProbeRouter, value)
    }

    /// # set a hiccup message that the socket will generate when connected peer temporarily disconnect `ZMQ_HICCUP_MSG`
    ///
    /// When set, the socket will generate a hiccup message when connect peer has been
    /// disconnected. You may set this on [`Dealer`], [`Client`] and [`Peer`] sockets. The
    /// combination with [`set_heartbeat_interval()`] is powerful and simplify protocols, when
    /// heartbeat recognize a connection drop it will generate a hiccup message that can match the
    /// protocol of the application.
    ///
    /// [`Dealer`]: DealerSocket
    /// [`Client`]: super::ClientSocket
    /// [`Peer`]: super::PeerSocket
    /// [`set_heartbeat_interval()`]: #method.set_heartbeat_interval
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_hiccup_message<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::HiccupMessage, value)
    }

    /// # set an hello message that will be sent when a new peer connect `ZMQ_HELLO_MSG`
    ///
    /// When set, the socket will automatically send an hello message when a new connection is made
    /// or accepted. You may set this on [`Dealer`], [`Router`], [`Client`], [`Server`] and [`Peer`]
    /// sockets. The combination with [`set_heartbeat_interval()`] is powerful and simplify
    /// protocols, as now heartbeat and sending the hello message can be left out of protocols and
    /// be handled by zeromq.
    ///
    /// [`Dealer`]: DealerSocket
    /// [`Router`]: super::RouterSocket
    /// [`Client`]: super::ClientSocket
    /// [`Server`]: super::ServerSocket
    /// [`Peer`]: super::PeerSocket
    /// [`set_heartbeat_interval()`]: #method.set_heartbeat_interval
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_hello_message<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::HelloMessage, value)
    }
}

#[cfg(test)]
mod dealer_tests {
    use super::DealerSocket;
    use crate::prelude::{
        Context, Message, MultipartReceiver, MultipartSender, RecvFlags, SendFlags, ZmqResult,
    };

    #[test]
    fn set_conflate_sets_conflate() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        socket.set_conflate(true)?;

        assert!(socket.conflate()?);

        Ok(())
    }

    #[test]
    fn set_routing_id_sets_routing_id() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        socket.set_routing_id("test123")?;

        assert_eq!(socket.routing_id()?, "test123");

        Ok(())
    }

    #[test]
    fn set_probe_router_sets_probe_router() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        socket.set_probe_router(true)?;

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_hiccup_message_sets_hiccup_message() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        socket.set_hiccup_message("test123")?;

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_hello_message_sets_hello_message() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        socket.set_hello_message("test123")?;

        Ok(())
    }

    #[test]
    fn dealer_dealer() -> ZmqResult<()> {
        let context = Context::new()?;

        let dealer_server = DealerSocket::from_context(&context)?;
        dealer_server.bind("tcp://127.0.0.1:*")?;
        let client_endpoint = dealer_server.last_endpoint()?;

        std::thread::spawn(move || {
            let mut multipart = dealer_server.recv_multipart(RecvFlags::empty()).unwrap();

            let content = multipart.pop_back().unwrap();
            assert!(!content.is_empty());
            assert_eq!(content.to_string(), "Hello");

            multipart.push_back("World".into());
            dealer_server
                .send_multipart(multipart, SendFlags::empty())
                .unwrap();
        });

        let dealer_client = DealerSocket::from_context(&context)?;
        dealer_client.connect(client_endpoint)?;

        let multipart: Vec<Message> = vec![vec![].into(), "Hello".into()];
        dealer_client.send_multipart(multipart, SendFlags::empty())?;

        let mut response = dealer_client.recv_multipart(RecvFlags::empty())?;

        let content = response.pop_back().unwrap();
        assert!(!content.is_empty());
        assert_eq!(content.to_string(), "World");

        Ok(())
    }

    #[cfg(feature = "futures")]
    #[test]
    fn dealer_dealer_async() -> ZmqResult<()> {
        let context = Context::new()?;

        let dealer_server = DealerSocket::from_context(&context)?;
        dealer_server.bind("tcp://127.0.0.1:*")?;
        let client_endpoint = dealer_server.last_endpoint()?;

        std::thread::spawn(move || {
            let mut multipart = dealer_server.recv_multipart(RecvFlags::empty()).unwrap();

            let content = multipart.pop_back().unwrap();
            assert!(!content.is_empty());
            assert_eq!(content.to_string(), "Hello");

            multipart.push_back("World".into());
            dealer_server
                .send_multipart(multipart, SendFlags::empty())
                .unwrap();
        });

        let dealer_client = DealerSocket::from_context(&context)?;
        dealer_client.connect(client_endpoint)?;

        futures::executor::block_on(async {
            let multipart: Vec<Message> = vec![vec![].into(), "Hello".into()];
            dealer_client
                .send_multipart_async(multipart, SendFlags::empty())
                .await;

            let mut response = dealer_client.recv_multipart_async().await;

            let content = response.pop_back().unwrap();
            assert!(!content.is_empty());
            assert_eq!(content.to_string(), "World");

            Ok(())
        })
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::DealerSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "DealerBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`DealerSocket`].\n\n")]
    #[allow(dead_code)]
    struct DealerConfig {
        socket_builder: SocketBuilder,
        #[builder(default = false)]
        conflate: bool,
        #[cfg(feature = "draft-api")]
        #[doc(cfg(feature = "draft-api"))]
        #[builder(setter(into), default = "Default::default()")]
        hiccup_msg: String,
        #[cfg(feature = "draft-api")]
        #[doc(cfg(feature = "draft-api"))]
        #[builder(setter(into), default = "Default::default()")]
        hello_message: String,
        #[builder(setter(into), default = "Default::default()")]
        routing_id: String,
    }

    impl DealerBuilder {
        pub fn apply(self, socket: &DealerSocket) -> ZmqResult<()> {
            if let Some(socket_builder) = self.socket_builder {
                socket_builder.apply(socket)?;
            }

            self.conflate
                .iter()
                .try_for_each(|conflate| socket.set_conflate(*conflate))?;

            #[cfg(feature = "draft-api")]
            self.hiccup_msg
                .iter()
                .try_for_each(|hiccup_msg| socket.set_hiccup_message(hiccup_msg))?;

            #[cfg(feature = "draft-api")]
            self.hello_message
                .iter()
                .try_for_each(|hello_message| socket.set_hello_message(hello_message))?;

            self.routing_id
                .iter()
                .try_for_each(|routing_id| socket.set_routing_id(routing_id))?;

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<DealerSocket> {
            let socket = DealerSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }

    #[cfg(test)]
    mod dealer_builder_tests {
        use super::DealerBuilder;
        use crate::prelude::{Context, SocketBuilder, ZmqResult};

        #[test]
        fn default_dealer_builder() -> ZmqResult<()> {
            let context = Context::new()?;

            let socket = DealerBuilder::default().build_from_context(&context)?;
            assert!(!socket.conflate()?);
            assert_eq!(socket.routing_id()?, "");

            Ok(())
        }

        #[test]
        fn dealer_builder_with_custom_value() -> ZmqResult<()> {
            let context = Context::new()?;

            let socket_builder = SocketBuilder::default();

            let dealer_builder = DealerBuilder::default()
                .socket_builder(socket_builder)
                .conflate(true)
                .routing_id("test123");

            #[cfg(feature = "draft-api")]
            let dealer_builder = dealer_builder
                .hello_message("hello123")
                .hiccup_msg("hiccup123");

            let socket = dealer_builder.build_from_context(&context)?;

            assert!(socket.conflate()?);
            assert_eq!(socket.routing_id()?, "test123");

            Ok(())
        }
    }
}
