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

#[cfg(test)]
mod dish_tests {
    use super::DishSocket;
    use crate::prelude::{
        Context, Message, RadioSocket, Receiver, RecvFlags, SendFlags, Sender, ZmqError, ZmqResult,
    };

    #[test]
    fn join_joins_group() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DishSocket::from_context(&context)?;
        socket.join("asdf")?;

        Ok(())
    }

    #[test]
    fn join_when_already_joined() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DishSocket::from_context(&context)?;
        socket.join("asdf")?;
        let result = socket.join("asdf");

        assert!(result.is_err_and(|err| err == ZmqError::InvalidArgument));

        Ok(())
    }

    #[test]
    fn leave_leaves_group() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DishSocket::from_context(&context)?;
        socket.join("asdf")?;
        socket.leave("asdf")?;

        Ok(())
    }

    #[test]
    fn leave_when_no_group_joined() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = DishSocket::from_context(&context)?;
        let result = socket.leave("asdf");

        assert!(result.is_err_and(|err| err == ZmqError::InvalidArgument));

        Ok(())
    }

    #[test]
    fn radio_dish() -> ZmqResult<()> {
        let context = Context::new()?;

        let radio = RadioSocket::from_context(&context)?;
        radio.bind("tcp://127.0.0.1:*")?;
        let dish_endpoint = radio.last_endpoint()?;

        std::thread::spawn(move || {
            loop {
                let message: Message = "radio-msg".into();
                message.set_group("asdf").unwrap();
                radio.send_msg(message, SendFlags::DONT_WAIT).unwrap();
            }
        });

        let dish = DishSocket::from_context(&context)?;
        dish.connect(dish_endpoint)?;
        dish.join("asdf")?;

        let msg = dish.recv_msg(RecvFlags::empty())?;
        assert_eq!(msg.group().unwrap(), "asdf");
        assert_eq!(msg.to_string(), "radio-msg");

        Ok(())
    }

    #[cfg(feature = "futures")]
    #[test]
    fn radio_dish_async() -> ZmqResult<()> {
        let context = Context::new()?;

        let radio = RadioSocket::from_context(&context)?;
        radio.bind("tcp://127.0.0.1:*")?;
        let dish_endpoint = radio.last_endpoint()?;

        std::thread::spawn(move || {
            futures::executor::block_on(async {
                loop {
                    let message: Message = "radio-msg".into();
                    message.set_group("asdf").unwrap();
                    radio.send_msg_async(message, SendFlags::DONT_WAIT).await;
                }
            })
        });

        let dish = DishSocket::from_context(&context)?;
        dish.connect(dish_endpoint)?;
        dish.join("asdf")?;

        futures::executor::block_on(async {
            loop {
                if let Some(msg) = dish.recv_msg_async().await {
                    assert_eq!(msg.group().unwrap(), "asdf");
                    assert_eq!(msg.to_string(), "radio-msg");
                    break;
                }
            }
        });

        Ok(())
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
        socket_builder: SocketBuilder,
        #[builder(setter(into), default = "Default::default()")]
        join: String,
    }

    impl DishBuilder {
        pub fn apply(self, socket: &DishSocket) -> ZmqResult<()> {
            if let Some(socket_builder) = self.socket_builder {
                socket_builder.apply(socket)?;
            }

            self.join.iter().try_for_each(|join| socket.join(join))?;

            Ok(())
        }

        pub fn build_from_context(self, context: &Context) -> ZmqResult<DishSocket> {
            let socket = DishSocket::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }
    #[cfg(test)]
    mod dish_builder_tests {
        use super::DishBuilder;
        use crate::prelude::{Context, ZmqResult};

        #[test]
        fn default_dish_builder() -> ZmqResult<()> {
            let context = Context::new()?;

            DishBuilder::default().build_from_context(&context)?;

            Ok(())
        }

        #[test]
        fn dish_builder_with_custom_settings() -> ZmqResult<()> {
            let context = Context::new()?;

            DishBuilder::default()
                .join("asdf")
                .build_from_context(&context)?;

            Ok(())
        }
    }
}
