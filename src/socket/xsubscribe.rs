#[cfg(feature = "draft-api")]
use super::SocketOption;
use super::{MultipartReceiver, MultipartSender, SendFlags, Sender, Socket, SocketType};
use crate::{ZmqResult, sealed};

/// # A XSubscribe socket `ZMQ_XSUB`
///
/// Same as [`Subscribe`] except that you subscribe by sending subscription messages to the socket.
/// Subscription message is a byte 1 (for subscriptions) or byte 0 (for unsubscriptions) followed
/// by the subscription body. Messages without a sub/unsub prefix may also be sent, but have no
/// effect on subscription status.
///
/// A socket of type [`Subscribe`] is used by a subscriber to subscribe to data distributed by a
/// [`Publish`]. Initially a [`Subscribe`] socket is not subscribed to any messages, use the
/// [`subscribe()`] function specify which messages to subscribe to.
///
/// [`Subscribe`]: super::SubscribeSocket
/// [`Publish`]: super::PublishSocket
/// [`subscribe()`]: #method.subscribe
pub type XSubscribeSocket = Socket<XSubscribe>;

pub struct XSubscribe {}

impl sealed::SenderFlag for XSubscribe {}
impl sealed::ReceiverFlag for XSubscribe {}

unsafe impl Sync for Socket<XSubscribe> {}
unsafe impl Send for Socket<XSubscribe> {}

impl MultipartSender for Socket<XSubscribe> {}
impl MultipartReceiver for Socket<XSubscribe> {}

impl sealed::SocketType for XSubscribe {
    fn raw_socket_type() -> SocketType {
        SocketType::XSubscribe
    }
}

impl Socket<XSubscribe> {
    /// # Process only first subscribe/unsubscribe in a multipart message `ZMQ_ONLY_FIRST_SUBSCRIBE`
    ///
    /// If set, only the first part of the multipart message is processed as a
    /// subscribe/unsubscribe message. The rest are forwarded as user data regardless of message
    /// contents.
    ///
    /// It not set (default), subscribe/unsubscribe messages in a multipart message are processed
    /// as such regardless of their number and order.
    #[cfg(feature = "draft-api")]
    pub fn set_only_first_subscribe(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::OnlyFirstSubscribe, value)
    }

    /// # Establish message filter `ZMQ_SUBSCRIBE`
    ///
    /// The [`subscribe()`] option shall establish a new message filter on a [`XSubscriber`] socket.
    /// Newly created [`XSubscriber`] sockets shall filter out all incoming messages, therefore you
    /// should call this option to establish an initial message filter.
    ///
    /// An empty `topic` of length zero shall subscribe to all incoming messages. A non-empty
    /// `topic` shall subscribe to all messages beginning with the specified prefix. Multiple
    /// filters may be attached to a single [`XSubscriber`] socket, in which case a message shall
    /// be accepted if it matches at least one filter.
    ///
    /// [`XSubscriber`]: XSubscribeSocket
    /// [`subscribe()`]: #method.subscribe
    pub fn subscribe<V>(&self, topic: V) -> ZmqResult<()>
    where
        V: AsRef<[u8]>,
    {
        let mut byte_string = vec![1];
        byte_string.extend_from_slice(topic.as_ref());
        self.send_msg(byte_string, SendFlags::empty())
    }

    /// # Establish message filter `ZMQ_SUBSCRIBE`
    ///
    /// This is the async variant of [`subscribe()`].
    ///
    /// [`subscribe()`]: #method.subscribe
    #[cfg(feature = "futures")]
    pub async fn subscribe_async<V>(&self, topic: V)
    where
        V: AsRef<[u8]>,
    {
        let mut byte_string = vec![1];
        byte_string.extend_from_slice(topic.as_ref());
        self.send_msg_async(byte_string, SendFlags::empty()).await;
    }

    /// # Remove message filter `ZMQ_UNSUBSCRIBE`
    ///
    /// The [`unsubscribe()`] option shall remove an existing message filter on a [`XSubscriber`]
    /// socket. The filter specified must match an existing filter previously established with the
    /// [`subscribe()`] option. If the socket has several instances of the same filter attached
    /// the [`unsubscribe()`] option shall remove only one instance, leaving the rest in place and
    /// functional.
    ///
    /// [`XSubscriber`]: XSubscribeSocket
    /// [`subscribe()`]: #method.subscribe
    /// [`unsubscribe()`]: #method.unsubscribe
    pub fn unsubscribe<V>(&self, topic: V) -> ZmqResult<()>
    where
        V: AsRef<[u8]>,
    {
        let mut byte_string = vec![0];
        byte_string.extend_from_slice(topic.as_ref());
        self.send_msg(byte_string, SendFlags::empty())
    }

    /// # Remove message filter `ZMQ_UNSUBSCRIBE`
    ///
    /// This is the async variant of [`unsubscribe()`].
    ///
    /// [`unsubscribe()`]: #method.unsubscribe
    #[cfg(feature = "futures")]
    pub async fn unsubscribe_async<V>(&self, topic: V)
    where
        V: AsRef<[u8]>,
    {
        let mut byte_string = vec![0];
        byte_string.extend_from_slice(topic.as_ref());
        self.send_msg_async(byte_string, SendFlags::empty()).await;
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
    /// [`Subscribe`]: super::SubscribeSocket
    /// [`Publish`]: super::PublishSocket
    /// [`XPublish`]: super::XPublishSocket
    /// [`XSubscribe`]: XSubscribeSocket
    #[cfg(feature = "draft-api")]
    pub fn topic_count(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TopicsCount)
    }

    /// # pass duplicate unsubscribe messages on [`XSubscribe`] socket `ZMQ_XSUB_VERBOSE_UNSUBSCRIBE`
    ///
    /// Sets the [`XSubscribe`] socket behaviour on duplicated unsubscriptions. If enabled, the
    /// socket passes all unsubscribe messages to the caller. If disabled, only the last
    /// unsubscription from each filter will be passed. The default is `false` (disabled).
    ///
    /// [`XSubscribe`]: XSubscribeSocket
    #[cfg(feature = "draft-api")]
    pub fn set_verbose_unsubscribe(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::XsubVerboseUnsubscribe, value)
    }
}

