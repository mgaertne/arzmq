use super::{MultipartReceiver, Socket, SocketOption, SocketType};
use crate::{ZmqResult, sealed};

/// # A Subscriber socket `ZMQ_SUB`
///
/// A socket of type [`Subscribe`] is used by a subscriber to subscribe to data distributed by a
/// [`Publish`]. Initially a [`Subscribe`] socket is not subscribed to any messages, use the
/// [`subscribe()`] function specify which messages to subscribe to.
///
/// [`Subscribe`]: SubscribeSocket
/// [`Publish`]: super::PublishSocket
/// [`subscribe()`]: #method.subscribe
pub type SubscribeSocket = Socket<Subscribe>;

pub struct Subscribe {}

impl sealed::ReceiverFlag for Subscribe {}

unsafe impl Sync for Socket<Subscribe> {}
unsafe impl Send for Socket<Subscribe> {}

impl sealed::SocketType for Subscribe {
    fn raw_socket_type() -> SocketType {
        SocketType::Subscribe
    }
}

impl MultipartReceiver for Socket<Subscribe> {}

impl Socket<Subscribe> {
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

    /// # Invert message filtering `ZMQ_INVERT_MATCHING`
    ///
    /// Reverses the filtering behavior of [`Publish`]-[`Subscribe`] sockets, when set to `true`.
    ///
    /// On [`Publish`] and [`XPublish`] sockets, this causes messages to be sent to all connected
    /// sockets *except* those subscribed to a prefix that matches the message. On [`Subscribe`]
    /// sockets, this causes only incoming messages that do *not* match any of the socket’s
    /// subscriptions to be received by the user.
    ///
    /// Whenever `ZMQ_INVERT_MATCHING` is set to `true` on a [`Publish`] socket, all [`Subscribe`]
    /// sockets connecting to it must also have the option set to `true`. Failure to do so will
    /// have the [`Subscribe`] sockets reject everything the [`Publish`] socket sends them.
    /// [`XSubscribe`] sockets do not need to do this because they do not filter incoming messages.
    ///
    /// [`Subscribe`]: SubscribeSocket
    /// [`Publish`]: super::PublishSocket
    /// [`XPublish`]: super::XPublishSocket
    /// [`XSubscribe`]: super::XSubscribeSocket
    pub fn set_invert_matching(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::InvertMatching, value)
    }

    /// # Retrieve inverted filtering status `ZMQ_INVERT_MATCHING`
    ///
    /// Returns the value of the `ZMQ_INVERT_MATCHING` option. A value of `true` means the socket
    /// uses inverted prefix matching.
    ///
    /// On [`Publish`] and [`XPublish`] sockets, this causes messages to be sent to all connected
    /// sockets *except* those subscribed to a prefix that matches the message. On [`Subscribe`]
    /// sockets, this causes only incoming messages that do *not* match any of the socket’s
    /// subscriptions to be received by the user.
    ///
    /// Whenever `ZMQ_INVERT_MATCHING` is set to `true` on a [`Publish`] socket, all [`Publish`]
    /// sockets connecting to it must also have the option set to `true`. Failure to do so will
    /// have the [`Subscribe`] sockets reject everything the [`Publish`] socket sends them.
    /// [`XSubscribe`] sockets do not need to do this because they do not filter incoming messages.
    ///
    /// [`Subscribe`]: SubscribeSocket
    /// [`Publish`]: super::PublishSocket
    /// [`XPublish`]: super::XPublishSocket
    /// [`XSubscribe`]: super::XSubscribeSocket
    pub fn invert_matching(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::InvertMatching)
    }

    /// # Establish message filter `ZMQ_SUBSCRIBE`
    ///
    /// The [`subscribe()`] option shall establish a new message filter on a [`Subscriber`] socket.
    /// Newly created [`Subscriber`] sockets shall filter out all incoming messages, therefore you
    /// should call this option to establish an initial message filter.
    ///
    /// An empty `topic` of length zero shall subscribe to all incoming messages. A non-empty
    /// `topic` shall subscribe to all messages beginning with the specified prefix. Multiple
    /// filters may be attached to a single [`Subscriber`] socket, in which case a message shall
    /// be accepted if it matches at least one filter.
    ///
    /// [`Subscriber`]: SubscribeSocket
    /// [`subscribe()`]: #method.subscribe
    pub fn subscribe<V>(&self, topic: V) -> ZmqResult<()>
    where
        V: AsRef<[u8]>,
    {
        self.set_sockopt_bytes(SocketOption::Subscribe, topic.as_ref())
    }

    /// # Remove message filter `ZMQ_UNSUBSCRIBE`
    ///
    /// The [`unsubscribe()`] option shall remove an existing message filter on a [`Subscriber`]
    /// socket. The filter specified must match an existing filter previously established with the
    /// [`subscribe()`] option. If the socket has several instances of the same filter attached
    /// the [`unsubscribe()`] option shall remove only one instance, leaving the rest in place and
    /// functional.
    ///
    /// [`Subscriber`]: SubscribeSocket
    /// [`subscribe()`]: #method.subscribe
    /// [`unsubscribe()`]: #method.unsubscribe
    pub fn unsubscribe<V>(&self, topic: V) -> ZmqResult<()>
    where
        V: AsRef<[u8]>,
    {
        self.set_sockopt_bytes(SocketOption::Unsubscribe, topic.as_ref())
    }

    /// # Number of topic subscriptions received `ZMQ_TOPICS_COUNT`
    ///
    /// Gets the number of topic (prefix) subscriptions either
    ///
    /// * received on a [`Publish`]/[`XPublish`] socket from all the connected
    ///   [`Subscribe`]/[`XSubscribe`] sockets or
    /// * acknowledged on an [`Publish`]/[`XPublish`] socket from all the connected
    ///   [`Subscribe`]/[`XSubscribe`] sockets
    ///
    /// [`Subscribe`]: SubscribeSocket
    /// [`Publish`]: super::PublishSocket
    /// [`XPublish`]: super::XPublishSocket
    /// [`XSubscribe`]: super::XSubscribeSocket
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn topic_count(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TopicsCount)
    }
}

