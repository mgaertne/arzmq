//! # 0MQ context
//!
//! The 0MQ [`Context`] keeps the list of sockets and manages the async I/O thread and internal
//! queries.
//!
//! Before using any 0MQ library functions you must create a 0MQ [`Context`].
//!
//! ## Multiple contexts
//! Multiple [`Context`] may coexist within a single application. Thus, an application can use 0MQ
//! directly and at the same time make use of any number of additional libraries or components
//! which themselves make use of 0MQ:
//!
//! ## Example
//! ```
//! # use arzmq::{ZmqResult, context::Context, socket::SubscribeSocket};
//! # fn main() -> ZmqResult<()> {
//! #
//! let context = Context::new()?;
//! context.set_blocky(false)?;
//!
//! let socket = SubscribeSocket::from_context(&context)?;
//!
//! # Ok(())
//! # }
//!
//! ```
//!
//! [`Context`]: Context

use alloc::sync::Arc;

#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use builder::ContextBuilder;
use derive_more::{Debug as DebugDeriveMore, Display as DisplayDeriveMore};
use num_traits::PrimInt;

use crate::{ZmqResult, ffi::RawContext, zmq_sys_crate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Options that can be set and/or retrieved on a 0MQ [`Context`]
pub enum ContextOption {
    /// Number of I/O threads on this context
    IoThreads,
    /// Maximum number of sockets on this context
    MaxSockets,
    /// Scheduling priority for I/O threads
    ThreadPriority,
    /// Scheduling policy for I/O threads
    ThreadSchedulingPolicy,
    /// Maximum message size
    MaxMessageSize,
    /// Add a CPI to list of affinity for I/O threads
    ThreadAffinityCPUAdd,
    /// Remove a CPI from list of affinity for I/O threads
    ThreadAffinityCPURemove,
    /// Name prefix for I/O threads
    ThreadNamePrefix,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Specify message decoding strategy
    ZeroCopyReceiving,
    /// Enable IPv6 support
    IPv6,
    /// Fix blocky behavior
    Blocky,
    /// Get maximum number of sockets
    SocketLimit,
}

impl From<ContextOption> for i32 {
    fn from(value: ContextOption) -> Self {
        match value {
            ContextOption::Blocky => zmq_sys_crate::ZMQ_BLOCKY as i32,
            ContextOption::IoThreads => zmq_sys_crate::ZMQ_IO_THREADS as i32,
            ContextOption::SocketLimit => zmq_sys_crate::ZMQ_SOCKET_LIMIT as i32,
            ContextOption::ThreadSchedulingPolicy => zmq_sys_crate::ZMQ_THREAD_SCHED_POLICY as i32,
            ContextOption::ThreadPriority => zmq_sys_crate::ZMQ_THREAD_PRIORITY as i32,
            ContextOption::ThreadAffinityCPUAdd => {
                zmq_sys_crate::ZMQ_THREAD_AFFINITY_CPU_ADD as i32
            }
            ContextOption::ThreadAffinityCPURemove => {
                zmq_sys_crate::ZMQ_THREAD_AFFINITY_CPU_REMOVE as i32
            }
            ContextOption::ThreadNamePrefix => zmq_sys_crate::ZMQ_THREAD_NAME_PREFIX as i32,
            ContextOption::MaxMessageSize => zmq_sys_crate::ZMQ_MAX_MSGSZ as i32,
            ContextOption::MaxSockets => zmq_sys_crate::ZMQ_MAX_SOCKETS as i32,
            ContextOption::IPv6 => zmq_sys_crate::ZMQ_IPV6 as i32,
            #[cfg(feature = "draft-api")]
            ContextOption::ZeroCopyReceiving => zmq_sys_crate::ZMQ_ZERO_COPY_RECV as i32,
        }
    }
}

#[derive(DebugDeriveMore, DisplayDeriveMore)]
#[debug("ZmqContext {{ ... }}")]
#[display("ZmqContext")]
/// # 0MQ context
///
/// The 0MQ [`Context`] keeps the list of sockets and manages the async I/O thread and internal
/// queries.
pub struct Context {
    pub(crate) inner: Arc<RawContext>,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
    pub fn new() -> ZmqResult<Self> {
        let inner = RawContext::new()?;
        Ok(Self::from_raw_context(inner))
    }

    pub(crate) fn from_raw_context(raw_context: RawContext) -> Self {
        Self {
            inner: raw_context.into(),
        }
    }

    pub(crate) fn as_raw(&self) -> &RawContext {
        &self.inner
    }

    /// # set context options
    ///
    /// Sets a [`ContextOption`] option on the context. The bool version is mostly suitable for 0/1
    /// options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`ContextOption`]: ContextOption
    pub fn set_option_bool(&self, option: ContextOption, value: bool) -> ZmqResult<()> {
        self.inner.set_ctxopt_bool(option.into(), value)
    }

    /// # set context options
    ///
    /// Sets a [`ContextOption`] option on the context. The int version is mostly suitable for
    /// integer options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`ContextOption`]: ContextOption
    pub fn set_option_int<V>(&self, option: ContextOption, value: V) -> ZmqResult<()>
    where
        V: PrimInt + Into<i32>,
    {
        self.inner.set_ctxopt_int(option.into(), value)
    }

    /// # set context options
    ///
    /// Sets a [`ContextOption`] option on the context. The string version is mostly suitable for
    /// character-based options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`ContextOption`]: ContextOption
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_option_string<V>(&self, option: ContextOption, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.inner.set_ctxopt_string(option.into(), value.as_ref())
    }

    /// # get context options
    ///
    /// Retrieves a [`ContextOption`] option on the context. The bool version is mostly suitable
    /// for 0/1 options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`ContextOption`]: ContextOption
    pub fn get_option_bool(&self, option: ContextOption) -> ZmqResult<bool> {
        self.inner.get_ctxpt_bool(option.into())
    }

    /// # get context options
    ///
    /// Retrieves a [`ContextOption`] option on the context. The bool version is mostly suitable
    /// for integer options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`ContextOption`]: ContextOption
    pub fn get_option_int<V>(&self, option: ContextOption) -> ZmqResult<V>
    where
        V: PrimInt + From<i32>,
    {
        self.inner.get_ctxopt_int(option.into())
    }

    /// # get context options
    ///
    /// Retrieves a [`ContextOption`] option on the context. The bool version is mostly suitable
    /// for character options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`ContextOption`]: ContextOption
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn get_option_string(&self, option: ContextOption) -> ZmqResult<String> {
        self.inner.get_ctxopt_string(option.into())
    }

    /// # Fix blocky behavior `ZMQ_BLOCKY`
    ///
    /// By default the context will block, forever, when dropped. The assumption behind this
    /// behavior is that abrupt termination will cause message loss. Most real applications use
    /// some form of handshaking to ensure applications receive termination messages, and then
    /// terminate the context with [`Linger`] set to zero on all sockets. This setting is an easier
    /// way to get the same result. When [`Blocky`] is set to `false`, all new sockets are given a
    /// linger timeout of zero.
    ///
    /// Default: `true` (old behavior)
    ///
    /// [`Linger`]: crate::socket::Socket::set_linger
    /// [`Blocky`]: ContextOption::Blocky
    pub fn set_blocky(&self, value: bool) -> ZmqResult<()> {
        self.set_option_bool(ContextOption::Blocky, value)
    }

    /// # Get blocky setting `ZMQ_BLOCKY`
    ///
    /// By default the context will block, forever, when dropped. The assumption behind this
    /// behavior is that abrupt termination will cause message loss. Most real applications use
    /// some form of handshaking to ensure applications receive termination messages, and then
    /// terminate the context with [`Linger`] set to zero on all sockets. This setting is an easier
    /// way to get the same result. When '[`Blocky`] is set to `false`, all new sockets are given a
    /// linger timeout of zero.
    ///
    /// Default: `true` (old behavior)
    ///
    /// [`Linger`]: crate::socket::Socket::set_linger
    /// [`Blocky`]: ContextOption::Blocky
    pub fn blocky(&self) -> ZmqResult<bool> {
        self.get_option_bool(ContextOption::Blocky)
    }

    /// # Set number of I/O threads `ZMQ_IO_THREADS`
    ///
    /// The [`IoThreads`] argument specifies the size of the 0MQ thread pool to handle I/O
    /// operations. If your application is using only the `inproc` transport for messaging you may
    /// set this to zero, otherwise set it to at least one. This option only applies before
    /// creating any sockets on the context.
    ///
    /// Default: `1`
    ///
    /// [`IoThreads`]: ContextOption::IoThreads
    pub fn set_io_threads(&self, value: i32) -> ZmqResult<()> {
        self.set_option_int(ContextOption::IoThreads, value)
    }

    /// # Retrieve the number of I/O threads `ZMQ_IO_THREADS`
    ///
    /// The [`IoThreads`] argument specifies the size of the 0MQ thread pool to handle I/O
    /// operations. This option only applies before creating any sockets on the context.
    ///
    /// Default: `1`
    ///
    /// [`IoThreads`]: ContextOption::IoThreads
    pub fn io_threads(&self) -> ZmqResult<i32> {
        self.get_option_int(ContextOption::IoThreads)
    }

    /// # Set maximum message size `ZMQ_MAX_MSGSZ`
    ///
    /// The [`MaxMessageSize`] argument sets the maximum allowed size of a message sent in the
    /// context. You can query the maximal allowed value with [`max_message_size()`].
    ///
    /// Default: [`i32::MAX`]
    ///
    /// [`MaxMessageSize`]: ContextOption::MaxMessageSize
    /// [`max_message_size()`]: #method.max_message_size
    /// [`i32::MAX`]: ::core::primitive::i32::MAX
    pub fn set_max_message_size(&self, value: i32) -> ZmqResult<()> {
        self.set_option_int(ContextOption::MaxMessageSize, value)
    }

    /// # Retrieve maximum message size `ZMQ_MAX_MSGSZ`
    ///
    /// [`max_message_size()`] returns the maximum size of a message allowed for this context.
    /// Default value is [`i32::MAX`].
    ///
    /// [`max_message_size()`]: #method.max_message_size
    /// [`i32::MAX`]: ::core::primitive::i32::MAX
    pub fn max_message_size(&self) -> ZmqResult<i32> {
        self.get_option_int(ContextOption::MaxMessageSize)
    }

    /// # Set maximum number of sockets `ZMQ_MAX_SOCKETS`
    ///
    /// The [`MaxSockets`] argument sets the maximum number of sockets allowed on the context. You
    /// can query the maximal allowed value with [`socket_limit()`] option.
    ///
    /// Default value: `1023`
    ///
    /// [`MaxSockets`]: ContextOption::MaxSockets
    /// [`socket_limit()`]: #method.socket_limit
    pub fn set_max_sockets(&self, value: i32) -> ZmqResult<()> {
        self.set_option_int(ContextOption::MaxSockets, value)
    }

    /// # Retrieve the maximum number of sockets `ZMQ_MAX_SOCKETS`
    ///
    /// Returns the maximum number of sockets allowed for this context.
    ///
    /// Default value: `1023`
    pub fn max_sockets(&self) -> ZmqResult<i32> {
        self.get_option_int(ContextOption::MaxSockets)
    }

    /// # Retreive the socket limit `ZMQ_SOCKET_LIMIT`
    ///
    /// Returns the largest number of sockets that [`set_max_sockets()`] will accept.
    ///
    /// [`set_max_sockets()`]: #method.set_max_sockets
    pub fn socket_limit(&self) -> ZmqResult<i32> {
        self.get_option_int(ContextOption::SocketLimit)
    }

    /// # Set IPv6 option `ZMQ_IPV6`
    ///
    /// The [`IPv6`] argument sets the IPv6 value for all sockets created in the context from this
    /// point onwards. A value of `true` means IPv6 is enabled, while `false` means the socket will
    /// use only IPv4. When IPv6 is enabled, a socket will connect to, or accept connections from,
    /// both IPv4 and IPv6 hosts.
    ///
    /// Default value: `false`
    ///
    /// [`IPv6`]: ContextOption::IPv6
    pub fn set_ipv6(&self, value: bool) -> ZmqResult<()> {
        self.set_option_bool(ContextOption::IPv6, value)
    }

    /// # Retrieve IPv6 option `ZMQ_IPV6`
    ///
    /// Returns the IPv6 option for the context.
    ///
    /// Default value: `false`
    pub fn ipv6(&self) -> ZmqResult<bool> {
        self.get_option_bool(ContextOption::IPv6)
    }

    /// # Specify message decoding strategy `ZMQ_ZERO_COPY_RECV`
    ///
    /// The [`ZeroCopyReceiving`] argument specifies whether the message decoder should use a zero
    /// copy strategy when receiving messages. The zero copy strategy can lead to increased memory
    /// usage in some cases. This option allows you to use the older copying strategy. You can
    /// query the value of this option with [`zero_copy_receiving()`].
    ///
    /// Default value: `true`
    ///
    /// [`ZeroCopyReceiving`]: ContextOption::ZeroCopyReceiving
    /// [`zero_copy_receiving()`]: #method.zero_copy_receiving
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_zero_copy_receiving(&self, value: bool) -> ZmqResult<()> {
        self.set_option_bool(ContextOption::ZeroCopyReceiving, value)
    }

    /// # Get message decoding strategy `ZMQ_ZERO_COPY_RECV`
    ///
    /// The [`ZeroCopyReceiving`] argument return whether message decoder uses a zero copy strategy
    /// when receiving messages.
    ///
    /// Default value: `true`
    ///
    /// [`ZeroCopyReceiving`]: ContextOption::ZeroCopyReceiving
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn zero_copy_receiving(&self) -> ZmqResult<bool> {
        self.get_option_bool(ContextOption::ZeroCopyReceiving)
    }

    /// # shutdown a 0MQ context
    ///
    /// The [`shutdown()`] function shall shutdown the 0MQ context.
    ///
    /// Context shutdown will cause any blocking operations currently in progress on sockets open
    /// within `context` to return immediately with an error code of [`ContextTerminated`]. Any
    /// further operations on sockets open within `context` shall fail with an error code of
    /// [`ContextTerminated`]. No further sockets can be created on a context for which
    /// [`shutdown()`] has been called, it will return `Err(`[`ContextTerminated`]`)`.
    ///
    /// [`shutdown()`]: #method.shutdown
    /// [`ContextTerminated`]: crate::ZmqError::ContextTerminated
    pub fn shutdown(&self) -> ZmqResult<()> {
        self.inner.shutdown()
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(feature = "builder")]
mod builder {
    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use crate::{ZmqResult, context::Context};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "ContextBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`Context`].\n\n")]
    #[allow(dead_code)]
    struct ContextConfig {
        #[builder(default = false)]
        /// Blocky behavior, see [`set_blocky()`].
        ///
        /// [`set_blocky()`]: Context::set_blocky
        blocky: bool,
        #[builder(setter(into), default = 1)]
        /// Number of I/O threads, see [`set_io_threads()`].
        ///
        /// [`set_io_threads()`]: Context::set_io_threads
        io_threads: i32,
        #[builder(setter(into), default = "i32::MAX")]
        /// Maximum message size, see [`set_max_message_size()`].
        ///
        /// [`set_max_message_size()`]: Context::set_max_message_size
        max_message_size: i32,
        #[cfg(feature = "draft-api")]
        #[doc(cfg(feature = "draft-api"))]
        #[builder(default = true)]
        /// Specify message decoding strategy, see [`set_zero_copy_receiving()`].
        ///
        /// [`set_zero_copy_receiving()`]: Context::set_zero_copy_receiving
        zero_copy_receiving: bool,
        #[builder(setter(into), default = 1023)]
        /// Maximum number of sockets, see [`set_max_sockets()`].
        ///
        /// [`set_max_sockets()`]: Context::set_max_sockets
        max_sockets: i32,
        #[builder(default = false)]
        /// IPv6 option, see [`set_ipv6()`].
        ///
        /// [`set_ipv6()`]: Context::set_ipv6
        ipv6: bool,
    }

    impl ContextBuilder {
        /// Applies this builder to the provided context
        pub fn apply(self, context: &Context) -> ZmqResult<()> {
            if let Some(blocky) = self.blocky {
                context.set_blocky(blocky)?;
            }

            if let Some(io_threads) = self.io_threads {
                context.set_io_threads(io_threads)?;
            }

            if let Some(max_msg_size) = self.max_message_size {
                context.set_max_message_size(max_msg_size)?;
            }

            if let Some(max_sockets) = self.max_sockets {
                context.set_max_sockets(max_sockets)?;
            }

            if let Some(ipv6) = self.ipv6 {
                context.set_ipv6(ipv6)?;
            }

            #[cfg(feature = "draft-api")]
            if let Some(zero_copy_receiving) = self.zero_copy_receiving {
                context.set_zero_copy_receiving(zero_copy_receiving)?;
            }

            Ok(())
        }

        /// Builds a new context and applies this builder to it.
        pub fn build(self) -> ZmqResult<Context> {
            let context = Context::new()?;

            self.apply(&context)?;

            Ok(context)
        }
    }
}
