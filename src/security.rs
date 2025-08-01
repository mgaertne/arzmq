//! 0MQ security mechanisms
//!
//! Currently only support [`Null`] and [`Plain`] across different platforms. Curve is
//! available with the <span class="stab portability"><code>curve</code></span> feature on Linux
//! and MacOS, but not on Windows.
//!
//! [`Null`]: SecurityMechanism::Null
//! [`Plain`]: SecurityMechanism::Plain

#[cfg(all(feature = "curve", not(windows)))]
use core::{ffi::c_char, hint::cold_path};

use derive_more::Display;

use crate::{
    ZmqError, ZmqResult, sealed,
    socket::{Socket, SocketOption},
    zmq_sys_crate,
};

#[derive(Default, Debug, Display, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "builder", derive(serde::Deserialize, serde::Serialize))]
#[repr(i32)]
#[non_exhaustive]
/// # 0MQ security mechanisms
///
/// A 0MQ socket can select a security mechanism. Both peers must use the same security mechanism.
pub enum SecurityMechanism {
    #[default]
    /// Null security
    Null,
    #[display("Plain {{ username = {username}, password = {password} }}")]
    /// Plain-textauthentication using username and password
    Plain { username: String, password: String },
    #[cfg(all(feature = "curve", not(windows)))]
    #[doc(cfg(all(feature = "curve", not(windows))))]
    #[display("CurveClient {{ ... }}")]
    /// Elliptic curve client authentication and encryption
    CurveClient {
        server_key: Vec<u8>,
        public_key: Vec<u8>,
        secret_key: Vec<u8>,
    },
    #[cfg(all(feature = "curve", not(windows)))]
    #[doc(cfg(all(feature = "curve", not(windows))))]
    #[display("CurveServer {{ ... }}")]
    /// Elliptic curve server authentication and encryption
    CurveServer { secret_key: Vec<u8> },
}

impl SecurityMechanism {
    /// Applies the security mechanism to the provided socket
    pub fn apply<T: sealed::SocketType>(&self, socket: &Socket<T>) -> ZmqResult<()> {
        match self {
            SecurityMechanism::Null => socket.set_sockopt_bool(SocketOption::PlainServer, false)?,
            SecurityMechanism::Plain { username, password } => {
                socket.set_sockopt_bool(SocketOption::PlainServer, true)?;
                socket.set_sockopt_string(SocketOption::PlainUsername, username)?;
                socket.set_sockopt_string(SocketOption::PlainPassword, password)?;
            }
            #[cfg(all(feature = "curve", not(windows)))]
            SecurityMechanism::CurveServer { secret_key } => {
                socket.set_sockopt_bool(SocketOption::CurveServer, true)?;
                socket.set_sockopt_bytes(SocketOption::CurveSecretKey, secret_key)?;
            }
            #[cfg(all(feature = "curve", not(windows)))]
            SecurityMechanism::CurveClient {
                server_key,
                public_key,
                secret_key,
            } => {
                socket.set_sockopt_bytes(SocketOption::CurveServerKey, server_key)?;
                socket.set_sockopt_bytes(SocketOption::CurvePublicKey, public_key)?;
                socket.set_sockopt_bytes(SocketOption::CurveSecretKey, secret_key)?;
            }
        }
        Ok(())
    }
}

impl<T: sealed::SocketType> TryFrom<&Socket<T>> for SecurityMechanism {
    type Error = ZmqError;

    fn try_from(socket: &Socket<T>) -> Result<Self, Self::Error> {
        match socket.get_sockopt_int::<i32>(SocketOption::Mechanism)? {
            value if value == zmq_sys_crate::ZMQ_NULL as i32 => Ok(Self::Null),
            value if value == zmq_sys_crate::ZMQ_PLAIN as i32 => {
                let username = socket.get_sockopt_string(SocketOption::PlainUsername)?;
                let password = socket.get_sockopt_string(SocketOption::PlainPassword)?;
                Ok(Self::Plain { username, password })
            }
            #[cfg(all(feature = "curve", not(windows)))]
            value if value == zmq_sys_crate::ZMQ_CURVE as i32 => {
                let secret_key = socket.get_sockopt_curve(SocketOption::CurveSecretKey)?;
                if socket.get_sockopt_bool(SocketOption::CurveServer)? {
                    Ok(Self::CurveServer { secret_key })
                } else {
                    let server_key = socket.get_sockopt_curve(SocketOption::CurveServerKey)?;
                    let public_key = socket.get_sockopt_curve(SocketOption::CurvePublicKey)?;
                    Ok(Self::CurveClient {
                        server_key,
                        public_key,
                        secret_key,
                    })
                }
            }
            _ => Err(ZmqError::Unsupported),
        }
    }
}

