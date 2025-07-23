use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOption, SocketType},
};

/// # A peer socket `ZMQ_PEER`
///
/// A [`Peer`] socket talks to a set of [`Peer`] sockets.
///
/// To connect and fetch the 'routing_id' of the peer use [`connect_peer()`].
///
/// Each received message has a 'routing_id' that is a 32-bit unsigned integer. The application can
/// fetch this with [`routing_id()`].
///
/// To send a message to a given [`Peer`] peer the application must set the peer’s 'routing_id' on
/// the message, using [`set_routing_id()`].
///
/// If the 'routing_id' is not specified, or does not refer to a connected client peer, the send
/// call will fail with [`HostUnreachable`]. If the outgoing buffer for the peer is full, the send
/// call shall block, unless [`DONT_WAIT`] is used in the send, in which case it shall fail with
/// [`Again`]. The [`Peer`] socket shall not drop messages in any case.
///
/// [`Peer`]: PeerSocket
/// [`connect_peer()`]: #method.connect_peer
/// [`routing_id()`]: crate::message::Message::routing_id()
/// [`set_routing_id()`]: crate::message::Message::set_routing_id()
/// [`HostUnreachable`]: crate::ZmqError::HostUnreachable
/// [`Again`]: crate::ZmqError::Again
/// [`DONT_WAIT`]: super::SendFlags::DONT_WAIT
pub type PeerSocket = Socket<Peer>;

pub struct Peer {}

impl sealed::SenderFlag for Peer {}
impl sealed::ReceiverFlag for Peer {}

impl sealed::SocketType for Peer {
    fn raw_socket_type() -> SocketType {
        SocketType::Peer
    }
}

unsafe impl Sync for Socket<Peer> {}
unsafe impl Send for Socket<Peer> {}

impl Socket<Peer> {
    /// # create outgoing connection from socket and return the connection routing id in thread-safe and atomic way.
    ///
    /// The [`connect_peer()`] function connects a [`Peer`] socket to an 'endpoint' and then
    /// returns the endpoint [`routing_id()`].
    ///
    /// The 'endpoint' is a string consisting of a 'transport'`://` followed by an 'address'. The
    /// 'transport' specifies the underlying protocol to use. The 'address' specifies the
    /// transport-specific address to connect to.
    ///
    /// The function is supported only on the [`Peer`] socket type and would return
    /// `Err(`[`Unsupported`]`)` otherwise.
    ///
    /// The [`connect_peer()`] support the following transports:
    ///
    /// * `tcp` unicast transport using TCP
    /// * `ipc` local inter-process communication transport
    /// * `inproc` local in-process (inter-thread) communication transport
    /// * `ws` unicast transport using WebSockets
    /// * `wss` unicast transport using WebSockets over TLS
    ///
    /// [`Peer`]: PeerSocket
    /// [`connect_peer()`]: #method.connect_peer
    /// [`Unsupported`]: crate::ZmqError::Unsupported
    /// [`routing_id()`]: crate::message::Message::routing_id
    pub fn connect_peer<V>(&self, endpoint: V) -> ZmqResult<u32>
    where
        V: AsRef<str>,
    {
        self.socket.connect_peer(endpoint.as_ref())
    }

    /// # set a hiccup message that the socket will generate when connected peer temporarily disconnect `ZMQ_HICCUP_MSG`
    ///
    /// When set, the socket will generate a hiccup message when connect peer has been
    /// disconnected. You may set this on [`Dealer`], [`Client`] and [`Peer`] sockets. The
    /// combination with [`set_heartbeat_interval()`] is powerful and simplify protocols, when
    /// heartbeat recognize a connection drop it will generate a hiccup message that can match the
    /// protocol of the application.
    ///
    /// [`Dealer`]: super::DealerSocket
    /// [`Client`]: super::ClientSocket
    /// [`Peer`]: PeerSocket
    /// [`set_heartbeat_interval()`]: #method.set_heartbeat_interval
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
    /// [`Dealer`]: super::DealerSocket
    /// [`Router`]: super::RouterSocket
    /// [`Client`]: super::ClientSocket
    /// [`Server`]: super::ServerSocket
    /// [`Peer`]: PeerSocket
    /// [`set_heartbeat_interval()`]: #method.set_heartbeat_interval
    pub fn set_hello_message<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::HelloMessage, value)
    }

    /// # set a disconnect message that the socket will generate when accepted peer disconnect `ZMQ_DISCONNECT_MSG`
    ///
    /// When set, the socket will generate a disconnect message when accepted peer has been
    /// disconnected. You may set this on [`Router`], [`Server`] and [`Peer`] sockets. The
    /// combination with [`set_heartbeat_interval()`] is powerful and simplify protocols, when heartbeat
    /// recognize a connection drop it will generate a disconnect message that can match the
    /// protocol of the application.
    ///
    /// [`Router`]: super::RouterSocket
    /// [`Server`]: super::ServerSocket
    /// [`Peer`]: PeerSocket
    /// [`set_heartbeat_interval()`]: #method.set_heartbeat_interval
    pub fn set_disconnect_message<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::DisconnectMessage, value)
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::PeerSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "PeerBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`PeerSocket`].\n\n")]
    #[allow(dead_code)]
    struct PeerConfig {
        socket_config: SocketBuilder,
        #[builder(setter(into), default = "Default::default()")]
        hiccup_msg: String,
        #[builder(setter(into), default = "Default::default()")]
        hello_message: String,
        #[builder(setter(into), default = "Default::default()")]
        disconnect_message: String,
    }

    impl PeerBuilder {
        pub fn apply(self, socket: &PeerSocket) -> ZmqResult<()> {
            if let Some(socket_config) = self.socket_config {
                socket_config.apply(socket)?;
            }

            if let Some(hiccup_message) = self.hiccup_msg {
                socket.set_hiccup_message(hiccup_message)?;
            }

            if let Some(hello_message) = self.hello_message {
                socket.set_hello_message(hello_message)?;
            }

            if let Some(disconnect_message) = self.disconnect_message {
                socket.set_disconnect_message(disconnect_message)?;
            }

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<PeerSocket> {
            let socket = PeerSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
}
