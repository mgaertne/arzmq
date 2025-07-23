use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOption, SocketType},
};

/// # A radio socket `ZMQ_RADIO`
///
/// A socket of type [`Radio`] is used by a publisher to distribute data. Each message belong to a
/// group, a group is specified with [`set_group()`]. Messages are distributed to all members of a
/// group.
///
/// When a [`Radio`] socket enters the 'mute' state due to having reached the high water mark for a
/// subscriber, then any messages that would be sent to the subscriber in question shall instead be
/// dropped until the mute state ends. The [`send_msg()`] function shall never block for this
/// socket type.
///
/// [`Radio`]: RadioSocket
/// [`set_group()`]: crate::message::Message::set_group
/// [`send_msg()`]: #method.send_msg
pub type RadioSocket = Socket<Radio>;

pub struct Radio {}

impl sealed::SenderFlag for Radio {}
impl sealed::SocketType for Radio {
    fn raw_socket_type() -> SocketType {
        SocketType::Radio
    }
}

unsafe impl Sync for Socket<Radio> {}
unsafe impl Send for Socket<Radio> {}

impl Socket<Radio> {
    /// # Retrieve multicast local loopback configuration `ZMQ_MULTICAST_LOOP`
    ///
    /// Retrieve the current multicast loopback configuration. A value of `true` means that the
    /// multicast packets sent on this socket will be looped back to local listening interface.
    pub fn multicast_loop(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::MulticastLoop)
    }

    /// # Control multicast local loopback `ZMQ_MULTICAST_LOOP`
    ///
    /// For multicast UDP sender sockets this option sets whether the data sent should be looped
    /// back on local listening sockets.
    pub fn set_multicast_loop(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::MulticastLoop, value)
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use core::default::Default;

    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::RadioSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "RadioBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`RadioSocket`].\n\n")]
    #[allow(dead_code)]
    struct RadioConfig {
        socket_config: SocketBuilder,
        #[builder(default = false)]
        multicast_loop: bool,
    }

    impl RadioBuilder {
        pub fn apply(self, socket: &RadioSocket) -> ZmqResult<()> {
            if let Some(socket_config) = self.socket_config {
                socket_config.apply(socket)?;
            }

            if let Some(multicast_loop) = self.multicast_loop {
                socket.set_multicast_loop(multicast_loop)?;
            }

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<RadioSocket> {
            let socket = RadioSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
}