#[cfg(test)]
mod xsubscribe_tests {
    use super::XSubscribeSocket;
    use crate::prelude::{
        Context, PublishSocket, Receiver, RecvFlags, SendFlags, Sender, XPublishSocket, ZmqResult,
    };

    #[test]
    fn subscribe_subscribes_to_topic() -> ZmqResult<()> {
        let context = Context::new()?;

        let publish = PublishSocket::from_context(&context)?;
        publish.bind("tcp://127.0.0.1:*")?;
        let xsubscribe_endpoint = publish.last_endpoint()?;

        std::thread::spawn(move || {
            loop {
                publish.send_msg("topic asdf", SendFlags::empty()).unwrap();
            }
        });

        let xsubscribe = XSubscribeSocket::from_context(&context)?;
        xsubscribe.connect(xsubscribe_endpoint)?;
        xsubscribe.subscribe("topic")?;

        let msg = xsubscribe.recv_msg(RecvFlags::empty())?;
        assert_eq!(msg.to_string(), "topic asdf");

        Ok(())
    }

    #[test]
    fn unsubscribe_unsubscribes_from_topic() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = XSubscribeSocket::from_context(&context)?;
        socket.unsubscribe("asdf")?;

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_only_first_subscribe_sets_only_first_subscribe() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = XSubscribeSocket::from_context(&context)?;
        socket.set_only_first_subscribe(true)?;

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn topic_count_returns_topic_count() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = XSubscribeSocket::from_context(&context)?;
        assert_eq!(socket.topic_count()?, 0);

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn topic_count_returns_topic_count_after_subscribe() -> ZmqResult<()> {
        let context = Context::new()?;

        let publish = PublishSocket::from_context(&context)?;
        publish.bind("tcp://127.0.0.1:*")?;
        let xsubscribe_endpoint = publish.last_endpoint()?;

        std::thread::spawn(move || {
            loop {
                publish.send_msg("topic2 asdf", SendFlags::empty()).unwrap();
            }
        });

        let xsubscribe = XSubscribeSocket::from_context(&context)?;
        xsubscribe.connect(xsubscribe_endpoint)?;
        xsubscribe.subscribe("topic1")?;
        xsubscribe.subscribe("topic2")?;
        xsubscribe.subscribe("topic3")?;

        assert_eq!(xsubscribe.topic_count()?, 3);

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_verbose_unsubscribe_sets_verbose_unsubscribe() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = XSubscribeSocket::from_context(&context)?;
        socket.set_verbose_unsubscribe(true)?;

        Ok(())
    }

