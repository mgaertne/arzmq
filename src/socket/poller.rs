//! Waiting for events on several sockets at once.

use crate::{
    ZmqResult,
    ffi::{RawSocket, poll_sockets},
    sealed,
    socket::{PollEvents, Socket},
};

/// # Wait for events on several sockets
///
/// Collects sockets together with the [`PollEvents`] to wait for, and waits until at least one of
/// them is ready, see `zmq_poll()`. The poller only borrows the sockets, so it cannot outlive them.
///
/// ## Example
/// ```
/// # use arzmq::prelude::{Context, PairSocket, SendFlags, Sender, ZmqResult};
/// # use arzmq::socket::{PollEvents, Poller};
/// # fn main() -> ZmqResult<()> {
/// let context = Context::new()?;
/// let server = PairSocket::from_context(&context)?;
/// server.bind("inproc://poller-doc-example")?;
/// let client = PairSocket::from_context(&context)?;
/// client.connect("inproc://poller-doc-example")?;
///
/// client.send_msg("Hello", SendFlags::empty())?;
///
/// let ready = Poller::new()
///     .with(&server, PollEvents::POLL_IN)
///     .with(&client, PollEvents::POLL_IN)
///     .poll(1000)?;
///
/// assert_eq!(ready, [PollEvents::POLL_IN, PollEvents::empty()]);
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct Poller<'a> {
    items: Vec<(&'a RawSocket, PollEvents)>,
}

impl<'a> Poller<'a> {
    /// Creates a poller without sockets.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a socket and the events to wait for. The results of [`poll()`] are in the order the
    /// sockets have been added.
    ///
    /// [`poll()`]: Poller::poll
    #[must_use]
    pub fn with<T: sealed::SocketType>(
        mut self,
        socket: &'a Socket<T>,
        events: PollEvents,
    ) -> Self {
        self.items.push((&*socket.socket, events));
        self
    }

    /// The number of sockets that are waited for.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether no sockets have been added.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// # Wait until a socket is ready
    ///
    /// Waits at most `timeout_ms` milliseconds, or indefinitely for `-1`, and returns the events
    /// that occurred for each socket. All sockets have no events when the timeout has passed.
    pub fn poll(&self, timeout_ms: i64) -> ZmqResult<Vec<PollEvents>> {
        poll_sockets(&self.items, timeout_ms)
    }
}

#[cfg(test)]
mod poller_tests {
    use super::Poller;
    use crate::{
        prelude::{Context, PairSocket, SendFlags, Sender, ZmqError, ZmqResult},
        socket::PollEvents,
    };

    fn pair(context: &Context, endpoint: &str) -> ZmqResult<(PairSocket, PairSocket)> {
        let server = PairSocket::from_context(context)?;
        server.bind(endpoint)?;
        let client = PairSocket::from_context(context)?;
        client.connect(endpoint)?;
        Ok((server, client))
    }

    #[test]
    fn poll_reports_ready_sockets_in_the_order_they_were_added() -> ZmqResult<()> {
        let context = Context::new()?;
        let (server1, client1) = pair(&context, "inproc://poller-first-test")?;
        let (server2, client2) = pair(&context, "inproc://poller-second-test")?;
        client2.send_msg("Hello", SendFlags::empty())?;

        let poller = Poller::new()
            .with(&server1, PollEvents::POLL_IN)
            .with(&server2, PollEvents::POLL_IN)
            .with(&client1, PollEvents::POLL_OUT);

        assert_eq!(poller.len(), 3);
        assert_eq!(
            poller.poll(1000)?,
            [
                PollEvents::empty(),
                PollEvents::POLL_IN,
                PollEvents::POLL_OUT
            ]
        );
        Ok(())
    }

    #[test]
    fn poll_returns_no_events_after_timeout() -> ZmqResult<()> {
        let context = Context::new()?;
        let (server, _client) = pair(&context, "inproc://poller-timeout-test")?;

        let ready = Poller::new().with(&server, PollEvents::POLL_IN).poll(10)?;

        assert_eq!(ready, [PollEvents::empty()]);
        Ok(())
    }

    #[test]
    fn empty_poller_waits_for_the_timeout() -> ZmqResult<()> {
        let poller = Poller::new();

        assert!(poller.is_empty());
        assert_eq!(poller.poll(1)?, []);
        Ok(())
    }

    #[test]
    fn poll_reports_terminated_context() -> ZmqResult<()> {
        let context = Context::new()?;
        let (server, _client) = pair(&context, "inproc://poller-term-test")?;
        context.shutdown()?;

        let result = Poller::new().with(&server, PollEvents::POLL_IN).poll(10);

        assert_eq!(result, Err(ZmqError::ContextTerminated));
        Ok(())
    }
}
