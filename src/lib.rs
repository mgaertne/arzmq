#![doc = include_str!("../README.md")]
#![doc = include_str!("../features.md")]
#![cfg_attr(
    nightly,
    feature(cold_path, doc_cfg, stmt_expr_attributes)
)]
#![allow(clippy::items_after_test_module)]
#![doc(test(no_crate_inject))]
#![deny(
    rustdoc::private_intra_doc_links,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::bare_urls,
    rustdoc::private_doc_tests,
    rustdoc::invalid_html_tags,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::unescaped_backticks,
    rustdoc::redundant_explicit_links
)]
extern crate alloc;
extern crate core;

pub mod auth;
pub mod context;
#[doc(hidden)]
pub mod error;
mod ffi;
pub mod message;
pub mod security;
pub mod socket;

use alloc::ffi::CString;
#[cfg(nightly)]
use core::hint::cold_path;
use core::ptr;

#[doc(hidden)]
pub(crate) use arzmq_sys as zmq_sys_crate;
use derive_more::Display;
#[doc(inline)]
pub use error::{ZmqError, ZmqResult};

pub mod prelude {
    #[cfg(feature = "builder")]
    pub use crate::context::ContextBuilder;
    #[cfg(all(feature = "draft-api", feature = "builder"))]
    pub use crate::socket::{
        ChannelBuilder, ClientBuilder, DishBuilder, GatherBuilder, PeerBuilder, RadioBuilder,
        ScatterBuilder, ServerBuilder,
    };
    #[cfg(feature = "draft-api")]
    pub use crate::socket::{
        ChannelSocket, ClientSocket, DishSocket, GatherSocket, PeerSocket, RadioSocket,
        ScatterSocket, ServerSocket,
    };
    #[cfg(feature = "builder")]
    pub use crate::socket::{
        DealerBuilder, PairBuilder, PublishBuilder, PullBuilder, PushBuilder, ReplyBuilder,
        RequestBuilder, RouterBuilder, SocketBuilder, StreamBuilder, SubscribeBuilder,
        XPublishBuilder, XSubscribeBuilder,
    };
    pub use crate::{
        ZmqError, ZmqResult,
        context::{Context, ContextOption},
        message::{Message, MultipartMessage},
        socket::{
            DealerSocket, MonitorFlags, MonitorReceiver, MonitorSocket, MonitorSocketEvent,
            MultipartReceiver, MultipartSender, PairSocket, PublishSocket, PullSocket, PushSocket,
            Receiver, RecvFlags, ReplySocket, RequestSocket, RouterSocket, SendFlags, Sender,
            Socket, SocketOption, StreamSocket, SubscribeSocket, XPublishSocket, XSubscribeSocket,
        },
    };
}

mod sealed {
    use crate::socket;

    pub trait ReceiverFlag {}
    pub trait SenderFlag {}
    pub trait SocketType {
        fn raw_socket_type() -> socket::SocketType;
    }
}

#[derive(Debug, Display, Clone, Eq, PartialEq)]
/// 0MQ capabilities for use with the [`has_capability()`] function.
///
/// [`has_capability()`]: has_capability
pub enum Capability {
    /// whether the library supports the `ipc://` protocol
    #[display("ipc")]
    Ipc,
    /// whether the library supports the `pgm://` protocol
    #[display("pgm")]
    Pgm,
    /// whether the library supports the `tipc://` protocol
    #[display("tipc")]
    Tipc,
    /// whether the library support the `vmci://` protocol
    #[display("vmci")]
    Vmci,
    /// whether the library supports the `norm://` protocol
    #[display("norm")]
    Norm,
    /// whether the library supports the CURVE security mechanism
    #[display("curve")]
    Curve,
    /// whether the library supports the GSSAPI security mechanism
    #[display("gssapi")]
    GssApi,
    /// whether the library is built with the draft api
    #[display("draft")]
    Draft,
}

/// # check a 0MQ capability
///
/// The [`has_capability()`] function shall report whether a specified capability is available in
/// the library. This allows bindings and applications to probe a library directly, for transport
/// and security options.
///
/// # Examples
///
/// Check whether the library provides support for `ipc` transports:
/// ```
/// use arzmq::{has_capability, Capability};
///
/// assert!(has_capability(Capability::Ipc));
/// ```
/// Check whether the library was built with draft capability:
/// ```
/// use arzmq::{has_capability, Capability};
///
/// assert_eq!(has_capability(Capability::Draft), cfg!(feature = "draft-api"));
/// ```
///
/// [`has_capability()`]: #method.has_capability
pub fn has_capability(capability: Capability) -> bool {
    let c_str = CString::new(capability.to_string()).unwrap();
    unsafe { zmq_sys_crate::zmq_has(c_str.as_ptr()) != 0 }
}

