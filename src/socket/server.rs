use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOption, SocketType},
};

/// # A server socket `ZMQ_SERVER`
///
/// A [`Server`] socket talks to a set of [`Client`] sockets. A [`Server`] socket can only reply to
/// an incoming message: the [`Client`] peer must always initiate a conversation.
///
/// Each received message has a [`routing_id()`] that is a 32-bit unsigned integer. The application
/// can fetch this with [`routing_id()`] To send a message to a given [`Client`] peer the
/// application must set the peer’s [`routing_id()`] on the message, using [`set_routing_id()`].
///
/// If the [`routing_id()`] is not specified, or does not refer to a connected client peer, the
/// send call will fail with [`HostUnreachable`]. If the outgoing buffer for the client peer is
/// full, the send call shall block, unless [`DONT_WAIT`] is used in the send, in which case it
/// shall fail with [`Again`]]. The [`Server`] socket shall not drop messages in any case.
///
/// [`Client`]: super::ClientSocket
/// [`Server`]: ServerSocket
/// [`routing_id()`]: crate::message::Message::routing_id
/// [`set_routing_id()`]: crate::message::Message::set_routing_id
/// [`HostUnreachable`]: crate::ZmqError::HostUnreachable
/// [`Again`]: crate::ZmqError::Again
/// [`DONT_WAIT`]: super::SendFlags::DONT_WAIT
pub type ServerSocket = Socket<Server>;

pub struct Server {}

impl sealed::SenderFlag for Server {}
impl sealed::ReceiverFlag for Server {}

impl sealed::SocketType for Server {
    fn raw_socket_type() -> SocketType {
        SocketType::Server
    }
}

unsafe impl Sync for Socket<Server> {}
unsafe impl Send for Socket<Server> {}

impl Socket<Server> {
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
    /// [`Server`]: ServerSocket
    /// [`Peer`]: super::PeerSocket
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
    /// [`Server`]: ServerSocket
    /// [`Peer`]: super::PeerSocket
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

    use super::ServerSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "ServerBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`ServerSocket`].\n\n")]
    #[allow(dead_code)]
    struct ServerConfig {
        socket_config: SocketBuilder,
        #[builder(setter(into), default = "Default::default()")]
        hello_message: String,
        #[builder(setter(into), default = "Default::default()")]
        disconnect_message: String,
    }

    impl ServerBuilder {
        pub fn apply(self, socket: &ServerSocket) -> ZmqResult<()> {
            if let Some(socket_config) = self.socket_config {
                socket_config.apply(socket)?;
            }

            if let Some(hello_message) = self.hello_message {
                socket.set_hello_message(hello_message)?;
            }

            if let Some(disconnect_message) = self.disconnect_message {
                socket.set_disconnect_message(disconnect_message)?;
            }

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<ServerSocket> {
            let socket = ServerSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
}
