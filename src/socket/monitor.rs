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

#[cfg(test)]
mod handshake_protocol_error_tests {
    use rstest::*;

    use super::HandshakeProtocolError;
    use crate::zmq_sys_crate;

    #[rstest]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED,
        HandshakeProtocolError::ZmtpUnspecified
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
        HandshakeProtocolError::ZmtpUnexpectedCommand
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE,
        HandshakeProtocolError::ZmtpInvalidSequence
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_KEY_EXCHANGE,
        HandshakeProtocolError::ZmtpKeyEchange
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED,
        HandshakeProtocolError::ZmtpMalformedCommandUnspecified
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE,
        HandshakeProtocolError::ZmtpMalformedCommandMessage
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO,
        HandshakeProtocolError::ZmtpMalformedCommandHello
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE,
        HandshakeProtocolError::ZmtpMalformedCommandInitiate
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR,
        HandshakeProtocolError::ZmtpMalformedCommandError
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_READY,
        HandshakeProtocolError::ZmtpMalformedCommandReady
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME,
        HandshakeProtocolError::ZmtpMalformedCommandWelcome
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA,
        HandshakeProtocolError::ZapInvalidMetadata
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC,
        HandshakeProtocolError::ZmtpCryptographic
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH,
        HandshakeProtocolError::ZmtpMechanismMismatch
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED,
        HandshakeProtocolError::ZapUnspecified
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY,
        HandshakeProtocolError::ZapMalformedReply
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID,
        HandshakeProtocolError::ZapBadRequestId
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION,
        HandshakeProtocolError::ZapBadVersion
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE,
        HandshakeProtocolError::ZapInvalidStatusCode
    )]
    #[case(
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA,
        HandshakeProtocolError::ZapInvalidMetadata
    )]
    #[case(666, HandshakeProtocolError::UnsupportedError(666))]
    fn converts_from_raw(#[case] raw_value: u32, #[case] expected: HandshakeProtocolError) {
        assert_eq!(HandshakeProtocolError::from(raw_value), expected);
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
            unreachable!();
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
            unreachable!();
        };

        let Some(event_value) = first_msg
            .bytes()
            .last_chunk::<4>()
            .map(|raw_event_value| u32::from_le_bytes(*raw_event_value))
        else {
            unreachable!();
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
            event_id => Ok(Self::UnSupported(event_id, event_value)),
        }
    }
}

#[cfg(test)]
mod monitor_socket_event_tests {
    use rstest::*;

    use super::{HandshakeProtocolError, MonitorSocketEvent};
    use crate::{
        prelude::{MonitorFlags, MultipartMessage, ZmqError, ZmqResult},
        zmq_sys_crate,
    };

    #[rstest]
    #[case(MonitorFlags::Connected, 0, Ok(MonitorSocketEvent::Connected))]
    #[case(
        MonitorFlags::ConnectDelayed,
        0,
        Ok(MonitorSocketEvent::ConnectDelayed)
    )]
    #[case(
        MonitorFlags::ConnectRetried,
        42,
        Ok(MonitorSocketEvent::ConnectRetried(42))
    )]
    #[case(MonitorFlags::Listening, 0, Ok(MonitorSocketEvent::Listening))]
    #[case(MonitorFlags::Accepted, 0, Ok(MonitorSocketEvent::Accepted))]
    #[case(
        MonitorFlags::AcceptFailed,
        14,
        Ok(MonitorSocketEvent::AcceptFailed(ZmqError::ContextInvalid))
    )]
    #[case(MonitorFlags::Closed, 0, Ok(MonitorSocketEvent::Closed))]
    #[case(
        MonitorFlags::CloseFailed,
        14,
        Ok(MonitorSocketEvent::CloseFailed(ZmqError::ContextInvalid))
    )]
    #[case(MonitorFlags::Disconnected, 0, Ok(MonitorSocketEvent::Disconnected))]
    #[case(
        MonitorFlags::MonitorStopped,
        0,
        Ok(MonitorSocketEvent::MonitorStopped)
    )]
    #[case(
        MonitorFlags::HandshakeFailedNoDetail,
        14,
        Ok(MonitorSocketEvent::HandshakeFailedNoDetail(ZmqError::ContextInvalid))
    )]
    #[case(
        MonitorFlags::HandshakeSucceeded,
        0,
        Ok(MonitorSocketEvent::HandshakeSucceeded)
    )]
    #[case(
        MonitorFlags::HandshakeFailedProtocol,
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED,
        Ok(MonitorSocketEvent::HandshakeFailedProtocol(HandshakeProtocolError::ZmtpUnspecified))
    )]
    #[case(
        MonitorFlags::HandshakeFailedAuth,
        404,
        Ok(MonitorSocketEvent::HandshakeFailedAuth(404))
    )]
    #[case(
        MonitorFlags::HandshakeFailedAuth | MonitorFlags::Connected,
        42,
        Ok(MonitorSocketEvent::UnSupported(MonitorFlags::HandshakeFailedAuth | MonitorFlags::Connected, 42))
    )]
    fn try_from_multipart_succeeds(
        #[case] upper_chunk: MonitorFlags,
        #[case] lower_chunk: u32,
        #[case] expected: ZmqResult<MonitorSocketEvent>,
    ) {
        let mut first = upper_chunk.bits().to_le_bytes().to_vec();
        first.extend(lower_chunk.to_le_bytes());
        let multipart: MultipartMessage = vec![first.into(), vec![].into()].into();

        assert_eq!(MonitorSocketEvent::try_from(multipart), expected);
    }

    #[test]
    fn try_from_mutipart_with_too_few_parts() {
        let multipart: MultipartMessage = vec!["asdf".into()].into();
        let result = MonitorSocketEvent::try_from(multipart);

        assert!(result.is_err_and(|err| err == ZmqError::InvalidArgument));
    }

    #[test]
    fn try_from_mutipart_with_too_many_parts() {
        let multipart: MultipartMessage = vec!["asdf".into(), "asdf".into(), "asdf".into()].into();
        let result = MonitorSocketEvent::try_from(multipart);

        assert!(result.is_err_and(|err| err == ZmqError::InvalidArgument));
    }

    #[test]
    fn try_from_mutipart_with_too_short_first_part() {
        let multipart: MultipartMessage = vec![vec![1, 2, 3, 4, 5].into(), "asdf".into()].into();
        let result = MonitorSocketEvent::try_from(multipart);

        assert!(result.is_err_and(|err| err == ZmqError::InvalidArgument));
    }

    #[test]
    fn try_from_mutipart_with_too_long_first_part() {
        let multipart: MultipartMessage =
            vec![vec![1, 2, 3, 4, 5, 6, 7].into(), "asdf".into()].into();
        let result = MonitorSocketEvent::try_from(multipart);

        assert!(result.is_err_and(|err| err == ZmqError::InvalidArgument));
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
