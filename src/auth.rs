//! # 0MQ authentification mechanisms
//!
//! Currently just [`ZapDomain`]s are supported.
//!
//! [`ZapDomain`]: ZapDomain
use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOption},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "builder", derive(serde::Serialize, serde::Deserialize))]
/// 0MQ authentification protocol (ZAP) domain reprensation
pub struct ZapDomain {
    domain: String,
}

impl ZapDomain {
    /// General constructor
    pub fn new(domain: String) -> Self {
        Self { domain }
    }

    /// Applies this ZAP domain to the provided socket.
    pub fn apply<T: sealed::SocketType>(&self, socket: &Socket<T>) -> ZmqResult<()> {
        socket.set_sockopt_string(SocketOption::ZapDomain, &self.domain)?;

        #[cfg(feature = "draft-api")]
        socket.set_sockopt_bool(SocketOption::ZapEnforceDomain, true)?;

        Ok(())
    }
}

impl<T: Into<String>> From<T> for ZapDomain {
    fn from(value: T) -> Self {
        ZapDomain::new(value.into())
    }
}