    #[test]
    fn xpublish_xsubscribe() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.bind("tcp://127.0.0.1:*")?;
        let xsubscribe_endpoint = xpublish.last_endpoint()?;

        std::thread::spawn(move || {
            let msg = xpublish.recv_msg(RecvFlags::empty()).unwrap();
            assert_eq!(msg.bytes()[0], 1);
            assert_eq!(&msg.to_string()[1..], "topic");

            loop {
                xpublish.send_msg("topic asdf", SendFlags::empty()).unwrap();
            }
        });

        let xsubscribe = XSubscribeSocket::from_context(&context)?;
        xsubscribe.connect(xsubscribe_endpoint)?;
        xsubscribe.subscribe("topic")?;

        let msg = xsubscribe.recv_msg(RecvFlags::empty())?;
        assert_eq!(msg.to_string(), "topic asdf");

        Ok(())
    }

    #[cfg(feature = "futures")]
    #[test]
    fn xpublish_xsubscribe_async() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.bind("tcp://127.0.0.1:*")?;
        let xsubscribe_endpoint = xpublish.last_endpoint()?;

        std::thread::spawn(move || {
            futures::executor::block_on(async {
                loop {
                    if let Some(msg) = xpublish.recv_msg_async().await {
                        assert_eq!(msg.bytes()[0], 1);
                        assert_eq!(&msg.to_string()[1..], "topic");
                        break;
                    }
                }

                loop {
                    xpublish
                        .send_msg_async("topic asdf", SendFlags::empty())
                        .await;
                }
            })
        });

        let xsubscribe = XSubscribeSocket::from_context(&context)?;
        xsubscribe.connect(xsubscribe_endpoint)?;

        futures::executor::block_on(async {
            xsubscribe.subscribe_async("topic").await;

            loop {
                if let Some(msg) = xsubscribe.recv_msg_async().await {
                    assert_eq!(msg.to_string(), "topic asdf");
                    break;
                }
            }

            xsubscribe.unsubscribe_async("topic").await;

            Ok(())
        })
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::XSubscribeSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "XSubscribeBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`XSubscribeSocket`].\n\n")]
    #[allow(dead_code)]
    struct XSubscribeConfig {
        socket_builder: SocketBuilder,
        #[builder(setter(into), default = "Default::default()")]
        subscribe: String,
        #[cfg(feature = "draft-api")]
        #[builder(default = false)]
        only_first_subscribe: bool,
    }

    impl XSubscribeBuilder {
        pub fn apply(self, socket: &XSubscribeSocket) -> ZmqResult<()> {
            if let Some(socket_builder) = self.socket_builder {
                socket_builder.apply(socket)?;
            }

            self.subscribe
                .iter()
                .try_for_each(|topic| socket.subscribe(topic))?;

            #[cfg(feature = "draft-api")]
            self.only_first_subscribe
                .iter()
                .try_for_each(|only_first_subscribe| {
                    socket.set_only_first_subscribe(*only_first_subscribe)
                })?;

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<XSubscribeSocket> {
            let socket = XSubscribeSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }

    #[cfg(test)]
    mod xsubscribe_builder_tests {
        use super::XSubscribeBuilder;
        use crate::prelude::{Context, SocketBuilder, ZmqResult};

        #[test]
        fn default_xsubscribe_builder() -> ZmqResult<()> {
            let context = Context::new()?;

            XSubscribeBuilder::default().build_from_context(&context)?;

            Ok(())
        }

        #[test]
        fn xsubscribe_builder_with_custom_values() -> ZmqResult<()> {
            let context = Context::new()?;

            let builder = XSubscribeBuilder::default()
                .socket_builder(SocketBuilder::default())
                .subscribe("asdf");

            #[cfg(feature = "draft-api")]
            let builder = builder.only_first_subscribe(true);

            builder.build_from_context(&context)?;

            Ok(())
        }
    }
}
