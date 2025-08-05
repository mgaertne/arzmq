use super::{MultipartReceiver, MultipartSender, Socket, SocketOption, SocketType};
use crate::{ZmqResult, sealed};

/// # A XSubscriber socket `ZMQ_XPUB`
///
/// Same as [`Publish`] except that you can receive subscriptions from the peers in form of
/// incoming messages. Subscription message is a byte 1 (for subscriptions) or byte 0 (for
/// unsubscriptions) followed by the subscription body. Messages without a sub/unsub prefix are
/// also received, but have no effect on subscription status.
///
/// A socket of type [`XPublish`] is used by a publisher to distribute data. Messages sent are
/// distributed in a fan out fashion to all connected peers.
///
/// When a [`XPublish`] sxocket enters the `mute` state due to having reached the high water mark
/// for a subscriber, then any messages that would be sent to the subscriber in question shall
/// instead be dropped until the mute state ends. The [`send_msg()`] function shall never block for
/// this socket type.
///
/// [`XPublish`]: XPublishSocket
/// [`Publish`]: super::PublishSocket
/// [`send_msg()`]: #impl-Sender-for-Socket<T>
pub type XPublishSocket = Socket<XPublish>;

pub struct XPublish {}

impl sealed::SenderFlag for XPublish {}
impl sealed::ReceiverFlag for XPublish {}
impl sealed::SocketType for XPublish {
    fn raw_socket_type() -> SocketType {
        SocketType::XPublish
    }
}

unsafe impl Sync for Socket<XPublish> {}
unsafe impl Send for Socket<XPublish> {}

impl MultipartSender for Socket<XPublish> {}
impl MultipartReceiver for Socket<XPublish> {}

impl Socket<XPublish> {
    /// # Establish message filter `ZMQ_SUBSCRIBE`
    ///
    /// The [`subscribe()`] option shall establish a new message filter on a [`XPublish`] socket
    /// if subscription management is set to manual via [`set_manual()`].
    ///
    /// [`XPublish`]: XPublishSocket
    /// [`set_manual()`]: #method.set_manual
    /// [`subscribe()`]: #method.subscribe
    pub fn subscribe<V>(&self, topic: V) -> ZmqResult<()>
    where
        V: AsRef<[u8]>,
    {
        self.set_sockopt_bytes(SocketOption::Subscribe, topic.as_ref())
    }

