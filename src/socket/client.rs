use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOption, SocketType},
};

/// # A client socket `ZMQ_CLIENT`
///
/// A [`Client`] socket talks to a [`Server`] socket. Either peer can connect, though the usual and
/// recommended model is to bind the [`Server`] and connect the [`Client`].
///
/// If the [`Client`] socket has established a connection, [`send_msg()`] will accept messages,
/// queue them, and send them as rapidly as the network allows. The outgoing buffer limit is
/// defined by the high water mark for the socket. If the outgoing buffer is full, or, for
/// connection-oriented transports, if the [`immediate()`] option is set and there is no connected
/// peer, [`send_msg()`] will block. The [`Client`] socket will not drop messages.
///
/// When a [`Client`] socket is connected to multiple [`Server`] sockets, outgoing messages are
/// distributed between connected peers on a round-robin basis. Likewise, the [`Client`] socket
/// receives messages fairly from each connected peer. This usage is sensible only for stateless
/// protocols.
///
/// [`Client`] sockets are threadsafe and can be used from multiple threads at the same time. Note
/// that replies from a [`Server`] socket will go to the first client thread that calls
/// [`recv_msg()`]. If you need to get replies back to the originating thread, use one [`Client`]
/// socket per thread.
///
/// [`Client`]: ClientSocket
/// [`Server`]: super::ServerSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #method.send_msg
/// [`recv_msg()`]: #method.recv_msg
pub type ClientSocket = Socket<Client>;

pub struct Client {}

impl sealed::SenderFlag for Client {}
impl sealed::ReceiverFlag for Client {}

impl sealed::SocketType for Client {
    fn raw_socket_type() -> SocketType {
        SocketType::Client
    }
}

unsafe impl Sync for Socket<Client> {}
unsafe impl Send for Socket<Client> {}

impl Socket<Client> {
    /// # set a hiccup message that the socket will generate when connected peer temporarily disconnect `ZMQ_HICCUP_MSG`
    ///
    /// When set, the socket will generate a hiccup message when connect peer has been
    /// disconnected. You may set this on [`Dealer`], [`Client`] and [`Peer`] sockets. The
    /// combination with [`set_heartbeat_interval()`] is powerful and simplify protocols, when
    /// heartbeat recognize a connection drop it will generate a hiccup message that can match the
    /// protocol of the application.
    ///
    /// [`Dealer`]: super::DealerSocket
    /// [`Client`]: ClientSocket
    /// [`Peer`]: super::PeerSocket
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
    /// [`Client`]: ClientSocket
    /// [`Server`]: super::ServerSocket
    /// [`Peer`]: super::PeerSocket
    /// [`set_heartbeat_interval()`]: #method.set_heartbeat_interval
    pub fn set_hello_message<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::HelloMessage, value)
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::ClientSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "ClientBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`ClientSocket`].\n\n")]
    #[allow(dead_code)]
    struct ClientConfig {
        socket_config: SocketBuilder,
        #[builder(setter(into), default)]
        hiccup_msg: String,
        #[builder(setter(into), default)]
        hello_message: String,
    }

    impl ClientBuilder {
        pub fn apply(self, socket: &ClientSocket) -> ZmqResult<()> {
            if let Some(socket_config) = self.socket_config {
                socket_config.apply(socket)?;
            }

            if let Some(hiccup_message) = self.hiccup_msg {
                socket.set_hiccup_message(hiccup_message)?;
            }

            if let Some(hello_message) = self.hello_message {
                socket.set_hello_message(hello_message)?;
            }

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<ClientSocket> {
            let socket = ClientSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
}
