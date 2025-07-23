#[cfg(feature = "futures")]
use core::{pin::Pin, task::Context, task::Poll};

#[cfg(feature = "futures")]
use async_trait::async_trait;
#[cfg(feature = "futures")]
use futures::FutureExt;

use super::{MonitorFlags, MultipartReceiver, RecvFlags, SocketType};
use crate::{
    ZmqError, ZmqResult, message::MultipartMessage, sealed, socket::Socket, zmq_sys_crate,
};

#[derive(Debug, Clone, Copy, PartialEq)]
/// Errors stemming from [`HandShakeFailedProtocol`]
///
/// [`HandShakeFailedProtocol`]: MonitorSocketEvent::HandshakeFailedProtocol
pub enum HandshakeProtocolError {
    ZmtpUnspecified,
    ZmtpUnexpectedCommand,
    ZmtpInvalidSequence,
    ZmtpKeyEchange,
    ZmtpMalformedCommandUnspecified,
    ZmtpMalformedCommandMessage,
    ZmtpMalformedCommandHello,
    ZmtpMalformedCommandInitiate,
    ZmtpMalformedCommandError,
    ZmtpMalformedCommandReady,
    ZmtpMalformedCommandWelcome,
    ZmtpInvalidMetadata,
    ZmtpCryptographic,
    ZmtpMechanismMismatch,
    ZapUnspecified,
    ZapMalformedReply,
    ZapBadRequestId,
    ZapBadVersion,
    ZapInvalidStatusCode,
    ZapInvalidMetadata,
    UnsupportedError(u32),
}

