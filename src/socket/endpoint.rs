//! Handles for endpoints a [`Socket`] has been bound or connected to.
//!
//! A 0MQ socket can be bound and connected to an arbitrary number of endpoints, so whether a socket
//! is "bound" or "connected" is no property of the socket itself. Instead, [`bind_endpoint()`] and
//! [`connect_endpoint()`] hand out a [`BoundEndpoint`] or [`ConnectedEndpoint`] handle for exactly
//! the endpoint that was set up. The handle is consumed by [`unbind()`] or [`disconnect()`], so
//! that an endpoint can only be released once, and only if it was actually established before.
//!
//! The handles do not keep the socket alive and do not release the endpoint when dropped; the
//! endpoint stays established until it is explicitly released or the socket is closed.
//!
//! [`bind_endpoint()`]: Socket::bind_endpoint
//! [`connect_endpoint()`]: Socket::connect_endpoint
//! [`unbind()`]: BoundEndpoint::unbind
//! [`disconnect()`]: ConnectedEndpoint::disconnect

use alloc::sync::{Arc, Weak};
use core::{fmt, marker::PhantomData};

use crate::{ZmqError, ZmqResult, ffi::RawSocket, sealed, socket::Socket};

/// # Handle of an endpoint a socket accepts connections on
///
/// Returned by [`Socket::bind_endpoint()`]. Wild-card endpoints are resolved to the actual
/// endpoint, so [`unbind()`] always addresses what has really been bound.
///
/// [`unbind()`]: BoundEndpoint::unbind
pub struct BoundEndpoint<T: sealed::SocketType> {
    socket: Weak<RawSocket>,
    endpoint: String,
    marker: PhantomData<fn() -> T>,
}

/// # Handle of an endpoint a socket has been connected to
///
/// Returned by [`Socket::connect_endpoint()`].
pub struct ConnectedEndpoint<T: sealed::SocketType> {
    socket: Weak<RawSocket>,
    endpoint: String,
    marker: PhantomData<fn() -> T>,
}

unsafe impl<T> Send for BoundEndpoint<T>
where
    T: sealed::SocketType + 'static,
    Socket<T>: Send,
{
}
unsafe impl<T> Sync for BoundEndpoint<T>
where
    T: sealed::SocketType + 'static,
    Socket<T>: Sync,
{
}
unsafe impl<T> Send for ConnectedEndpoint<T>
where
    T: sealed::SocketType + 'static,
    Socket<T>: Send,
{
}
unsafe impl<T> Sync for ConnectedEndpoint<T>
where
    T: sealed::SocketType + 'static,
    Socket<T>: Sync,
{
}

impl<T: sealed::SocketType> BoundEndpoint<T> {
    /// The endpoint that has been bound, with wild-cards resolved.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// # Stop accepting connections on this endpoint
    ///
    /// Unbinds the socket from the endpoint, see [`Socket::unbind()`].
    ///
    /// Returns [`ZmqError::SocketNull`] when the socket has been closed in the meantime.
    pub fn unbind(self) -> ZmqResult<()> {
        self.socket
            .upgrade()
            .ok_or(ZmqError::SocketNull)?
            .unbind(&self.endpoint)
    }
}

impl<T: sealed::SocketType> ConnectedEndpoint<T> {
    /// The endpoint the socket has been connected to.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// # Disconnect the socket from this endpoint
    ///
    /// Disconnects the socket from the endpoint, see [`Socket::disconnect()`].
    ///
    /// Returns [`ZmqError::SocketNull`] when the socket has been closed in the meantime.
    pub fn disconnect(self) -> ZmqResult<()> {
        self.socket
            .upgrade()
            .ok_or(ZmqError::SocketNull)?
            .disconnect(&self.endpoint)
    }
}

impl<T: sealed::SocketType> fmt::Debug for BoundEndpoint<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoundEndpoint")
            .field("endpoint", &self.endpoint)
            .finish_non_exhaustive()
    }
}

impl<T: sealed::SocketType> fmt::Debug for ConnectedEndpoint<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectedEndpoint")
            .field("endpoint", &self.endpoint)
            .finish_non_exhaustive()
    }
}

impl<T: sealed::SocketType> Socket<T> {
    /// # Accept connections on an endpoint and return its handle
    ///
    /// Like [`bind()`], but returns a [`BoundEndpoint`] that can later be handed to
    /// [`BoundEndpoint::unbind()`]. For wild-card endpoints the handle contains the resolved
    /// endpoint, taken from [`last_endpoint()`] right after binding. When other threads bind the
    /// same socket concurrently, the resolved endpoint might not be the one bound by this call.
    ///
    /// ## Example
    ///
    /// ```
    /// # use arzmq::prelude::{Context, PairSocket, ZmqResult};
    /// # fn main() -> ZmqResult<()> {
    /// let context = Context::new()?;
    /// let socket = PairSocket::from_context(&context)?;
    ///
    /// let bound = socket.bind_endpoint("inproc://bind-endpoint-doc")?;
    /// assert_eq!(bound.endpoint(), "inproc://bind-endpoint-doc");
    /// bound.unbind()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`bind()`]: #method.bind
    /// [`last_endpoint()`]: #method.last_endpoint
    pub fn bind_endpoint<E>(&self, endpoint: E) -> ZmqResult<BoundEndpoint<T>>
    where
        E: AsRef<str>,
    {
        let endpoint = endpoint.as_ref();
        self.bind(endpoint)?;

        let endpoint = if endpoint.contains('*') {
            self.last_endpoint()?
        } else {
            endpoint.to_owned()
        };
        Ok(BoundEndpoint {
            socket: Arc::downgrade(&self.socket),
            endpoint,
            marker: PhantomData,
        })
    }

