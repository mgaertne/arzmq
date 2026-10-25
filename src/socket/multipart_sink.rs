//! A [`Sink`] for the multipart messages sent to a socket.

use alloc::{collections::VecDeque, sync::Arc};
use core::{
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
    task::{Context, Poll},
};

use futures::sink::Sink;

use crate::{
    ZmqError, ZmqResult,
    message::{Message, MultipartMessage},
    sealed,
    socket::{MultipartSender, SendFlags, Socket},
};

/// # Sink for the multipart messages sent to a socket
///
/// Created by [`Socket::multipart_sink()`] or [`Socket::into_multipart_sink()`]. The sink holds one
/// message at a time: when the socket cannot accept it right away, for example because the high
/// water mark has been reached, [`poll_ready`] suspends the task until the previous message has
/// been queued. That passes the back pressure of the socket on to the producer.
///
/// 0MQ accepts all parts of a multipart message or none, so a message is never queued partially.
/// Tasks are woken up by the readiness notification of the [`FileDescriptor`] option, so the sink
/// works with any async runtime.
///
/// [`poll_ready`]: Sink::poll_ready
/// [`FileDescriptor`]: crate::socket::SocketOption::FileDescriptor
#[derive(Debug)]
pub struct MultipartSink<S, T: sealed::SocketType> {
    socket: S,
    pending: VecDeque<Message>,
    marker: PhantomData<fn() -> T>,
}

impl<T> Socket<T>
where
    T: sealed::SocketType,
    Socket<T>: MultipartSender,
{
    /// # Sink the multipart messages to this socket
    ///
    /// See [`MultipartSink`].
    pub fn multipart_sink(&self) -> MultipartSink<&Socket<T>, T> {
        MultipartSink::new(self)
    }

    /// # Sink the multipart messages to a shared socket
    ///
    /// Like [`multipart_sink()`], but keeps the socket alive itself.
    ///
    /// [`multipart_sink()`]: Socket::multipart_sink
    pub fn into_multipart_sink(self: Arc<Self>) -> MultipartSink<Arc<Socket<T>>, T> {
        MultipartSink::new(self)
    }
}

impl<S, T> MultipartSink<S, T>
where
    S: Deref<Target = Socket<T>>,
    T: sealed::SocketType,
{
    fn new(socket: S) -> Self {
        Self {
            socket,
            pending: VecDeque::new(),
            marker: PhantomData,
        }
    }

    fn poll_send_pending(&mut self, ctx: &mut Context<'_>) -> Poll<ZmqResult<()>> {
        while let Some(part) = self.pending.front() {
            let flags = if self.pending.len() > 1 {
                SendFlags::SEND_MORE
            } else {
                SendFlags::empty()
            };
            let socket = &*self.socket;
            match socket.readiness.poll_send(&socket.socket, part, flags, ctx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(error)) => {
                    self.pending.clear();
                    return Poll::Ready(Err(error));
                }
                Poll::Ready(Ok(())) => {
                    self.pending.pop_front();
                }
            }
        }
        Poll::Ready(Ok(()))
    }
}

impl<S, T, M> Sink<M> for MultipartSink<S, T>
where
    S: Deref<Target = Socket<T>> + Unpin,
    T: sealed::SocketType,
    Socket<T>: MultipartSender,
    M: Into<MultipartMessage>,
{
    type Error = ZmqError;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.poll_send_pending(ctx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: M) -> Result<(), Self::Error> {
        self.pending.extend(item.into());
        Ok(())
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.poll_send_pending(ctx)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.poll_send_pending(ctx)
    }
}

#[cfg(test)]
mod multipart_sink_tests {
    use alloc::sync::Arc;
    use std::{thread, time::Duration};

    use futures::{SinkExt, StreamExt, executor::block_on, stream};

    use crate::prelude::{
        Context, MultipartMessage, MultipartReceiver, PairSocket, PullSocket, PushSocket, Receiver,
        RecvFlags, SendFlags, SocketOption, ZmqError, ZmqResult,
    };

    fn pair(context: &Context, endpoint: &str) -> ZmqResult<(PairSocket, PairSocket)> {
        let server = PairSocket::from_context(context)?;
        server.bind(endpoint)?;
        let client = PairSocket::from_context(context)?;
        client.connect(endpoint)?;
        Ok((server, client))
    }

