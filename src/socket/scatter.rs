use crate::{
    sealed,
    socket::{Socket, SocketType},
};

/// # A radio socket `ZMQ_SCATTER`
///
/// A socket of type [`Scatter`] is used by a scatter-gather node to send messages to downstream
/// scatter-gather nodes. Messages are round-robined to all connected downstream nodes.
///
/// When a [`Scatter`] socket enters the 'mute' state due to having reached the high water mark
/// for all downstream nodes, or, for connection-oriented transports, if the [`immediate()`]
/// option is set and there are no downstream nodes at all, then any [`send_msg()`] operations on
/// the socket shall block until the mute state ends or at least one downstream node becomes
/// available for sending; messages are not discarded.
///
/// [`Scatter`]: ScatterSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #method.send_msg
pub type ScatterSocket = Socket<Scatter>;

pub struct Scatter {}

impl sealed::SenderFlag for Scatter {}
impl sealed::SocketType for Scatter {
    fn raw_socket_type() -> SocketType {
        SocketType::Scatter
    }
}

unsafe impl Sync for Socket<Scatter> {}
unsafe impl Send for Socket<Scatter> {}

impl Socket<Scatter> {}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use crate::socket::SocketBuilder;

    /// Builder for [`ScatterSocket`](super::ScatterSocket)
    pub type ScatterBuilder = SocketBuilder;

    #[cfg(test)]
    mod scatter_builder_tests {
        use super::ScatterBuilder;
        use crate::{
            auth::ZapDomain,
            prelude::{Context, ScatterSocket, SocketOption, ZmqResult},
            security::SecurityMechanism,
        };

        #[test]
        fn builder_from_default() -> ZmqResult<()> {
            let context = Context::new()?;

            let socket: ScatterSocket = ScatterBuilder::default().build_from_context(&context)?;

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
