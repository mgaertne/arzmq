use alloc::ffi::{IntoStringError, NulError};
use core::{ffi::FromBytesUntilNulError, num::ParseIntError};

use thiserror::Error;

use crate::zmq_sys_crate;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// 0MQ error codes
pub enum ZmqError {
    /// EAGAIN
    #[error("Again")]
    Again,
    /// EFAULT
    #[error("Context is invalid")]
    ContextInvalid,
    /// EINVAL
    #[error("Invalid argument")]
    InvalidArgument,
    /// ENOTSUP
    #[error("Not supported")]
    Unsupported,
    /// EPROTONOSUPPORT
    #[error("Protocol not supported")]
    ProtocolNotSupported,
    /// ENOBUFS
    #[error("No buffer space available")]
    NoBufferSpaceAvailable,
    /// ENETDOWN
    #[error("Network is down")]
    NetworkDown,
    /// EADDRINUSE
    #[error("Address in use")]
    AddressInUse,
    /// EADDRNOTAVAIL
    #[error("Address not available")]
    AddressNotAvailable,
    /// ECONNREFUSED
    #[error("Connection refused")]
    ConnectionRefused,
    /// EINPROGRESS
    #[error("Operation in progress")]
    OperationInProgress,
    /// ENOTSOCK
    #[error("The provided socket was null")]
    SocketNull,
    /// EMSGSIZE
    #[error("Message too long")]
    MessageTooLong,
    /// EAFNOSUPPORT
    #[error("Address family not supported by protocol")]
    AddressFamilyNotSupported,
    /// ENETUNREACH
    #[error("Network is unreachable")]
    NetworkUnreachable,
    /// ECONNABORTED
    #[error("Software caused connection abort")]
    ConnectionAborted,
    /// ECONNRESET
    #[error("Connection reset by peer")]
    ConnectionReset,
    /// ENOTCONN
    #[error("Transport endpoint is not connected")]
    NotConnected,
    /// ETIMEDOUT
    #[error("Connection timed out")]
    ConnectionTimeout,
    /// EHOSTUNREACH
    #[error("Host unreachable")]
    HostUnreachable,
    /// ENETRESET
    #[error("Network dropped connection on reset")]
    NetworkReset,
    /// EFSM
    #[error("Operation cannot be accomplished in current state")]
    OperationNotPossible,
    /// ENOCOMPATPROTO
    #[error("The protocol is not compatible with the socket type")]
    ProtocolIncompatible,
    /// ETERM
    #[error("Context was terminated")]
    ContextTerminated,
    /// EMTHREAD
    #[error("I/O thread unavaible")]
    IoThreadUnavailable,
    /// ENOENT
    #[error("Endpoint not in use")]
    EndpointNotInUse,
    /// EINTR
    #[error("Interrupted function call")]
    Interrupted,
    /// EMFILE
    #[error("Too many open files")]
    TooManyOpenFiles,
    /// EPROTO
    #[error("Transport protocol not supported")]
    TransportNotSupported,
    /// ENODEV
    #[error("Interface not existent")]
    NonExistentInterface,
    /// ENOMEM
    #[error("Insufficient memory")]
    InsufficientMemory,
    #[error("other")]
    Other(i32),
}

impl From<i32> for ZmqError {
    fn from(code: i32) -> Self {
        match code {
            zmq_sys_crate::errno::EAGAIN => Self::Again,
            zmq_sys_crate::errno::EFAULT => Self::ContextInvalid,
            zmq_sys_crate::errno::EINVAL => Self::InvalidArgument,
            zmq_sys_crate::errno::ENOTSUP => Self::Unsupported,
            zmq_sys_crate::errno::EPROTONOSUPPORT => Self::ProtocolNotSupported,
            zmq_sys_crate::errno::ENOBUFS => Self::NoBufferSpaceAvailable,
            zmq_sys_crate::errno::ENETDOWN => Self::NetworkDown,
            zmq_sys_crate::errno::EADDRINUSE => Self::AddressInUse,
            zmq_sys_crate::errno::EADDRNOTAVAIL => Self::AddressNotAvailable,
            zmq_sys_crate::errno::ECONNREFUSED => Self::ConnectionRefused,
            zmq_sys_crate::errno::EINPROGRESS => Self::OperationInProgress,
            zmq_sys_crate::errno::ENOTSOCK => Self::SocketNull,
            zmq_sys_crate::errno::EMSGSIZE => Self::MessageTooLong,
            zmq_sys_crate::errno::EAFNOSUPPORT => Self::AddressFamilyNotSupported,
            zmq_sys_crate::errno::ENETUNREACH => Self::NetworkUnreachable,
            zmq_sys_crate::errno::ECONNABORTED => Self::ConnectionAborted,
            zmq_sys_crate::errno::ECONNRESET => Self::ConnectionReset,
            zmq_sys_crate::errno::ENOTCONN => Self::NotConnected,
            zmq_sys_crate::errno::ETIMEDOUT => Self::ConnectionTimeout,
            zmq_sys_crate::errno::EHOSTUNREACH => Self::HostUnreachable,
            zmq_sys_crate::errno::ENETRESET => Self::NetworkReset,
            zmq_sys_crate::errno::EFSM => Self::OperationNotPossible,
            zmq_sys_crate::errno::ENOCOMPATPROTO => Self::ProtocolIncompatible,
            zmq_sys_crate::errno::ETERM => Self::ContextTerminated,
            zmq_sys_crate::errno::EMTHREAD => Self::IoThreadUnavailable,
            zmq_sys_crate::errno::ENOENT => Self::EndpointNotInUse,
            zmq_sys_crate::errno::EINTR => Self::Interrupted,
            zmq_sys_crate::errno::EMFILE => Self::TooManyOpenFiles,
            zmq_sys_crate::errno::EPROTO => Self::TransportNotSupported,
            zmq_sys_crate::errno::ENODEV => Self::NonExistentInterface,
            zmq_sys_crate::errno::ENOMEM => Self::InsufficientMemory,
            errno => Self::Other(errno),
        }
    }
}