    /// # Connect to an endpoint and return its handle
    ///
    /// Like [`connect()`], but returns a [`ConnectedEndpoint`] that can later be handed to
    /// [`ConnectedEndpoint::disconnect()`].
    ///
    /// The handle only tells that the connection has been set up, not that a peer is reachable;
    /// use a [`monitor()`] to follow the state of the actual connection.
    ///
    /// ## Example
    ///
    /// ```
    /// # use arzmq::prelude::{Context, PairSocket, ZmqResult};
    /// # fn main() -> ZmqResult<()> {
    /// let context = Context::new()?;
    /// let socket = PairSocket::from_context(&context)?;
    ///
    /// let connected = socket.connect_endpoint("inproc://connect-endpoint-doc")?;
    /// assert_eq!(connected.endpoint(), "inproc://connect-endpoint-doc");
    /// connected.disconnect()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`connect()`]: #method.connect
    /// [`monitor()`]: #method.monitor
    pub fn connect_endpoint<E>(&self, endpoint: E) -> ZmqResult<ConnectedEndpoint<T>>
    where
        E: AsRef<str>,
    {
        let endpoint = endpoint.as_ref();
        self.connect(endpoint)?;

        Ok(ConnectedEndpoint {
            socket: Arc::downgrade(&self.socket),
            endpoint: endpoint.to_owned(),
            marker: PhantomData,
        })
    }
}

#[cfg(test)]
mod endpoint_tests {
    use super::{BoundEndpoint, ConnectedEndpoint};
    use crate::{
        prelude::{
            Context, PairSocket, Receiver, RecvFlags, SendFlags, Sender, ZmqError, ZmqResult,
        },
        socket::pair::Pair,
    };

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn handles_are_send_and_sync() {
        assert_send_sync::<BoundEndpoint<Pair>>();
        assert_send_sync::<ConnectedEndpoint<Pair>>();
    }

    #[test]
    fn bind_endpoint_returns_bound_endpoint() -> ZmqResult<()> {
        let context = Context::new()?;
        let socket = PairSocket::from_context(&context)?;

        let bound = socket.bind_endpoint("inproc://bind-endpoint-test")?;

        assert_eq!(bound.endpoint(), "inproc://bind-endpoint-test");
        assert_eq!(socket.last_endpoint()?, "inproc://bind-endpoint-test");
        bound.unbind()
    }

    #[test]
    fn bind_endpoint_resolves_wildcard_endpoint() -> ZmqResult<()> {
        let context = Context::new()?;
        let socket = PairSocket::from_context(&context)?;

        let bound = socket.bind_endpoint("tcp://127.0.0.1:*")?;

        assert!(!bound.endpoint().contains('*'));
        assert_eq!(bound.endpoint(), socket.last_endpoint()?);
        bound.unbind()
    }

    #[test]
    fn bind_endpoint_with_invalid_endpoint_fails() -> ZmqResult<()> {
        let context = Context::new()?;
        let socket = PairSocket::from_context(&context)?;

        assert!(socket.bind_endpoint("invalid://endpoint").is_err());

        Ok(())
    }

    #[test]
    fn connect_endpoint_returns_connected_endpoint() -> ZmqResult<()> {
        let context = Context::new()?;
        let socket = PairSocket::from_context(&context)?;

        let connected = socket.connect_endpoint("inproc://connect-endpoint-test")?;

        assert_eq!(connected.endpoint(), "inproc://connect-endpoint-test");
        connected.disconnect()
    }

    #[test]
    fn bound_and_connected_endpoint_exchange_messages() -> ZmqResult<()> {
        let endpoint = "inproc://endpoint-exchange-test";
        let context = Context::new()?;

        let server = PairSocket::from_context(&context)?;
        let bound = server.bind_endpoint(endpoint)?;
        let client = PairSocket::from_context(&context)?;
        let connected = client.connect_endpoint(endpoint)?;

        client.send_msg("Hello", SendFlags::empty())?;
        assert_eq!(server.recv_msg(RecvFlags::empty())?.to_string(), "Hello");

        connected.disconnect()?;
        bound.unbind()
    }

    #[test]
    fn unbind_after_socket_was_closed_fails() -> ZmqResult<()> {
        let context = Context::new()?;
        let socket = PairSocket::from_context(&context)?;
        let bound = socket.bind_endpoint("inproc://unbind-closed-test")?;
        drop(socket);

        assert_eq!(bound.unbind(), Err(ZmqError::SocketNull));

        Ok(())
    }

    #[test]
    fn disconnect_after_socket_was_closed_fails() -> ZmqResult<()> {
        let context = Context::new()?;
        let socket = PairSocket::from_context(&context)?;
        let connected = socket.connect_endpoint("inproc://disconnect-closed-test")?;
        drop(socket);

        assert_eq!(connected.disconnect(), Err(ZmqError::SocketNull));

        Ok(())
    }

    #[test]
    fn debug_shows_endpoint() -> ZmqResult<()> {
        let context = Context::new()?;
        let socket = PairSocket::from_context(&context)?;
        let connected = socket.connect_endpoint("inproc://debug-test")?;

        assert_eq!(
            format!("{connected:?}"),
            "ConnectedEndpoint { endpoint: \"inproc://debug-test\", .. }"
        );

        Ok(())
    }
}
