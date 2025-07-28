use crate::{
    sealed,
    socket::{Socket, SocketType},
};

/// # A channel socket `ZMQ_CHANNEL`
///
/// A socket of type [`Channel`] can only be connected to a single peer at any one time. No
/// message routing or filtering is performed on messages sent over a [`Channel`] socket.
///
/// When a [`Channel`] socket enters the 'mute' state due to having reached the high water mark
/// for the connected peer, or, for connection-oriented transports, if the [`immediate()`]
/// option is set and there is no connected peer, then any [`send_msg()`] operation on the socket
/// shall block until the peer becomes available for sending; messages are not discarded.
///
/// While [`Channel`] sockets can be used over transports other than `inproc`, their inability to
/// auto-reconnect coupled with the fact new incoming connections will be terminated while any
/// previous connections (including ones in a closing state) exist makes them unsuitable for TCP
/// in most cases.
///
/// [`Channel`]: ChannelSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #method.send_msg
pub type ChannelSocket = Socket<Channel>;

pub struct Channel {}

impl sealed::SenderFlag for Channel {}
impl sealed::ReceiverFlag for Channel {}

impl sealed::SocketType for Channel {
    fn raw_socket_type() -> SocketType {
        SocketType::Channel
    }
}

unsafe impl Sync for Socket<Channel> {}
unsafe impl Send for Socket<Channel> {}

impl Socket<Channel> {}

#[cfg(test)]
mod channel_tests {
    use super::ChannelSocket;
    use crate::prelude::{Context, Receiver, RecvFlags, SendFlags, Sender, ZmqResult};

    #[test]
    fn channel_channel() -> ZmqResult<()> {
        let endpoint = "inproc://channel_channel";

        let context = Context::new()?;

        let channel_server = ChannelSocket::from_context(&context)?;
        channel_server.bind(endpoint)?;

        std::thread::spawn(move || {
            let msg = channel_server.recv_msg(RecvFlags::empty()).unwrap();
            assert_eq!(msg.to_string(), "Hello");

            channel_server
                .send_msg("World", SendFlags::empty())
                .unwrap();
        });

        let channel_client = ChannelSocket::from_context(&context)?;
        channel_client.connect(endpoint)?;

        channel_client
            .send_msg("Hello", SendFlags::empty())
            .unwrap();
        let msg = channel_client.recv_msg(RecvFlags::empty()).unwrap();
        assert_eq!(msg.to_string(), "World");

        Ok(())
    }

    #[cfg(feature = "futures")]
    #[test]
    fn channel_channel_async() -> ZmqResult<()> {
        let endpoint = "inproc://channel_channel";

        let context = Context::new()?;

        let channel_server = ChannelSocket::from_context(&context)?;
        channel_server.bind(endpoint)?;

        std::thread::spawn(move || {
            let msg = channel_server.recv_msg(RecvFlags::empty()).unwrap();
            assert_eq!(msg.to_string(), "Hello");

            channel_server
                .send_msg("World", SendFlags::empty())
                .unwrap();
        });

        let channel_client = ChannelSocket::from_context(&context)?;
        channel_client.connect(endpoint)?;

        futures::executor::block_on(async {
            channel_client
                .send_msg_async("Hello", SendFlags::empty())
                .await;
            loop {
                if let Some(msg) = channel_client.recv_msg_async().await {
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

    /// Builder for [`ChannelSocket`](super::ChannelSocket)
    pub type ChannelBuilder = SocketBuilder;

    #[cfg(test)]
    mod channel_builder_tests {
        use super::ChannelBuilder;
        use crate::{
            auth::ZapDomain,
            prelude::{ChannelSocket, Context, SocketOption, ZmqResult},
            security::SecurityMechanism,
        };

        #[test]
        fn builder_from_default() -> ZmqResult<()> {
            let context = Context::new()?;

            let socket: ChannelSocket = ChannelBuilder::default().build_from_context(&context)?;

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
