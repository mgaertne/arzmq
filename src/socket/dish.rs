use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketType},
};

/// # A dish socket `ZMQ_DISH`
///
/// A socket of type [`Dish`] is used by a subscriber to subscribe to groups distributed by a
/// radio. Initially a [`Dish`] socket is not subscribed to any groups, use [`join()`] to join a
/// group. To get the group the message belong to call [`group()`].
///
/// [`Dish`]: DishSocket
/// [`join()`]: #method.join
/// [`group()`]: crate::message::Message::group
pub type DishSocket = Socket<Dish>;

pub struct Dish {}

impl sealed::ReceiverFlag for Dish {}
impl sealed::SocketType for Dish {
    fn raw_socket_type() -> SocketType {
        SocketType::Dish
    }
}

unsafe impl Sync for Socket<Dish> {}
unsafe impl Send for Socket<Dish> {}

impl Socket<Dish> {
    pub fn join<G>(&self, group: G) -> ZmqResult<()>
    where
        G: AsRef<str>,
    {
        self.socket.join(group.as_ref())
    }

    pub fn leave<G>(&self, group: G) -> ZmqResult<()>
    where
        G: AsRef<str>,
    {
        self.socket.leave(group.as_ref())
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use super::DishSocket;
    use crate::{ZmqResult, context::Context, socket::SocketBuilder};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "DishBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`DishSocket`].\n\n")]
    #[allow(dead_code)]
    struct DishConfig {
        socket_config: SocketBuilder,
        #[builder(setter(into), default = "Default::default()")]
        join: String,
    }

    impl DishBuilder {
        pub fn apply(self, socket: &DishSocket) -> ZmqResult<()> {
            if let Some(socket_config) = self.socket_config {
                socket_config.apply(socket)?;
            }

            if let Some(join) = self.join {
                socket.join(&join)?;
            }

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<DishSocket> {
            let socket = DishSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
}