    /// Invert message filtering `ZMQ_INVERT_MATCHING`
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
    /// [`Subscribe`]: super::SubscribeSocket
    /// [`Publish`]: super::PublishSocket
    /// [`XPublish`]: XPublishSocket
    /// [`XSubscribe`]: super::XSubscribeSocket
    pub fn set_invert_matching(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::InvertMatching, value)
    }

    /// Retrieve inverted filtering status `ZMQ_INVERT_MATCHING`
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
    /// [`Subscribe`]: super::SubscribeSocket
    /// [`Publish`]: super::PublishSocket
    /// [`XPublish`]: XPublishSocket
    /// [`XSubscribe`]: super::XSubscribeSocket
    pub fn invert_matching(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::InvertMatching)
    }

    /// # do not silently drop messages if [`send_highwater_mark()`] is reached `ZMQ_XPUB_NODROP`
    ///
    /// Sets the [`XPublish`] socket behaviour to return error [`Again`] if
    /// [`send_highwater_mark()`] is reached and the message could not be send.
    ///
    /// A value of `false` is the default and drops the message silently when the peers
    /// [`send_highwater_mark()`] is reached. A value of `true` returns an [`Again`] error code if
    /// the [`send_highwater_mark()`] is reached and [`DONT_WAIT`] was used.
    ///
    /// [`XPublish`]: XPublishSocket
    /// [`send_highwater_mark()`]: #method.send_highwater_mark
    /// [`Again`]: crate::ZmqError::Again
    /// [`DONT_WAIT`]: super::SendFlags::DONT_WAIT
    pub fn set_nodrop(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::XpubNoDrop, value)
    }

    /// # pass duplicate subscribe messages on [`XPublish`] socket `ZMQ_XPUB_VERBOSE`
    ///
    /// Sets the [`XPublish`] socket behaviour on new duplicated subscriptions. If enabled, the
    /// socket passes all subscribe messages to the caller. If disabled, only the first
    /// subscription to each filter will be passed. The default is `false` (disabled).
    ///
    /// [`XPublish`]: XPublishSocket
    pub fn set_verbose(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::XpubVerbose, value)
    }

    /// # pass duplicate subscribe and unsubscribe messages on [`XPublish`] socket `ZMQ_XPUB_VERBOSER`
    ///
    /// Sets the [`XPublish`] socket behaviour on new duplicated subscriptions and unsubscriptions.
    /// If enabled, the socket passes all subscribe and unsubscribe messages to the caller. If
    /// disabled, only the first subscription to each filter and the last unsubscription from each
    /// filter will be passed. The default is `false` (disabled).
    ///
    /// [`XPublish`]: XPublishSocket
    pub fn set_verboser(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::XpubVerboser, value)
    }

    /// # change the subscription handling to manual `ZMQ_XPUB_MANUAL`
    ///
    /// Sets the [`XPublish`] socket subscription handling mode manual/automatic. A value of
    /// `false` is the default and subscription requests will be handled automatically. A value of
    /// `true` will change the subscription requests handling to manual, with manual mode
    /// subscription requests are not added to the subscription list. To add subscription the user
    /// need to call [`subscribe()`] on [`XPublish`] socket.
    ///
    /// [`XPublish`]: XPublishSocket
    /// [`subscribe()`]: #method.subscribe
    pub fn set_manual(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::XpubManual, value)
    }

    /// # change the subscription handling to manual `ZMQ_XPUB_MANUAL_LAST_VALUE`
    ///
    /// This option is similar to [`set_manual()`]. The difference is that
    /// [`set_manual_last_value()`] changes the [`XPublish`] socket behaviour to send the first
    /// message to the last subscriber after the socket receives a subscription and call setsockopt
    /// with [`subscribe()`] on [`XPublish`] socket. This prevents duplicated messages when using
    /// last value caching (LVC).
    ///
    /// [`XPublish`]: XPublishSocket
    /// [`set_manual_last_value()`]: #method.set_manual_last_value
    /// [`set_manual()`]: #method.set_manual
    /// [`subscribe()`]: #method.subscribe
    #[cfg(feature = "draft-api")]
    pub fn set_manual_last_value(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::XpubManualLastValue, value)
    }

    /// # set welcome message that will be received by subscriber when connecting `ZMQ_XPUB_WELCOME_MSG`
    ///
    /// Sets a welcome message that will be received by subscriber when connecting. Subscriber must
    /// subscribe to the Welcome message before connecting. Welcome message will also be sent on
    /// reconnecting. For welcome message to work well the user must poll on incoming subscription
    /// messages on the [`XPublish`] socket and handle them.
    ///
    /// Use a length of zero to disable welcome message.
    ///
    /// [`XPublish`]: XPublishSocket
    #[cfg(feature = "draft-api")]
    pub fn set_welcome_msg<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::XpubWelcomeMessage, value)
    }

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
    /// [`XPublish`]: XPublishSocket
    /// [`XSubscribe`]: super::XSubscribeSocket
    #[cfg(feature = "draft-api")]
    pub fn topic_count(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TopicsCount)
    }
}

#[cfg(test)]
mod xpublish_tests {
    use super::XPublishSocket;
    use crate::prelude::{
        Context, Receiver, RecvFlags, SendFlags, Sender, SubscribeSocket, ZmqResult,
    };

    #[test]
    fn subscribe_acknowledges_manual_subscriptions() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.bind("tcp://127.0.0.1:*")?;
        let subscribe_endpoint = xpublish.last_endpoint()?;
        xpublish.set_manual(true)?;

        std::thread::spawn(move || {
            let msg = xpublish.recv_msg(RecvFlags::empty()).unwrap();
            assert_eq!(msg.bytes()[0], 1);
            assert_eq!(&msg.to_string()[1..], "topic");

            xpublish.subscribe("topic").unwrap();

            xpublish.send_msg("topic asdf", SendFlags::empty()).unwrap();
        });

        let subscribe = SubscribeSocket::from_context(&context)?;
        subscribe.connect(subscribe_endpoint)?;
        subscribe.subscribe("topic")?;

        let msg = subscribe.recv_msg(RecvFlags::empty())?;

        assert_eq!(msg.to_string(), "topic asdf");