impl From<u32> for HandshakeProtocolError {
    fn from(value: u32) -> Self {
        match value {
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED => Self::ZmtpUnspecified,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND => {
                Self::ZmtpUnexpectedCommand
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE => Self::ZmtpInvalidSequence,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_KEY_EXCHANGE => Self::ZmtpKeyEchange,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED => {
                Self::ZmtpMalformedCommandUnspecified
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE => {
                Self::ZmtpMalformedCommandMessage
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO => {
                Self::ZmtpMalformedCommandHello
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE => {
                Self::ZmtpMalformedCommandInitiate
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR => {
                Self::ZmtpMalformedCommandError
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_READY => {
                Self::ZmtpMalformedCommandReady
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME => {
                Self::ZmtpMalformedCommandWelcome
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA => Self::ZapInvalidMetadata,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC => Self::ZmtpCryptographic,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH => {
                Self::ZmtpMechanismMismatch
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED => Self::ZapUnspecified,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY => Self::ZapMalformedReply,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID => Self::ZapBadRequestId,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION => Self::ZapBadVersion,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE => Self::ZapInvalidStatusCode,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA => Self::ZapInvalidMetadata,
            other => Self::UnsupportedError(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Monitor events that can be received from a monitor socket
pub enum MonitorSocketEvent {
    /// The socket has successfully connected to a remote peer. The event value is the file
    /// descriptor (FD) of the underlying network socket.
    ///
    /// <div class="warning">
    ///
    /// Warning:
    ///
    /// There is no guarantee that the FD is still valid by the time your code receives this
    /// event.
    ///
    /// </div>
    Connected,
    /// A connect request on the socket is pending. The event value is unspecified.
    ConnectDelayed,
    /// A connect request failed, and is now being retried. The event value is the reconnect
    /// interval in milliseconds.
    ///
    /// Note that the reconnect interval is recalculated at each retry.
    ConnectRetried(u32),
    /// The socket was successfully bound to a network interface. The event value is the FD of
    /// the underlying network socket.
    ///
    /// <div class="warning">
    ///
    /// Warning:
    ///
    /// There is no guarantee that the FD is still valid by the time your code receives this
    /// event.
    ///
    /// </div>
    Listening,
    /// The socket could not bind to a given interface. The event value is the errno generated
    /// by the system bind call.
    BindFailed,
    /// The socket has accepted a connection from a remote peer. The event value is the FD of
    /// the underlying network socket.
    ///
    /// <div class="warning">
    ///
    /// Warning:
    ///
    /// There is no guarantee that the FD is still valid by the time your code receives this
    /// event.
    ///
    /// </div>
    Accepted,
    /// The socket has rejected a connection from a remote peer. The event value is the errno
    /// generated by the accept call.
    AcceptFailed(ZmqError),
    /// The socket was closed. The event value is the FD of the (now closed) network socket.
    Closed,
    /// The socket close failed. The event value is the errno returned by the system call.
    ///
    /// Note that this event occurs only on IPC transports.
    CloseFailed(ZmqError),
    /// The socket was disconnected unexpectedly. The event value is the FD of the underlying
    /// network socket.
    ///
    /// <div class="warning">
    ///
    /// Warning:
    ///
    /// This socket will be closed.
    ///
    /// </div>
    Disconnected,
    /// Monitoring on this socket ended.
    MonitorStopped,
    /// Unspecified error during handshake. The event value is an errno.
    HandshakeFailedNoDetail(ZmqError),
    /// The ZMTP security mechanism handshake succeeded. The event value is unspecified.
    HandshakeSucceeded,
    /// The ZMTP security mechanism handshake failed due to some mechanism protocol error,
    /// either between the ZMTP mechanism peers, or between the mechanism server and the ZAP
    /// handler. This indicates a configuration or implementation error in either peer resp.
    /// the ZAP handler.
    HandshakeFailedProtocol(HandshakeProtocolError),
    /// The ZMTP security mechanism handshake failed due to an authentication failure. The
    /// event value is the status code returned by the ZAP handler (i.e. `300`, `400` or `500`).
    HandshakeFailedAuth(u32),
    UnSupported(MonitorFlags, u32),
}

impl TryFrom<MultipartMessage> for MonitorSocketEvent {
    type Error = ZmqError;

    fn try_from(zmq_msgs: MultipartMessage) -> Result<Self, Self::Error> {
        if zmq_msgs.len() != 2 {
            return Err(ZmqError::InvalidArgument);
        }

        let Some(first_msg) = zmq_msgs.get(0) else {
            return Err(ZmqError::InvalidArgument);
        };

        if first_msg.len() != 6 {
            return Err(ZmqError::InvalidArgument);
        }

        let Some(event_id) = first_msg
            .bytes()
            .first_chunk::<2>()
            .map(|raw_event_id| u16::from_le_bytes(*raw_event_id))
            .map(MonitorFlags::from)
        else {
            return Err(ZmqError::InvalidArgument);
        };

        let Some(event_value) = first_msg
            .bytes()
            .last_chunk::<4>()
            .map(|raw_event_value| u32::from_le_bytes(*raw_event_value))
        else {
            return Err(ZmqError::InvalidArgument);
        };

        match event_id {
            MonitorFlags::Connected => Ok(Self::Connected),
            MonitorFlags::ConnectDelayed => Ok(Self::ConnectDelayed),
            MonitorFlags::ConnectRetried => Ok(Self::ConnectRetried(event_value)),
            MonitorFlags::Listening => Ok(Self::Listening),
            MonitorFlags::Accepted => Ok(Self::Accepted),
            MonitorFlags::AcceptFailed => {
                Ok(Self::AcceptFailed(ZmqError::from(event_value as i32)))
            }
            MonitorFlags::Closed => Ok(Self::Closed),
            MonitorFlags::CloseFailed => Ok(Self::CloseFailed(ZmqError::from(event_value as i32))),
            MonitorFlags::Disconnected => Ok(Self::Disconnected),
            MonitorFlags::MonitorStopped => Ok(Self::MonitorStopped),
            MonitorFlags::HandshakeFailedNoDetail => Ok(Self::HandshakeFailedNoDetail(
                ZmqError::from(event_value as i32),
            )),
            MonitorFlags::HandshakeSucceeded => Ok(Self::HandshakeSucceeded),
            MonitorFlags::HandshakeFailedProtocol => {
                Ok(Self::HandshakeFailedProtocol(event_value.into()))
            }
            MonitorFlags::HandshakeFailedAuth => Ok(Self::HandshakeFailedAuth(event_value)),
            event_id => Ok(Self::UnSupported(event_id, 0)),
        }
    }
}

/// # A monitor socket `ZMQ_PAIR`
pub type MonitorSocket = Socket<Monitor>;

pub struct Monitor {}

impl sealed::ReceiverFlag for Monitor {}

unsafe impl Sync for Socket<Monitor> {}
unsafe impl Send for Socket<Monitor> {}

impl MultipartReceiver for Socket<Monitor> {}

impl sealed::SocketType for Monitor {
    fn raw_socket_type() -> SocketType {
        SocketType::Pair
    }
}

impl Socket<Monitor> {}

#[cfg_attr(feature = "futures", async_trait)]
/// Trait for receiving [`MonitorSocketEvent`] from a monitor socket
///
/// [`MonitorSocketEvent`]: MonitorSocketEvent
pub trait MonitorReceiver {
    fn recv_monitor_event(&self) -> ZmqResult<MonitorSocketEvent>;

    #[cfg(feature = "futures")]
    #[doc(cfg(feature = "futures"))]
    async fn recv_monitor_event_async(&self) -> Option<MonitorSocketEvent>;
}

#[cfg_attr(feature = "futures", async_trait)]
impl MonitorReceiver for MonitorSocket {
    fn recv_monitor_event(&self) -> ZmqResult<MonitorSocketEvent> {
        self.recv_multipart(RecvFlags::DONT_WAIT)
            .and_then(MonitorSocketEvent::try_from)
    }

    #[cfg(feature = "futures")]
    async fn recv_monitor_event_async(&self) -> Option<MonitorSocketEvent> {
        MonitorSocketEventFuture { receiver: self }.now_or_never()
    }
}

#[cfg(feature = "futures")]
struct MonitorSocketEventFuture<'a> {
    receiver: &'a MonitorSocket,
}

#[cfg(feature = "futures")]
impl Future for MonitorSocketEventFuture<'_> {
    type Output = MonitorSocketEvent;

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.receiver.recv_monitor_event() {
            Ok(event) => Poll::Ready(event),
            _ => Poll::Pending,
        }
    }
}
