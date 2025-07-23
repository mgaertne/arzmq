//! # Asynchronous Rust bindings for 0MQ (arzmq)

#![feature(cold_path, doc_cfg, stmt_expr_attributes)]
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
use core::{hint::cold_path, ptr};

#[doc(hidden)]
pub(crate) use arzmq_sys as zmq_sys_crate;
use derive_more::Display;
#[doc(inline)]
pub use error::{ZmqError, ZmqResult};

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
        assert_eq!(
            has_capability(Capability::Curve),
            cfg!(all(feature = "curve", not(windows)))
        );
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
pub fn proxy<T: sealed::SocketType>(
    frontend: Socket<T>,
    backend: Socket<T>,
    capture: Option<Socket<T>>,
) -> ZmqResult<()> {
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

    Ok(())
}
