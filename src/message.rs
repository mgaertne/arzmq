//! 0MQ messages

use alloc::collections::{
    VecDeque,
    vec_deque::{Drain, IntoIter, Iter, IterMut},
};
use core::ops::RangeBounds;

use derive_more::{Debug as DebugDeriveMore, Display as DisplayDeriveMore};
use parking_lot::FairMutex;

use crate::{
    ZmqResult,
    ffi::RawMessage,
    sealed,
    socket::{MultipartSender, Socket},
};

#[derive(DebugDeriveMore, DisplayDeriveMore)]
#[debug("Message {{ {:?} }}", inner.lock())]
#[display("{}", inner.lock())]
/// 0MQ single-part message
pub struct Message {
    inner: FairMutex<RawMessage>,
}

unsafe impl Send for Message {}
unsafe impl Sync for Message {}

impl Message {
    pub fn new() -> Self {
        Self::default()
    }

    /// initialise 0MQ message of a specified size
    pub fn with_size(len: usize) -> ZmqResult<Self> {
        Ok(Self::from_raw_msg(RawMessage::with_size(len)))
    }

    pub(crate) fn from_raw_msg(raw_msg: RawMessage) -> Self {
        Self {
            inner: raw_msg.into(),
        }
    }

    /// returns the message underlying byte representation
    pub fn bytes(&self) -> Vec<u8> {
        let msg_guard = self.inner.lock();
        (*msg_guard).as_ref().to_vec()
    }

    /// returns the message length
    pub fn len(&self) -> usize {
        let msg_guard = self.inner.lock();
        msg_guard.len()
    }

    /// returns whether this message is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// returns whether there are more parts to this message that can be received
    pub fn get_more(&self) -> bool {
        let msg_guard = self.inner.lock();
        msg_guard.get_more()
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Set the routing id of the message. Used for interactions on [`Server`] and [`Peer`] sockets.
    ///
    /// [`Server`]: crate::socket::ServerSocket
    /// [`Peer`]: crate::socket::PeerSocket
    pub fn set_routing_id(&self, value: u32) -> ZmqResult<()> {
        let mut msg_guard = self.inner.lock();
        msg_guard.set_routing_id(value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Retrieve the routing id of the message. Used for interactions on [`Server`] and [`Peer`]
    /// sockets.
    ///
    /// [`Server`]: crate::socket::ServerSocket
    /// [`Peer`]: crate::socket::PeerSocket
    pub fn routing_id(&self) -> Option<u32> {
        let msg_guard = self.inner.lock();
        msg_guard.routing_id()
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Sets the group for the message. Used by [`Radio`] sockets.
    ///
    /// [`Radio`]: crate::socket::RadioSocket
    pub fn set_group<V: AsRef<str>>(&self, value: V) -> ZmqResult<()> {
        let mut msg_guard = self.inner.lock();
        msg_guard.set_group(value.as_ref())
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Retrieves the group for the message. Used by [`Dish`] sockets.
    ///
    /// [`Dish`]: crate::socket::DishSocket
    pub fn group(&self) -> Option<String> {
        let msg_guard = self.inner.lock();
        msg_guard.group()
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::from_raw_msg(RawMessage::default())
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        let msg_guard = self.inner.lock();
        Message {
            inner: (*msg_guard).clone().into(),
        }
    }
}

impl<T: Into<RawMessage>> From<T> for Message {
    fn from(value: T) -> Self {
        let raw_msg = value.into();
        Self {
            inner: raw_msg.into(),
        }
    }
}

/// convenicen trait for sendable messages, including single- and multipart ones.
pub trait Sendable<S: sealed::SocketType + sealed::SenderFlag> {
    /// send the message on the provided socket
    fn send(self, socket: &Socket<S>, flags: i32) -> ZmqResult<()>;
}

impl<M, S: sealed::SocketType + sealed::SenderFlag> Sendable<S> for M
where
    M: Into<Message>,
{
    fn send(self, socket: &Socket<S>, flags: i32) -> ZmqResult<()> {
        let zmq_msg = self.into();
        let mut raw_msg = zmq_msg.inner.lock();

        socket.socket.send(&mut raw_msg, flags)?;
        Ok(())
    }
}

#[derive(Default, DebugDeriveMore, DisplayDeriveMore)]
#[debug("MultipartMessage {{ {inner:?} }}")]
#[display("MultipartMessage {{ {inner:?} }}")]
/// 0MQ multipart message
pub struct MultipartMessage {
    inner: VecDeque<Message>,
}

unsafe impl Send for MultipartMessage {}
unsafe impl Sync for MultipartMessage {}

impl MultipartMessage {
    pub fn new() -> Self {
        MultipartMessage::default()
    }

    pub fn into_inner(self) -> VecDeque<Message> {
        self.inner
    }

    /// get the message part at `index`
    pub fn get(&self, index: usize) -> Option<&Message> {
        self.inner.get(index)
    }

    /// removes the first part of this multipart message and returns it.
    pub fn pop_front(&mut self) -> Option<Message> {
        self.inner.pop_front()
    }

    /// removes the last part of this multipart message and returns it.
    pub fn pop_back(&mut self) -> Option<Message> {
        self.inner.pop_back()
    }

    /// inserts a new part at the front of this multipart message.
    pub fn push_front(&mut self, msg: Message) {
        self.inner.push_front(msg)
    }

    /// inserts a new part at the back of this multipart message.
    pub fn push_back(&mut self, msg: Message) {
        self.inner.push_back(msg)
    }

    /// returns whether this multipart message is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// returns an iterator over the parts of this multipart message
    pub fn iter(&self) -> Iter<'_, Message> {
        self.inner.iter()
    }

    /// returns a mutable iterator over the parts of this multipart message
    pub fn iter_mut(&mut self) -> IterMut<'_, Message> {
        self.inner.iter_mut()
    }

    /// returns the number of parts in this multipart message
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// clears this multipart message in the given range
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, Message>
    where
        R: RangeBounds<usize>,
    {
        self.inner.drain(range)
    }
}

impl From<Message> for MultipartMessage {
    fn from(msg: Message) -> Self {
        let mut multipart = MultipartMessage::new();
        multipart.push_back(msg);
        multipart
    }
}

impl From<Vec<Message>> for MultipartMessage {
    fn from(v: Vec<Message>) -> Self {
        MultipartMessage { inner: v.into() }
    }
}

impl<'a> IntoIterator for &'a MultipartMessage {
    type IntoIter = Iter<'a, Message>;
    type Item = &'a Message;

    fn into_iter(self) -> Iter<'a, Message> {
        self.iter()
    }
}

impl IntoIterator for MultipartMessage {
    type IntoIter = IntoIter<Message>;
    type Item = Message;

    fn into_iter(self) -> IntoIter<Message> {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut MultipartMessage {
    type IntoIter = IterMut<'a, Message>;
    type Item = &'a mut Message;

    fn into_iter(self) -> IterMut<'a, Message> {
        self.iter_mut()
    }
}

impl<S> Sendable<S> for MultipartMessage
where
    S: sealed::SocketType + sealed::SenderFlag,
    Socket<S>: MultipartSender,
{
    fn send(self, socket: &Socket<S>, flags: i32) -> ZmqResult<()> {
        socket.send_multipart(self, flags)
    }
}