impl From<FromBytesUntilNulError> for ZmqError {
    fn from(_err: FromBytesUntilNulError) -> Self {
        Self::InvalidArgument
    }
}

impl From<IntoStringError> for ZmqError {
    fn from(_err: IntoStringError) -> Self {
        Self::InvalidArgument
    }
}

impl From<NulError> for ZmqError {
    fn from(_err: NulError) -> Self {
        Self::InvalidArgument
    }
}

impl From<ParseIntError> for ZmqError {
    fn from(_err: ParseIntError) -> Self {
        Self::InvalidArgument
    }
}

/// 0MQ specific result type
pub type ZmqResult<T, E = ZmqError> = Result<T, E>;

#[cfg(test)]
mod error_tests {
    use alloc::ffi::CString;
    use core::ffi::CStr;

    use rstest::*;

    use super::ZmqError;
    use crate::zmq_sys_crate;

    #[rstest]
    #[case(zmq_sys_crate::errno::EAGAIN, ZmqError::Again)]
    #[case(zmq_sys_crate::errno::EFAULT, ZmqError::ContextInvalid)]
    #[case(zmq_sys_crate::errno::EINVAL, ZmqError::InvalidArgument)]
    #[case(zmq_sys_crate::errno::ENOTSUP, ZmqError::Unsupported)]
    #[case(zmq_sys_crate::errno::EPROTONOSUPPORT, ZmqError::ProtocolNotSupported)]
    #[case(zmq_sys_crate::errno::ENOBUFS, ZmqError::NoBufferSpaceAvailable)]
    #[case(zmq_sys_crate::errno::ENETDOWN, ZmqError::NetworkDown)]
    #[case(zmq_sys_crate::errno::EADDRINUSE, ZmqError::AddressInUse)]
    #[case(zmq_sys_crate::errno::EADDRNOTAVAIL, ZmqError::AddressNotAvailable)]
    #[case(zmq_sys_crate::errno::ECONNREFUSED, ZmqError::ConnectionRefused)]
    #[case(zmq_sys_crate::errno::EINPROGRESS, ZmqError::OperationInProgress)]
    #[case(zmq_sys_crate::errno::ENOTSOCK, ZmqError::SocketNull)]
    #[case(zmq_sys_crate::errno::EMSGSIZE, ZmqError::MessageTooLong)]
    #[case(
        zmq_sys_crate::errno::EAFNOSUPPORT,
        ZmqError::AddressFamilyNotSupported
    )]
    #[case(zmq_sys_crate::errno::ENETUNREACH, ZmqError::NetworkUnreachable)]
    #[case(zmq_sys_crate::errno::ECONNABORTED, ZmqError::ConnectionAborted)]
    #[case(zmq_sys_crate::errno::ECONNRESET, ZmqError::ConnectionReset)]
    #[case(zmq_sys_crate::errno::ENOTCONN, ZmqError::NotConnected)]
    #[case(zmq_sys_crate::errno::ETIMEDOUT, ZmqError::ConnectionTimeout)]
    #[case(zmq_sys_crate::errno::EHOSTUNREACH, ZmqError::HostUnreachable)]
    #[case(zmq_sys_crate::errno::ENETRESET, ZmqError::NetworkReset)]
    #[case(zmq_sys_crate::errno::EFSM, ZmqError::OperationNotPossible)]
    #[case(zmq_sys_crate::errno::ENOCOMPATPROTO, ZmqError::ProtocolIncompatible)]
    #[case(zmq_sys_crate::errno::ETERM, ZmqError::ContextTerminated)]
    #[case(zmq_sys_crate::errno::EMTHREAD, ZmqError::IoThreadUnavailable)]
    #[case(zmq_sys_crate::errno::ENOENT, ZmqError::EndpointNotInUse)]
    #[case(zmq_sys_crate::errno::EINTR, ZmqError::Interrupted)]
    #[case(zmq_sys_crate::errno::EMFILE, ZmqError::TooManyOpenFiles)]
    #[case(zmq_sys_crate::errno::EPROTO, ZmqError::TransportNotSupported)]
    #[case(zmq_sys_crate::errno::ENODEV, ZmqError::NonExistentInterface)]
    #[case(zmq_sys_crate::errno::ENOMEM, ZmqError::InsufficientMemory)]
    fn test_zmq_error(#[case] errno: i32, #[case] expected: ZmqError) {
        assert_eq!(ZmqError::from(errno), expected);
    }

    #[test]
    fn from_bytes_until_nul_error() {
        assert_eq!(
            ZmqError::from(CStr::from_bytes_until_nul(&[]).unwrap_err()),
            ZmqError::InvalidArgument
        );
    }

    #[test]
    fn from_into_string_error() {
        let invalid_utf8 = vec![b'f', 0xff, b'o', b'o'];
        let cstring = CString::new(invalid_utf8).expect("CString::new failed");
        let err = cstring
            .into_string()
            .expect_err("into_string().err() failed");
        assert_eq!(ZmqError::from(err), ZmqError::InvalidArgument);
    }

    #[test]
    fn from_nul_error() {
        assert_eq!(
            ZmqError::from(CString::new(b"f\0oo".to_vec()).unwrap_err()),
            ZmqError::InvalidArgument
        );
    }

    #[test]
    fn from_parse_int_error() {
        assert_eq!(
            ZmqError::from("".parse::<i32>().unwrap_err()),
            ZmqError::InvalidArgument
        );
    }
}