        Ok(())
    }

    #[test]
    fn set_invert_matching_sets_invert_matching() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.set_invert_matching(true)?;

        assert!(xpublish.invert_matching()?);

        Ok(())
    }

    #[test]
    fn set_no_drop_sets_no_drop() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.set_nodrop(true)?;

        Ok(())
    }

    #[test]
    fn set_verbose_sets_verbose() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.set_verbose(true)?;

        Ok(())
    }

    #[test]
    fn set_verboser_sets_verboser() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.set_verboser(true)?;

        Ok(())
    }

    #[test]
    fn set_manual_sets_manual() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.set_manual(true)?;

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_manual_last_value_sets_manual_last_value() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.set_manual_last_value(true)?;

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_welcome_msg_sets_welcome_msg() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.set_welcome_msg("welcome")?;

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_only_first_subscribe_sets_only_first_subscribe() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.set_only_first_subscribe(true)?;

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn topic_count_with_no_subscriber() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.set_manual(true)?;

        assert_eq!(xpublish.topic_count()?, 0);

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn topic_count_returns_subscribed_topic_count() -> ZmqResult<()> {
        let context = Context::new()?;

        let xpublish = XPublishSocket::from_context(&context)?;
        xpublish.bind("tcp://127.0.0.1:*")?;
        let subscribe_endpoint = xpublish.last_endpoint()?;
        xpublish.set_manual(true)?;

        let subscribe = SubscribeSocket::from_context(&context)?;
        subscribe.connect(subscribe_endpoint)?;

        subscribe.subscribe("topic1")?;
        let msg = xpublish.recv_msg(RecvFlags::empty())?;
        assert_eq!(msg.bytes()[0], 1);
        assert_eq!(&msg.to_string()[1..], "topic1");
        xpublish.subscribe("topic")?;

        subscribe.subscribe("topic2")?;
        let msg = xpublish.recv_msg(RecvFlags::empty())?;
        assert_eq!(msg.bytes()[0], 1);
        assert_eq!(&msg.to_string()[1..], "topic2");
        xpublish.subscribe("topic2")?;

        subscribe.subscribe("topic3")?;
        let msg = xpublish.recv_msg(RecvFlags::empty())?;
        assert_eq!(msg.bytes()[0], 1);
        assert_eq!(&msg.to_string()[1..], "topic3");
        xpublish.subscribe("topic3")?;

        assert_eq!(xpublish.topic_count()?, 3);

        Ok(())
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::XPublishSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "XPublishBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`XPublishSocket`].\n\n")]
    #[allow(dead_code)]
    struct XPublishConfig {
        socket_builder: SocketBuilder,
        #[builder(default = false)]
        invert_matching: bool,
        #[builder(default = false)]
        nodrop: bool,
        #[builder(default = false)]
        verbose: bool,
        #[builder(default = false)]
        verboser: bool,
        #[builder(default = false)]
        manual: bool,
        #[cfg(feature = "draft-api")]
        #[builder(default = false)]
        manual_last_value: bool,
        #[cfg(feature = "draft-api")]
        #[builder(default = "Default::default()")]
        welcome_msg: String,
        #[cfg(feature = "draft-api")]
        #[builder(default = false)]
        only_first_subscribe: bool,
    }

    impl XPublishBuilder {
        pub fn apply(self, socket: &XPublishSocket) -> ZmqResult<()> {
            if let Some(socket_builder) = self.socket_builder {
                socket_builder.apply(socket)?;
            }

            self.invert_matching
                .iter()
                .try_for_each(|invert_matching| socket.set_invert_matching(*invert_matching))?;

            self.nodrop
                .iter()
                .try_for_each(|nodrop| socket.set_nodrop(*nodrop))?;

            self.verbose
                .iter()
                .try_for_each(|verbose| socket.set_verbose(*verbose))?;

            self.verboser
                .iter()
                .try_for_each(|verboser| socket.set_verboser(*verboser))?;

            self.manual
                .iter()
                .try_for_each(|manual| socket.set_manual(*manual))?;

            #[cfg(feature = "draft-api")]
            self.manual_last_value
                .iter()
                .try_for_each(|manual_last_value| {
                    socket.set_manual_last_value(*manual_last_value)
                })?;

            #[cfg(feature = "draft-api")]
            self.welcome_msg
                .iter()
                .try_for_each(|welcome_msg| socket.set_welcome_msg(welcome_msg))?;

            #[cfg(feature = "draft-api")]
            self.only_first_subscribe
                .iter()
                .try_for_each(|only_first_subscribe| {
                    socket.set_only_first_subscribe(*only_first_subscribe)
                })?;

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<XPublishSocket> {
            let socket = XPublishSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }

    #[cfg(test)]
    mod xpublish_builder_tests {
        use super::XPublishBuilder;
        use crate::prelude::{Context, SocketBuilder, ZmqResult};

        #[test]
        fn default_xpublish_builder() -> ZmqResult<()> {
            let context = Context::new()?;

            let socket = XPublishBuilder::default().build_from_context(&context)?;

            assert!(!socket.invert_matching()?);

            Ok(())
        }

        #[test]
        fn xpublish_builder_with_custom_values() -> ZmqResult<()> {
            let context = Context::new()?;

            let builder = XPublishBuilder::default()
                .socket_builder(SocketBuilder::default())
                .invert_matching(true)
                .nodrop(true)
                .verbose(true)
                .verboser(true)
                .manual(true);

            #[cfg(feature = "draft-api")]
            let builder = builder
                .manual_last_value(true)
                .welcome_msg("test".to_string())
                .only_first_subscribe(true);

            let socket = builder.build_from_context(&context)?;

            assert!(socket.invert_matching()?);

            Ok(())
        }
    }
}
