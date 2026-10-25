//! A [`Stream`] of the multipart messages a socket receives.

use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::stream::{FusedStream, Stream};

use crate::{
    ZmqError, ZmqResult,
    message::MultipartMessage,
    sealed,
    socket::{MultipartReceiver, Receiver, RecvFlags, Socket},
};

/// # Stream of the multipart messages received by a socket
///
/// Created by [`Socket::multipart_stream()`]. 0MQ delivers all parts of a multipart message or
/// none, so each item is a complete [`MultipartMessage`]; the remaining parts are read right
/// after the first one has arrived, without suspending the task in between. Dropping the stream
/// or a pending [`next()`] therefore never loses a message, and a single-part message is a
/// multipart message with one part.
///
/// The stream ends when the [`Context`] of the socket is terminated. Any other error is yielded as
/// an item and the stream keeps polling the socket.
///
/// As for [`recv_msg_wait()`], tasks are woken up by the readiness notification of the
/// [`FileDescriptor`] option, so the stream works with any async runtime.
///
/// [`Context`]: crate::context::Context
/// [`next()`]: futures::stream::StreamExt::next
/// [`recv_msg_wait()`]: Receiver::recv_msg_wait
/// [`FileDescriptor`]: crate::socket::SocketOption::FileDescriptor
#[derive(Debug)]
pub struct MultipartStream<'a, T: sealed::SocketType> {
    socket: &'a Socket<T>,
    terminated: bool,
}

impl<T> Socket<T>
where
    T: sealed::SocketType,
    Socket<T>: MultipartReceiver,
{
    /// # Stream the multipart messages of this socket
    ///
    /// See [`MultipartStream`].
    pub fn multipart_stream(&self) -> MultipartStream<'_, T> {
        MultipartStream {
            socket: self,
            terminated: false,
        }
    }
}

impl<T> Stream for MultipartStream<'_, T>
where
    T: sealed::SocketType,
    Socket<T>: MultipartReceiver,
{
    type Item = ZmqResult<MultipartMessage>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.terminated {
            return Poll::Ready(None);
        }

        let socket = self.socket;
        let first = match socket.readiness.poll_recv(&socket.socket, ctx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(ZmqError::ContextTerminated)) => {
                self.terminated = true;
                return Poll::Ready(None);
            }
            Poll::Ready(first) => first,
        };

        Poll::Ready(Some(first.and_then(|first| {
            let mut parts = MultipartMessage::new();
            let mut got_more = first.get_more();
            parts.push_back(first);
            while got_more {
                let part = socket.recv_msg(RecvFlags::DONT_WAIT)?;
                got_more = part.get_more();
                parts.push_back(part);
            }
            Ok(parts)
        })))
    }
}

impl<T> FusedStream for MultipartStream<'_, T>
where
    T: sealed::SocketType,
    Socket<T>: MultipartReceiver,
{
    fn is_terminated(&self) -> bool {
        self.terminated
    }
}

#[cfg(test)]
mod multipart_stream_tests {
    use std::{thread, time::Duration};

    use futures::{StreamExt, executor::block_on};

    use crate::prelude::{
        Context, MultipartMessage, MultipartSender, PairSocket, SendFlags, Sender, ZmqResult,
    };

    fn pair(context: &Context, endpoint: &str) -> ZmqResult<(PairSocket, PairSocket)> {
        let server = PairSocket::from_context(context)?;
        server.bind(endpoint)?;
        let client = PairSocket::from_context(context)?;
        client.connect(endpoint)?;
        Ok((server, client))
    }

    #[test]
    fn stream_yields_complete_multipart_messages_in_order() -> ZmqResult<()> {
        let context = Context::new()?;
        let (server, client) = pair(&context, "inproc://multipart-stream-order-test")?;

        let sender = thread::spawn(move || {
            for i in 0..50 {
                let multipart = MultipartMessage::from([format!("head{i}"), format!("tail{i}")]);
                client
                    .send_multipart(multipart, SendFlags::empty())
                    .unwrap();
                if i % 10 == 0 {
                    thread::sleep(Duration::from_millis(5));
                }
            }
            client
        });

        let received = block_on(server.multipart_stream().take(50).collect::<Vec<_>>());

        sender.join().unwrap();
        assert_eq!(received.len(), 50);
        for (i, multipart) in received.into_iter().enumerate() {
            let expected = MultipartMessage::from([format!("head{i}"), format!("tail{i}")]);
            assert_eq!(multipart?, expected);
        }
        Ok(())
    }

    #[test]
    fn stream_yields_single_part_message_as_one_part() -> ZmqResult<()> {
        let context = Context::new()?;
        let (server, client) = pair(&context, "inproc://multipart-stream-single-test")?;
        client.send_msg("Hello", SendFlags::empty())?;

        let multipart = block_on(server.multipart_stream().next()).unwrap()?;

        assert_eq!(multipart.len(), 1);
        assert_eq!(multipart[0], "Hello");
        Ok(())
    }

    #[test]
    fn pending_stream_does_not_lose_messages_when_dropped() -> ZmqResult<()> {
        let context = Context::new()?;
        let (server, client) = pair(&context, "inproc://multipart-stream-cancel-test")?;

        let timed_out = block_on(async {
            let mut stream = server.multipart_stream();
            let outcome = futures::future::select(stream.next(), futures::future::ready(())).await;
            matches!(outcome, futures::future::Either::Right(_))
        });
        assert!(timed_out);

        client.send_multipart(MultipartMessage::from(["a", "b"]), SendFlags::empty())?;
        let multipart = block_on(server.multipart_stream().next()).unwrap()?;

        assert_eq!(multipart, MultipartMessage::from(["a", "b"]));
        Ok(())
    }

    #[test]
    fn stream_ends_when_context_is_shut_down() -> ZmqResult<()> {
        use futures::stream::FusedStream;

        let context = Context::new()?;
        let (server, _client) = pair(&context, "inproc://multipart-stream-shutdown-test")?;
        let mut stream = server.multipart_stream();
        assert!(!stream.is_terminated());

        context.shutdown()?;

        assert!(block_on(stream.next()).is_none());
        assert!(stream.is_terminated());
        assert!(block_on(stream.next()).is_none());
        Ok(())
    }

    #[test]
    fn stream_borrowing_socket_is_unpin_and_send() -> ZmqResult<()> {
        fn assert_unpin_send<S: Unpin + Send>(_: &S) {}

        let context = Context::new()?;
        let (server, _client) = pair(&context, "inproc://multipart-stream-send-test")?;

        assert_unpin_send(&server.multipart_stream());
        Ok(())
    }
}
