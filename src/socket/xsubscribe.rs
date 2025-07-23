use super::{MultipartReceiver, MultipartSender, Socket, SocketOption, SocketType};
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
        SocketType::Subscribe
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
    #[doc(cfg(feature = "draft-api"))]
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
        self.set_sockopt_bytes(SocketOption::Subscribe, topic.as_ref())
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
    /// [`Subscribe`]: super::SubscribeSocket
    /// [`Publish`]: super::PublishSocket
    /// [`XPublish`]: super::XPublishSocket
    /// [`XSubscribe`]: XSubscribeSocket
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
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
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_verbose_unsubscribe(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::XsubVerboseUnsubscribe, value)
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
        socket_config: SocketBuilder,
        #[builder(setter(into), default = "Default::default()")]
        subscribe: String,
        #[cfg(feature = "draft-api")]
        #[doc(cfg(feature = "draft-api"))]
        #[builder(default = false)]
        only_first_subscribe: bool,
    }

    impl XSubscribeBuilder {
        pub fn apply(self, socket: &XSubscribeSocket) -> ZmqResult<()> {
            if let Some(socket_config) = self.socket_config {
                socket_config.apply(socket)?;
            }

            #[cfg(feature = "draft-api")]
            if let Some(only_first_subscribe) = self.only_first_subscribe {
                socket.set_only_first_subscribe(only_first_subscribe)?;
            }

            if let Some(subscribe) = self.subscribe {
                socket.subscribe(&subscribe)?;
            }

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<XSubscribeSocket> {
            let socket = XSubscribeSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
}
