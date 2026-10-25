//! Readiness notification of a socket for the waiting `futures` operations.
//!
//! 0MQ signals state changes of a socket on the file descriptor of the [`FileDescriptor`] option.
//! The descriptor is edge-triggered, so after each wake-up the socket has to be asked for its
//! state again; the receiving operations do that by simply retrying.
//!
//! The reactor remembers only one waker per direction. All waiting tasks therefore hand the reactor
//! the same broadcast waker, which wakes every task registered in [`Waiters`].
//!
//! [`FileDescriptor`]: super::SocketOption::FileDescriptor

use alloc::{sync::Arc, task::Wake};
use core::{
    mem,
    task::{Context, Poll, Waker},
};
use std::{
    os::fd::{AsFd, BorrowedFd, RawFd},
    sync::OnceLock,
};

use async_io::Async;
use parking_lot::Mutex;

use crate::{
    ZmqError, ZmqResult, ffi::RawSocket, message::Message, socket::RecvFlags, zmq_sys_crate,
};

/// The descriptor is owned by 0MQ, so it is never closed from here.
pub(super) struct EventFd(RawFd);

impl AsFd for EventFd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.0) }
    }
}

#[derive(Default)]
struct Waiters(Mutex<Vec<Waker>>);

impl Waiters {
    fn register(&self, waker: &Waker) {
        let mut wakers = self.0.lock();
        if !wakers.iter().any(|registered| registered.will_wake(waker)) {
            wakers.push(waker.clone());
        }
    }
}

impl Wake for Waiters {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let wakers = mem::take(&mut *self.0.lock());
        wakers.into_iter().for_each(Waker::wake);
    }
}

/// Registration of the socket descriptor with the reactor, created on first use.
pub(super) struct Readiness {
    source: OnceLock<ZmqResult<Async<EventFd>>>,
    waiters: Arc<Waiters>,
    broadcast: Waker,
}

impl Default for Readiness {
    fn default() -> Self {
        let waiters = Arc::new(Waiters::default());
        Self {
            source: OnceLock::new(),
            broadcast: Waker::from(waiters.clone()),
            waiters,
        }
    }
}

