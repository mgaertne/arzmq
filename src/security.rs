//! 0MQ security mechanisms
//!
//! Currently only support [`Null`] and [`Plain`] across different platforms. Curve is
//! available with the <span class="stab portability"><code>curve</code></span> feature on Linux
//! and MacOS, but not on Windows. [`GSSAPI`] values are available across the crate, but are not
//! compiled in.
//!
//! [`Null`]: SecurityMechanism::Null
//! [`Plain`]: SecurityMechanism::Plain
//! [`GSSAPI`]: SecurityMechanism::GssApiServer

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
    #[doc(cfg(zmq_have_gssapi))]
    #[display("GssApiClient {{ ... }}")]
    /// GSSAPI client authentication and encryption
    GssApiClient { service_principal: String },
    #[doc(cfg(zmq_have_gssapi))]
    #[display("GssApiServer {{ ... }}")]
    /// GSSAPI server authentication and encryption
    GssApiServer,
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
            SecurityMechanism::GssApiClient { service_principal } if cfg!(zmq_have_gssapi) => {
                socket
                    .set_sockopt_string(SocketOption::GssApiServicePrincipal, service_principal)?;
            }
            SecurityMechanism::GssApiServer if cfg!(zmq_have_gssapi) => {
                socket.set_sockopt_bool(SocketOption::GssApiServer, true)?;
            }
            _ => (),
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
                if socket.get_sockopt_bool(SocketOption::CurveServer)? {
                    Ok(Self::CurveServer {
                        secret_key: Default::default(),
                    })
                } else {
                    Ok(Self::CurveClient {
                        server_key: Default::default(),
                        public_key: Default::default(),
                        secret_key: Default::default(),
                    })
                }
            }
            #[cfg(zmq_have_gssapi)]
            value if value == zmq_sys_crate::ZMQ_GSSAPI as i32 => {
                if socket.get_sockopt_bool(SocketOption::GssApiServer)? {
                    Ok(Self::GssApiServer)
                } else {
                    let service_principal =
                        socket.get_sockopt_string(SocketOption::GssApiServicePrincipal)?;
                    Ok(Self::GssApiClient { service_principal })
                }
            }
            _ => Err(ZmqError::Unsupported),
        }
    }
}

#[cfg(test)]
mod security_mechanism_tests {
    use super::SecurityMechanism;
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
        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        let security = SecurityMechanism::CurveServer {
            secret_key: vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 0,
            ],
        };
        security.apply(&socket)?;

        assert_eq!(
            socket.get_sockopt_int::<i32>(SocketOption::Mechanism)?,
            zmq_sys_crate::ZMQ_CURVE as i32
        );
        assert!(socket.get_sockopt_bool(SocketOption::CurveServer)?);

        Ok(())
    }

    #[cfg(all(feature = "curve", not(windows)))]
    #[test]
    fn apply_curve_client_security() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DealerSocket::from_context(&context)?;
        let security = SecurityMechanism::CurveClient {
            server_key: vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 0,
            ],
            public_key: vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 0,
            ],
            secret_key: vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 0,
            ],
        };
        security.apply(&socket)?;

        assert_eq!(
            socket.get_sockopt_int::<i32>(SocketOption::Mechanism)?,
            zmq_sys_crate::ZMQ_CURVE as i32
        );
        assert!(!socket.get_sockopt_bool(SocketOption::CurveServer)?);

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
pub fn curve_keypair() -> ZmqResult<(Vec<c_char>, Vec<c_char>)> {
    let mut public_key: [c_char; 41] = [0; 41];
    let mut secret_key: [c_char; 41] = [0; 41];

    if unsafe { zmq_sys_crate::zmq_curve_keypair(public_key.as_mut_ptr(), secret_key.as_mut_ptr()) }
        == -1
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
pub fn curve_public<T>(mut secret_key: T) -> ZmqResult<Vec<c_char>>
where
    T: AsMut<[c_char]>,
{
    let mut public_key: [c_char; 41] = [0; 41];
    let secret_key_array = secret_key.as_mut();

    if unsafe {
        zmq_sys_crate::zmq_curve_public(public_key.as_mut_ptr(), secret_key_array.as_mut_ptr())
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

#[doc(cfg(zmq_have_gssapi))]
#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
#[repr(i32)]
/// # name types for GSSAPI
pub enum GssApiNametype {
    /// the name is interpreted as a host based name
    NtHostbased,
    /// the name is interpreted as a local user name
    NtUsername,
    /// the name is interpreted as an unparsed principal name string (valid only with the krb5
    /// GSSAPI mechanism).
    NtKrb5Principal,
}

#[doc(cfg(zmq_have_gssapi))]
impl TryFrom<i32> for GssApiNametype {
    type Error = ZmqError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            _ if value == zmq_sys_crate::ZMQ_GSSAPI_NT_HOSTBASED as i32 => Ok(Self::NtHostbased),
            _ if value == zmq_sys_crate::ZMQ_GSSAPI_NT_USER_NAME as i32 => Ok(Self::NtUsername),
            _ if value == zmq_sys_crate::ZMQ_GSSAPI_NT_KRB5_PRINCIPAL as i32 => {
                Ok(Self::NtKrb5Principal)
            }
            _ => Err(ZmqError::Unsupported),
        }
    }
}

#[cfg(test)]
mod gss_api_nametype_tests {
    use rstest::*;

    use super::GssApiNametype;
    use crate::{
        prelude::{ZmqError, ZmqResult},
        zmq_sys_crate,
    };

    #[rstest]
    #[case(zmq_sys_crate::ZMQ_GSSAPI_NT_HOSTBASED as i32, Ok(GssApiNametype::NtHostbased))]
    #[case(zmq_sys_crate::ZMQ_GSSAPI_NT_USER_NAME as i32, Ok(GssApiNametype::NtUsername))]
    #[case(zmq_sys_crate::ZMQ_GSSAPI_NT_KRB5_PRINCIPAL as i32, Ok(GssApiNametype::NtKrb5Principal))]
    #[case(666, Err(ZmqError::Unsupported))]
    fn nametype_try_from(#[case] value: i32, #[case] expected: ZmqResult<GssApiNametype>) {
        assert_eq!(expected, GssApiNametype::try_from(value));
    }
}