#[cfg(test)]
mod subscribe_tests {
    use super::SubscribeSocket;
    use crate::{
        prelude::{Context, PublishSocket, Receiver, SendFlags, Sender, SocketOption, ZmqResult},
        socket::RecvFlags,
    };

    #[test]
    fn set_conflate_sets_conflate() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = SubscribeSocket::from_context(&context)?;
        socket.set_conflate(true)?;

        assert!(socket.get_sockopt_bool(SocketOption::Conflate)?);

        Ok(())
    }

    #[test]
    fn set_invert_matching_sets_invert_matching() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = SubscribeSocket::from_context(&context)?;
        socket.set_invert_matching(true)?;

        assert!(socket.invert_matching()?);

        Ok(())
    }

    #[test]
    fn subscribe_sets_subscribe() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = SubscribeSocket::from_context(&context)?;
        socket.subscribe("topic")?;

        Ok(())
    }

    #[test]
    fn unsubscribe_sets_unsubscribe() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = SubscribeSocket::from_context(&context)?;
        socket.subscribe("topic")?;
        socket.unsubscribe("topic")?;

        Ok(())
    }

    #[test]
    fn unsubscribe_when_not_subscribed() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = SubscribeSocket::from_context(&context)?;
        socket.unsubscribe("topic")?;

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn topic_count_with_no_subscriptions() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = SubscribeSocket::from_context(&context)?;

        assert_eq!(socket.topic_count()?, 0);

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn topic_count_when_subscribed() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = SubscribeSocket::from_context(&context)?;
        socket.subscribe("topic")?;
        socket.subscribe("topic2")?;

        assert_eq!(socket.topic_count()?, 2);

        Ok(())
    }

    #[test]
    fn publish_subscribe() -> ZmqResult<()> {
        let context = Context::new()?;

        let publish = PublishSocket::from_context(&context)?;
        publish.bind("tcp://127.0.0.1:*")?;
        let subscribe_endpoint = publish.last_endpoint()?;

        std::thread::spawn(move || {
            loop {
                publish.send_msg("topic asdf", SendFlags::empty()).unwrap();
            }
        });

        let subscribe = SubscribeSocket::from_context(&context)?;
        subscribe.connect(&subscribe_endpoint)?;
        subscribe.subscribe("topic")?;

        let msg = subscribe.recv_msg(RecvFlags::empty())?;

        assert_eq!(msg.to_string().split_once(' ').unwrap(), ("topic", "asdf"));

        Ok(())
    }

    #[cfg(feature = "futures")]
    #[test]
    fn publish_subscribe_async() -> ZmqResult<()> {
        let context = Context::new()?;

        let publish = PublishSocket::from_context(&context)?;
        publish.bind("tcp://127.0.0.1:*")?;
        let subscribe_endpoint = publish.last_endpoint()?;

        std::thread::spawn(move || {
            futures::executor::block_on(async {
                loop {
                    publish
                        .send_msg_async("topic asdf", SendFlags::empty())
                        .await;
                }
            })
        });

        let subscribe = SubscribeSocket::from_context(&context)?;
        subscribe.connect(&subscribe_endpoint)?;
        subscribe.subscribe("topic")?;

        futures::executor::block_on(async {
            loop {
                if let Some(msg) = subscribe.recv_msg_async().await {
                    assert_eq!(msg.to_string().split_once(' ').unwrap(), ("topic", "asdf"));
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

    use super::SubscribeSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "SubscribeBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`SubscribeSocket`].\n\n")]
    #[allow(dead_code)]
    struct SubscribeConfig {
        socket_builder: SocketBuilder,
        #[builder(default = false)]
        conflate: bool,
        #[builder(default = false)]
        invert_matching: bool,
        #[builder(setter(into), default = "Default::default()")]
        subscribe: String,
    }

    impl SubscribeBuilder {
        pub fn apply(self, socket: &SubscribeSocket) -> ZmqResult<()> {
            if let Some(socket_builder) = self.socket_builder {
                socket_builder.apply(socket)?;
            }

            self.conflate
                .iter()
                .try_for_each(|conflate| socket.set_conflate(*conflate))?;

            self.invert_matching
                .iter()
                .try_for_each(|invert_matching| socket.set_invert_matching(*invert_matching))?;

            self.subscribe
                .iter()
                .try_for_each(|subscribe| socket.subscribe(subscribe.as_bytes()))?;

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<SubscribeSocket> {
            let socket = SubscribeSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }

    #[cfg(test)]
    mod subscribe_builder_tests {
        use super::SubscribeBuilder;
        use crate::prelude::{Context, SocketBuilder, SocketOption, ZmqResult};

        #[test]
        fn default_subscribe_builder() -> ZmqResult<()> {
            let context = Context::new()?;

            let socket = SubscribeBuilder::default().build_from_context(&context)?;

            assert!(!socket.get_sockopt_bool(SocketOption::Conflate)?);
            assert!(!socket.invert_matching()?);

            Ok(())
        }

        #[test]
        fn subscribe_builder_with_custom_values() -> ZmqResult<()> {
            let context = Context::new()?;

            let socket = SubscribeBuilder::default()
                .socket_builder(SocketBuilder::default())
                .conflate(true)
                .invert_matching(true)
                .subscribe("topic")
                .build_from_context(&context)?;

            assert!(socket.get_sockopt_bool(SocketOption::Conflate)?);
            assert!(socket.invert_matching()?);

            Ok(())
        }
    }
}