#[cfg(test)]
mod security_mechanism_tests {
    use super::SecurityMechanism;
    #[cfg(all(feature = "curve", not(windows)))]
    use super::curve_keypair;
    use crate::{
        prelude::{Context, DealerSocket, SocketOption, ZmqResult},
        zmq_sys_crate,
    };

    #[test]
    fn apply_null_security() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;

        SecurityMechanism::Null.apply(&socket)?;

        assert_eq!(
            socket.get_sockopt_int::<i32>(SocketOption::Mechanism)?,
            zmq_sys_crate::ZMQ_NULL as i32
        );

        Ok(())
    }

    #[test]
    fn apply_plain_security() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        let security = SecurityMechanism::Plain {
            username: "username".to_string(),
            password: "password".to_string(),
        };

        security.apply(&socket)?;

        assert_eq!(
            socket.get_sockopt_int::<i32>(SocketOption::Mechanism)?,
            zmq_sys_crate::ZMQ_PLAIN as i32
        );
        assert_eq!(
            socket.get_sockopt_string(SocketOption::PlainUsername)?,
            "username"
        );
        assert_eq!(
            socket.get_sockopt_string(SocketOption::PlainPassword)?,
            "password"
        );

        Ok(())
    }

    #[cfg(all(feature = "curve", not(windows)))]
    #[test]
    fn apply_curve_server_security() -> ZmqResult<()> {
        let (_, secret_key) = curve_keypair()?;

        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        let security = SecurityMechanism::CurveServer {
            secret_key: secret_key.clone(),
        };
        security.apply(&socket)?;

        assert_eq!(
            socket.get_sockopt_int::<i32>(SocketOption::Mechanism)?,
            zmq_sys_crate::ZMQ_CURVE as i32
        );
        assert!(socket.get_sockopt_bool(SocketOption::CurveServer)?);
        assert_eq!(
            socket.get_sockopt_curve(SocketOption::CurveSecretKey)?,
            secret_key
        );

        Ok(())
    }

    #[cfg(all(feature = "curve", not(windows)))]
    #[test]
    fn apply_curve_client_security() -> ZmqResult<()> {
        let (_, server_key) = curve_keypair()?;
        let (public_key, secret_key) = curve_keypair()?;

        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        let security = SecurityMechanism::CurveClient {
            server_key: server_key.clone(),
            public_key: public_key.clone(),
            secret_key: secret_key.clone(),
        };
        security.apply(&socket)?;

        assert_eq!(
            socket.get_sockopt_int::<i32>(SocketOption::Mechanism)?,
            zmq_sys_crate::ZMQ_CURVE as i32
        );
        assert!(!socket.get_sockopt_bool(SocketOption::CurveServer)?);
        assert_eq!(
            socket.get_sockopt_curve(SocketOption::CurveServerKey)?,
            server_key
        );
        assert_eq!(
            socket.get_sockopt_curve(SocketOption::CurvePublicKey)?,
            public_key
        );
        assert_eq!(
            socket.get_sockopt_curve(SocketOption::CurveSecretKey)?,
            secret_key
        );

        Ok(())
    }

    #[test]
    fn try_from_socket_with_no_security() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;

        assert_eq!(
            SecurityMechanism::try_from(&socket)?,
            SecurityMechanism::Null
        );

        Ok(())
    }

    #[test]
    fn try_from_socket_with_plain_security() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        socket.set_sockopt_string(SocketOption::PlainUsername, "username")?;
        socket.set_sockopt_string(SocketOption::PlainPassword, "password")?;

        assert_eq!(
            SecurityMechanism::try_from(&socket)?,
            SecurityMechanism::Plain {
                username: "username".to_string(),
                password: "password".to_string(),
            }
        );

        Ok(())
    }

    #[cfg(all(feature = "curve", not(windows)))]
    #[test]
    fn try_from_socket_with_curve_security() -> ZmqResult<()> {
        let (_, secret_key) = curve_keypair()?;

        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;

        socket.set_sockopt_bytes(SocketOption::CurveSecretKey, secret_key.clone())?;
        socket.set_sockopt_bool(SocketOption::CurveServer, true)?;
        assert_eq!(
            SecurityMechanism::try_from(&socket)?,
            SecurityMechanism::CurveServer {
                secret_key: secret_key.clone(),
            }
        );

        Ok(())
    }

    #[cfg(all(feature = "curve", not(windows)))]
    #[test]
    fn try_from_socket_with_curve_client_security() -> ZmqResult<()> {
        let (_, server_key) = curve_keypair()?;
        let (public_key, secret_key) = curve_keypair()?;

        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        socket.set_sockopt_bool(SocketOption::CurveServer, false)?;
        socket.set_sockopt_bytes(SocketOption::CurveServerKey, server_key.clone())?;
        socket.set_sockopt_bytes(SocketOption::CurvePublicKey, public_key.clone())?;
        socket.set_sockopt_bytes(SocketOption::CurveSecretKey, secret_key.clone())?;
        assert_eq!(
            SecurityMechanism::try_from(&socket)?,
            SecurityMechanism::CurveClient {
                server_key: server_key.clone(),
                public_key: public_key.clone(),
                secret_key: secret_key.clone(),
            }
        );

        Ok(())
    }
}

