use super::{MultipartSender, Socket, SocketOption, SocketType};
use crate::{ZmqResult, sealed};

/// # A Subscriber socket `ZMQ_PUB`
///
/// A socket of type [`Publish`] is used by a publisher to distribute data. Messages sent are
/// distributed in a fan out fashion to all connected peers.
///
/// When a [`Publish`] socket enters the `mute` state due to having reached the high water mark
/// for a subscriber, then any messages that would be sent to the subscriber in question shall
/// instead be dropped until the mute state ends. The [`send_msg()`] function shall never block for
/// this socket type.
///
/// [`Publish`]: PublishSocket
/// [`send_msg()`]: #impl-Sender-for-Socket<T>
pub type PublishSocket = Socket<Publish>;

pub struct Publish {}

impl sealed::SenderFlag for Publish {}
impl sealed::SocketType for Publish {
    fn raw_socket_type() -> SocketType {
        SocketType::Publish
    }
}

unsafe impl Sync for Socket<Publish> {}
unsafe impl Send for Socket<Publish> {}

impl MultipartSender for Socket<Publish> {}

impl Socket<Publish> {
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
    /// [`Publish`]: PublishSocket
    /// [`XPublish`]: super::XPublishSocket
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
    /// [`Publish`]: PublishSocket
    /// [`XPublish`]: super::XPublishSocket
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
    /// [`XPublish`]: super::XPublishSocket
    /// [`send_highwater_mark()`]: #method.send_highwater_mark
    /// [`Again`]: crate::ZmqError::Again
    /// [`DONT_WAIT`]: super::SendFlags::DONT_WAIT
    pub fn set_nodrop(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::XpubNoDrop, value)
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
    /// [`Publish`]: PublishSocket
    /// [`XPublish`]: super::XPublishSocket
    /// [`XSubscribe`]: super::XSubscribeSocket
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn topic_count(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TopicsCount)
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::PublishSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "PublishBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`PublishSocket`].\n\n")]
    #[allow(dead_code)]
    struct PublishConfig {
        socket_config: SocketBuilder,
        #[builder(default = false)]
        conflate: bool,
        #[builder(default = false)]
        invert_matching: bool,
        #[builder(default = false)]
        nodrop: bool,
    }

    impl PublishBuilder {
        pub fn apply(self, socket: &PublishSocket) -> ZmqResult<()> {
            if let Some(socket_config) = self.socket_config {
                socket_config.apply(socket)?;
            }

            if let Some(conflate) = self.conflate {
                socket.set_conflate(conflate)?;
            }

            if let Some(invert_matching) = self.invert_matching {
                socket.set_invert_matching(invert_matching)?;
            }

            if let Some(nodrop) = self.nodrop {
                socket.set_nodrop(nodrop)?;
            }

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<PublishSocket> {
            let socket = PublishSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
}
