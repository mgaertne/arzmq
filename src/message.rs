//! 0MQ messages

use alloc::{
    collections::{
        VecDeque,
        vec_deque::{Drain, Iter, IterMut},
    },
    string::FromUtf8Error,
};
use core::{
    ops::{Deref, RangeBounds},
    ptr,
    str::{self, Utf8Error},
};

use derive_more::{Debug as DebugDeriveMore, Display as DisplayDeriveMore, Index, IntoIterator};
use parking_lot::{Mutex, MutexGuard};

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
    inner: Mutex<RawMessage>,
}

unsafe impl Send for Message {}
unsafe impl Sync for Message {}

/// # Borrowed bytes of a [`Message`]
///
/// Returned by [`Message::as_bytes()`]. It holds the lock of the message, so the bytes cannot be
/// changed or sent from another thread while they are borrowed. The guard is not [`Send`], which
/// keeps it from being held across an `.await` in a future that may move between threads:
///
/// ```compile_fail
/// # use arzmq::prelude::Message;
/// fn assert_send<T: Send>(_: T) {}
///
/// let msg = Message::from("Hello");
/// assert_send(msg.as_bytes());
/// ```
pub struct MessageBytes<'a> {
    guard: MutexGuard<'a, RawMessage>,
}

impl MessageBytes<'_> {
    /// Interprets the borrowed bytes as UTF-8 text.
    pub fn to_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(self)
    }
}

impl Deref for MessageBytes<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.guard.as_ref()
    }
}

impl AsRef<[u8]> for MessageBytes<'_> {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

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

    /// Borrows the message underlying byte representation without copying it.
    ///
    /// The returned [`MessageBytes`] keeps the message locked until it is dropped. Do not call
    /// other methods of the same message, or send it, while holding on to it, as those wait for
    /// the lock as well.
    ///
    /// ## Example
    ///
    /// ```
    /// # use arzmq::prelude::Message;
    /// let msg = Message::from("Hello");
    ///
    /// assert_eq!(&*msg.as_bytes(), b"Hello");
    /// ```
    pub fn as_bytes(&self) -> MessageBytes<'_> {
        MessageBytes {
            guard: self.inner.lock(),
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

    /// Set the routing id of the message. Used for interactions on [`Server`] and [`Peer`] sockets.
    ///
    /// [`Server`]: crate::socket::ServerSocket
    /// [`Peer`]: crate::socket::PeerSocket
    #[cfg(feature = "draft-api")]
    pub fn set_routing_id(&self, value: u32) -> ZmqResult<()> {
        let mut msg_guard = self.inner.lock();
        msg_guard.set_routing_id(value)
    }

    /// Retrieve the routing id of the message. Used for interactions on [`Server`] and [`Peer`]
    /// sockets.
    ///
    /// [`Server`]: crate::socket::ServerSocket
    /// [`Peer`]: crate::socket::PeerSocket
    #[cfg(feature = "draft-api")]
    pub fn routing_id(&self) -> Option<u32> {
        let msg_guard = self.inner.lock();
        msg_guard.routing_id()
    }

    /// Sets the group for the message. Used by [`Radio`] sockets.
    ///
    /// [`Radio`]: crate::socket::RadioSocket
    #[cfg(feature = "draft-api")]
    pub fn set_group<V: AsRef<str>>(&self, value: V) -> ZmqResult<()> {
        let mut msg_guard = self.inner.lock();
        msg_guard.set_group(value.as_ref())
    }

    /// Retrieves the group for the message. Used by [`Dish`] sockets.
    ///
    /// [`Dish`]: crate::socket::DishSocket
    #[cfg(feature = "draft-api")]
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

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        if ptr::eq(self, other) {
            return true;
        }

        let (first, second) = if ptr::from_ref(self) < ptr::from_ref(other) {
            (self, other)
        } else {
            (other, self)
        };
        *first.as_bytes() == *second.as_bytes()
    }
}

impl Eq for Message {}

impl PartialEq<[u8]> for Message {
    fn eq(&self, other: &[u8]) -> bool {
        *self.as_bytes() == *other
    }
}

impl PartialEq<&[u8]> for Message {
    fn eq(&self, other: &&[u8]) -> bool {
        *self.as_bytes() == **other
    }
}

