use crate::{
    sealed,
    socket::{Socket, SocketType},
};

/// # A gather socket `ZMQ_GATHER`
///
/// A socket of type [`Gather`] is used by a scatter-gather node to receive messages from upstream
/// scatter-gather nodes. Messages are fair-queued from among all connected upstream nodes.
///
/// [`Gather`]: GatherSocket
pub type GatherSocket = Socket<Gather>;

pub struct Gather {}

impl sealed::ReceiverFlag for Gather {}
impl sealed::SocketType for Gather {
    fn raw_socket_type() -> SocketType {
        SocketType::Gather
    }
}

unsafe impl Sync for Socket<Gather> {}
unsafe impl Send for Socket<Gather> {}

impl Socket<Gather> {}

#[cfg(test)]
mod gather_tests {
    use super::GatherSocket;
    use crate::prelude::{
        Context, Receiver, RecvFlags, ScatterSocket, SendFlags, Sender, ZmqResult,
    };

    #[test]
    fn scatter_gather() -> ZmqResult<()> {
        let context = Context::new()?;

        let scatter = ScatterSocket::from_context(&context)?;
        scatter.bind("tcp://127.0.0.1:*")?;
        let gather_endpoint = scatter.last_endpoint()?;

        std::thread::spawn(move || {
            loop {
                scatter.send_msg("asdf", SendFlags::empty()).unwrap();
            }
        });

        let gather = GatherSocket::from_context(&context)?;
        gather.connect(gather_endpoint)?;

        let msg = gather.recv_msg(RecvFlags::empty())?;
        assert_eq!(msg.to_string(), "asdf");

        Ok(())
    }

    #[cfg(feature = "futures")]
    #[test]
    fn scatter_gather_async() -> ZmqResult<()> {
        let context = Context::new()?;

        let scatter = ScatterSocket::from_context(&context)?;
        scatter.bind("tcp://127.0.0.1:*")?;
        let gather_endpoint = scatter.last_endpoint()?;

        std::thread::spawn(move || {
            futures::executor::block_on(async {
                loop {
                    scatter.send_msg_async("asdf", SendFlags::empty()).await;
                }
            })
        });

        let gather = GatherSocket::from_context(&context)?;
        gather.connect(gather_endpoint)?;

        futures::executor::block_on(async {
            loop {
                if let Some(msg) = gather.recv_msg_async().await {
                    assert_eq!(msg.to_string(), "asdf");
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

    /// Builder for [`GatherSocket`](super::GatherSocket)
    pub type GatherBuilder = SocketBuilder;

    #[cfg(test)]
    mod gather_builder_tests {
        use super::GatherBuilder;
        use crate::{
            auth::ZapDomain,
            prelude::{Context, GatherSocket, SocketOption, ZmqResult},
            security::SecurityMechanism,
        };

        #[test]
        fn builder_from_default() -> ZmqResult<()> {
            let context = Context::new()?;

            let socket: GatherSocket = GatherBuilder::default().build_from_context(&context)?;

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