#[cfg(test)]
mod has_capability_tests {
    use super::{Capability, has_capability};

    #[test]
    fn has_ipc_capability() {
        assert!(has_capability(Capability::Ipc));
    }

    #[test]
    fn has_curve_capability() {
        assert_eq!(has_capability(Capability::Curve), cfg!(zmq_has = "curve"));
    }

    #[test]
    fn has_draft_capability() {
        assert_eq!(
            has_capability(Capability::Draft),
            cfg!(feature = "draft-api")
        );
    }
}

/// Return the current zeromq version, as `(major, minor, patch)`.
pub fn version() -> (i32, i32, i32) {
    let mut major = Default::default();
    let mut minor = Default::default();
    let mut patch = Default::default();

    unsafe { zmq_sys_crate::zmq_version(&mut major, &mut minor, &mut patch) };

    (major, minor, patch)
}

#[cfg(test)]
mod version_tests {
    use super::{version, zmq_sys_crate};

    #[test]
    fn version_returns_sys_values() {
        let (major, minor, patch) = version();
        assert_eq!(major, zmq_sys_crate::ZMQ_VERSION_MAJOR as i32);
        assert_eq!(minor, zmq_sys_crate::ZMQ_VERSION_MINOR as i32);
        assert_eq!(patch, zmq_sys_crate::ZMQ_VERSION_PATCH as i32);
    }
}

use crate::socket::Socket;

/// # Start built-in 0MQ proxy
///
/// The [`proxy()`] function starts the built-in 0MQ proxy in the current application thread.
///
/// The proxy connects a frontend socket to a backend socket. Conceptually, data flows from
/// frontend to backend. Depending on the socket types, replies may flow in the opposite direction.
/// The direction is conceptual only; the proxy is fully symmetric and there is no technical
/// difference between frontend and backend.
///
/// Before calling [`proxy()`] you must set any socket options, and connect or bind both frontend
/// and backend sockets.
///
/// [`proxy()`] runs in the current thread and returns only if/when the current context is closed.
///
/// If the capture socket is not `None`, the proxy shall send all messages, received on both
/// frontend and backend, to the capture socket. The capture socket should be a [`Publish`],
/// [`Dealer`], [`Push`], or [`Pair`] socket.
///
/// [`proxy()`]: #method.proxy
/// [`Publish`]: socket::PublishSocket
/// [`Dealer`]: socket::DealerSocket
/// [`Push`]: socket::PushSocket
/// [`Pair`]: socket::PairSocket
pub fn proxy<T, U, V>(
    frontend: &Socket<T>,
    backend: &Socket<U>,
    capture: Option<&Socket<V>>,
) -> ZmqResult<()>
where
    T: sealed::SocketType,
    U: sealed::SocketType,
    V: sealed::SocketType,
{
    let frontend_guard = frontend.socket.socket.lock();
    let backend_guard = backend.socket.socket.lock();
    let return_code = match capture {
        None => unsafe {
            zmq_sys_crate::zmq_proxy(*frontend_guard, *backend_guard, ptr::null_mut())
        },
        Some(capture) => {
            let capture_guard = capture.socket.socket.lock();
            unsafe { zmq_sys_crate::zmq_proxy(*frontend_guard, *backend_guard, *capture_guard) }
        }
    };

    if return_code == -1 {
        #[cfg(nightly)]
        cold_path();
        match unsafe { zmq_sys_crate::zmq_errno() } {
            errno @ (zmq_sys_crate::errno::ETERM
            | zmq_sys_crate::errno::EINTR
            | zmq_sys_crate::errno::EFAULT) => {
                return Err(ZmqError::from(errno));
            }
            _ => unreachable!(),
        }
    }

    unreachable!()
}

#[cfg(test)]
mod proxy_tests {
    use std::thread;

    use super::{ZmqError, proxy};
    use crate::prelude::{
        Context, DealerSocket, MultipartReceiver, PairSocket, RecvFlags, RouterSocket, SendFlags,
        Sender, ZmqResult,
    };