#[cfg(all(feature = "curve", not(windows)))]
#[doc(cfg(all(feature = "curve", not(windows))))]
/// Z85 decoding error
pub use z85::DecodeError as Z85DecodeError;
#[cfg(all(feature = "curve", not(windows)))]
#[doc(cfg(all(feature = "curve", not(windows))))]
pub use z85::{decode as z85_decode, encode as z85_encode};

/// # generate a new CURVE keypair
///
/// The [`curve_keypair()`] function returns a newly generated random keypair consisting of a
/// public key and a secret key. The keys are encoded using [`z85_encode()`].
///
/// [`curve_keypair()`]: curve_keypair
/// [`z85_encode()`]: z85_encode
#[cfg(all(feature = "curve", not(windows)))]
#[doc(cfg(all(feature = "curve", not(windows))))]
pub fn curve_keypair() -> ZmqResult<(Vec<u8>, Vec<u8>)> {
    let mut public_key: [u8; 41] = [0; 41];
    let mut secret_key: [u8; 41] = [0; 41];

    if unsafe {
        zmq_sys_crate::zmq_curve_keypair(
            public_key.as_mut_ptr() as *mut c_char,
            secret_key.as_mut_ptr() as *mut c_char,
        )
    } == -1
    {
        cold_path();
        match unsafe { zmq_sys_crate::zmq_errno() } {
            errno @ zmq_sys_crate::errno::ENOTSUP => return Err(ZmqError::from(errno)),
            _ => unreachable!(),
        }
    }

    Ok((public_key.to_vec(), secret_key.to_vec()))
}

/// # derive the public key from a private key
///
/// The [`curve_public()`] function shall derive the public key from a private key. The keys are
/// encoded using [`z85_encode()`].
///
/// [`curve_public()`]: curve_public
/// [`z85_encode()`]: z85_encode
#[cfg(all(feature = "curve", not(windows)))]
#[doc(cfg(all(feature = "curve", not(windows))))]
pub fn curve_public<T>(mut secret_key: T) -> ZmqResult<Vec<u8>>
where
    T: AsMut<[u8]>,
{
    let mut public_key: [u8; 41] = [0; 41];
    let secret_key_array = secret_key.as_mut();

    if unsafe {
        zmq_sys_crate::zmq_curve_public(
            public_key.as_mut_ptr() as *mut c_char,
            secret_key_array.as_ptr() as *const c_char,
        )
    } == -1
    {
        cold_path();
        match unsafe { zmq_sys_crate::zmq_errno() } {
            errno @ zmq_sys_crate::errno::ENOTSUP => return Err(ZmqError::from(errno)),
            _ => unreachable!(),
        }
    }

    Ok(public_key.to_vec())
}

#[cfg(all(feature = "curve", not(windows)))]
#[cfg(test)]
mod curve_keypair_tests {
    use super::{curve_keypair, curve_public};
    use crate::prelude::ZmqResult;

    #[test]
    fn curve_keypair_generate_curve_keypair() -> ZmqResult<()> {
        let (public_key, secret_key) = curve_keypair()?;

        let pub_key = curve_public(secret_key)?;

        assert_eq!(public_key, pub_key);

        Ok(())
    }
}