impl Readiness {
    fn source(&self, socket: &RawSocket) -> ZmqResult<&Async<EventFd>> {
        self.source
            .get_or_init(|| {
                let fd = socket.get_sockopt_int::<RawFd>(zmq_sys_crate::ZMQ_FD as i32)?;
                Async::new_nonblocking(EventFd(fd)).map_err(io_error)
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    pub(super) fn poll_recv(
        &self,
        socket: &RawSocket,
        ctx: &mut Context<'_>,
    ) -> Poll<ZmqResult<Message>> {
        let source = match self.source(socket) {
            Ok(source) => source,
            Err(error) => return Poll::Ready(Err(error)),
        };
        let mut broadcast_ctx = Context::from_waker(&self.broadcast);

        loop {
            match socket.recv(RecvFlags::DONT_WAIT.bits()) {
                Ok(raw_msg) => return Poll::Ready(Ok(Message::from_raw_msg(raw_msg))),
                Err(ZmqError::Again) => (),
                Err(error) => return Poll::Ready(Err(error)),
            }

            self.waiters.register(ctx.waker());
            match source.poll_readable(&mut broadcast_ctx) {
                Poll::Ready(Ok(())) => (),
                Poll::Ready(Err(error)) => return Poll::Ready(Err(io_error(error))),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

fn io_error(error: std::io::Error) -> ZmqError {
    ZmqError::Other(error.raw_os_error().unwrap_or(-1))
}

#[cfg(test)]
mod readiness_tests {
    use std::{sync::Arc, thread, time::Duration};

    use futures::executor::block_on;

    use crate::prelude::{Context, PairSocket, Receiver, SendFlags, Sender, ZmqResult};

    fn pair(context: &Context, endpoint: &str) -> ZmqResult<(PairSocket, PairSocket)> {
        let server = PairSocket::from_context(context)?;
        server.bind(endpoint)?;
        let client = PairSocket::from_context(context)?;
        client.connect(endpoint)?;
        Ok((server, client))
    }

    fn assert_send<T: Send>(_: &T) {}

    #[test]
    fn several_tasks_can_wait_on_one_socket() -> ZmqResult<()> {
        const WAITERS: usize = 4;
        const MESSAGES: usize = 400;

        let context = Context::new()?;
        let (server, client) = pair(&context, "inproc://multi-waiter-test")?;
        let server = Arc::new(server);

        let receivers: Vec<_> = (0..WAITERS)
            .map(|_| {
                let server = server.clone();
                thread::spawn(move || {
                    let mut received = 0;
                    while block_on(server.recv_msg_wait()).unwrap() != "stop" {
                        received += 1;
                    }
                    received
                })
            })
            .collect();

        thread::sleep(Duration::from_millis(100));
        for i in 0..MESSAGES {
            client.send_msg("data", SendFlags::empty())?;
            if i % 10 == 0 {
                thread::sleep(Duration::from_millis(1));
            }
        }
        for _ in 0..WAITERS {
            client.send_msg("stop", SendFlags::empty())?;
        }

        let received: usize = receivers.into_iter().map(|r| r.join().unwrap()).sum();
        assert_eq!(received, MESSAGES);
        Ok(())
    }

    #[test]
    fn async_operations_are_send() -> ZmqResult<()> {
        let context = Context::new()?;
        let (server, client) = pair(&context, "inproc://send-futures-test")?;

        assert_send(&server.recv_msg_async());
        assert_send(&server.recv_msg_wait());
        assert_send(&client.send_msg_async("Hello", SendFlags::empty()));

        Ok(())
    }

    #[test]
    fn socket_is_usable_from_multiple_threads_at_once() -> ZmqResult<()> {
        const SENDERS: usize = 4;
        const PER_SENDER: usize = 250;

        let context = Context::new()?;
        let (server, client) = pair(&context, "inproc://shared-socket-test")?;
        let client = Arc::new(client);

        let senders: Vec<_> = (0..SENDERS)
            .map(|_| {
                let client = client.clone();
                thread::spawn(move || {
                    for _ in 0..PER_SENDER {
                        client.send_msg("Hello", SendFlags::empty()).unwrap();
                    }
                })
            })
            .collect();

        let received = block_on(async {
            let mut received = 0;
            while received < SENDERS * PER_SENDER {
                server.recv_msg_wait().await.unwrap();
                received += 1;
            }
            received
        });

        senders.into_iter().for_each(|s| s.join().unwrap());
        assert_eq!(received, SENDERS * PER_SENDER);
        Ok(())
    }

    #[test]
    fn recv_msg_wait_returns_available_message() -> ZmqResult<()> {
        let context = Context::new()?;
        let (server, client) = pair(&context, "inproc://wait-available-test")?;
        client.send_msg("Hello", SendFlags::empty())?;

        let msg = block_on(server.recv_msg_wait())?;

        assert_eq!(msg.to_string(), "Hello");
        Ok(())
    }

    #[test]
    fn recv_msg_wait_waits_for_message_from_other_thread() -> ZmqResult<()> {
        let context = Context::new()?;
        let (server, client) = pair(&context, "inproc://wait-later-test")?;

        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            client.send_msg("Later", SendFlags::empty()).unwrap();
            client
        });

        let msg = block_on(server.recv_msg_wait())?;

        assert_eq!(msg.to_string(), "Later");
        sender.join().unwrap();
        Ok(())
    }

    #[test]
    fn recv_msg_wait_receives_all_messages_of_a_stream() -> ZmqResult<()> {
        const COUNT: usize = 2000;

        let context = Context::new()?;
        let (server, client) = pair(&context, "inproc://wait-stream-test")?;

        let sender = thread::spawn(move || {
            for i in 0..COUNT {
                client.send_msg(i.to_string(), SendFlags::empty()).unwrap();
                if i % 50 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
            client
        });

        block_on(async {
            for i in 0..COUNT {
                let msg = server.recv_msg_wait().await.unwrap();
                assert_eq!(msg.to_string(), i.to_string());
            }
        });

        sender.join().unwrap();
        Ok(())
    }
}