    #[test]
    fn sink_sends_multipart_messages_in_order() -> ZmqResult<()> {
        let context = Context::new()?;
        let (server, client) = pair(&context, "inproc://multipart-sink-order-test")?;

        let mut sink = client.multipart_sink();
        block_on(async {
            for i in 0..20 {
                sink.send(MultipartMessage::from([format!("a{i}"), format!("b{i}")]))
                    .await
                    .unwrap();
            }
        });

        for i in 0..20 {
            let received = server.recv_multipart(RecvFlags::DONT_WAIT)?;
            assert_eq!(
                received,
                MultipartMessage::from([format!("a{i}"), format!("b{i}")])
            );
        }
        Ok(())
    }

    #[test]
    fn sink_accepts_a_stream_of_messages() -> ZmqResult<()> {
        let context = Context::new()?;
        let (server, client) = pair(&context, "inproc://multipart-sink-stream-test")?;
        let client = Arc::new(client);

        let messages = (0..5).map(|i| Ok(MultipartMessage::from([i.to_string()])));
        block_on(stream::iter(messages).forward(client.into_multipart_sink())).unwrap();

        for i in 0..5 {
            let received = server.recv_multipart(RecvFlags::DONT_WAIT)?;
            assert_eq!(received, MultipartMessage::from([i.to_string()]));
        }
        Ok(())
    }

    #[test]
    fn sink_waits_for_the_socket_instead_of_failing_at_the_high_water_mark() -> ZmqResult<()> {
        let context = Context::new()?;
        let pull = PullSocket::from_context(&context)?;
        pull.set_sockopt_int(SocketOption::ReceiveHighWatermark, 1)?;
        pull.bind("inproc://multipart-sink-hwm-test")?;
        let push = PushSocket::from_context(&context)?;
        push.set_sockopt_int(SocketOption::SendHighWatermark, 1)?;
        push.connect("inproc://multipart-sink-hwm-test")?;

        const COUNT: usize = 50;
        let receiver = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            (0..COUNT)
                .map(|_| block_on(pull.recv_msg_wait()).unwrap().to_string())
                .collect::<Vec<_>>()
        });

        let mut sink = push.multipart_sink();
        block_on(async {
            for i in 0..COUNT {
                sink.send(MultipartMessage::from([i.to_string()]))
                    .await
                    .unwrap();
            }
            SinkExt::<MultipartMessage>::close(&mut sink).await.unwrap();
        });

        let received = receiver.join().unwrap();
        let expected: Vec<_> = (0..COUNT).map(|i| i.to_string()).collect();
        assert_eq!(received, expected);
        Ok(())
    }

    #[test]
    fn send_msg_wait_waits_until_the_message_is_queued() -> ZmqResult<()> {
        let context = Context::new()?;
        let pull = PullSocket::from_context(&context)?;
        pull.set_sockopt_int(SocketOption::ReceiveHighWatermark, 1)?;
        pull.bind("inproc://send-wait-hwm-test")?;
        let push = PushSocket::from_context(&context)?;
        push.set_sockopt_int(SocketOption::SendHighWatermark, 1)?;
        push.connect("inproc://send-wait-hwm-test")?;

        use crate::prelude::Sender;
        let receiver = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            (0..10)
                .map(|_| pull.recv_msg(RecvFlags::empty()).unwrap().to_string())
                .collect::<Vec<_>>()
        });

        block_on(async {
            for i in 0..10 {
                push.send_msg_wait(i.to_string(), SendFlags::empty())
                    .await
                    .unwrap();
            }
        });

        assert_eq!(receiver.join().unwrap().len(), 10);
        Ok(())
    }

    #[test]
    fn sink_reports_termination_and_drops_the_pending_message() -> ZmqResult<()> {
        let context = Context::new()?;
        let (_server, client) = pair(&context, "inproc://multipart-sink-term-test")?;
        let mut sink = client.multipart_sink();

        context.shutdown()?;

        let result = block_on(sink.send(MultipartMessage::from(["a", "b"])));
        assert_eq!(result, Err(ZmqError::ContextTerminated));
        assert_eq!(
            block_on(SinkExt::<MultipartMessage>::flush(&mut sink)),
            Ok(())
        );
        Ok(())
    }

    #[test]
    fn sink_is_send_and_unpin() -> ZmqResult<()> {
        fn assert_send_unpin<S: Send + Unpin>(_: &S) {}

        let context = Context::new()?;
        let (_server, client) = pair(&context, "inproc://multipart-sink-send-test")?;

        assert_send_unpin(&client.multipart_sink());
        assert_send_unpin(&Arc::new(client).into_multipart_sink());
        Ok(())
    }
}
