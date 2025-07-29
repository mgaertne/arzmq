use crate::{
    ZmqResult, sealed,
    socket::{MultipartReceiver, Socket, SocketOption, SocketType},
};

/// # A pull socket `ZMQ_PULL`
///
/// A socket of type [`Pull`] is used by a pipeline node to receive messages from upstream pipeline
/// nodes. Messages are fair-queued from among all connected upstream nodes. The `send_msg()`
/// function is not implemented for this socket type.
///
/// [`Pull`]: PullSocket
pub type PullSocket = Socket<Pull>;

pub struct Pull {}

impl sealed::ReceiverFlag for Pull {}

unsafe impl Sync for Socket<Pull> {}
unsafe impl Send for Socket<Pull> {}

impl sealed::SocketType for Pull {
    fn raw_socket_type() -> SocketType {
        SocketType::Pull
    }
}

impl MultipartReceiver for Socket<Pull> {}

impl Socket<Pull> {
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
    /// If [`recv_msg()`] is not called on the inbound socket, the queue and memory will grow with
    /// each message received. Use [`events()`] to trigger the conflation of the messages.
    ///
    /// [`receive_highwater_mark()`]: #method.receive_highwater_mark
    /// [`send_highwater_mark()`]: #method.send_highwater_mark
    /// [`recv_msg()`]: #method.recv_msg
    /// [`events()`]: #method.events
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::Conflate, value)
    }
}

#[cfg(test)]
mod pull_tests {
    use super::PullSocket;
    use crate::prelude::{
        Context, PushSocket, Receiver, RecvFlags, SendFlags, Sender, SocketOption, ZmqResult,
    };

    #[test]
    fn set_conflate_sets_conflate() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PullSocket::from_context(&context)?;
        socket.set_conflate(true)?;

        assert!(socket.get_sockopt_bool(SocketOption::Conflate)?);

        Ok(())
    }

    #[test]
    fn push_pull() -> ZmqResult<()> {
        let context = Context::new()?;

        let push = PushSocket::from_context(&context)?;
        push.bind("tcp://127.0.0.1:*")?;
        let pull_endpoint = push.last_endpoint()?;

        std::thread::spawn(move || {
            loop {
                push.send_msg("Hello", SendFlags::empty()).unwrap();
            }
        });

        let pull = PullSocket::from_context(&context)?;
        pull.connect(pull_endpoint)?;

        let msg = pull.recv_msg(RecvFlags::empty())?;
        assert_eq!(msg.to_string(), "Hello");

        Ok(())
    }

    #[cfg(feature = "futures")]
    #[test]
    fn push_pull_async() -> ZmqResult<()> {
        let context = Context::new()?;

        let push = PushSocket::from_context(&context)?;
        push.bind("tcp://127.0.0.1:*")?;
        let pull_endpoint = push.last_endpoint()?;

        std::thread::spawn(move || {
            futures::executor::block_on(async {
                loop {
                    push.send_msg_async("Hello", SendFlags::empty()).await;
                }
            })
        });

        let pull = PullSocket::from_context(&context)?;
        pull.connect(pull_endpoint)?;

        futures::executor::block_on(async {
            loop {
                if let Some(msg) = pull.recv_msg_async().await {
                    assert_eq!(msg.to_string(), "Hello");
                    break;
                }
            }

            Ok(())
        })
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::PullSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "PullBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`PullSocket`].\n\n")]
    #[allow(dead_code)]
    struct PullConfig {
        socket_builder: SocketBuilder,
        #[builder(default = false)]
        conflate: bool,
    }

    impl PullBuilder {
        pub fn apply(self, socket: &PullSocket) -> ZmqResult<()> {
            if let Some(socket_builder) = self.socket_builder {
                socket_builder.apply(socket)?;
            }

            self.conflate
                .iter()
                .try_for_each(|conflate| socket.set_conflate(*conflate))?;

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<PullSocket> {
            let socket = PullSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }

    #[cfg(test)]
    mod pull_builder_tests {
        use super::PullBuilder;
        use crate::prelude::{Context, SocketBuilder, SocketOption, ZmqResult};

        #[test]
        fn default_pull_builder() -> ZmqResult<()> {
            let context = Context::new()?;

            let socket = PullBuilder::default().build_from_context(&context)?;

            assert!(!socket.get_sockopt_bool(SocketOption::Conflate)?);

            Ok(())
        }

        #[test]
        fn pull_builder_with_custom_settings() -> ZmqResult<()> {
            let context = Context::new()?;

            let socket = PullBuilder::default()
                .socket_builder(SocketBuilder::default())
                .conflate(true)
                .build_from_context(&context)?;

            assert!(socket.get_sockopt_bool(SocketOption::Conflate)?);

            Ok(())
        }
    }
}
