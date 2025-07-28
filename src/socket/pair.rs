use crate::{
    sealed,
    socket::{MultipartReceiver, MultipartSender, Socket, SocketType},
};

/// # A pair socket `ZMQ_PAIR`
///
/// A socket of type [`Pair`] can only be connected to a single peer at any one time. No message
/// routing or filtering is performed on messages sent over a [`Pair`] socket.
///
/// When a [`Pair`] socket enters the 'mute' state due to having reached the high water mark for
/// the connected peer, or, for connection-oriented transports, if the
/// [`immediate()`] option is set and there is no connected peer, then any
/// [`send_msg()`] operations on the socket shall block until the peer becomes available for
/// sending; messages are not discarded.
///
/// While [`Pair`] sockets can be used over transports other than `inproc`, their inability to
/// auto-reconnect coupled with the fact new incoming connections will be terminated while any
/// previous connections (including ones in a closing state) exist makes them unsuitable for TCP
/// in most cases.
///
/// <div class="warning">
///
/// [`Pair`] sockets are designed for inter-thread communication across the `inproc` transport
/// and do not implement functionality such as auto-reconnection.
///
/// </div>
///
/// [`Pair`]: PairSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #impl-Sender-for-Socket<T>
pub type PairSocket = Socket<Pair>;

pub struct Pair {}

impl sealed::SenderFlag for Pair {}
impl sealed::ReceiverFlag for Pair {}

impl sealed::SocketType for Pair {
    fn raw_socket_type() -> SocketType {
        SocketType::Pair
    }
}

unsafe impl Sync for Socket<Pair> {}
unsafe impl Send for Socket<Pair> {}

impl MultipartSender for Socket<Pair> {}
impl MultipartReceiver for Socket<Pair> {}

impl Socket<Pair> {}

#[cfg(test)]
mod pair_tests {
    use super::PairSocket;
    use crate::prelude::{Context, Receiver, RecvFlags, SendFlags, Sender, ZmqResult};

    #[test]
    fn pair_pair() -> ZmqResult<()> {
        let endpoint = "inproc://pair-test";

        let context = Context::new()?;

        let pair_server = PairSocket::from_context(&context)?;
        pair_server.bind(endpoint)?;

        std::thread::spawn(move || {
            let msg = pair_server.recv_msg(RecvFlags::empty()).unwrap();

            assert_eq!(msg.to_string(), "Hello");
            pair_server.send_msg("World", SendFlags::empty()).unwrap();
        });

        let pair_client = PairSocket::from_context(&context)?;
        pair_client.connect(endpoint)?;

        pair_client.send_msg("Hello", SendFlags::empty())?;
        let msg = pair_client.recv_msg(RecvFlags::empty())?;

        assert_eq!(msg.to_string(), "World");

        Ok(())
    }

    #[test]
    fn pair_pair_async() -> ZmqResult<()> {
        let endpoint = "inproc://pair-test";

        let context = Context::new()?;

        let pair_server = PairSocket::from_context(&context)?;
        pair_server.bind(endpoint)?;

        std::thread::spawn(move || {
            let msg = pair_server.recv_msg(RecvFlags::empty()).unwrap();

            assert_eq!(msg.to_string(), "Hello");
            pair_server.send_msg("World", SendFlags::empty()).unwrap();
        });

        let pair_client = PairSocket::from_context(&context)?;
        pair_client.connect(endpoint)?;

        futures::executor::block_on(async {
            pair_client
                .send_msg_async("Hello", SendFlags::empty())
                .await;

            loop {
                if let Some(msg) = pair_client.recv_msg_async().await {
                    assert_eq!(msg.to_string(), "World");
                    break;
                }
            }

            Ok(())
        })
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use crate::socket::SocketBuilder;

    /// Builder for [`PairSocket`](super::PairSocket)
    pub type PairBuilder = SocketBuilder;

    #[cfg(test)]
    mod pair_builder_tests {
        use super::PairBuilder;
        use crate::{
            auth::ZapDomain,
            prelude::{Context, PairSocket, SocketOption, ZmqResult},
            security::SecurityMechanism,
        };

        #[test]
        fn builder_from_default() -> ZmqResult<()> {
            let context = Context::new()?;

            let socket: PairSocket = PairBuilder::default().build_from_context(&context)?;

            assert_eq!(socket.connect_timeout()?, 0);
            assert_eq!(socket.handshake_interval()?, 30_000);
            assert_eq!(
                socket.get_sockopt_int::<i32>(SocketOption::HeartbeatInterval)?,
                0
            );
            assert_eq!(
                socket.get_sockopt_int::<i32>(SocketOption::HeartbeatTimeout)?,
                -1
            );
            assert_eq!(
                socket.get_sockopt_int::<i32>(SocketOption::HeartbeatTimeToLive)?,
                0
            );
            assert!(!socket.immediate()?);
            assert!(!socket.ipv6()?);
            assert_eq!(socket.linger()?, -1);
            assert_eq!(socket.max_message_size()?, -1);
            assert_eq!(socket.receive_buffer()?, -1);
            assert_eq!(socket.receive_highwater_mark()?, 1_000);
            assert_eq!(socket.receive_timeout()?, -1);
            assert_eq!(socket.reconnect_interval()?, 100);
            assert_eq!(socket.reconnect_interval_max()?, 0);
            assert_eq!(socket.send_buffer()?, -1);
            assert_eq!(socket.send_highwater_mark()?, 1_000);
            assert_eq!(socket.send_timeout()?, -1);
            assert_eq!(socket.zap_domain()?, ZapDomain::new("".into()));
            assert_eq!(socket.security_mechanism()?, SecurityMechanism::Null);

            Ok(())
        }
    }
}