    #[test]
    fn proxy_between_frontend_and_backend() -> ZmqResult<()> {
        let context = Context::new()?;

        let frontend_router = RouterSocket::from_context(&context)?;
        frontend_router.bind("inproc://proxy-frontend")?;

        let external_dealer = DealerSocket::from_context(&context)?;
        external_dealer.connect("inproc://proxy-frontend")?;

        let backend_dealer = DealerSocket::from_context(&context)?;
        backend_dealer.bind("inproc://proxy-backend")?;

        let receiving_dealer = DealerSocket::from_context(&context)?;
        receiving_dealer.connect("inproc://proxy-backend")?;

        thread::spawn(move || {
            let _ = proxy(&frontend_router, &backend_dealer, None::<&PairSocket>);
        });

        external_dealer.send_msg("proxied msg", SendFlags::empty())?;

        let mut received = receiving_dealer.recv_multipart(RecvFlags::empty())?;

        assert_eq!(
            received
                .pop_back()
                .expect("this should not happen")
                .to_string(),
            "proxied msg"
        );

        Ok(())
    }

    #[test]
    fn proxy_between_frontend_and_backend_with_capture() -> ZmqResult<()> {
        let context = Context::new()?;

        let frontend_router = RouterSocket::from_context(&context)?;
        frontend_router.bind("inproc://proxy-frontend")?;

        let external_dealer = DealerSocket::from_context(&context)?;
        external_dealer.connect("inproc://proxy-frontend")?;

        let backend_dealer = DealerSocket::from_context(&context)?;
        backend_dealer.bind("inproc://proxy-backend")?;

        let receiving_dealer = DealerSocket::from_context(&context)?;
        receiving_dealer.connect("inproc://proxy-backend")?;

        let capture_socket = PairSocket::from_context(&context)?;
        capture_socket.bind("inproc://proxy-capture")?;

        let capture_pair = PairSocket::from_context(&context)?;
        capture_pair.connect("inproc://proxy-capture")?;

        thread::spawn(move || {
            let _ = proxy(&frontend_router, &backend_dealer, Some(&capture_socket));
        });

        external_dealer.send_msg("proxied msg", SendFlags::empty())?;

        let mut captured = capture_pair.recv_multipart(RecvFlags::empty())?;
        assert!(
            captured
                .pop_back()
                .is_some_and(|message| message.to_string() == "proxied msg")
        );

        let mut received = receiving_dealer.recv_multipart(RecvFlags::empty())?;

        assert!(
            received
                .pop_back()
                .is_some_and(|message| message.to_string() == "proxied msg")
        );

        Ok(())
    }

    #[test]
    fn proxy_when_frontend_context_is_terminated() -> ZmqResult<()> {
        let context = Context::new()?;

        let frontend_router = RouterSocket::from_context(&context)?;
        let backend_dealer = DealerSocket::from_context(&context)?;

        context.shutdown()?;

        let result = proxy(&frontend_router, &backend_dealer, None::<&PairSocket>);

        assert!(result.is_err_and(|err| err == ZmqError::ContextTerminated));

        Ok(())
    }

    #[test]
    fn proxy_when_backend_context_is_terminated() -> ZmqResult<()> {
        let frontend_context = Context::new()?;
        let frontend_router = RouterSocket::from_context(&frontend_context)?;

        let backend_context = Context::new()?;
        let backend_dealer = DealerSocket::from_context(&backend_context)?;

        backend_context.shutdown()?;

        let result = proxy(&frontend_router, &backend_dealer, None::<&PairSocket>);

        assert!(result.is_err_and(|err| err == ZmqError::ContextTerminated));

        Ok(())
    }

    #[test]
    fn proxy_when_context_is_shutdown_while_running() -> ZmqResult<()> {
        let context = Context::new()?;

        let frontend_router = RouterSocket::from_context(&context)?;

        let backend_dealer = DealerSocket::from_context(&context)?;

        let handle =
            thread::spawn(move || proxy(&frontend_router, &backend_dealer, None::<&PairSocket>));

        context.shutdown()?;

        let result = handle.join();

        assert!(
            result.is_ok_and(|result| result.is_err_and(|err| err == ZmqError::ContextTerminated))
        );

        Ok(())
    }
}