impl PartialEq<str> for Message {
    fn eq(&self, other: &str) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl PartialEq<&str> for Message {
    fn eq(&self, other: &&str) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl From<Message> for Vec<u8> {
    fn from(message: Message) -> Self {
        message.bytes()
    }
}

impl TryFrom<&Message> for String {
    type Error = FromUtf8Error;

    fn try_from(message: &Message) -> Result<Self, Self::Error> {
        String::from_utf8(message.bytes())
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

impl Message {
    #[cfg(all(feature = "futures", unix))]
    pub(crate) fn send_by_ref(&self, socket: &crate::ffi::RawSocket, flags: i32) -> ZmqResult<()> {
        socket.send(&mut self.inner.lock(), flags)
    }
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

#[cfg(test)]
mod message_tests {
    use super::Message;
    #[cfg(feature = "draft-api")]
    use crate::prelude::ZmqResult;

    #[test]
    fn with_size_creates_message_with_correct_size() {
        let msg = Message::with_size(42).unwrap();
        assert_eq!(msg.len(), 42);
    }

    #[test]
    fn bytes_returns_correct_bytes() {
        let msg: Message = "asdf".into();
        assert_eq!(msg.bytes(), "asdf".as_bytes());
    }

    #[test]
    fn as_bytes_borrows_the_message_bytes() {
        let msg: Message = "asdf".into();
        let copied = msg.bytes();
        assert_eq!(&*msg.as_bytes(), "asdf".as_bytes());
        assert_eq!(msg.as_bytes().as_ref(), copied.as_slice());
    }

    #[test]
    fn message_from_owned_and_fixed_size_data() {
        let from_string = Message::from(String::from("asdf"));
        let from_cow = Message::from(std::borrow::Cow::Borrowed("asdf"));
        let from_array = Message::from(*b"asdf");
        let from_array_ref = Message::from(b"asdf");

        assert_eq!(from_string, "asdf");
        assert_eq!(from_cow, "asdf");
        assert_eq!(from_array, "asdf");
        assert_eq!(from_array_ref, "asdf");
    }

    #[test]
    fn messages_compare_by_their_bytes() {
        let msg: Message = "asdf".into();

        assert_eq!(msg, msg);
        assert_eq!(msg, Message::from("asdf"));
        assert_ne!(msg, Message::from("qwertz"));
        assert_eq!(msg, *"asdf".as_bytes());
        assert_eq!(msg, "asdf".as_bytes());
        assert_eq!(msg, *"asdf");
        assert_eq!(msg, "asdf");
    }

    #[test]
    fn message_converts_back_into_data() {
        let msg: Message = "asdf".into();

        assert_eq!(String::try_from(&msg).unwrap(), "asdf");
        assert_eq!(Vec::<u8>::from(msg), b"asdf");
        assert!(String::try_from(&Message::from(vec![0xFF_u8, 0xFE])).is_err());
    }

    #[test]
    fn as_bytes_to_str_interprets_utf8() {
        let msg: Message = "asdf".into();
        assert_eq!(msg.as_bytes().to_str(), Ok("asdf"));
        assert!(Message::from(vec![0xFF_u8]).as_bytes().to_str().is_err());
    }

    #[test]
    fn as_bytes_guard_is_released_on_drop() {
        let msg: Message = "asdf".into();
        drop(msg.as_bytes());
        assert_eq!(msg.len(), 4);
    }

    #[test]
    fn is_empty_for_empty_message() {
        let msg = Message::new();
        assert!(msg.is_empty());
    }

    #[test]
    fn is_empty_for_message_with_data() {
        let msg: Message = "asdf".into();
        assert!(!msg.is_empty());
    }

    #[test]
    fn get_more_for_single_message() {
        let msg = Message::new();
        assert!(!msg.get_more());
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_routing_id_sets_routing_id() -> ZmqResult<()> {
        let msg = Message::new();
        msg.set_routing_id(123)?;
        assert_eq!(msg.routing_id(), Some(123));

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn routing_id_defaults_to_none() {
        let msg = Message::new();
        assert_eq!(msg.routing_id(), None);
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_group_sets_group() -> ZmqResult<()> {
        let msg = Message::new();
        msg.set_group("asdf")?;
        assert_eq!(msg.group(), Some("asdf".to_string()));

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn group_defaults_to_none() {
        let msg = Message::new();
        assert_eq!(msg.group(), None);
    }

    #[test]
    fn message_can_be_cloned() {
        let msg: Message = "asdf".into();
        assert_eq!(msg.clone().to_string(), "asdf");
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn message_can_be_cloned_with_routing_id_and_group() -> ZmqResult<()> {
        let msg: Message = "asdf".into();
        msg.set_routing_id(123)?;
        msg.set_group("asdf")?;

        let cloned_msg = msg.clone();
        assert_eq!(cloned_msg.to_string(), "asdf");
        assert_eq!(cloned_msg.routing_id(), Some(123));
        assert_eq!(cloned_msg.group(), Some("asdf".into()));

        Ok(())
    }
}

/// convenicen trait for sendable messages, including single- and multipart ones.
pub trait Sendable<S: sealed::SocketType + sealed::SenderFlag> {
    /// send the message on the provided socket
    fn send(self, socket: &Socket<S>, flags: i32) -> ZmqResult<()>;
}

#[derive(
    Default, Clone, PartialEq, Eq, DebugDeriveMore, DisplayDeriveMore, Index, IntoIterator,
)]
#[debug("MultipartMessage {{ {inner:?} }}")]
#[display("MultipartMessage {{ {inner:?} }}")]
/// 0MQ multipart message
pub struct MultipartMessage {
    #[into_iterator(owned, ref, ref_mut)]
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

impl<T: Into<Message>, const N: usize> From<[T; N]> for MultipartMessage {
    fn from(parts: [T; N]) -> Self {
        parts.into_iter().collect()
    }
}

impl<T: Into<Message>> FromIterator<T> for MultipartMessage {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        MultipartMessage {
            inner: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl<T: Into<Message>> Extend<T> for MultipartMessage {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.inner.extend(iter.into_iter().map(Into::into));
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

#[cfg(test)]
mod multipart_message_tests {
    use super::{Message, MultipartMessage};

    #[test]
    fn multipart_message_collects_from_convertible_parts() {
        let msg: MultipartMessage = ["asdf", "qwertz"].into_iter().collect();

        assert_eq!(msg.len(), 2);
        assert_eq!(msg[0], "asdf");
        assert_eq!(msg[1], "qwertz");
    }

    #[test]
    fn multipart_message_from_array_of_convertible_parts() {
        let msg = MultipartMessage::from(["asdf", "qwertz"]);

        assert_eq!(msg.len(), 2);
        assert_eq!(msg[1], "qwertz");
    }

    #[test]
    fn multipart_message_extends_with_convertible_parts() {
        let mut msg = MultipartMessage::from(["asdf"]);
        msg.extend(["qwertz", "yxcv"]);

        assert_eq!(msg.len(), 3);
        assert_eq!(msg[2], "yxcv");
    }

    #[test]
    fn multipart_message_clones_and_compares_by_parts() {
        let msg = MultipartMessage::from(["asdf", "qwertz"]);

        assert_eq!(msg.clone(), msg);
        assert_ne!(msg, MultipartMessage::from(["asdf"]));
    }

    #[test]
    fn empty_multipart_message_is_empty() {
        let msg = MultipartMessage::new();
        assert!(msg.is_empty());
    }

    #[test]
    fn get_for_empty_messahge() {
        let msg = MultipartMessage::new();
        assert!(msg.get(0).is_none());
    }

    #[test]
    fn get_for_non_empty_message() {
        let msg: MultipartMessage = vec!["asdf".into(), vec![].into(), "qwertz".into()].into();
        assert_eq!(msg.get(0).unwrap().to_string(), "asdf");
        assert_eq!(msg.get(2).unwrap().to_string(), "qwertz");
    }

    #[test]
    fn pop_front_for_empty_message() {
        let mut msg = MultipartMessage::new();
        assert!(msg.pop_front().is_none());
    }

    #[test]
    fn pop_front_for_non_empty_message() {
        let mut msg: MultipartMessage = vec!["asdf".into(), vec![].into(), "qwertz".into()].into();
        assert!(msg.pop_front().is_some_and(|message| message == "asdf"));
    }

    #[test]
    fn pop_back_for_empty_message() {
        let mut msg = MultipartMessage::new();
        assert!(msg.pop_back().is_none());
    }

    #[test]
    fn pop_back_for_non_empty_message() {
        let mut msg: MultipartMessage = vec!["asdf".into(), vec![].into(), "qwertz".into()].into();
        assert!(msg.pop_back().is_some_and(|message| message == "qwertz"));
    }

    #[test]
    fn push_front_for_empty_message() {
        let mut msg = MultipartMessage::new();
        msg.push_front("asdf".into());
        assert!(msg.get(0).is_some_and(|message| message == "asdf"));
    }

    #[test]
    fn push_front_for_non_empty_message() {
        let mut msg: MultipartMessage = vec!["qwertz".into()].into();
        msg.push_front("asdf".into());
        assert!(msg.get(0).is_some_and(|message| message == "asdf"));
        assert!(msg.get(1).is_some_and(|message| message == "qwertz"));
    }

    #[test]
    fn push_back_for_empty_message() {
        let mut msg = MultipartMessage::new();
        msg.push_back("asdf".into());
        assert!(msg.get(0).is_some_and(|message| message == "asdf"));
    }

    #[test]
    fn push_back_for_non_empty_message() {
        let mut msg: MultipartMessage = vec!["asdf".into()].into();
        msg.push_back("qwertz".into());
        assert!(msg.get(0).is_some_and(|message| message == "asdf"));
        assert!(msg.get(1).is_some_and(|message| message == "qwertz"));
    }

    #[test]
    fn is_empty_for_empty_message() {
        let msg = MultipartMessage::new();
        assert!(msg.is_empty());
    }

    #[test]
    fn is_empty_for_non_empty_message() {
        let msg: MultipartMessage = vec!["asdf".into(), vec![].into(), "qwertz".into()].into();
        assert!(!msg.is_empty());
    }

    #[test]
    fn iter_for_empty_message() {
        let msg = MultipartMessage::new();
        assert!(msg.iter().next().is_none());
    }

    #[test]
    fn iter_for_non_empty_message() {
        let msg: MultipartMessage = vec!["asdf".into(), vec![].into(), "qwertz".into()].into();
        assert_eq!(
            msg.iter()
                .map(|message| message.to_string())
                .collect::<Vec<_>>(),
            vec!["asdf", "", "qwertz"]
        );
    }

    #[test]
    fn iter_mut_for_empty_message() {
        let mut msg = MultipartMessage::new();
        assert!(msg.iter_mut().next().is_none());
    }

    #[test]
    fn iter_mut_for_non_empty_message() {
        let mut msg: MultipartMessage = vec!["asdf".into(), vec![].into(), "qwertz".into()].into();
        for message in msg.iter_mut() {
            *message = "asdf".into();
        }

        assert_eq!(
            msg.iter_mut()
                .map(|message| message.to_string())
                .collect::<Vec<_>>(),
            vec!["asdf", "asdf", "asdf"]
        );
    }

    #[test]
    fn len_for_empty_message() {
        let msg = MultipartMessage::new();
        assert_eq!(msg.len(), 0);
    }

    #[test]
    fn len_for_non_empty_message() {
        let msg: MultipartMessage = vec!["asdf".into(), vec![].into(), "qwertz".into()].into();
        assert_eq!(msg.len(), 3);
    }

    #[test]
    fn drain_for_empty_message() {
        let mut msg = MultipartMessage::new();
        assert!(msg.drain(..).next().is_none());
    }

    #[test]
    fn drain_for_non_empty_message() {
        let mut msg: MultipartMessage = vec!["asdf".into(), vec![].into(), "qwertz".into()].into();
        assert_eq!(
            msg.drain(..)
                .map(|message| message.to_string())
                .collect::<Vec<_>>(),
            vec!["asdf", "", "qwertz"]
        );
        assert!(msg.is_empty());
    }

    #[test]
    fn multipart_can_be_constructed_from_single_message() {
        let msg: Message = "asdf".into();
        let multipart: MultipartMessage = msg.into();
        assert_eq!(multipart.len(), 1);
        assert!(multipart.get(0).is_some_and(|message| message == "asdf"));
    }

    #[test]
    fn multipart_can_be_constructed_from_multiple_messages() {
        let msg: Message = "asdf".into();
        let msg2: Message = vec![].into();
        let msg3: Message = "qwertz".into();
        let multipart: MultipartMessage = vec![msg, msg2, msg3].into();
        assert_eq!(multipart.len(), 3);
        assert!(multipart.get(0).is_some_and(|message| message == "asdf"));
        assert!(multipart.get(1).is_some_and(|message| message.is_empty()));
        assert!(multipart.get(2).is_some_and(|message| message == "qwertz"));
    }

    #[test]
    fn multipart_into_iter_for_references_returns_correct_values() {
        let multipart: MultipartMessage =
            vec!["asdf".into(), vec![].into(), "qwertz".into()].into();
        assert_eq!(
            (&multipart)
                .into_iter()
                .map(|message| message.to_string())
                .collect::<Vec<_>>(),
            vec!["asdf", "", "qwertz"]
        )
    }

    #[test]
    fn multipart_into_iter_returns_correct_values() {
        let multipart: MultipartMessage =
            vec!["asdf".into(), vec![].into(), "qwertz".into()].into();
        assert_eq!(
            multipart
                .into_iter()
                .map(|message| message.to_string())
                .collect::<Vec<_>>(),
            vec!["asdf", "", "qwertz"]
        )
    }

    #[test]
    fn multipart_into_mut_iter_returns_correct_values() {
        let mut multipart: MultipartMessage =
            vec!["asdf".into(), vec![].into(), "qwertz".into()].into();
        for message in (&mut multipart).into_iter() {
            *message = "asdf".into();
        }

        assert_eq!(
            multipart
                .iter()
                .map(|message| message.to_string())
                .collect::<Vec<String>>(),
            vec!["asdf", "asdf", "asdf"]
        );
    }
}
