//! # 0MQ sockets
//!
//! Sockets are the main entry point to interact with other 0MQ sockets. They come in different
//! [`SocketType`]s for interactions.
//!
//! # Key differences to conventional sockets
//! Generally speaking, conventional sockets present a synchronous interface to either
//! connection-oriented reliable byte streams (SOCK_STREAM), or connection-less unreliable
//! datagrams (SOCK_DGRAM). In comparison, 0MQ sockets present an abstraction of an asynchronous
//! message queue, with the exact queueing semantics depending on the socket type in use. Where
//! conventional sockets transfer streams of bytes or discrete datagrams, 0MQ sockets transfer
//! discrete messages.
//!
//! 0MQ sockets being asynchronous means that the timings of the physical connection setup and tear
//! down, reconnect and effective delivery are transparent to the user and organized by 0MQ itself.
//! Further, messages may be queued in the event that a peer is unavailable to receive them.
//!
//! Conventional sockets allow only strict one-to-one (two peers), many-to-one (many clients, one
//! server), or in some cases one-to-many (multicast) relationships. With the exception of
//! [`Pair`] and [`Channel`], 0MQ sockets may be connected to multiple endpoints using
//! [`connect()`], while simultaneously accepting incoming connections from multiple endpoints
//! bound to the socket using [`bind()`], thus allowing many-to-many relationships.
//!
//! # Socket types
//! The following sections present the socket types defined by 0MQ, grouped by the general
//! messaging pattern which is built from related socket types.
//!
//! [`SocketType`]: SocketType
//! [`Pair`]: PairSocket
//! [`Channel`]: ChannelSocket
//! [`connect()`]: Socket::connect
//! [`bind()`]: Socket::bind
//!
//! ## Publish-Subscribe pattern
//! The publish-subscribe pattern is used for one-to-many distribution of data from a single
//! [`Publish`]er to multiple [`Subscribe`]ers in a fan out fashion.
//!
//! ### Examples:
//! Publish-Subscribe:
//! ```
//! # use core::sync::atomic::{Ordering, AtomicBool};
//! # use std::thread;
//! #
//! # use arzmq::prelude::{ZmqResult, Context, PublishSocket, SubscribeSocket, Sender, SendFlags, Receiver, RecvFlags};
//! #
//! static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
//!
//! const SUBSCRIBED_TOPIC: &str = "arzmq-example";
//!
//! pub fn run_publisher<S>(socket: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: Sender,
//! {
//!     while KEEP_RUNNING.load(Ordering::Acquire) {
//!         socket.send_msg(msg, SendFlags::empty())?;
//!     }
//!
//!     Ok(())
//! }
//!
//! pub fn run_subscribe_client<S>(socket: &S, subscribed_topic: &str) -> ZmqResult<()>
//! where
//!    S: Receiver,
//! {
//!     let zmq_msg = socket.recv_msg(RecvFlags::empty())?;
//!     let zmq_str = zmq_msg.to_string();
//!     let pubsub_item = zmq_str.split_once(" ");
//!     assert_eq!(Some((subscribed_topic, "important update")), pubsub_item);
//!
//!     Ok(())
//! }
//!
//! fn main() -> ZmqResult<()> {
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let publish = PublishSocket::from_context(&context)?;
//!     publish.bind("tcp://127.0.0.1:*")?;
//!     let subscribe_endpoint = publish.last_endpoint()?;
//!
//!     thread::spawn(move || {
//!         let published_msg = format!("{SUBSCRIBED_TOPIC} important update");
//!         run_publisher(&publish, &published_msg).unwrap();
//!     });
//!
//!     let subscribe = SubscribeSocket::from_context(&context)?;
//!     subscribe.connect(subscribe_endpoint)?;
//!     subscribe.subscribe(SUBSCRIBED_TOPIC)?;
//!
//!     (0..iterations).try_for_each(|number| {
//!         run_subscribe_client(&subscribe, SUBSCRIBED_TOPIC)
//!     })?;
//!
//!     KEEP_RUNNING.store(false, Ordering::Release);
//!
//!     Ok(())
//! }
//! ```
//!
//! XPublish-XSubscribe:
//! ```
//! # use core::sync::atomic::{Ordering, AtomicBool};
//! # use std::thread;
//! #
//! # use arzmq::prelude::{ZmqResult, Context, Receiver, RecvFlags, SendFlags, Sender, XPublishSocket, XSubscribeSocket};
//! #
//! static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
//!
//! const SUBSCRIBED_TOPIC: &str = "arzmq-example";
//!
//! fn run_xpublish_socket(xpublish: &XPublishSocket, msg: &str) -> ZmqResult<()> {
//!     while KEEP_RUNNING.load(Ordering::Acquire) {
//!         let published_msg = format!("{SUBSCRIBED_TOPIC} {msg}");
//!         xpublish.send_msg(&published_msg, SendFlags::empty())?;
//!     }
//!
//!     Ok(())
//! }
//!
//! pub fn run_subscribe_client<S>(socket: &S, subscribed_topic: &str) -> ZmqResult<()>
//! where
//!    S: Receiver,
//! {
//!     let zmq_msg = socket.recv_msg(RecvFlags::empty())?;
//!     let zmq_str = zmq_msg.to_string();
//!     let pubsub_item = zmq_str.split_once(" ");
//!     assert_eq!(Some((subscribed_topic, "important update")), pubsub_item);
//!
//!     Ok(())
//! }
//!
//! fn main() -> ZmqResult<()> {
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let xpublish = XPublishSocket::from_context(&context)?;
//!     xpublish.bind("tcp://127.0.0.1:*")?;
//!     let xsubscribe_endpoint = xpublish.last_endpoint()?;
//!
//!     thread::spawn(move || {
//!         run_xpublish_socket(&xpublish, "important update").unwrap();
//!     });
//!
//!     let xsubscribe = XSubscribeSocket::from_context(&context)?;
//!     xsubscribe.connect(xsubscribe_endpoint)?;
//!     xsubscribe.subscribe(SUBSCRIBED_TOPIC)?;
//!
//!     (0..iterations).try_for_each(|number| {
//!         run_subscribe_client(&xsubscribe, SUBSCRIBED_TOPIC)
//!     })?;
//!
//!     KEEP_RUNNING.store(false, Ordering::Release);
//!
//!     Ok(())
//! }
//! ```
//!
//! [`Publish`]: PublishSocket
//! [`Subscribe`]: SubscribeSocket
//!
//! ## Client-Server pattern <span class="stab portability"><code>draft-api</code></span>
//! The client-server pattern is used to allow a single [`Server`] server talk to one or more
//! [`Client`] clients. The client always starts the conversation, after which either peer can send
//! messages asynchronously, to the other.
//!
//! ### Examples:
//! ```
//! # #[cfg(feature = "draft-api")]
//! # use core::sync::atomic::{AtomicBool, Ordering};
//! # #[cfg(feature = "draft-api")]
//! # use std::thread;
//! #
//! # #[cfg(feature = "draft-api")]
//! # use arzmq::prelude::{ZmqError, ZmqResult, Context, Message, ClientSocket, Receiver, RecvFlags, SendFlags, Sender, ServerSocket};
//! #
//! # #[cfg(feature = "draft-api")]
//! static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
//!
//! # #[cfg(feature = "draft-api")]
//! fn run_server_socket(server: &ServerSocket, reply: &str) -> ZmqResult<()> {
//!     while KEEP_RUNNING.load(Ordering::Acquire) {
//!         let message = server.recv_msg(RecvFlags::empty())?;
//!         assert_eq!(message.to_string(), "Hello");
//!
//!         let returned: Message = reply.into();
//!         returned.set_routing_id(message.routing_id().unwrap())?;
//!         server.send_msg(returned, SendFlags::empty())?;
//!     }
//!
//!     Ok(())
//! }
//!
//! # #[cfg(feature = "draft-api")]
//! pub fn run_send_recv<S>(send_recv: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: Sender + Receiver,
//! {
//!     send_recv.send_msg(msg, SendFlags::empty())?;
//!
//!     let message = send_recv.recv_msg(RecvFlags::empty())?;
//!     assert_eq!(message.to_string(), "World");
//!
//!     Ok(())
//! }
//!
//! # #[cfg(feature = "draft-api")]
//! fn main() -> ZmqResult<()> {
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let server = ServerSocket::from_context(&context)?;
//!     server.bind("tcp://127.0.0.1:*")?;
//!     let client_endpoint = server.last_endpoint()?;
//!
//!     thread::spawn(move || {
//!             run_server_socket(&server, "World").unwrap();
//!     });
//!
//!     let client = ClientSocket::from_context(&context)?;
//!     client.connect(client_endpoint)?;
//!
//!     (0..iterations).try_for_each(|number| {
//!         run_send_recv(&client, "Hello")
//!      })?;
//!
//!     KEEP_RUNNING.store(false, Ordering::Release);
//!
//!     Ok(())
//! }
//! # #[cfg(not(feature = "draft-api"))]
//! # fn main() {}
//! ```
//!
//! [`Server`]: ServerSocket
//! [`Client`]: ClientSocket
//!
//! ## Radio-Dish pattern <span class="stab portability"><code>draft-api</code></span>
//!
//! The radio-dish pattern is used for one-to-many distribution of data from a single publisher to
//! multiple subscribers in a fan out fashion.
//!
//! Radio-dish is using groups (vs Pub-sub topics), [`Dish`] sockets can #[`join()`] a group and
//! each message sent by [´Radio´] sockets belong to a group.
//!
//! Groups are null terminated strings limited to 16 chars length (including null). The intention
//! is to increase the length to 40 chars (including null). The encoding of groups shall be UTF8.
//!
//! ### Example
//!
//! ```
//! # #[cfg(feature = "draft-api")]
//! # use core::sync::atomic::{AtomicBool, Ordering};
//! # #[cfg(feature = "draft-api")]
//! # use std::thread;
//! #
//! # #[cfg(feature = "draft-api")]
//! # use arzmq::prelude::{ZmqError, ZmqResult, Context, Message, DishSocket, RadioSocket, Receiver, RecvFlags, SendFlags, Sender};
//! #
//! # #[cfg(feature = "draft-api")]
//! static GROUP: &str = "radio-dish-ex";
//!
//! # #[cfg(feature = "draft-api")]
//! static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
//!
//! # #[cfg(feature = "draft-api")]
//! fn run_radio_socket(radio: &RadioSocket, message: &str) -> ZmqResult<()> {
//!     while KEEP_RUNNING.load(Ordering::Acquire) {
//!         let msg: Message = message.into();
//!         msg.set_group(GROUP).unwrap();
//!
//!         radio.send_msg(msg, SendFlags::empty()).unwrap();
//!     }
//!
//!     Ok(())
//! }
//!
//! # #[cfg(feature = "draft-api")]
//! fn main() -> ZmqResult<()> {
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let radio = RadioSocket::from_context(&context)?;
//!     radio.bind("tcp://127.0.0.1:*")?;
//!     let dish_endpoint = radio.last_endpoint()?;
//!
//!     thread::spawn(move || {
//!         run_radio_socket(&radio, "radio msg").unwrap();
//!     });
//!
//!     let dish = DishSocket::from_context(&context)?;
//!     dish.connect(dish_endpoint)?;
//!     dish.join(GROUP)?;
//!
//!     (0..iterations).try_for_each(|_| {
//!         let msg = dish.recv_msg(RecvFlags::empty())?;
//!         assert_eq!(msg.to_string(), "radio msg");
//!         Ok::<(), ZmqError>(())
//!     })?;
//!
//!     KEEP_RUNNING.store(false, Ordering::Release);
//!
//!     Ok(())
//! }
//! # #[cfg(not(feature = "draft-api"))]
//! # fn main() {}
//! ```
//!
//! [`Radio`]: RadioSocket
//! [`Dish`]: DishSocket
//! [`join()`]: DishSocket::join
//!
//! ## Pipeline pattern ([`Push`]/[`Pull`])
//! The pipeline pattern is used for distributing data to nodes arranged in a pipeline. Data always
//! flows down the pipeline, and each stage of the pipeline is connected to at least one node. When
//! a pipeline stage is connected to multiple nodes data is round-robined among all connected nodes
//!
//! ### Example
//! ```
//! # use core::sync::atomic::{Ordering, AtomicBool};
//! # use std::thread;
//! #
//! # use arzmq::prelude::{ZmqError, ZmqResult, Context, PullSocket, PushSocket, Sender, SendFlags, Receiver, RecvFlags};
//! #
//! static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
//!
//! fn main() -> ZmqResult<()> {
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let push = PushSocket::from_context(&context)?;
//!     push.bind("tcp://127.0.0.1:*")?;
//!     let pull_endpoint = push.last_endpoint()?;
//!
//!     thread::spawn(move || {
//!         while KEEP_RUNNING.load(Ordering::Acquire) {
//!             push.send_msg("important update", SendFlags::empty()).unwrap();
//!         }
//!     });
//!
//!     let pull = PullSocket::from_context(&context)?;
//!     pull.connect(pull_endpoint)?;
//!
//!     (0..iterations).try_for_each(|_| {
//!         let msg = pull.recv_msg(RecvFlags::empty())?;
//!         assert_eq!(msg.to_string(), "important update");
//!
//!         Ok::<(), ZmqError>(())
//!     })?;
//!
//!     KEEP_RUNNING.store(false, Ordering::Release);
//!
//!     Ok(())
//! }
//! ```
//!
//! [`Push`]: PushSocket
//! [`Pull`]: PullSocket
//!
//! ## Scatter-gather pattern <span class="stab portability"><code>draft-api</code></span>
//! The [`Scatter`]-[`Gather`] pattern is the thread-safe version of the pipeline pattern. The
//! scatter-gather pattern is used for distributing data to nodes arranged in a pipeline. Data
//! always flows down the pipeline, and each stage of the pipeline is connected to at least one
//! node. When a pipeline stage is connected to multiple nodes data is round-robined among all
//! connected nodes.
//!
//! ### Example
//! ```
//! # #[cfg(feature = "draft-api")]
//! # use core::sync::atomic::{AtomicBool, Ordering};
//! # #[cfg(feature = "draft-api")]
//! # use std::thread;
//! #
//! # #[cfg(feature = "draft-api")]
//! # use arzmq::prelude::{ZmqResult, ZmqError, Context, GatherSocket, Receiver, RecvFlags, ScatterSocket, SendFlags, Sender};
//! #
//! # #[cfg(feature = "draft-api")]
//! static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
//!
//! # #[cfg(feature = "draft-api")]
//! pub fn run_publisher<S>(socket: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: Sender,
//! {
//!     while KEEP_RUNNING.load(Ordering::Acquire) {
//!         socket.send_msg(msg, SendFlags::empty())?;
//!     }
//!
//!     Ok(())
//! }
//!
//! # #[cfg(feature = "draft-api")]
//! fn main() -> ZmqResult<()> {
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let scatter = ScatterSocket::from_context(&context)?;
//!     scatter.bind("tcp://127.0.0.1:*")?;
//!     let gather_endpoint = scatter.last_endpoint()?;
//!
//!     thread::spawn(move || {
//!         run_publisher(&scatter, "important update").unwrap();
//!     });
//!
//!     let gather = GatherSocket::from_context(&context)?;
//!     gather.connect(gather_endpoint)?;
//!
//!     (0..iterations).try_for_each(|_| {
//!         let msg = gather.recv_msg(RecvFlags::empty())?;
//!         assert_eq!(msg.to_string(), "important update");
//!
//!         Ok::<(), ZmqError>(())
//!     })?;
//!
//!     KEEP_RUNNING.store(false, Ordering::Release);
//!
//!     Ok(())
//! }
//! # #[cfg(not(feature = "draft-api"))]
//! # fn main() {}
//! ```
//!
//! [`Scatter`]: ScatterSocket
//! [`Gather`]: GatherSocket
//!
//! ## Exclusive pair pattern
//! The exclusive [`Pair`] pattern is used to connect a peer to precisely one other peer. This pattern
//! is used for inter-thread communication across the inproc transport.
//!
//! ### Example
//! ```
//! # use std::thread;
//! #
//! # use arzmq::prelude::{ZmqResult, Context, PairSocket, Sender, Receiver, SendFlags, RecvFlags};
//! #
//! pub fn run_recv_send<S>(recv_send: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: Receiver + Sender,
//! {
//!     let message = recv_send.recv_msg(RecvFlags::empty())?;
//!     assert_eq!(message.to_string(), "Hello");
//!
//!     recv_send.send_msg(msg, SendFlags::empty())
//! }
//!
//! pub fn run_send_recv<S>(send_recv: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: Sender + Receiver,
//! {
//!     send_recv.send_msg(msg, SendFlags::empty())?;
//!
//!     let message = send_recv.recv_msg(RecvFlags::empty())?;
//!     assert_eq!(message.to_string(), "World");
//!
//!     Ok(())
//! }
//!
//! fn main() -> ZmqResult<()> {
//!     let endpoint = "inproc://arzmq-example-pair";
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let pair_server = PairSocket::from_context(&context)?;
//!     pair_server.bind(endpoint)?;
//!
//!     thread::spawn(move || {
//!         (0..iterations)
//!             .try_for_each(|_| run_recv_send(&pair_server, "World"))
//!             .unwrap();
//!     });
//!
//!     let pair_client = PairSocket::from_context(&context)?;
//!     pair_client.connect(endpoint)?;
//!
//!     (0..iterations).try_for_each(|_| run_send_recv(&pair_client, "Hello"))
//! }
//! ```
//!
//! [`Pair`]: PairSocket
//!
//! ## Peer-to-peer pattern <span class="stab portability"><code>draft-api</code></span>
//! The peer-to-peer pattern is used to connect a [`Peer`] to multiple peers. Peer can both connect
//! and bind and mix both of them with the same socket. The peer-to-peer pattern is useful to build
//! peer-to-peer networks (e.g zyre, bitcoin, torrent) where a peer can both accept connections
//! from other peers or connect to them.
//!
//! ### Example
//! ```
//! # #[cfg(feature = "draft-api")]
//! # use std::thread;
//! #
//! # #[cfg(feature = "draft-api")]
//! # use arzmq::prelude::{ZmqResult, Context, Message, PeerSocket, Receiver, RecvFlags, SendFlags, Sender};
//! #
//! # #[cfg(feature = "draft-api")]
//! fn run_peer_server(peer: &PeerSocket, msg: &str) -> ZmqResult<()> {
//!     let message = peer.recv_msg(RecvFlags::empty())?;
//!     assert_eq!(message.to_string(), "Hello");
//!
//!     let response: Message = msg.into();
//!     response.set_routing_id(message.routing_id().unwrap())?;
//!     peer.send_msg(response, SendFlags::empty())
//! }
//!
//! # #[cfg(feature = "draft-api")]
//! fn run_peer_client(peer: &PeerSocket, routing_id: u32, msg: &str) -> ZmqResult<()> {
//!     let request: Message = msg.into();
//!     request.set_routing_id(routing_id)?;
//!     peer.send_msg(request, SendFlags::empty())?;
//!
//!     let message = peer.recv_msg(RecvFlags::empty())?;
//!     assert_eq!(message.to_string(), "World");
//!
//!     Ok(())
//! }
//!
//! # #[cfg(feature = "draft-api")]
//! fn main() -> ZmqResult<()> {
//!     let endpoint = "inproc://arzmq-example-peer";
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let peer_server = PeerSocket::from_context(&context)?;
//!     peer_server.bind(endpoint)?;
//!
//!     thread::spawn(move || {
//!         (0..iterations).try_for_each(|_| {
//!             run_peer_server(&peer_server, "World")
//!         }).unwrap();
//!     });
//!
//!     let peer_client = PeerSocket::from_context(&context)?;
//!     let routing_id = peer_client.connect_peer(endpoint)?;
//!
//!     (0..iterations).try_for_each(|_| run_peer_client(&peer_client, routing_id, "Hello"))
//! }
//! # #[cfg(not(feature = "draft-api"))]
//! # fn main() {}
//! ```
//!
//! [`Peer`]: PeerSocket
//!
//! ## Channel pattern <span class="stab portability"><code>draft-api</code></span>
//! The [`Channel`] pattern is the thread-safe version of the exclusive [`Pair`] pattern. The
//! channel pattern is used to connect a peer to precisely one other peer. This pattern is used for
//! inter-thread communication across the inproc transport.
//!
//! ### Example
//! ```
//! # #[cfg(feature = "draft-api")]
//! # use std::thread;
//! #
//! # #[cfg(feature = "draft-api")]
//! # use arzmq::prelude::{ZmqResult, Context, ChannelSocket, Sender, Receiver, SendFlags, RecvFlags};
//! #
//! # #[cfg(feature = "draft-api")]
//! pub fn run_recv_send<S>(recv_send: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: Receiver + Sender,
//! {
//!     let message = recv_send.recv_msg(RecvFlags::empty())?;
//!     assert_eq!(message.to_string(), "Hello");
//!
//!     recv_send.send_msg(msg, SendFlags::empty())
//! }
//!
//! # #[cfg(feature = "draft-api")]
//! pub fn run_send_recv<S>(send_recv: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: Sender + Receiver,
//! {
//!     send_recv.send_msg(msg, SendFlags::empty())?;
//!
//!     let message = send_recv.recv_msg(RecvFlags::empty())?;
//!     assert_eq!(message.to_string(), "World");
//!
//!     Ok(())
//! }
//!
//! # #[cfg(feature = "draft-api")]
//! fn main() -> ZmqResult<()> {
//!     let endpoint = "inproc://arzmq-example-channel";
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let channel_server = ChannelSocket::from_context(&context)?;
//!     channel_server.bind(endpoint)?;
//!
//!     thread::spawn(move || {
//!         (0..iterations)
//!             .try_for_each(|_| run_recv_send(&channel_server, "World"))
//!             .unwrap();
//!     });
//!
//!     let channel_client = ChannelSocket::from_context(&context)?;
//!     channel_client.connect(endpoint)?;
//!
//!     (0..iterations).try_for_each(|_| run_send_recv(&channel_client, "Hello"))
//! }
//! # #[cfg(not(feature = "draft-api"))]
//! # fn main() {}
//! ```
//!
//! [`Channel`]: ChannelSocket
//! [`Pair`]: PairSocket
//!
//! ## Native pattern ([`Stream`])
//! The native pattern is used for communicating with TCP peers and allows asynchronous requests
//! and replies in either direction.
//!
//! ### Example
//! Stream as client:
//! ```
//! # use core::error::Error;
//! # use std::{io::prelude::*, net::TcpListener, thread};
//! #
//! # use arzmq::prelude::{ZmqResult, Context, MultipartMessage, MultipartReceiver, MultipartSender, RecvFlags, SendFlags, StreamSocket};
//! #
//! fn run_tcp_server(endpoint: &str) -> Result<(), Box<dyn Error>> {
//!     let tcp_listener = TcpListener::bind(endpoint)?;
//!     thread::spawn(move || {
//!         let (mut tcp_stream, _socket_addr) = tcp_listener.accept().unwrap();
//!         tcp_stream.write_all("".as_bytes()).unwrap();
//!         loop {
//!             let mut buffer = [0; 256];
//!             if let Ok(length) = tcp_stream.read(&mut buffer) {
//!                 if length == 0 {
//!                     break;
//!                 }
//!                 let recevied_msg = &buffer[..length];
//!                 assert_eq!(str::from_utf8(recevied_msg).unwrap(), "Hello");
//!                 tcp_stream.write_all("World".as_bytes()).unwrap();
//!             }
//!         }
//!     });
//!
//!     Ok(())
//! }
//!
//! fn run_stream_socket(zmq_stream: &StreamSocket, routing_id: &[u8], msg: &str) -> ZmqResult<()> {
//!     let mut multipart = MultipartMessage::new();
//!     multipart.push_back(routing_id.into());
//!     multipart.push_back(msg.into());
//!     zmq_stream.send_multipart(multipart, SendFlags::empty())?;
//!
//!     let mut message = zmq_stream.recv_multipart(RecvFlags::empty())?;
//!     assert_eq!(message.pop_back().unwrap().to_string(), "World");
//!
//!     Ok(())
//! }
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let port = 5558;
//!     let iterations = 10;
//!
//!     let tcp_endpoint = format!("127.0.0.1:{port}");
//!     run_tcp_server(&tcp_endpoint)?;
//!
//!     let context = Context::new()?;
//!
//!     let zmq_stream = StreamSocket::from_context(&context)?;
//!
//!     let stream_endpoint = format!("tcp://127.0.0.1:{port}");
//!     zmq_stream.connect(stream_endpoint)?;
//!
//!     let mut connect_msg = zmq_stream.recv_multipart(RecvFlags::empty())?;
//!     let routing_id = connect_msg.pop_front().unwrap();
//!
//!     (0..iterations)
//!         .try_for_each(|_| run_stream_socket(&zmq_stream, &routing_id.bytes(), "Hello"))?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Stream as server:
//! ```
//! # use core::error::Error;
//! # use std::{io::prelude::*, net::TcpStream, thread};
//! #
//! # use arzmq::prelude::{ZmqResult, Context, MultipartReceiver, MultipartSender, RecvFlags, SendFlags, StreamSocket};
//! #
//! fn run_stream_socket(zmq_stream: &StreamSocket, _routing_id: &[u8], msg: &str) -> ZmqResult<()> {
//!     let mut message = zmq_stream.recv_multipart(RecvFlags::empty())?;
//!     assert_eq!(message.pop_back().unwrap().to_string(), "Hello");
//!
//!     message.push_back(msg.into());
//!     zmq_stream.send_multipart(message, SendFlags::empty())
//! }
//!
//! fn run_tcp_client(endpoint: &str, iterations: i32) -> Result<(), Box<dyn Error>> {
//!     let mut tcp_stream = TcpStream::connect(endpoint)?;
//!     (0..iterations).try_for_each(|request_no| {
//!         println!("Sending requrst {request_no}");
//!         tcp_stream.write_all("Hello".as_bytes()).unwrap();
//!
//!         let mut buffer = [0; 256];
//!         if let Ok(length) = tcp_stream.read(&mut buffer)
//!             && length != 0
//!         {
//!             let recevied_msg = &buffer[..length];
//!             assert_eq!(str::from_utf8(recevied_msg).unwrap(), "World");
//!         }
//!
//!         Ok::<(), Box<dyn Error>>(())
//!     })?;
//!
//!     Ok(())
//! }
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let zmq_stream = StreamSocket::from_context(&context)?;
//!     zmq_stream.bind("tcp://127.0.0.1:*")?;
//!     let tcp_endpoint = zmq_stream.last_endpoint()?;
//!
//!     thread::spawn(move || {
//!         let mut connect_msg = zmq_stream.recv_multipart(RecvFlags::empty()).unwrap();
//!         let routing_id = connect_msg.pop_front().unwrap();
//!
//!         loop {
//!             run_stream_socket(&zmq_stream, &routing_id.bytes(), "World").unwrap();
//!         }
//!     });
//!
//!     run_tcp_client(tcp_endpoint.strip_prefix("tcp://").unwrap(), iterations)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! [`Stream`]: StreamSocket
//!
//! ## Request-Reply pattern
//! The request-reply pattern is used for sending requests from a [`Request`] client to one or more
//! [`Reply`] services, and receiving subsequent replies to each request sent.
//!
//! ### Examples
//! Request-Reply sockets
//! ```
//! # use std::thread;
//! #
//! # use arzmq::prelude::{ZmqResult, Context, ReplySocket, RequestSocket, Sender, Receiver, SendFlags, RecvFlags};
//! #
//! pub fn run_recv_send<S>(recv_send: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: Receiver + Sender,
//! {
//!     let message = recv_send.recv_msg(RecvFlags::empty())?;
//!     assert_eq!(message.to_string(), "Hello");
//!
//!     recv_send.send_msg(msg, SendFlags::empty())
//! }
//!
//! pub fn run_send_recv<S>(send_recv: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: Sender + Receiver,
//! {
//!     send_recv.send_msg(msg, SendFlags::empty())?;
//!
//!     let message = send_recv.recv_msg(RecvFlags::empty())?;
//!     assert_eq!(message.to_string(), "World");
//!
//!     Ok(())
//! }
//!
//! fn main() -> ZmqResult<()> {
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let reply = ReplySocket::from_context(&context)?;
//!     reply.bind("tcp://127.0.0.1:*")?;
//!     let request_endpoint = reply.last_endpoint()?;
//!
//!     thread::spawn(move || {
//!         (1..=iterations)
//!             .try_for_each(|_| run_recv_send(&reply, "World"))
//!             .unwrap();
//!     });
//!
//!     let request = RequestSocket::from_context(&context)?;
//!     request.connect(request_endpoint)?;
//!
//!     (0..iterations).try_for_each(|_| run_send_recv(&request, "Hello"))
//! }
//! ```
//!
//! Request-Router sockets
//! ```
//! # use std::thread;
//! #
//! # use arzmq::prelude::{ZmqResult, Context, RequestSocket, RouterSocket, Sender, Receiver, SendFlags, RecvFlags, MultipartSender, MultipartReceiver};
//! #
//! pub fn run_multipart_recv_reply<S>(recv_send: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: MultipartSender + MultipartReceiver,
//! {
//!     let mut multipart = recv_send.recv_multipart(RecvFlags::empty())?;
//!
//!     let content = multipart.pop_back().unwrap();
//!     if !content.is_empty() {
//!         assert_eq!(content.to_string(), "Hello");
//!     }
//!
//!     multipart.push_back(msg.into());
//!     recv_send.send_multipart(multipart, SendFlags::empty())
//! }
//!
//! pub fn run_send_recv<S>(send_recv: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: Sender + Receiver,
//! {
//!     send_recv.send_msg(msg, SendFlags::empty())?;
//!
//!     let message = send_recv.recv_msg(RecvFlags::empty())?;
//!     assert_eq!(message.to_string(), "World");
//!
//!     Ok(())
//! }
//!
//! fn main() -> ZmqResult<()> {
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let router = RouterSocket::from_context(&context)?;
//!     router.bind("tcp://127.0.0.1:*")?;
//!     let request_endpoint = router.last_endpoint()?;
//!
//!     thread::spawn(move || {
//!         (0..iterations)
//!             .try_for_each(|_| run_multipart_recv_reply(&router, "World"))
//!             .unwrap();
//!     });
//!
//!     let request = RequestSocket::from_context(&context)?;
//!     request.connect(request_endpoint)?;
//!
//!     (0..iterations).try_for_each(|_| run_send_recv(&request, "Hello"))
//! }
//! ```
//!
//! Dealer-Router sockets
//! ```
//! # use std::thread;
//! #
//! # use arzmq::prelude::{ZmqResult, Context, Message, DealerSocket, RouterSocket, MultipartReceiver, MultipartSender, SendFlags, RecvFlags};
//! #
//! pub fn run_multipart_recv_reply<S>(recv_send: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: MultipartSender + MultipartReceiver,
//! {
//!     let mut multipart = recv_send.recv_multipart(RecvFlags::empty())?;
//!
//!     let content = multipart.pop_back().unwrap();
//!     if !content.is_empty() {
//!         assert_eq!(content.to_string(), "Hello");
//!     }
//!
//!     multipart.push_back(msg.into());
//!     recv_send.send_multipart(multipart, SendFlags::empty())
//! }
//!
//! pub fn run_multipart_send_recv<S>(send_recv: &S, msg: &str) -> ZmqResult<()>
//! where
//!     S: MultipartReceiver + MultipartSender,
//! {
//!     println!("Sending message {msg:?}");
//!     let multipart: Vec<Message> = vec![vec![].into(), msg.into()];
//!     send_recv.send_multipart(multipart, SendFlags::empty())?;
//!
//!     let mut multipart = send_recv.recv_multipart(RecvFlags::empty())?;
//!     let content = multipart.pop_back().unwrap();
//!     if !content.is_empty() {
//!         assert_eq!(content.to_string(), "World");
//!     }
//!
//!     Ok(())
//! }
//!
//! fn main() -> ZmqResult<()> {
//!     let iterations = 10;
//!
//!     let context = Context::new()?;
//!
//!     let router = RouterSocket::from_context(&context)?;
//!     router.bind("tcp://127.0.0.1:*")?;
//!     let dealer_endpoint = router.last_endpoint()?;
//!
//!     thread::spawn(move || {
//!         (0..iterations)
//!             .try_for_each(|_| run_multipart_recv_reply(&router, "World"))
//!             .unwrap();
//!     });
//!
//!     let dealer = DealerSocket::from_context(&context)?;
//!     dealer.connect(dealer_endpoint)?;
//!
//!     (0..iterations).try_for_each(|_| run_multipart_send_recv(&dealer, "Hello"))
//! }
//! ```
//!
//! [`Request`]: RequestSocket
//! [`Reply`]: ReplySocket

use alloc::sync::Arc;
use core::{iter, marker::PhantomData, ops::ControlFlow};

#[cfg(feature = "futures")]
use ::futures::{FutureExt, TryStreamExt};
#[cfg(feature = "futures")]
use async_trait::async_trait;
use bitflags::bitflags;
use derive_more::From;
use num_traits::PrimInt;

use crate::{
    ZmqError, ZmqResult,
    context::Context,
    ffi::RawSocket,
    message::{Message, MultipartMessage, Sendable},
    sealed, zmq_sys_crate,
};

#[cfg(feature = "draft-api")]
mod channel;
#[cfg(feature = "draft-api")]
mod client;
mod dealer;
#[cfg(feature = "draft-api")]
mod dish;
#[cfg(feature = "draft-api")]
mod gather;
pub(crate) mod monitor;
mod pair;
#[cfg(feature = "draft-api")]
mod peer;
mod publish;
mod pull;
mod push;
#[cfg(feature = "draft-api")]
mod radio;
mod reply;
mod request;
mod router;
#[cfg(feature = "draft-api")]
mod scatter;
#[cfg(feature = "draft-api")]
mod server;
mod stream;
mod subscribe;
mod xpublish;
mod xsubscribe;

#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use builder::SocketBuilder;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use channel::ChannelSocket;
#[cfg(all(feature = "draft-api", feature = "builder"))]
#[doc(cfg(all(feature = "draft-api", feature = "builder")))]
pub use channel::builder::ChannelBuilder;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use client::ClientSocket;
#[cfg(all(feature = "draft-api", feature = "builder"))]
#[doc(cfg(all(feature = "draft-api", feature = "builder")))]
pub use client::builder::ClientBuilder;
pub use dealer::DealerSocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use dealer::builder::DealerBuilder;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use dish::DishSocket;
#[cfg(all(feature = "draft-api", feature = "builder"))]
#[doc(cfg(all(feature = "draft-api", feature = "builder")))]
pub use dish::builder::DishBuilder;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use gather::GatherSocket;
#[cfg(all(feature = "draft-api", feature = "builder"))]
#[doc(cfg(all(feature = "draft-api", feature = "builder")))]
pub use gather::builder::GatherBuilder;
use monitor::Monitor;
pub use monitor::{HandshakeProtocolError, MonitorReceiver, MonitorSocket, MonitorSocketEvent};
pub use pair::PairSocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use pair::builder::PairBuilder;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use peer::PeerSocket;
#[cfg(all(feature = "draft-api", feature = "builder"))]
#[doc(cfg(all(feature = "draft-api", feature = "builder")))]
pub use peer::builder::PeerBuilder;
pub use publish::PublishSocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use publish::builder::PublishBuilder;
pub use pull::PullSocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use pull::builder::PullBuilder;
pub use push::PushSocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use push::builder::PushBuilder;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use radio::RadioSocket;
#[cfg(all(feature = "draft-api", feature = "builder"))]
#[doc(cfg(all(feature = "draft-api", feature = "builder")))]
pub use radio::builder::RadioBuilder;
pub use reply::ReplySocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use reply::builder::ReplyBuilder;
pub use request::RequestSocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use request::builder::RequestBuilder;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use router::RouterNotify;
pub use router::RouterSocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use router::builder::RouterBuilder;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use scatter::ScatterSocket;
#[cfg(all(feature = "draft-api", feature = "builder"))]
#[doc(cfg(all(feature = "draft-api", feature = "builder")))]
pub use scatter::builder::ScatterBuilder;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use server::ServerSocket;
#[cfg(all(feature = "draft-api", feature = "builder"))]
#[doc(cfg(all(feature = "draft-api", feature = "builder")))]
pub use server::builder::ServerBuilder;
pub use stream::StreamSocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use stream::builder::StreamBuilder;
pub use subscribe::SubscribeSocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use subscribe::builder::SubscribeBuilder;
pub use xpublish::XPublishSocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use xpublish::builder::XPublishBuilder;
pub use xsubscribe::XSubscribeSocket;
#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use xsubscribe::builder::XSubscribeBuilder;

use crate::{auth::ZapDomain, security::SecurityMechanism};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Socket type
pub enum SocketType {
    /// [`PairSocket`]
    Pair,
    /// [`PublishSocket`]
    Publish,
    /// [`SubscribeSocket`]
    Subscribe,
    /// [`RequestSocket`]
    Request,
    /// [`ReplySocket`]
    Reply,
    /// [`DealerSocket`]
    Dealer,
    /// [`RouterSocket`]
    Router,
    /// [`PullSocket`]
    Pull,
    /// [`PushSocket`]
    Push,
    /// [`XPublishSocket`]
    XPublish,
    /// [`XSubscribeSocket`]
    XSubscribe,
    /// [`StreamSocket`]
    Stream,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// [`ServerSocket`]
    Server,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// [`ClientSocket`]
    Client,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// [`RadioSocket`]
    Radio,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// [`DishSocket`]
    Dish,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// [`GatherSocket`]
    Gather,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// [`ScatterSocket`]
    Scatter,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// DGRAM sockets
    Datagram,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// [`PeerSocket`]
    Peer,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// [`ChannelSocket`]
    Channel,
}

impl From<SocketType> for i32 {
    fn from(value: SocketType) -> Self {
        match value {
            SocketType::Pair => zmq_sys_crate::ZMQ_PAIR as i32,
            SocketType::Publish => zmq_sys_crate::ZMQ_PUB as i32,
            SocketType::Subscribe => zmq_sys_crate::ZMQ_SUB as i32,
            SocketType::Request => zmq_sys_crate::ZMQ_REQ as i32,
            SocketType::Reply => zmq_sys_crate::ZMQ_REP as i32,
            SocketType::Dealer => zmq_sys_crate::ZMQ_DEALER as i32,
            SocketType::Router => zmq_sys_crate::ZMQ_ROUTER as i32,
            SocketType::Pull => zmq_sys_crate::ZMQ_PULL as i32,
            SocketType::Push => zmq_sys_crate::ZMQ_PUSH as i32,
            SocketType::XPublish => zmq_sys_crate::ZMQ_XPUB as i32,
            SocketType::XSubscribe => zmq_sys_crate::ZMQ_XSUB as i32,
            SocketType::Stream => zmq_sys_crate::ZMQ_STREAM as i32,
            #[cfg(feature = "draft-api")]
            SocketType::Server => zmq_sys_crate::ZMQ_SERVER as i32,
            #[cfg(feature = "draft-api")]
            SocketType::Client => zmq_sys_crate::ZMQ_CLIENT as i32,
            #[cfg(feature = "draft-api")]
            SocketType::Radio => zmq_sys_crate::ZMQ_RADIO as i32,
            #[cfg(feature = "draft-api")]
            SocketType::Dish => zmq_sys_crate::ZMQ_DISH as i32,
            #[cfg(feature = "draft-api")]
            SocketType::Gather => zmq_sys_crate::ZMQ_GATHER as i32,
            #[cfg(feature = "draft-api")]
            SocketType::Scatter => zmq_sys_crate::ZMQ_SCATTER as i32,
            #[cfg(feature = "draft-api")]
            SocketType::Datagram => zmq_sys_crate::ZMQ_DGRAM as i32,
            #[cfg(feature = "draft-api")]
            SocketType::Peer => zmq_sys_crate::ZMQ_PEER as i32,
            #[cfg(feature = "draft-api")]
            SocketType::Channel => zmq_sys_crate::ZMQ_CHANNEL as i32,
        }
    }
}

#[cfg(test)]
mod socket_type_tests {
    use rstest::*;

    use super::SocketType;
    use crate::zmq_sys_crate;

    #[rstest]
    #[case(SocketType::Pair, zmq_sys_crate::ZMQ_PAIR as i32)]
    #[case(SocketType::Publish, zmq_sys_crate::ZMQ_PUB as i32)]
    #[case(SocketType::Subscribe, zmq_sys_crate::ZMQ_SUB as i32)]
    #[case(SocketType::Request, zmq_sys_crate::ZMQ_REQ as i32)]
    #[case(SocketType::Reply, zmq_sys_crate::ZMQ_REP as i32)]
    #[case(SocketType::Dealer, zmq_sys_crate::ZMQ_DEALER as i32)]
    #[case(SocketType::Router, zmq_sys_crate::ZMQ_ROUTER as i32)]
    #[case(SocketType::Pull, zmq_sys_crate::ZMQ_PULL as i32)]
    #[case(SocketType::Push, zmq_sys_crate::ZMQ_PUSH as i32)]
    #[case(SocketType::XPublish, zmq_sys_crate::ZMQ_XPUB as i32)]
    #[case(SocketType::XSubscribe, zmq_sys_crate::ZMQ_XSUB as i32)]
    #[case(SocketType::Stream, zmq_sys_crate::ZMQ_STREAM as i32)]
    #[cfg_attr(feature = "draft-api", case(SocketType::Server, zmq_sys_crate::ZMQ_SERVER as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketType::Client, zmq_sys_crate::ZMQ_CLIENT as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketType::Radio, zmq_sys_crate::ZMQ_RADIO as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketType::Dish, zmq_sys_crate::ZMQ_DISH as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketType::Gather, zmq_sys_crate::ZMQ_GATHER as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketType::Scatter, zmq_sys_crate::ZMQ_SCATTER as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketType::Datagram, zmq_sys_crate::ZMQ_DGRAM as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketType::Peer, zmq_sys_crate::ZMQ_PEER as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketType::Channel, zmq_sys_crate::ZMQ_CHANNEL as i32))]
    fn converts_to_raw(#[case] socket_type: SocketType, #[case] raw: i32) {
        assert_eq!(<SocketType as Into<i32>>::into(socket_type), raw);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
/// Options that can be set or retrieved on a 0MQ socket
pub enum SocketOption {
    /// I/O thread affinity
    Affinity,
    /// Socket routing id
    RoutingId,
    /// Establish message filter
    Subscribe,
    /// Remove message filter
    Unsubscribe,
    /// Multicast data rate
    Rate,
    /// Multicast recovery interval
    RecoveryInterval,
    /// Kernel transmit buffer size
    SendBuffer,
    /// Kernel receive buffer size
    ReceiveBuffer,
    /// more message data to follow
    ReceiveMore,
    /// File descriptor associated with the socket
    FileDescriptor,
    /// Socket event state
    Events,
    /// Socket type
    Type,
    /// Linger period for socket shutdown
    Linger,
    /// Reconnection interval
    ReconnectInterval,
    /// Maximum length of the queue of outstanding connections
    Backlog,
    /// Maximum reconnection interval
    ReconnectIntervalMax,
    /// Maximum acceptable inbound message size
    MaxMessageSize,
    /// High water mark for outbound messages
    SendHighWatermark,
    /// High water mark for inbound messages
    ReceiveHighWatermark,
    /// Maximum network hops for multicast packets
    MulticastHops,
    /// Maximum time before a socket operation returns with [`Again`](crate::ZmqError::Again)
    ReceiveTimeout,
    /// Maximum time before a socket operation returns with [`Again`](crate::ZmqError::Again)
    SendTimeout,
    /// Last endpoint set
    LastEndpoint,
    /// Accept only routable messages on [`Rocker`](RouterSocket) sockets
    RouterMandatory,
    /// Overrides SO_KEEPALIVE socket option
    TcpKeepalive,
    /// Override TCP_KEEPCNT sockt option
    TcpKeepaliveCount,
    /// Override TCP_KEEPIDLE sockt option
    TcpKeepaliveIdle,
    /// Override TCP_KEEPINTVL sockt option
    TcpKeepaliveInterval,
    /// Assign filters to allow new TCP connections
    TcpAcceptFilter,
    /// Queue messages only to completed connections
    Immediate,
    /// Pass duplicate subscribe messages on [`XPublish`](XPublishSocket) sockets
    XpubVerbose,
    /// IPv6 setting
    IPv6,
    /// Current security mechanism
    Mechanism,
    /// Current PLAIN server role
    PlainServer,
    /// Current PLAIN username
    PlainUsername,
    /// Current PLAIN password
    PlainPassword,
    #[cfg(feature = "curve")]
    #[doc(cfg(all(feature = "curve", not(windows))))]
    /// Current CURVE public key
    CurvePublicKey,
    #[cfg(feature = "curve")]
    #[doc(cfg(all(feature = "curve", not(windows))))]
    /// Current CURVE secret key
    CurveSecretKey,
    #[cfg(feature = "curve")]
    #[doc(cfg(all(feature = "curve", not(windows))))]
    /// Current CURVE server role
    CurveServer,
    #[cfg(feature = "curve")]
    #[doc(cfg(all(feature = "curve", not(windows))))]
    /// Current CURVE server key
    CurveServerKey,
    /// Bootstrap connections to [`Router`](RouterSocket) sockets
    ProbeRouter,
    /// Match replies with requests on [`Request`](RequestSocket) sockets
    RequestCorrelate,
    /// Relax strict alternation between reques and reply
    RequestRelaxed,
    /// Keep only last message
    Conflate,
    /// RFC27 authentifcation domain
    ZapDomain,
    /// Handle duplicate client routing ids on [`Router`](RouterSocket) sockets
    RouterHandover,
    /// Type-of-service on the underlying socket
    TypeOfService,
    #[doc(cfg(zmq_have_ipc))]
    /// Process ID filters to allow new IPC connections
    IpcFilterProcessId,
    #[doc(cfg(zmq_have_ipc))]
    /// User ID filters to allow new IPC connections
    IpcFilterUserId,
    #[doc(cfg(zmq_have_ipc))]
    /// Group ID filters to allow new IPC connections
    IpcFilterGroupId,
    /// Next outbound routing id
    ConnectRoutingId,
    /// Maximum handshake interval
    HandshakeInterval,
    /// SOCKS5 proxy address
    SocksProxy,
    /// Do not silently drop message if SendHighWatermark is reached on [`XPublish`](XPublishSocket)
    /// sockets
    XpubNoDrop,
    /// Change the subscription handling to manual
    XpubManual,
    /// Welcome message that will be received by [`Subscribe`](SubscribeSocket) when connecting to a
    /// [`XPublish`](XPublishSocket) socket.
    XpubWelcomeMessage,
    /// Send connect and disconnect notifications
    StreamNotify,
    /// Invert message filtering
    InvertMatching,
    /// Interval between sending ZMTP heartbeats
    HeartbeatInterval,
    /// Time-to-live for ZMTP heartbeats
    HeartbeatTimeToLive,
    /// Timeout for ZMTP heartbeats
    HeartbeatTimeout,
    /// Pass duplicate subscribe and unsubscribe message on [`XPublish`](XPublishSocket) sockets
    XpubVerboser,
    /// Connect timeout
    ConnectTimeout,
    /// TCP maximum retransmit timeout
    MaxTcpRetransmitTimeout,
    /// Retrieve socket thread-safety
    ThreadSafe,
    /// Maximum transport data unit size for multicast packets
    MulticastMaxTransportDataUnitSize,
    #[doc(cfg(zmq_have_vmci))]
    /// Buffer size of the VMCI socket
    VmciBufferSize,
    #[doc(cfg(zmq_have_vmci))]
    /// Minimum buffer size of the VMCI socket
    VmciBufferMinSize,
    #[doc(cfg(zmq_have_vmci))]
    /// Maximum buffer size of the VMCI socket
    VmciBufferMaxSize,
    #[doc(cfg(zmq_have_vmci))]
    /// Connection timeout of the VMCI socket
    VmciConntectTimeout,
    /// Retrive the pre-allocated socket file descriptor
    UseFd,
    /// Name of the devive to bind the socket to
    BindToDevice,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Strict ZAP domain handling
    ZapEnforceDomain,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Application metadata properties on the socket
    Metadata,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Control multicast local loopback
    MulticastLoop,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Send connect and disconnect notifications
    RouterNotify,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Change the subscription handling to manual
    XpubManualLastValue,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// SOCKS username and select basic authentification
    SocksUsername,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// SOCKS basic authentification password
    SocksPassword,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Maximum receive batch size
    InBatchSize,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Maximum send batch size
    OutBatchSize,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Process only first subscribe/unsubscribe in a multipart message
    OnlyFirstSubscribe,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Set condition when reconnection will stop
    ReconnectStop,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Set a hello message that will be sent when a new peer connects
    HelloMessage,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Set a disconnect message that the socket will generate when accepted peer disconnect
    DisconnectMessage,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Set the priority on the socket
    Priority,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// This removes delays caused by the interrupt and the resultant context switch
    BusyPoll,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Set a hiccup message that the socket will generate when connected peer temprarily
    /// disconnects
    HiccupMessage,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Pass duplicate unsubscribe messages on [`XSubscribe`](XSubscribeSocket) sockets
    XsubVerboseUnsubscribe,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    /// Number of topic subscriptions received
    TopicsCount,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    /// NORM sender mode
    NormMode,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    /// NORM unicast NACK mode
    NormUnicastNack,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    /// NORM buffer size
    NormBufferSize,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    /// NORM segment size
    NormSegmentSize,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    /// NORM block size
    NormBlockSize,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    /// NORM parity segment setting
    NormNumnParity,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    /// Proactive NORM parity segment setting
    NormNumnAutoParity,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    /// NORM push mode
    NormPush,
}

impl From<SocketOption> for i32 {
    fn from(value: SocketOption) -> Self {
        match value {
            SocketOption::Affinity => zmq_sys_crate::ZMQ_AFFINITY as i32,
            SocketOption::RoutingId => zmq_sys_crate::ZMQ_ROUTING_ID as i32,
            SocketOption::Subscribe => zmq_sys_crate::ZMQ_SUBSCRIBE as i32,
            SocketOption::Unsubscribe => zmq_sys_crate::ZMQ_UNSUBSCRIBE as i32,
            SocketOption::Rate => zmq_sys_crate::ZMQ_RATE as i32,
            SocketOption::RecoveryInterval => zmq_sys_crate::ZMQ_RECOVERY_IVL as i32,
            SocketOption::SendBuffer => zmq_sys_crate::ZMQ_SNDBUF as i32,
            SocketOption::ReceiveBuffer => zmq_sys_crate::ZMQ_RCVBUF as i32,
            SocketOption::ReceiveMore => zmq_sys_crate::ZMQ_RCVMORE as i32,
            SocketOption::FileDescriptor => zmq_sys_crate::ZMQ_FD as i32,
            SocketOption::Events => zmq_sys_crate::ZMQ_EVENTS as i32,
            SocketOption::Type => zmq_sys_crate::ZMQ_TYPE as i32,
            SocketOption::Linger => zmq_sys_crate::ZMQ_LINGER as i32,
            SocketOption::ReconnectInterval => zmq_sys_crate::ZMQ_RECONNECT_IVL as i32,
            SocketOption::Backlog => zmq_sys_crate::ZMQ_BACKLOG as i32,
            SocketOption::ReconnectIntervalMax => zmq_sys_crate::ZMQ_RECONNECT_IVL_MAX as i32,
            SocketOption::MaxMessageSize => zmq_sys_crate::ZMQ_MAXMSGSIZE as i32,
            SocketOption::SendHighWatermark => zmq_sys_crate::ZMQ_SNDHWM as i32,
            SocketOption::ReceiveHighWatermark => zmq_sys_crate::ZMQ_RCVHWM as i32,
            SocketOption::MulticastHops => zmq_sys_crate::ZMQ_MULTICAST_HOPS as i32,
            SocketOption::ReceiveTimeout => zmq_sys_crate::ZMQ_RCVTIMEO as i32,
            SocketOption::SendTimeout => zmq_sys_crate::ZMQ_SNDTIMEO as i32,
            SocketOption::LastEndpoint => zmq_sys_crate::ZMQ_LAST_ENDPOINT as i32,
            SocketOption::RouterMandatory => zmq_sys_crate::ZMQ_ROUTER_MANDATORY as i32,
            SocketOption::TcpKeepalive => zmq_sys_crate::ZMQ_TCP_KEEPALIVE as i32,
            SocketOption::TcpKeepaliveCount => zmq_sys_crate::ZMQ_TCP_KEEPALIVE_CNT as i32,
            SocketOption::TcpKeepaliveIdle => zmq_sys_crate::ZMQ_TCP_KEEPALIVE_IDLE as i32,
            SocketOption::TcpKeepaliveInterval => zmq_sys_crate::ZMQ_TCP_KEEPALIVE_INTVL as i32,
            SocketOption::TcpAcceptFilter => zmq_sys_crate::ZMQ_TCP_ACCEPT_FILTER as i32,
            SocketOption::Immediate => zmq_sys_crate::ZMQ_IMMEDIATE as i32,
            SocketOption::XpubVerbose => zmq_sys_crate::ZMQ_XPUB_VERBOSE as i32,
            SocketOption::IPv6 => zmq_sys_crate::ZMQ_IPV6 as i32,
            SocketOption::Mechanism => zmq_sys_crate::ZMQ_MECHANISM as i32,
            SocketOption::PlainServer => zmq_sys_crate::ZMQ_PLAIN_SERVER as i32,
            SocketOption::PlainUsername => zmq_sys_crate::ZMQ_PLAIN_USERNAME as i32,
            SocketOption::PlainPassword => zmq_sys_crate::ZMQ_PLAIN_PASSWORD as i32,
            #[cfg(feature = "curve")]
            SocketOption::CurvePublicKey => zmq_sys_crate::ZMQ_CURVE_PUBLICKEY as i32,
            #[cfg(feature = "curve")]
            SocketOption::CurveSecretKey => zmq_sys_crate::ZMQ_CURVE_SECRETKEY as i32,
            #[cfg(feature = "curve")]
            SocketOption::CurveServer => zmq_sys_crate::ZMQ_CURVE_SERVER as i32,
            #[cfg(feature = "curve")]
            SocketOption::CurveServerKey => zmq_sys_crate::ZMQ_CURVE_SERVERKEY as i32,
            SocketOption::ProbeRouter => zmq_sys_crate::ZMQ_PROBE_ROUTER as i32,
            SocketOption::RequestCorrelate => zmq_sys_crate::ZMQ_REQ_CORRELATE as i32,
            SocketOption::RequestRelaxed => zmq_sys_crate::ZMQ_REQ_RELAXED as i32,
            SocketOption::Conflate => zmq_sys_crate::ZMQ_CONFLATE as i32,
            SocketOption::ZapDomain => zmq_sys_crate::ZMQ_ZAP_DOMAIN as i32,
            SocketOption::RouterHandover => zmq_sys_crate::ZMQ_ROUTER_HANDOVER as i32,
            SocketOption::TypeOfService => zmq_sys_crate::ZMQ_TOS as i32,
            SocketOption::IpcFilterProcessId => zmq_sys_crate::ZMQ_IPC_FILTER_PID as i32,
            SocketOption::IpcFilterUserId => zmq_sys_crate::ZMQ_IPC_FILTER_UID as i32,
            SocketOption::IpcFilterGroupId => zmq_sys_crate::ZMQ_IPC_FILTER_GID as i32,
            SocketOption::ConnectRoutingId => zmq_sys_crate::ZMQ_CONNECT_ROUTING_ID as i32,
            SocketOption::HandshakeInterval => zmq_sys_crate::ZMQ_HANDSHAKE_IVL as i32,
            SocketOption::SocksProxy => zmq_sys_crate::ZMQ_SOCKS_PROXY as i32,
            SocketOption::XpubNoDrop => zmq_sys_crate::ZMQ_XPUB_NODROP as i32,
            SocketOption::XpubManual => zmq_sys_crate::ZMQ_XPUB_MANUAL as i32,
            SocketOption::XpubWelcomeMessage => zmq_sys_crate::ZMQ_XPUB_WELCOME_MSG as i32,
            SocketOption::StreamNotify => zmq_sys_crate::ZMQ_STREAM_NOTIFY as i32,
            SocketOption::InvertMatching => zmq_sys_crate::ZMQ_INVERT_MATCHING as i32,
            SocketOption::HeartbeatInterval => zmq_sys_crate::ZMQ_HEARTBEAT_IVL as i32,
            SocketOption::HeartbeatTimeToLive => zmq_sys_crate::ZMQ_HEARTBEAT_TTL as i32,
            SocketOption::HeartbeatTimeout => zmq_sys_crate::ZMQ_HEARTBEAT_TIMEOUT as i32,
            SocketOption::XpubVerboser => zmq_sys_crate::ZMQ_XPUB_VERBOSER as i32,
            SocketOption::ConnectTimeout => zmq_sys_crate::ZMQ_CONNECT_TIMEOUT as i32,
            SocketOption::MaxTcpRetransmitTimeout => zmq_sys_crate::ZMQ_TCP_MAXRT as i32,
            SocketOption::MulticastMaxTransportDataUnitSize => {
                zmq_sys_crate::ZMQ_MULTICAST_MAXTPDU as i32
            }
            SocketOption::ThreadSafe => zmq_sys_crate::ZMQ_THREAD_SAFE as i32,
            SocketOption::VmciBufferSize => zmq_sys_crate::ZMQ_VMCI_BUFFER_SIZE as i32,
            SocketOption::VmciBufferMinSize => zmq_sys_crate::ZMQ_VMCI_BUFFER_MIN_SIZE as i32,
            SocketOption::VmciBufferMaxSize => zmq_sys_crate::ZMQ_VMCI_BUFFER_MAX_SIZE as i32,
            SocketOption::VmciConntectTimeout => zmq_sys_crate::ZMQ_VMCI_CONNECT_TIMEOUT as i32,
            SocketOption::UseFd => zmq_sys_crate::ZMQ_USE_FD as i32,
            SocketOption::BindToDevice => zmq_sys_crate::ZMQ_BINDTODEVICE as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::ZapEnforceDomain => zmq_sys_crate::ZMQ_ZAP_ENFORCE_DOMAIN as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::Metadata => zmq_sys_crate::ZMQ_METADATA as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::MulticastLoop => zmq_sys_crate::ZMQ_MULTICAST_LOOP as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::RouterNotify => zmq_sys_crate::ZMQ_ROUTER_NOTIFY as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::XpubManualLastValue => zmq_sys_crate::ZMQ_XPUB_MANUAL_LAST_VALUE as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::SocksUsername => zmq_sys_crate::ZMQ_SOCKS_USERNAME as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::SocksPassword => zmq_sys_crate::ZMQ_SOCKS_PASSWORD as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::InBatchSize => zmq_sys_crate::ZMQ_IN_BATCH_SIZE as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::OutBatchSize => zmq_sys_crate::ZMQ_OUT_BATCH_SIZE as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::OnlyFirstSubscribe => zmq_sys_crate::ZMQ_ONLY_FIRST_SUBSCRIBE as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::ReconnectStop => zmq_sys_crate::ZMQ_RECONNECT_STOP as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::HelloMessage => zmq_sys_crate::ZMQ_HELLO_MSG as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::DisconnectMessage => zmq_sys_crate::ZMQ_DISCONNECT_MSG as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::Priority => zmq_sys_crate::ZMQ_PRIORITY as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::BusyPoll => zmq_sys_crate::ZMQ_BUSY_POLL as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::HiccupMessage => zmq_sys_crate::ZMQ_HICCUP_MSG as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::XsubVerboseUnsubscribe => {
                zmq_sys_crate::ZMQ_XSUB_VERBOSE_UNSUBSCRIBE as i32
            }
            #[cfg(feature = "draft-api")]
            SocketOption::TopicsCount => zmq_sys_crate::ZMQ_TOPICS_COUNT as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::NormMode => zmq_sys_crate::ZMQ_NORM_MODE as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::NormUnicastNack => zmq_sys_crate::ZMQ_NORM_UNICAST_NACK as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::NormBufferSize => zmq_sys_crate::ZMQ_NORM_BUFFER_SIZE as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::NormSegmentSize => zmq_sys_crate::ZMQ_NORM_SEGMENT_SIZE as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::NormBlockSize => zmq_sys_crate::ZMQ_NORM_BLOCK_SIZE as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::NormNumnParity => zmq_sys_crate::ZMQ_NORM_NUM_PARITY as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::NormNumnAutoParity => zmq_sys_crate::ZMQ_NORM_NUM_AUTOPARITY as i32,
            #[cfg(feature = "draft-api")]
            SocketOption::NormPush => zmq_sys_crate::ZMQ_NORM_PUSH as i32,
        }
    }
}

#[cfg(test)]
mod socket_option_tests {
    use rstest::*;

    use super::SocketOption;
    use crate::zmq_sys_crate;

    #[rstest]
    #[case(SocketOption::Affinity, zmq_sys_crate::ZMQ_AFFINITY as i32)]
    #[case(SocketOption::RoutingId, zmq_sys_crate::ZMQ_ROUTING_ID as i32)]
    #[case(SocketOption::Subscribe, zmq_sys_crate::ZMQ_SUBSCRIBE as i32)]
    #[case(SocketOption::Unsubscribe, zmq_sys_crate::ZMQ_UNSUBSCRIBE as i32)]
    #[case(SocketOption::Rate, zmq_sys_crate::ZMQ_RATE as i32)]
    #[case(SocketOption::RecoveryInterval, zmq_sys_crate::ZMQ_RECOVERY_IVL as i32)]
    #[case(SocketOption::SendBuffer, zmq_sys_crate::ZMQ_SNDBUF as i32)]
    #[case(SocketOption::ReceiveBuffer, zmq_sys_crate::ZMQ_RCVBUF as i32)]
    #[case(SocketOption::ReceiveMore, zmq_sys_crate::ZMQ_RCVMORE as i32)]
    #[case(SocketOption::FileDescriptor, zmq_sys_crate::ZMQ_FD as i32)]
    #[case(SocketOption::Events, zmq_sys_crate::ZMQ_EVENTS as i32)]
    #[case(SocketOption::Type, zmq_sys_crate::ZMQ_TYPE as i32)]
    #[case(SocketOption::Linger, zmq_sys_crate::ZMQ_LINGER as i32)]
    #[case(SocketOption::ReconnectInterval, zmq_sys_crate::ZMQ_RECONNECT_IVL as i32)]
    #[case(SocketOption::Backlog, zmq_sys_crate::ZMQ_BACKLOG as i32)]
    #[case(SocketOption::ReconnectIntervalMax, zmq_sys_crate::ZMQ_RECONNECT_IVL_MAX as i32)]
    #[case(SocketOption::MaxMessageSize, zmq_sys_crate::ZMQ_MAXMSGSIZE as i32)]
    #[case(SocketOption::SendHighWatermark, zmq_sys_crate::ZMQ_SNDHWM as i32)]
    #[case(SocketOption::ReceiveHighWatermark, zmq_sys_crate::ZMQ_RCVHWM as i32)]
    #[case(SocketOption::MulticastHops, zmq_sys_crate::ZMQ_MULTICAST_HOPS as i32)]
    #[case(SocketOption::ReceiveTimeout, zmq_sys_crate::ZMQ_RCVTIMEO as i32)]
    #[case(SocketOption::SendTimeout, zmq_sys_crate::ZMQ_SNDTIMEO as i32)]
    #[case(SocketOption::LastEndpoint, zmq_sys_crate::ZMQ_LAST_ENDPOINT as i32)]
    #[case(SocketOption::RouterMandatory, zmq_sys_crate::ZMQ_ROUTER_MANDATORY as i32)]
    #[case(SocketOption::TcpKeepalive, zmq_sys_crate::ZMQ_TCP_KEEPALIVE as i32)]
    #[case(SocketOption::TcpKeepaliveCount, zmq_sys_crate::ZMQ_TCP_KEEPALIVE_CNT as i32)]
    #[case(SocketOption::TcpKeepaliveIdle, zmq_sys_crate::ZMQ_TCP_KEEPALIVE_IDLE as i32)]
    #[case(SocketOption::TcpKeepaliveInterval, zmq_sys_crate::ZMQ_TCP_KEEPALIVE_INTVL as i32)]
    #[case(SocketOption::TcpAcceptFilter, zmq_sys_crate::ZMQ_TCP_ACCEPT_FILTER as i32)]
    #[case(SocketOption::Immediate, zmq_sys_crate::ZMQ_IMMEDIATE as i32)]
    #[case(SocketOption::XpubVerbose, zmq_sys_crate::ZMQ_XPUB_VERBOSE as i32)]
    #[case(SocketOption::IPv6, zmq_sys_crate::ZMQ_IPV6 as i32)]
    #[case(SocketOption::Mechanism, zmq_sys_crate::ZMQ_MECHANISM as i32)]
    #[case(SocketOption::PlainServer, zmq_sys_crate::ZMQ_PLAIN_SERVER as i32)]
    #[case(SocketOption::PlainUsername, zmq_sys_crate::ZMQ_PLAIN_USERNAME as i32)]
    #[case(SocketOption::PlainPassword, zmq_sys_crate::ZMQ_PLAIN_PASSWORD as i32)]
    #[cfg_attr(feature = "curve", case(SocketOption::CurvePublicKey, zmq_sys_crate::ZMQ_CURVE_PUBLICKEY as i32))]
    #[cfg_attr(feature = "curve", case(SocketOption::CurveSecretKey, zmq_sys_crate::ZMQ_CURVE_SECRETKEY as i32))]
    #[cfg_attr(feature = "curve", case(SocketOption::CurveServer, zmq_sys_crate::ZMQ_CURVE_SERVER as i32))]
    #[cfg_attr(feature = "curve", case(SocketOption::CurveServerKey, zmq_sys_crate::ZMQ_CURVE_SERVERKEY as i32))]
    #[case(SocketOption::ProbeRouter, zmq_sys_crate::ZMQ_PROBE_ROUTER as i32)]
    #[case(SocketOption::RequestCorrelate, zmq_sys_crate::ZMQ_REQ_CORRELATE as i32)]
    #[case(SocketOption::RequestRelaxed, zmq_sys_crate::ZMQ_REQ_RELAXED as i32)]
    #[case(SocketOption::Conflate, zmq_sys_crate::ZMQ_CONFLATE as i32)]
    #[case(SocketOption::ZapDomain, zmq_sys_crate::ZMQ_ZAP_DOMAIN as i32)]
    #[case(SocketOption::RouterHandover, zmq_sys_crate::ZMQ_ROUTER_HANDOVER as i32)]
    #[case(SocketOption::TypeOfService, zmq_sys_crate::ZMQ_TOS as i32)]
    #[case(SocketOption::IpcFilterProcessId, zmq_sys_crate::ZMQ_IPC_FILTER_PID as i32)]
    #[case(SocketOption::IpcFilterUserId, zmq_sys_crate::ZMQ_IPC_FILTER_UID as i32)]
    #[case(SocketOption::IpcFilterGroupId, zmq_sys_crate::ZMQ_IPC_FILTER_GID as i32)]
    #[case(SocketOption::ConnectRoutingId, zmq_sys_crate::ZMQ_CONNECT_ROUTING_ID as i32)]
    #[case(SocketOption::HandshakeInterval, zmq_sys_crate::ZMQ_HANDSHAKE_IVL as i32)]
    #[case(SocketOption::SocksProxy, zmq_sys_crate::ZMQ_SOCKS_PROXY as i32)]
    #[case(SocketOption::XpubNoDrop, zmq_sys_crate::ZMQ_XPUB_NODROP as i32)]
    #[case(SocketOption::XpubManual, zmq_sys_crate::ZMQ_XPUB_MANUAL as i32)]
    #[case(SocketOption::XpubWelcomeMessage, zmq_sys_crate::ZMQ_XPUB_WELCOME_MSG as i32)]
    #[case(SocketOption::StreamNotify, zmq_sys_crate::ZMQ_STREAM_NOTIFY as i32)]
    #[case(SocketOption::InvertMatching, zmq_sys_crate::ZMQ_INVERT_MATCHING as i32)]
    #[case(SocketOption::HeartbeatInterval, zmq_sys_crate::ZMQ_HEARTBEAT_IVL as i32)]
    #[case(SocketOption::HeartbeatTimeToLive, zmq_sys_crate::ZMQ_HEARTBEAT_TTL as i32)]
    #[case(SocketOption::HeartbeatTimeout, zmq_sys_crate::ZMQ_HEARTBEAT_TIMEOUT as i32)]
    #[case(SocketOption::XpubVerboser, zmq_sys_crate::ZMQ_XPUB_VERBOSER as i32)]
    #[case(SocketOption::ConnectTimeout, zmq_sys_crate::ZMQ_CONNECT_TIMEOUT as i32)]
    #[case(SocketOption::MaxTcpRetransmitTimeout, zmq_sys_crate::ZMQ_TCP_MAXRT as i32)]
    #[case(SocketOption::MulticastMaxTransportDataUnitSize, zmq_sys_crate::ZMQ_MULTICAST_MAXTPDU as i32)]
    #[case(SocketOption::ThreadSafe, zmq_sys_crate::ZMQ_THREAD_SAFE as i32)]
    #[case(SocketOption::VmciBufferSize, zmq_sys_crate::ZMQ_VMCI_BUFFER_SIZE as i32)]
    #[case(SocketOption::VmciBufferMinSize, zmq_sys_crate::ZMQ_VMCI_BUFFER_MIN_SIZE as i32)]
    #[case(SocketOption::VmciBufferMaxSize, zmq_sys_crate::ZMQ_VMCI_BUFFER_MAX_SIZE as i32)]
    #[case(SocketOption::VmciConntectTimeout, zmq_sys_crate::ZMQ_VMCI_CONNECT_TIMEOUT as i32)]
    #[case(SocketOption::UseFd, zmq_sys_crate::ZMQ_USE_FD as i32)]
    #[case(SocketOption::BindToDevice, zmq_sys_crate::ZMQ_BINDTODEVICE as i32)]
    #[cfg_attr(feature = "draft-api", case(SocketOption::ZapEnforceDomain, zmq_sys_crate::ZMQ_ZAP_ENFORCE_DOMAIN as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::Metadata, zmq_sys_crate::ZMQ_METADATA as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::MulticastLoop, zmq_sys_crate::ZMQ_MULTICAST_LOOP as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::RouterNotify, zmq_sys_crate::ZMQ_ROUTER_NOTIFY as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::XpubManualLastValue, zmq_sys_crate::ZMQ_XPUB_MANUAL_LAST_VALUE as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::SocksUsername, zmq_sys_crate::ZMQ_SOCKS_USERNAME as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::SocksPassword, zmq_sys_crate::ZMQ_SOCKS_PASSWORD as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::InBatchSize, zmq_sys_crate::ZMQ_IN_BATCH_SIZE as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::OutBatchSize, zmq_sys_crate::ZMQ_OUT_BATCH_SIZE as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::OnlyFirstSubscribe, zmq_sys_crate::ZMQ_ONLY_FIRST_SUBSCRIBE as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::ReconnectStop, zmq_sys_crate::ZMQ_RECONNECT_STOP as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::HelloMessage, zmq_sys_crate::ZMQ_HELLO_MSG as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::DisconnectMessage, zmq_sys_crate::ZMQ_DISCONNECT_MSG as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::Priority, zmq_sys_crate::ZMQ_PRIORITY as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::BusyPoll, zmq_sys_crate::ZMQ_BUSY_POLL as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::HiccupMessage, zmq_sys_crate::ZMQ_HICCUP_MSG as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::XsubVerboseUnsubscribe, zmq_sys_crate::ZMQ_XSUB_VERBOSE_UNSUBSCRIBE as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::TopicsCount, zmq_sys_crate::ZMQ_TOPICS_COUNT as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::NormMode, zmq_sys_crate::ZMQ_NORM_MODE as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::NormUnicastNack, zmq_sys_crate::ZMQ_NORM_UNICAST_NACK as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::NormBufferSize, zmq_sys_crate::ZMQ_NORM_BUFFER_SIZE as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::NormSegmentSize, zmq_sys_crate::ZMQ_NORM_SEGMENT_SIZE as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::NormBlockSize, zmq_sys_crate::ZMQ_NORM_BLOCK_SIZE as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::NormNumnParity, zmq_sys_crate::ZMQ_NORM_NUM_PARITY as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::NormNumnAutoParity, zmq_sys_crate::ZMQ_NORM_NUM_AUTOPARITY as i32))]
    #[cfg_attr(feature = "draft-api", case(SocketOption::NormPush, zmq_sys_crate::ZMQ_NORM_PUSH as i32))]
    fn converts_to_raw(#[case] option: SocketOption, #[case] expected: i32) {
        assert_eq!(<SocketOption as Into<i32>>::into(option), expected);
    }
}

/// generic 0MQ socket
pub struct Socket<T: sealed::SocketType> {
    context: Context,
    pub(crate) socket: Arc<RawSocket>,
    marker: PhantomData<T>,
}

impl<T: sealed::SocketType> Socket<T> {
    /// General constructor
    pub fn from_context(context: &Context) -> ZmqResult<Self> {
        let socket = RawSocket::from_ctx(&context.inner, T::raw_socket_type() as i32)?;
        Ok(Self {
            context: context.clone(),
            socket: socket.into(),
            marker: PhantomData,
        })
    }

    /// # set 0MQ socket options
    ///
    /// Sets a [`SocketOption`] option on the socket. The bytes version is mostly suitable for
    /// binary data options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn set_sockopt_bytes<V>(&self, option: SocketOption, value: V) -> ZmqResult<()>
    where
        V: AsRef<[u8]>,
    {
        self.socket.set_sockopt_bytes(option.into(), value.as_ref())
    }

    /// # set 0MQ socket options
    ///
    /// Sets a [`SocketOption`] option on the socket. The string version is mostly suitable for
    /// character string options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn set_sockopt_string<V>(&self, option: SocketOption, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.socket
            .set_sockopt_string(option.into(), value.as_ref())
    }

    /// # set 0MQ socket options
    ///
    /// Sets a [`SocketOption`] option on the socket. The int version is mostly suitable for
    /// integer options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn set_sockopt_int<V>(&self, option: SocketOption, value: V) -> ZmqResult<()>
    where
        V: PrimInt,
    {
        self.socket.set_sockopt_int(option.into(), value)
    }

    /// # set 0MQ socket options
    ///
    /// Sets a [`SocketOption`] option on the socket. The bool version is mostly suitable for
    /// 0/1 integer options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn set_sockopt_bool(&self, option: SocketOption, value: bool) -> ZmqResult<()> {
        self.socket.set_sockopt_bool(option.into(), value)
    }

    #[cfg(all(feature = "curve", not(windows)))]
    pub(crate) fn get_sockopt_curve(&self, option: SocketOption) -> ZmqResult<Vec<u8>> {
        self.socket.get_sockopt_curve(option.into())
    }

    /// # get 0MQ socket options
    ///
    /// Gets a [`SocketOption`] option on the socket. The bytes version is mostly suitable for
    /// binary data options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn get_sockopt_bytes(&self, option: SocketOption) -> ZmqResult<Vec<u8>> {
        self.socket.get_sockopt_bytes(option.into())
    }

    /// # get 0MQ socket options
    ///
    /// Gets a [`SocketOption`] option on the socket. The string version is mostly suitable for
    /// character string options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn get_sockopt_string(&self, option: SocketOption) -> ZmqResult<String> {
        self.socket.get_sockopt_string(option.into())
    }

    /// # get 0MQ socket options
    ///
    /// Gets a [`SocketOption`] option on the socket. The int version is mostly suitable for
    /// integer options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn get_sockopt_int<V>(&self, option: SocketOption) -> ZmqResult<V>
    where
        V: PrimInt + Default,
    {
        self.socket.get_sockopt_int(option.into())
    }

    /// # get 0MQ socket options
    ///
    /// Gets a [`SocketOption`] option on the socket. The bool version is mostly suitable for
    /// 0/1 integer options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn get_sockopt_bool(&self, option: SocketOption) -> ZmqResult<bool> {
        self.socket.get_sockopt_bool(option.into())
    }

    /// # Set I/O thread affinity `ZMQ_AFFINITY`
    ///
    /// The [`Affinity`] option shall set the I/O thread affinity for newly created connections on
    /// the specified [`Socket`].
    ///
    /// Affinity determines which threads from the 0MQ I/O thread pool associated with the
    /// socket’s context shall handle newly created connections. A value of zero specifies no
    /// affinity, meaning that work shall be distributed fairly among all 0MQ I/O threads in the
    /// thread pool. For non-zero values, the lowest bit corresponds to thread 1, second lowest bit
    /// to thread 2 and so on. For example, a value of 3 specifies that subsequent connections on
    /// [`Socket`] shall be handled exclusively by I/O threads 1 and 2.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 0             | N/A                     |
    ///
    /// [`Socket`]: Socket
    /// [`Affinity`]: SocketOption::Affinity
    pub fn set_affinity(&self, value: u64) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::Affinity, value)
    }

    /// # Retrieve I/O thread affinity `ZMQ_AFFINITY`
    ///
    /// The [`Affinity`] option shall retrieve the I/O thread affinity for newly
    /// created connections on the specified [`Socket`].
    ///
    /// Affinity determines which threads from the 0MQ I/O thread pool associated with the
    /// socket’s context shall handle newly created connections. A value of zero specifies no
    /// affinity, meaning that work shall be distributed fairly among all 0MQ I/O threads in the
    /// thread pool. For non-zero values, the lowest bit corresponds to thread 1, second lowest bit
    /// to thread 2 and so on. For example, a value of 3 specifies that subsequent connections on
    /// [`Socket`] shall be handled exclusively by I/O threads 1 and 2.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 0             | N/A                     |
    ///
    /// [`Socket`]: Socket
    /// [`Affinity`]: SocketOption::Affinity
    pub fn affinity(&self) -> ZmqResult<u64> {
        self.get_sockopt_int(SocketOption::Affinity)
    }

    /// # Set maximum length of the queue of outstanding connections `ZMQ_BACKLOG`
    ///
    /// The [`Backlog`] option shall set the maximum length of the queue of outstanding peer
    /// connections for the specified [`Socket`]; this only applies to connection-oriented
    /// transports. For details refer to your operating system documentation for the `listen`
    /// function.
    ///
    /// | Default value           | Applicable socket types                       |
    /// | :---------------------: | :-------------------------------------------: |
    /// | 100                     | all, only for connection-oriented transports. |
    ///
    /// [`Socket`]: Socket
    /// [`Backlog`]: SocketOption::Backlog
    pub fn set_backlog(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::Backlog, value)
    }

    /// # Retrieve maximum length of the queue of outstanding connections `ZMQ_BACKLOG`
    ///
    /// The [`Backlog`] option shall retrieve the maximum length of the queue of outstanding peer
    /// connections for the specified [`Socket`]; this only applies to connection-oriented
    /// transports. For details refer to your operating system documentation for the `listen`
    /// function.
    ///
    /// | Default value           | Applicable socket types                       |
    /// | :---------------------: | :-------------------------------------------: |
    /// | 100                     | all, only for connection-oriented transports. |
    ///
    /// [`Socket`]: Socket
    /// [`Backlog`]: SocketOption::Backlog
    pub fn backlog(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::Backlog)
    }

    /// # This removes delays caused by the interrupt and the resultant context switch. `ZMQ_BUSY_POLL`
    ///
    /// Busy polling helps reduce latency in the network receive path by allowing socket layer code
    /// to poll the receive queue of a network device, and disabling network interrupts. This
    /// removes delays caused by the interrupt and the resultant context switch. However, it also
    /// increases CPU utilization. Busy polling also prevents the CPU from sleeping, which can
    /// incur additional power consumption.
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_busy_poll(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::BusyPoll, value)
    }

    /// # Set connect() timeout `ZMQ_CONNECT_TIMEOUT`
    ///
    /// Sets how long to wait before timing-out a [`connect()`] system call. The [`connect()`]
    /// system call normally takes a long time before it returns a time out error. Setting this
    /// option allows the library to time out the call at an earlier interval.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | 0 ms (disabled)         | all, when using TCP transports. |
    ///
    /// [`connect()`]: #method.connect
    pub fn set_connect_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ConnectTimeout, value)
    }

    /// # Retrieve connect() timeout `ZMQ_CONNECT_TIMEOUT`
    ///
    /// Retrieves how long to wait before timing-out a [`connect()`] system call. The [`connect()`]
    /// system call normally takes a long time before it returns a time out error. Setting this
    /// option allows the library to time out the call at an earlier interval.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | 0 ms (disabled)         | all, when using TCP transports. |
    ///
    /// [`connect()`]: #method.connect
    pub fn connect_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ConnectTimeout)
    }

    /// # Retrieve socket event state `ZMQ_EVENTS`
    ///
    /// The [`events()`] option shall retrieve the event state for the specified `Socket`. The
    /// returned value is a bit mask constructed by OR’ing a combination of the following event
    /// flags:
    ///
    /// * [`POLL_IN`] Indicates that at least one message may be received from the specified socket
    ///   without blocking.
    /// * [`POLL_OUT`] Indicates that at least one message may be sent to the specified socket
    ///   without blocking.
    ///
    /// The combination of a file descriptor returned by the [`FileDescriptor`] option being ready
    /// for reading but no actual events returned by a subsequent retrieval of the [`events()`]
    /// option is valid; applications should simply ignore this case and restart their polling
    /// operation/event loop.
    ///
    /// | Default value | Applicable socket types         |
    /// | :-----------: | :-----------------------------: |
    /// | None          | all                             |
    ///
    /// [`events()`]: #method.events
    /// [`POLL_IN`]: PollEvents::POLL_IN
    /// [`POLL_OUT`]: PollEvents::POLL_OUT
    /// [`FileDescriptor`]: SocketOption::FileDescriptor
    pub fn events(&self) -> ZmqResult<PollEvents> {
        self.get_sockopt_int::<i32>(SocketOption::Events)?
            .try_into()
            .map(PollEvents::from_bits_truncate)
            .map_err(|_err| ZmqError::InvalidArgument)
    }

    /// # Set maximum handshake interval `ZMQ_HANDSHAKE_IVL`
    ///
    /// The [`HandshakeInterval`] option shall set the maximum handshake interval for the `Socket`.
    /// Handshaking is the exchange of socket configuration information (socket type, routing id,
    /// security) that occurs when a connection is first opened, only for connection-oriented
    /// transports. If handshaking does not complete within the configured time, the connection
    /// shall be closed. The value `0` means no handshake time limit.
    ///
    /// | Default value | Applicable socket types                                     |
    /// | :-----------: | :---------------------------------------------------------: |
    /// | 30_000 ms     | all but [`Stream`], only for connection-oriented transports |
    ///
    /// [`HandshakeInterval`]: SocketOption::HandshakeInterval
    /// [`Stream`]: StreamSocket
    pub fn set_handshake_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HandshakeInterval, value)
    }

    /// # Retrieve maximum handshake interval `ZMQ_HANDSHAKE_IVL`
    ///
    /// The [`HandshakeInterval`] option shall retrieve the maximum handshake interval for the
    /// `Socket`. Handshaking is the exchange of socket configuration information (socket type,
    /// routing id, security) that occurs when a connection is first opened, only for
    /// connection-oriented transports. If handshaking does not complete within the configured
    /// time, the connection shall be closed. The value `0` means no handshake time limit.
    ///
    /// | Default value | Applicable socket types                                     |
    /// | :-----------: | :---------------------------------------------------------: |
    /// | 30_000 ms     | all but [`Stream`], only for connection-oriented transports |
    ///
    /// [`HandshakeInterval`]: SocketOption::HandshakeInterval
    /// [`Stream`]: StreamSocket
    pub fn handshake_interval(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::HandshakeInterval)
    }

    /// # Set interval between sending ZMTP heartbeats `ZMQ_HEARTBEAT_IVL`
    ///
    /// The [`HeartbeatInterval`] option shall set the interval between sending ZMTP heartbeats for
    /// the `Socket`. If this option is set and is greater than `0`, then a `PING` ZMTP command
    /// will be sent every [`heartbeat_interval()`] milliseconds.
    ///
    /// | Default value | Applicable socket types                        |
    /// | :-----------: | :--------------------------------------------: |
    /// | ß ms          | all, when using connection-oriented transports |
    ///
    /// [`HeartbeatInterval`]: SocketOption::HeartbeatInterval
    /// [`heartbeat_interval()`]: #method.heartbeat_interval
    pub fn set_heartbeat_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HeartbeatInterval, value)
    }

    /// # Retrieve interval between sending ZMTP heartbeats `ZMQ_HEARTBEAT_IVL`
    ///
    /// The [`HeartbeatInterval`] option returns the interval between sending ZMTP heartbeats for
    /// the `Socket`. If this option is set and is greater than `0`, then a `PING` ZMTP command
    /// will be sent every [`heartbeat_interval()`] milliseconds.
    ///
    /// | Default value | Applicable socket types                        |
    /// | :-----------: | :--------------------------------------------: |
    /// | ß ms          | all, when using connection-oriented transports |
    ///
    /// [`HeartbeatInterval`]: SocketOption::HeartbeatInterval
    /// [`heartbeat_interval()`]: #method.heartbeat_interval
    pub fn heartbeat_interval(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::HeartbeatInterval)
    }

    /// # Set timeout for ZMTP heartbeats `ZMQ_HEARTBEAT_TIMEOUT`
    ///
    /// The [`HeartbeatTimeout`] option shall set how long to wait before timing-out a connection
    /// after sending a `PING` ZMTP command and not receiving any traffic. This option is only
    /// valid if [`HeartbeatInterval`] is also set, and is greater than `0`. The connection will
    /// time out if there is no traffic received after sending the `PING` command, but the received
    /// traffic does not have to be a `PONG` command - any received traffic will cancel the timeout.
    ///
    /// | Default value                                     | Applicable socket types                        |
    /// | :-----------------------------------------------: | :--------------------------------------------: |
    /// | 0 ms normally, [`HeartbeatInterval`] if it is set | all, when using connection-oriented transports |
    ///
    /// [`HeartbeatTimeout`]: SocketOption::HeartbeatTimeout
    /// [`HeartbeatInterval`]: SocketOption::HeartbeatInterval
    pub fn set_heartbeat_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HeartbeatTimeout, value)
    }

    /// # Retrieve timeout for ZMTP heartbeats `ZMQ_HEARTBEAT_TIMEOUT`
    ///
    /// The [`HeartbeatTimeout`] option returns how long to wait before timing-out a connection
    /// after sending a `PING` ZMTP command and not receiving any traffic. The connection will time
    /// out if there is no traffic received after sending the `PING` command, but the received
    /// traffic does not have to be a `PONG` command - any received traffic will cancel the timeout.
    ///
    /// | Default value                                     | Applicable socket types                        |
    /// | :-----------------------------------------------: | :--------------------------------------------: |
    /// | 0 ms normally, [`HeartbeatInterval`] if it is set | all, when using connection-oriented transports |
    ///
    /// [`HeartbeatTimeout`]: SocketOption::HeartbeatTimeout
    /// [`HeartbeatInterval`]: SocketOption::HeartbeatInterval
    pub fn heartbeat_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::HeartbeatTimeout)
    }

    /// # Set the TTL value for ZMTP heartbeats `ZMQ_HEARTBEAT_TTL`
    ///
    /// The [`HeartbeatTimeToLive`] option shall set the timeout on the remote peer for ZMTP
    /// heartbeats. If this option is greater than 0, the remote side shall time out the connection
    /// if it does not receive any more traffic within the TTL period. This option does not have
    /// any effect if [`HeartbeatInterval`] is not set or is `0`. Internally, this value is rounded
    /// down to the nearest decisecond, any value less than `100` will have no effect.
    ///
    /// | Default value                           | Applicable socket types                        |
    /// | :-------------------------------------: | :--------------------------------------------: |
    /// | 6_553_599 (which is 2^16-1 deciseconds) | all, when using connection-oriented transports |
    ///
    /// [`HeartbeatTimeToLive`]: SocketOption::HeartbeatTimeToLive
    /// [`HeartbeatInterval`]: SocketOption::HeartbeatInterval
    pub fn set_heartbeat_timetolive(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HeartbeatTimeToLive, value)
    }

    /// # Retrieve the TTL value for ZMTP heartbeats `ZMQ_HEARTBEAT_TTL`
    ///
    /// The [`HeartbeatTimeToLive`] option returns the timeout on the remote peer for ZMTP
    /// heartbeats. If this option is greater than 0, the remote side shall time out the connection
    /// if it does not receive any more traffic within the TTL period. This option does not have
    /// any effect if [`HeartbeatInterval`] is not set or is `0`. Internally, this value is rounded
    /// down to the nearest decisecond, any value less than `100` will have no effect.
    ///
    /// | Default value                           | Applicable socket types                        |
    /// | :-------------------------------------: | :--------------------------------------------: |
    /// | 6_553_599 (which is 2^16-1 deciseconds) | all, when using connection-oriented transports |
    ///
    /// [`HeartbeatTimeToLive`]: SocketOption::HeartbeatTimeToLive
    /// [`HeartbeatInterval`]: SocketOption::HeartbeatInterval
    pub fn heartbeat_timetolive(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::HeartbeatTimeToLive)
    }

    /// # Queue messages only to completed connections `ZMQ_IMMEDIATE`
    ///
    /// By default queues will fill on outgoing connections even if the connection has not
    /// completed. This can lead to "lost" messages on sockets with round-robin routing
    /// ([`Request`], [`Push`], [`Dealer`]). If this option is set to `true`, messages shall be
    /// queued only to completed connections. This will cause the socket to block if there are no
    /// other connections, but will prevent queues from filling on pipes awaiting connection.
    ///
    /// | Default value | Applicable socket types                        |
    /// | :-----------: | :--------------------------------------------: |
    /// | false         | all, when using connection-oriented transports |
    ///
    /// [`Request`]: RequestSocket
    /// [`Push`]: PushSocket
    /// [`Dealer`]: DealerSocket
    pub fn set_immediate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::Immediate, value)
    }

    /// # Retrieve attach-on-connect value `ZMQ_IMMEDIATE`
    ///
    /// Retrieve the state of the attach on connect value. If set to 1`true`, will delay the
    /// attachment of a pipe on connect until the underlying connection has completed. This will
    /// cause the socket to block if there are no other connections, but will prevent queues from
    /// filling on pipes awaiting connection.
    ///
    /// | Default value | Applicable socket types                        |
    /// | :-----------: | :--------------------------------------------: |
    /// | false         | all, when using connection-oriented transports |
    pub fn immediate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::Immediate)
    }

    /// # Enable IPv6 on socket `ZMQ_IPV6`
    ///
    /// Set the IPv6 option for the socket. A value of `true` means IPv6 is enabled on the socket,
    /// while `false` means the socket will use only IPv4. When IPv6 is enabled the socket will
    /// connect to, or accept connections from, both IPv4 and IPv6 hosts.
    ///
    /// | Default value | Applicable socket types         |
    /// | :-----------: | :-----------------------------: |
    /// | false         | all, when using TCP transports. |
    pub fn set_ipv6(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::IPv6, value)
    }

    /// # Retrieve IPv6 socket status `ZMQ_IPV6`
    ///
    /// Retrieve the IPv6 option for the socket. A value of `true` means IPv6 is enabled on the
    /// socket, while `false` means the socket will use only IPv4. When IPv6 is enabled the socket
    /// will connect to, or accept connections from, both IPv4 and IPv6 hosts.
    ///
    /// | Default value | Applicable socket types         |
    /// | :-----------: | :-----------------------------: |
    /// | false         | all, when using TCP transports. |
    pub fn ipv6(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::IPv6)
    }

    /// # Set linger period for socket shutdown `ZMQ_LINGER`
    ///
    /// The [`Linger`] option shall set the linger period for the `Socket`. The linger period
    /// determines how long pending messages which have yet to be sent to a peer shall linger in
    /// memory after a socket is disconnected with [`disconnect()`] or closed, and further affects
    /// the termination of the socket’s context. The following outlines the different behaviours:
    ///
    /// * A value of `-1` specifies an infinite linger period. Pending messages shall not be
    ///   discarded after a call to [`disconnect()`]; attempting to terminate the socket’s context
    ///   shall block until all pending messages have been sent to a peer.
    /// * The value of `0` specifies no linger period. Pending messages shall be discarded
    ///   immediately after a call to [`disconnect()`].
    /// * Positive values specify an upper bound for the linger period in milliseconds. Pending
    ///   messages shall not be discarded after a call to [`disconnect()`]; attempting to terminate
    ///   the socket’s context shall block until either all pending messages have been sent to a
    ///   peer, or the linger period expires, after which any pending messages shall be discarded.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1 (infinite) | all                     |
    ///
    /// [`disconnect()`]: #method.disconnect
    /// [`Linger`]: SocketOption::Linger
    pub fn set_linger(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::Linger, value)
    }

    /// # Retrieve linger period for socket shutdown `ZMQ_LINGER`
    ///
    /// The [`Linger`] option shall retrieve the linger period for the `Socket`. The linger period
    /// determines how long pending messages which have yet to be sent to a peer shall linger in
    /// memory after a socket is closed, and further affects the termination of the socket’s
    /// context. The following outlines the different behaviours:
    ///
    /// * A value of `-1` specifies an infinite linger period. Pending messages shall not be
    ///   discarded after a call to [`disconnect()`]; attempting to terminate the socket’s context
    ///   shall block until all pending messages have been sent to a peer.
    /// * The value of `0` specifies no linger period. Pending messages shall be discarded
    ///   immediately after a call to [`disconnect()`].
    /// * Positive values specify an upper bound for the linger period in milliseconds. Pending
    ///   messages shall not be discarded after a call to [`disconnect()`]; attempting to terminate
    ///   the socket’s context shall block until either all pending messages have been sent to a
    ///   peer, or the linger period expires, after which any pending messages shall be discarded.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1 (infinite) | all                     |
    ///
    /// [`disconnect()`]: #method.disconnect
    /// [`Linger`]: SocketOption::Linger
    pub fn linger(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::Linger)
    }

    /// # Retrieve the last endpoint set `ZMQ_LAST_ENDPOINT`
    ///
    /// The [`LastEndpoint`] option shall retrieve the last endpoint bound for TCP and IPC
    /// transports. The returned value will be a string in the form of a ZMQ DSN. Note that if the
    /// TCP host is INADDR_ANY, indicated by a *, then the returned address will be `0.0.0.0`
    /// (for IPv4). Note: not supported on GNU/Hurd with IPC due to non-working getsockname().
    ///
    /// | Default value | Applicable socket types                 |
    /// | :-----------: | :-------------------------------------: |
    /// | None          | all, when binding TCP or IPC transports |
    ///
    /// [`LastEndpoint`]: SocketOption::LastEndpoint
    pub fn last_endpoint(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOption::LastEndpoint)
    }

    /// # Maximum acceptable inbound message size `ZMQ_MAXMSGSIZE`
    ///
    /// Limits the size of the inbound message. If a peer sends a message larger than
    /// [`MaxMessageSize`] it is disconnected. Value of `-1` means 'no limit'.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1 (bytes)    | all                     |
    ///
    /// [`MaxMessageSize`]: SocketOption::MaxMessageSize
    pub fn set_max_message_size(&self, value: i64) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::MaxMessageSize, value)
    }

    /// # Maximum acceptable inbound message size `ZMQ_MAXMSGSIZE`
    ///
    /// The option shall retrieve limit for the inbound messages. If a peer sends a message larger
    /// than [`MaxMessageSize`] it is disconnected. Value of `-1` means 'no limit'.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1 (bytes)    | all                     |
    ///
    /// [`MaxMessageSize`]: SocketOption::MaxMessageSize
    pub fn max_message_size(&self) -> ZmqResult<i64> {
        self.get_sockopt_int(SocketOption::MaxMessageSize)
    }

    /// # Set the security mechanism `ZMQ_MECHANISM`
    ///
    /// Sets the security [`Mechanism`] option for the socket based on the provided
    /// [`SecurityMechanism`].
    ///
    /// [`Mechanism`]: SocketOption::Mechanism
    /// [ SecurityMechanism`]: SecurityMechanism
    pub fn set_security_mechanism(&self, security: &SecurityMechanism) -> ZmqResult<()> {
        security.apply(self)
    }

    /// # Retrieve current security mechanism `ZMQ_MECHANISM`
    ///
    /// The [`Mechanism`] option shall retrieve the current security mechanism for the socket.
    ///
    /// [`Mechanism`]: SocketOption::Mechanism
    pub fn security_mechanism(&self) -> ZmqResult<SecurityMechanism> {
        SecurityMechanism::try_from(self)
    }

    /// # Maximum network hops for multicast packets `ZMQ_MULTICAST_HOPS`
    ///
    /// Sets the time-to-live field in every multicast packet sent from this socket. The default
    /// is `1` which means that the multicast packets don’t leave the local network.
    ///
    /// | Default value | Applicable socket types              |
    /// | :-----------: | :----------------------------------: |
    /// | 1             | all, when using multicast transports |
    pub fn set_multicast_hops(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::MulticastHops, value)
    }

    /// # Maximum network hops for multicast packets `ZMQ_MULTICAST_HOPS`
    ///
    /// The option shall retrieve time-to-live used for outbound multicast packets. The default of
    /// `1` means that the multicast packets don’t leave the local network.
    ///
    /// | Default value | Applicable socket types              |
    /// | :-----------: | :----------------------------------: |
    /// | 1             | all, when using multicast transports |
    pub fn multicast_hops(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::MulticastHops)
    }

    /// # Set multicast data rate `ZMQ_RATE`
    ///
    /// The [`Rate`] option shall set the maximum send or receive data rate for multicast
    /// transports such as `pgm` using the `Socket`.
    ///
    /// | Default value      | Applicable socket types              |
    /// | :----------------: | :----------------------------------: |
    /// | 100 (kilobits/sec) | all, when using multicast transports |
    ///
    /// [`Rate`]: SocketOption::Rate
    pub fn set_rate(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::Rate, value)
    }

    /// # Retrieve multicast data rate `ZMQ_RATE`
    ///
    /// The [`Rate`] option shall retrieve the maximum send or receive data rate for multicast
    /// transports using the `Socket`.
    ///
    /// | Default value      | Applicable socket types              |
    /// | :----------------: | :----------------------------------: |
    /// | 100 (kilobits/sec) | all, when using multicast transports |
    ///
    /// [`Rate`]: SocketOption::Rate
    pub fn rate(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::Rate)
    }

    /// # Set kernel receive buffer size `ZMQ_RCVBUF`
    /// The [`ReceiveBuffer`] option shall set the underlying kernel receive buffer size for the
    /// `Socket` to the specified size in bytes. A value of `-1` means leave the OS default
    /// unchanged. For details refer to your operating system documentation for the `SO_RCVBUF`
    /// socket option.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1            | all                     |
    ///
    /// [`ReceiveBuffer`]: SocketOption::ReceiveBuffer
    pub fn set_receive_buffer(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReceiveBuffer, value)
    }

    /// # Retrieve kernel receive buffer size `ZMQ_RCVBUF`
    ///
    /// The [`ReceiveBuffer`] option shall retrieve the underlying kernel receive buffer size for
    /// the `Socket`. For details refer to your operating system documentation for the `SO_RCVBUF`
    /// socket option.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1            | all                     |
    ///
    /// [`ReceiveBuffer`]: SocketOption::ReceiveBuffer
    pub fn receive_buffer(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReceiveBuffer)
    }

    /// # Set high water mark for inbound messages `ZMQ_RCVHWM`
    ///
    /// The [`ReceiveHighWatermark`] option shall set the high water mark for inbound messages on
    /// the `Ssocket`. The high water mark is a hard limit on the maximum number of outstanding
    /// messages 0MQ shall queue in memory for any single peer that the specified `Socket` is
    /// communicating with. A value of zero means no limit.
    ///
    /// If this limit has been reached the socket shall enter an exceptional state and depending on
    /// the socket type, 0MQ shall take appropriate action such as blocking or dropping sent
    /// messages. Refer to the individual socket descriptions for details on the exact action taken
    /// for each socket type.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 1000          | all                     |
    ///
    /// [`ReceiveHighWatermark`]: SocketOption::ReceiveHighWatermark
    pub fn set_receive_highwater_mark(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReceiveHighWatermark, value)
    }

    /// # Retrieve high water mark for inbound messages `ZMQ_RCVHWM`
    ///
    /// The [`ReceiveHighWatermark`] option shall return the high water mark for inbound messages
    /// on the `Socket`. The high water mark is a hard limit on the maximum number of outstanding
    /// messages 0MQ shall queue in memory for any single peer that the specified `Socket` is
    /// communicating with. A value of zero means no limit.
    ///
    /// If this limit has been reached the socket shall enter an exceptional state and depending on
    /// the socket type, 0MQ shall take appropriate action such as blocking or dropping sent
    /// messages. Refer to the individual socket descriptions for details on the exact action taken
    /// for each socket type.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 1000          | all                     |
    ///
    /// [`ReceiveHighWatermark`]: SocketOption::ReceiveHighWatermark
    pub fn receive_highwater_mark(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReceiveHighWatermark)
    }

    ///# Maximum time before a recv operation returns with [`Again`] `ZMQ_RCVTIMEO`
    ///
    /// Sets the timeout for receive operation on the socket. If the value is 0, [`recv_msg()`]
    /// will return immediately, with a [`Again`] error if there is no message to receive. If the
    /// value is `-1`, it will block until a message is available. For all other values, it will
    /// wait for a message for that amount of time before returning with an [`Again`] error.
    ///
    /// | Default value    | Applicable socket types |
    /// | :--------------: | :---------------------: |
    /// | -1 ms (infinite) | all                     |
    ///
    /// [`Again`]: crate::ZmqError::Again
    /// [`recv_msg()`]: #method.recv_msg
    pub fn set_receive_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReceiveTimeout, value)
    }

    /// # Maximum time before a socket operation returns with [`Again`] `ZMQ_RCVTIMEO`
    ///
    /// Retrieve the timeout for recv operation on the socket. If the value is `0`, [`recv_msg()`]
    /// will return immediately, with a [`Again`] error if there is no message to receive. If the
    /// value is `-1`, it will block until a message is available. For all other values, it will
    /// wait for a message for that amount of time before returning with an [`Again`] error.
    ///
    /// | Default value    | Applicable socket types |
    /// | :--------------: | :---------------------: |
    /// | -1 ms (infinite) | all                     |
    ///
    /// [`Again`]: crate::ZmqError::Again
    /// [`recv_msg()`]: #method.recv_msg
    pub fn receive_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReceiveTimeout)
    }

    /// # Set reconnection interval `ZMQ_RECONNECT_IVL`
    /// The [`ReconnectInterval`] option shall set the initial reconnection interval for the
    /// `Socket`. The reconnection interval is the period 0MQ shall wait between attempts to
    /// reconnect disconnected peers when using connection-oriented transports. The value `-1`
    /// means no reconnection.
    ///
    /// | Default value | Applicable socket types                      |
    /// | :-----------: | :------------------------------------------: |
    /// | 100 ms        | all, only for connection-oriented transports |
    ///
    /// [`ReconnectInterval`]: SocketOption::ReconnectInterval
    pub fn set_reconnect_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReconnectInterval, value)
    }

    /// # Retrieve reconnection interval `ZMQ_RECONNECT_IVL`
    ///
    /// The [`ReconnectInterval`] option shall retrieve the initial reconnection interval for the
    /// `Socket`. The reconnection interval is the period 0MQ shall wait between attempts to
    /// reconnect disconnected peers when using connection-oriented transports. The value `-1`
    /// means no reconnection.
    ///
    /// | Default value | Applicable socket types                      |
    /// | :-----------: | :------------------------------------------: |
    /// | 100 ms        | all, only for connection-oriented transports |
    ///
    /// [`ReconnectInterval`]: SocketOption::ReconnectInterval
    pub fn reconnect_interval(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReconnectInterval)
    }

    /// # Set max reconnection interval `ZMQ_RECONNECT_IVL_MAX`
    ///
    /// The [`ReconnectIntervalMax`] option shall set the max reconnection interval for the
    /// `Socket`. 0MQ shall wait at most the configured interval between reconnection attempts. The
    /// interval grows exponentionally (i.e.: it is doubled) with each attempt until it reaches
    /// [`ReconnectIntervalMax`]. Default value means that the reconnect interval is based
    /// exclusively on [`ReconnectInterval`] and no exponential backoff is performed.
    ///
    /// | Default value                             | Applicable socket types                      |
    /// | :---------------------------------------: | :------------------------------------------: |
    /// | 0 ms ([`ReconnectInterval`] will be used) | all, only for connection-oriented transports |
    ///
    /// [`ReconnectIntervalMax`]: SocketOption::ReconnectIntervalMax
    /// [`ReconnectInterval`]: SocketOption::ReconnectInterval
    pub fn set_reconnect_interval_max(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReconnectIntervalMax, value)
    }

    /// # Retrieve max reconnection interval `ZMQ_RECONNECT_IVL_MAX`
    ///
    /// The [`ReconnectIntervalMax`] option shall retrieve the max reconnection interval for the
    /// `Socket`. 0MQ shall wait at most the configured interval between reconnection attempts. The
    /// interval grows exponentionally (i.e.: it is doubled) with each attempt until it reaches
    /// [`ReconnectIntervalMax`]. Default value means that the reconnect interval is based
    /// exclusively on [`ReconnectInterval`] and no exponential backoff is performed.
    ///
    /// | Default value                             | Applicable socket types                      |
    /// | :---------------------------------------: | :------------------------------------------: |
    /// | 0 ms ([`ReconnectInterval`] will be used) | all, only for connection-oriented transports |
    ///
    /// [`ReconnectIntervalMax`]: SocketOption::ReconnectIntervalMax
    /// [`ReconnectInterval`]: SocketOption::ReconnectInterval
    pub fn reconnect_interval_max(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReconnectIntervalMax)
    }

    /// # Set condition where reconnection will stop `ZMQ_RECONNECT_STOP`
    ///
    /// The [`ReconnectStop`] option shall set the conditions under which automatic reconnection
    /// will stop. This can be useful when a process binds to a wild-card port, where the OS
    /// supplies an ephemeral port.
    ///
    /// | Default value | Applicable socket types                      |
    /// | :-----------: | :------------------------------------------: |
    /// | 0             | all, only for connection-oriented transports |
    ///
    /// [`ReconnectStop`]: SocketOption::ReconnectStop
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_reconnect_stop(&self, value: ReconnectStop) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReconnectStop, value.bits())
    }

    /// # ZMQ_RECONNECT_STOP: Retrieve condition where reconnection will stop
    ///
    /// The [`ReconnectStop`] option shall retrieve the conditions under which automatic
    /// reconnection will stop.
    ///
    /// | Default value | Applicable socket types                      |
    /// | :-----------: | :------------------------------------------: |
    /// | 0             | all, only for connection-oriented transports |
    ///
    /// [`ReconnectStop`]: SocketOption::ReconnectStop
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn reconnect_stop(&self) -> ZmqResult<ReconnectStop> {
        self.get_sockopt_int(SocketOption::ReconnectStop)
            .map(ReconnectStop::from_bits_truncate)
    }

    /// # Set multicast recovery interval `ZMQ_RECOVERY_IVL`
    ///
    /// The [`RecoveryInterval`] option shall set the recovery interval for multicast transports
    /// using the `Socket`. The recovery interval determines the maximum time in milliseconds that
    /// a receiver can be absent from a multicast group before unrecoverable data loss will occur.
    ///
    /// | Default value | Applicable socket types              |
    /// | :-----------: | :----------------------------------: |
    /// | 10_000 ms     | all, when using multicast transports |
    ///
    /// [`RecoveryInterval`]: SocketOption::RecoveryInterval
    pub fn set_recovery_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::RecoveryInterval, value)
    }

    /// # Get multicast recovery interval `ZMQ_RECOVERY_IVL`
    ///
    /// The [`RecoveryInterval`] option shall retrieve the recovery interval for multicast
    /// transports using the `Socket`. The recovery interval determines the maximum time in
    /// milliseconds that a receiver can be absent from a multicast group before unrecoverable data
    /// loss will occur.
    ///
    /// | Default value | Applicable socket types              |
    /// | :-----------: | :----------------------------------: |
    /// | 10_000 ms     | all, when using multicast transports |
    ///
    /// [`RecoveryInterval`]: SocketOption::RecoveryInterval
    pub fn recovery_interval(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::RecoveryInterval)
    }

    /// # Set kernel transmit buffer size `ZMQ_SNDBUF`
    ///
    /// The [`SendBuffer`] option shall set the underlying kernel transmit buffer size for the
    /// `Socket` to the specified size in bytes. A value of `-1` means leave the OS default
    /// unchanged. For details please refer to your operating system documentation for the
    /// `SO_SNDBUF` socket option.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1            | all                     |
    ///
    /// [`SendBuffer`]: SocketOption::SendBuffer
    pub fn set_send_buffer(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::SendBuffer, value)
    }

    /// # Retrieve kernel transmit buffer size `ZMQ_SNDBUF`
    ///
    /// The [`SendBuffer`] option shall retrieve the underlying kernel transmit buffer size for the
    /// `Socket`. For details refer to your operating system documentation for the `SO_SNDBUF`
    /// socket option.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1            | all                     |
    ///
    /// [`SendBuffer`]: SocketOption::SendBuffer
    pub fn send_buffer(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::SendBuffer)
    }

    /// # Set high water mark for outbound messages `ZMQ_SNDHWM`
    ///
    /// The [`SendHighWatermark`] option shall set the high water mark for outbound messages on the
    /// `Socket`. The high water mark is a hard limit on the maximum number of outstanding messages
    /// 0MQ shall queue in memory for any single peer that the `Socket` is communicating with. A
    /// value of zero means no limit.
    ///
    /// If this limit has been reached the socket shall enter an exceptional state and depending on
    /// the socket type, 0MQ shall take appropriate action such as blocking or dropping sent
    /// messages. Refer to the individual socket descriptions for details on the exact action taken
    /// for each socket type.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 1000          | all                     |
    ///
    /// [`SendHighWatermark`]: SocketOption::SendHighWatermark
    pub fn set_send_highwater_mark(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::SendHighWatermark, value)
    }

    /// # Retrieves high water mark for outbound messages `ZMQ_SNDHWM`
    ///
    /// The [`SendHighWatermark`] option shall return the high water mark for outbound messages on
    /// the `Socket`. The high water mark is a hard limit on the maximum number of outstanding
    /// messages 0MQ shall queue in memory for any single peer that the `Socket` is communicating
    /// with. A value of zero means no limit.
    ///
    /// If this limit has been reached the socket shall enter an exceptional state and depending on
    /// the socket type, 0MQ shall take appropriate action such as blocking or dropping sent
    /// messages. Refer to the individual socket descriptions for details on the exact action taken
    /// for each socket type.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 1000          | all                     |
    ///
    /// [`SendHighWatermark`]: SocketOption::SendHighWatermark
    pub fn send_highwater_mark(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::SendHighWatermark)
    }

    /// # Maximum time before a send operation returns with [`Again`] `ZMQ_SNDTIMEO`
    ///
    /// Sets the timeout for send operation on the socket. If the value is `0`, [`send_msg()`] will
    /// return immediately, with a [`Again`] error if the message cannot be sent. If the value is
    /// `-1`, it will block until the message is sent. For all other values, it will try to send
    /// the message for that amount of time before returning with an EAGAIN error.
    ///
    /// | Default value    | Applicable socket types |
    /// | :--------------: | :---------------------: |
    /// | -1 ms (infinite) | all                     |
    ///
    /// [`Again`]: crate::ZmqError::Again
    /// [`send_msg()`]: #method.send_msg
    pub fn set_send_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::SendTimeout, value)
    }

    /// # Maximum time before a socket operation returns with [`Again`] `ZMQ_SNDTIMEO`
    ///
    /// Retrieve the timeout for send operation on the socket. If the value is `0`, [`send_msg()`]
    /// will return immediately, with a [`Again`] error if the message cannot be sent. If the value
    /// is `-1`, it will block until the message is sent. For all other values, it will try to send
    /// the message for that amount of time before returning with an [`Again`] error.
    ///
    /// | Default value    | Applicable socket types |
    /// | :--------------: | :---------------------: |
    /// | -1 ms (infinite) | all                     |
    ///
    /// [`Again`]: crate::ZmqError::Again
    /// [`send_msg()`]: #method.send_msg
    pub fn send_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::SendTimeout)
    }

    /// # Set SOCKS5 proxy address `ZMQ_SOCKS_PROXY`
    ///
    /// Sets the SOCKS5 proxy address that shall be used by the socket for the TCP connection(s).
    /// Supported authentication methods are: no authentication or basic authentication when setup
    /// with [`SocksUsername`]. If the endpoints are domain names instead of addresses they shall
    /// not be resolved and they shall be forwarded unchanged to the SOCKS proxy service in the
    /// client connection request message (address type 0x03 domain name).
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | not set       | all, when using TCP transport |
    ///
    /// [`SocksUsername`]: SocketOption::SocksUsername
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_socks_proxy<V>(&self, value: Option<V>) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        match value {
            None => self.set_sockopt_string(SocketOption::SocksUsername, ""),
            Some(ref_value) => self.set_sockopt_string(SocketOption::SocksProxy, ref_value),
        }
    }

    /// # Retrieve SOCKS5 proxy address `ZMQ_SOCKS_PROXY`
    ///
    /// The [`SocksProxy`] option shall retrieve the SOCKS5 proxy address in string format. The
    /// returned value MAY be empty.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | not set       | all, when using TCP transport |
    ///
    /// [`SocksProxy`]: SocketOption::SocksProxy
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn socks_proxy(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOption::SocksProxy)
    }

    /// # Set SOCKS username and select basic authentication `ZMQ_SOCKS_USERNAME`
    ///
    /// Sets the username for authenticated connection to the SOCKS5 proxy. If you set this to a
    /// non-null and non-empty value, the authentication method used for the SOCKS5 connection
    /// shall be basic authentication. In this case, use [`set_socks_password()`] option in order
    /// to set the password. If you set this to a null value or empty value, the authentication
    /// method shall be no authentication, the default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | not set       | all, when using TCP transport |
    ///
    /// [`set_socks_password()`]: #method.set_socks_password
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_socks_username<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::SocksUsername, value.as_ref())
    }

    /// # Retrieve SOCKS username and select basic authentication `ZMQ_SOCKS_USERNAME`
    ///
    /// Returns the username for authenticated connection to the SOCKS5 proxy. If set to a non-null
    /// and non-empty value, the authentication method used for the SOCKS5 connection shall be
    /// basic authentication. If set to a null value or empty value, the authentication method
    /// shall be no authentication, the default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | not set       | all, when using TCP transport |
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn socks_username(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOption::SocksUsername)
    }

    /// # Set SOCKS basic authentication password `ZMQ_SOCKS_PASSWORD`
    ///
    /// Sets the password for authenticating to the SOCKS5 proxy server. This is used only when the
    /// SOCK5 authentication method has been set to basic authentication through the
    /// [`set_socks_username()`] option. Setting this to a null value (the default) is equivalent
    /// to an empty password string.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | not set       | all, when using TCP transport |
    ///
    /// [`set_socks_username()`]: #method.set_socks_username
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_socks_password<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::SocksPassword, value.as_ref())
    }

    /// # Retrieve SOCKS basic authentication password `ZMQ_SOCKS_PASSWORD`
    ///
    /// Returns the password for authenticating to the SOCKS5 proxy server. This is used only when
    /// the SOCK5 authentication method has been set to basic authentication through the
    /// [`set_socks_username()`] option. A null value (the default) is equivalent to an empty
    /// password string.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | not set       | all, when using TCP transport |
    ///
    /// [`set_socks_username()`]: #method.set_socks_username
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn socks_password(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOption::SocksPassword)
    }

    /// # Override `SO_KEEPALIVE` socket option `ZMQ_TCP_KEEPALIVE`
    ///
    /// Override `SO_KEEPALIVE` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn set_tcp_keepalive(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepalive, value)
    }

    /// # Override `SO_KEEPALIVE` socket option `ZMQ_TCP_KEEPALIVE`
    ///
    /// Override `SO_KEEPALIVE` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn tcp_keepalive(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TcpKeepalive)
    }

    /// # Override `TCP_KEEPCNT` socket option `ZMQ_TCP_KEEPALIVE_CNT`
    ///
    /// Override `TCP_KEEPCNT` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn set_tcp_keepalive_count(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepaliveCount, value)
    }

    ///  Override `TCP_KEEPCNT` socket option `ZMQ_TCP_KEEPALIVE_CNT`
    ///
    /// Override `TCP_KEEPCNT` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn tcp_keepalive_count(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TcpKeepaliveCount)
    }

    /// # Override `TCP_KEEPIDLE` (or `TCP_KEEPALIVE` on some OS) `ZMQ_TCP_KEEPALIVE_IDLE`
    ///
    /// Override `TCP_KEEPIDLE` (or `TCP_KEEPALIVE` on some OS) socket option (where supported by
    /// OS). The default value of `-1` means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn set_tcp_keepalive_idle(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepaliveIdle, value)
    }

    /// # Override `TCP_KEEPIDLE` (or `TCP_KEEPALIVE` on some OS) `ZMQ_TCP_KEEPALIVE_IDLE`
    ///
    /// Override `TCP_KEEPIDLE` (or `TCP_KEEPALIVE` on some OS) socket option (where supported by
    /// OS). The default value of `-1` means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn tcp_keepalive_idle(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TcpKeepaliveIdle)
    }

    /// # Override `TCP_KEEPINTVL` socket option `ZMQ_TCP_KEEPALIVE_INTVL`
    ///
    /// Override `TCP_KEEPINTVL` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn set_tcp_keepalive_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepaliveInterval, value)
    }

    /// # Override `TCP_KEEPINTVL` socket option `ZMQ_TCP_KEEPALIVE_INTVL`
    ///
    /// Override `TCP_KEEPINTVL` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn tcp_keepalive_interval(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TcpKeepaliveInterval)
    }

    /// # Set TCP Maximum Retransmit Timeout `ZMQ_TCP_MAXRT`
    ///
    /// On OSes where it is supported, sets how long before an unacknowledged TCP retransmit times
    /// out. The system normally attempts many TCP retransmits following an exponential backoff
    /// strategy. This means that after a network outage, it may take a long time before the
    /// session can be re-established. Setting this option allows the timeout to happen at a
    /// shorter interval.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | 0 ms                    | all, when using TCP transports. |
    pub fn set_tcp_max_retransmit_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::MaxTcpRetransmitTimeout, value)
    }

    /// # Retrieve Max TCP Retransmit Timeout `ZMQ_TCP_MAXRT`
    ///
    /// On OSes where it is supported, retrieves how long before an unacknowledged TCP retransmit
    /// times out. The system normally attempts many TCP retransmits following an exponential
    /// backoff strategy. This means that after a network outage, it may take a long time before
    /// the session can be re-established. Setting this option allows the timeout to happen at a
    /// shorter interval.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | 0 ms                    | all, when using TCP transports. |
    pub fn tcp_max_retransmit_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::MaxTcpRetransmitTimeout)
    }

    /// # Set the Type-of-Service on socket `ZMQ_TOS`
    ///
    /// Sets the ToS fields (Differentiated services (DS) and Explicit Congestion Notification
    /// (ECN) field of the IP header. The ToS field is typically used to specify a packets
    /// priority. The availability of this option is dependent on intermediate network equipment
    /// that inspect the ToS field and provide a path for low-delay, high-throughput,
    /// highly-reliable service, etc.
    ///
    /// | Default value           | Applicable socket types                      |
    /// | :---------------------: | :------------------------------------------: |
    /// | 0                       | all, only for connection-oriented transports |
    pub fn set_type_of_service(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TypeOfService, value)
    }

    /// # Retrieve the Type-of-Service socket override status `ZMQ_TOS`
    ///
    /// Retrieve the IP_TOS option for the socket.
    ///
    /// | Default value           | Applicable socket types                      |
    /// | :---------------------: | :------------------------------------------: |
    /// | 0                       | all, only for connection-oriented transports |
    pub fn type_of_service(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TypeOfService)
    }

    /// # Set RFC 27 authentication domain `ZMQ_ZAP_DOMAIN`
    ///
    /// Sets the domain for ZAP (ZMQ RFC 27) authentication. A ZAP domain must be specified to
    /// enable authentication. When the ZAP domain is empty, which is the default, ZAP
    /// authentication is disabled.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | empty                   | all, when using TCP transports. |
    pub fn set_zap_domain(&self, domain: &ZapDomain) -> ZmqResult<()> {
        domain.apply(self)
    }

    /// # Retrieve RFC 27 authentication domain `ZMQ_ZAP_DOMAIN`
    ///
    /// The [`zap_domain()`] option shall retrieve the last ZAP domain set for the socket. The
    /// returned value shall be a NULL-terminated string and MAY be empty. An empty string means
    /// that ZAP authentication is disabled. The returned size SHALL include the terminating null
    /// byte.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | not set                 | all, when using TCP transports. |
    ///
    /// [`zap_domain()`]: #method.zap_domain
    pub fn zap_domain(&self) -> ZmqResult<ZapDomain> {
        self.get_sockopt_string(SocketOption::ZapDomain)
            .map(ZapDomain::from)
    }

    /// # accept incoming connections on a socket
    ///
    /// The [`bind()`] function binds the `Socket` to a local `endpoint` and then accepts incoming
    /// connections on that endpoint.
    ///
    /// The `endpoint` is a string consisting of a `transport://` followed by an `address`. The
    /// `transport` specifies the underlying protocol to use. The `address` specifies the
    /// transport-specific address to bind to.
    ///
    /// 0MQ provides the following transports:
    ///
    /// * `tcp` unicast transport using TCP
    /// * `ipc` local inter-process communication transport
    /// * `inproc` local in-process (inter-thread) communication transport
    /// * `pgm`, `epgm` reliable multicast transport using PGM
    /// * `vmci` virtual machine communications interface (VMCI)
    /// * `udp` unreliable unicast and multicast using UDP
    ///
    /// Every 0MQ socket type except [`Pair`] and [`Channel`] supports one-to-many and many-to-one
    /// semantics.
    ///
    /// The `ipc`, `tcp`, `vmci` and `udp` transports accept wildcard addresses.
    ///
    /// [`bind()`]: #method.bind
    /// [`Pair`]: PairSocket
    /// [`Channel`]: ChannelSocket
    pub fn bind<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.bind(endpoint.as_ref())
    }

    /// # Stop accepting connections on a socket
    ///
    /// The [`unbind()`] function shall unbind a socket from the endpoint specified by the
    /// `endpoint` argument.
    ///
    /// Additionally the incoming message queue associated with the endpoint will be discarded.
    /// This means that after unbinding an endpoint it is possible to received messages originating
    /// from that same endpoint if they were already present in the incoming message queue before
    /// unbinding.
    ///
    /// The `endpoint` argument is as described in [`bind()`].
    ///
    /// ## Unbinding wild-card address from a socket
    ///
    /// When wild-card * `endpoint` was used in [`bind()`], the caller should use real `endpoint`
    /// obtained from the [`last_endpoint()`] socket option to unbind this `endpoint` from a socket.
    ///
    /// [`unbind()`]: #method.unbind
    /// [`bind()`]: #method.bind
    /// [`last_endpoint()`]: #method.last_endpoint
    pub fn unbind<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.unbind(endpoint.as_ref())
    }

    /// # create outgoing connection from socket
    ///
    /// The [`connect()`] function connects the `Socket` to an `endpoint` and then accepts incoming
    /// connections on that endpoint.
    ///
    /// The `endpoint` is a string consisting of a `transport://` followed by an `address`. The
    /// `transport` specifies the underlying protocol to use. The `address` specifies the
    /// transport-specific address to connect to.
    ///
    /// 0MQ provides the the following transports:
    ///
    /// * `tcp` unicast transport using TCP
    /// * `ipc` local inter-process communication transport
    /// * `inproc` local in-process (inter-thread) communication transport
    /// * `pgm`, `epgm` reliable multicast transport using PGM
    /// * `vmci` virtual machine communications interface (VMCI)
    /// * `udp` unreliable unicast and multicast using UDP
    ///
    /// Every 0MQ socket type except [`Pair`] and [`Channel`] supports one-to-many and many-to-one
    /// semantics.
    ///
    /// [`connect()`]: #method.connect
    /// [`Pair`]: PairSocket
    /// [`Channel`]: ChannelSocket
    pub fn connect<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.connect(endpoint.as_ref())
    }

    /// # Disconnect a socket from an endpoint
    ///
    /// The [`disconnect()`] function shall disconnect a socket from the endpoint specified by the
    /// `endpoint` argument. Note the actual disconnect system call might occur at a later time.
    ///
    /// Upon disconnection the will also stop receiving messages originating from this endpoint.
    /// Moreover, the socket will no longer be able to queue outgoing messages to this endpoint.
    /// The outgoing message queue associated with the endpoint will be discarded. However, if the
    /// socket’s [`linger()`] period is non-zero, libzmq will still attempt to transmit these discarded
    /// messages, until the linger period expires.
    ///
    /// The `endpoint` argument is as described in [`connect()`]
    ///
    /// [`disconnect()`]: #method.disconnect
    /// [`linger()`]: #method.linger
    /// [`connect()`]: #method.connect
    pub fn disconnect<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.disconnect(endpoint.as_ref())
    }

    /// # monitor socket events
    ///
    /// The [`monitor()`] method lets an application thread track socket events (like connects) on
    /// a ZeroMQ socket. Each call to this method creates a [`Monitor`] socket connected to the
    /// socket.
    ///
    /// The `events` argument is a bitmask of the socket events you wish to monitor. To monitor all
    /// events, use the event value [`MonitorFlags::all()`]. NOTE: as new events are added, the
    /// catch-all value will start returning them. An application that relies on a strict and fixed
    /// sequence of events must not use [`MonitorFlags::all()`] in order to guarantee compatibility
    /// with future versions.
    ///
    /// [`monitor()`]: #method.monitor
    /// [`Monitor`]: MonitorSocket
    /// [`MonitorFlags::all()`]: MonitorFlags::all
    pub fn monitor<F>(&self, events: F) -> ZmqResult<MonitorSocket>
    where
        F: Into<MonitorFlags>,
    {
        let fd = self.get_sockopt_int::<usize>(SocketOption::FileDescriptor)?;
        let monitor_endpoint = format!("inproc://monitor.s-{fd}");

        self.socket
            .monitor(&monitor_endpoint, events.into().bits() as i32)?;

        let monitor = RawSocket::from_ctx(
            self.context.as_raw(),
            <Monitor as sealed::SocketType>::raw_socket_type() as i32,
        )?;

        monitor.connect(&monitor_endpoint)?;

        Ok(Socket {
            context: self.context.clone(),
            socket: monitor.into(),
            marker: PhantomData,
        })
    }

    /// # input/output multiplexing
    ///
    /// Poll this socket for input/output events.
    pub fn poll<E>(&self, events: E, timeout_ms: i64) -> ZmqResult<PollEvents>
    where
        E: Into<PollEvents>,
    {
        self.socket
            .poll(events.into(), timeout_ms)?
            .try_into()
            .map(PollEvents::from_bits_truncate)
            .map_err(|_err| ZmqError::InvalidArgument)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
#[from(u16)]
/// Flags for setting up monitor sockets
pub struct MonitorFlags(u16);

bitflags! {
    impl MonitorFlags: u16 {
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
        const Connected                 = 0b0000_0000_0000_0001;
        /// A connect request on the socket is pending. The event value is unspecified.
        const ConnectDelayed            = 0b0000_0000_0000_0010;
        /// A connect request failed, and is now being retried. The event value is the reconnect
        /// interval in milliseconds.
        ///
        /// Note that the reconnect interval is recalculated at each retry.
        const ConnectRetried            = 0b0000_0000_0000_0100;
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
        const Listening                 = 0b0000_0000_0000_1000;
        /// The socket could not bind to a given interface. The event value is the errno generated
        /// by the system bind call.
        const BindFailed                = 0b0000_0000_0001_0000;
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
        const Accepted                  = 0b0000_0000_0010_0000;
        /// The socket has rejected a connection from a remote peer. The event value is the errno
        /// generated by the accept call.
        const AcceptFailed              = 0b0000_0000_0100_0000;
        /// The socket was closed. The event value is the FD of the (now closed) network socket.
        const Closed                    = 0b0000_0000_1000_0000;
        /// The socket close failed. The event value is the errno returned by the system call.
        ///
        /// Note that this event occurs only on IPC transports.
        const CloseFailed               = 0b0000_0001_0000_0000;
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
        const Disconnected              = 0b0000_0010_0000_0000;
        /// Monitoring on this socket ended.
        const MonitorStopped            = 0b0000_0100_0000_0000;
        /// Unspecified error during handshake. The event value is an errno.
        const HandshakeFailedNoDetail   = 0b0000_1000_0000_0000;
        /// The ZMTP security mechanism handshake succeeded. The event value is unspecified.
        const HandshakeSucceeded        = 0b0001_0000_0000_0000;
        /// The ZMTP security mechanism handshake failed due to some mechanism protocol error,
        /// either between the ZMTP mechanism peers, or between the mechanism server and the ZAP
        /// handler. This indicates a configuration or implementation error in either peer resp.
        /// the ZAP handler.
        const HandshakeFailedProtocol   = 0b0010_0000_0000_0000;
        /// The ZMTP security mechanism handshake failed due to an authentication failure. The
        /// event value is the status code returned by the ZAP handler (i.e. `300`, `400` or `500`).
        const HandshakeFailedAuth       = 0b0100_0000_0000_0000;
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
/// Flag options for receive operations
pub struct RecvFlags(i32);

bitflags! {
    impl RecvFlags: i32 {
        /// Specifies that the operation should be performed in non-blocking mode.
        const DONT_WAIT = 0b00000001;
    }
}

#[cfg_attr(feature = "futures", async_trait)]
/// Trait for receiving single part messages
pub trait Receiver {
    fn recv_msg<F>(&self, flags: F) -> ZmqResult<Message>
    where
        F: Into<RecvFlags> + Copy;

    #[cfg(feature = "futures")]
    #[doc(cfg(feature = "futures"))]
    async fn recv_msg_async(&self) -> Option<Message>;
}

#[cfg_attr(feature = "futures", async_trait)]
impl<T> Receiver for Socket<T>
where
    T: sealed::SocketType + sealed::ReceiverFlag + Unpin,
    Socket<T>: Sync,
{
    fn recv_msg<F>(&self, flags: F) -> ZmqResult<Message>
    where
        F: Into<RecvFlags> + Copy,
    {
        self.socket
            .recv(flags.into().bits())
            .map(Message::from_raw_msg)
    }

    #[cfg(feature = "futures")]
    async fn recv_msg_async(&self) -> Option<Message> {
        futures::MessageReceivingFuture { receiver: self }.now_or_never()
    }
}

#[cfg_attr(feature = "futures", async_trait)]
/// Trait for receiving multipart messages
pub trait MultipartReceiver: Receiver {
    fn recv_multipart<F>(&self, flags: F) -> ZmqResult<MultipartMessage>
    where
        F: Into<RecvFlags> + Copy,
    {
        iter::repeat_with(|| self.recv_msg(flags))
            .try_fold(
                MultipartMessage::new(),
                |mut parts, zmq_result| match zmq_result {
                    Err(e) => ControlFlow::Break(Err(e)),
                    Ok(zmq_msg) => {
                        let got_more = zmq_msg.get_more();
                        parts.push_back(zmq_msg);
                        if got_more {
                            ControlFlow::Continue(parts)
                        } else {
                            ControlFlow::Break(Ok(parts))
                        }
                    }
                },
            )
            .break_value()
            .unwrap()
    }

    #[cfg(feature = "futures")]
    #[doc(cfg(feature = "futures"))]
    async fn recv_multipart_async(&self) -> MultipartMessage {
        ::futures::stream::repeat_with(|| Ok(self.recv_msg_async()))
            .try_fold(MultipartMessage::new(), |mut parts, zmq_msg| async move {
                if let Some(msg) = zmq_msg.await {
                    let got_more = msg.get_more();
                    parts.push_back(msg);
                    if !got_more {
                        return Err(parts);
                    }
                }
                Ok(parts)
            })
            .await
            .unwrap_err()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
/// Flag options for send operations
pub struct SendFlags(i32);

bitflags! {
    impl SendFlags: i32 {
        /// For socket types ([`Dealer`], [`Push`]) that block (either with [`Immediate`] option
        /// set and no peer available, or all peers having full high-water mark), specifies that
        /// the operation should be performed in non-blocking mode.
        ///
        /// [`Dealer`]: DealerSocket
        /// [`Push`]: PushSocket
        /// [`Immediate`]: SocketOption::Immediate
        const DONT_WAIT = 0b00000001;
        /// Specifies that the message being sent is a multi-part message, and that further message
        /// parts are to follow.
        const SEND_MORE = 0b00000010;
    }
}

#[cfg_attr(feature = "futures", async_trait)]
/// Trait for sending single part messages
pub trait Sender {
    fn send_msg<M, F>(&self, msg: M, flags: F) -> ZmqResult<()>
    where
        M: Into<Message>,
        F: Into<SendFlags> + Copy;

    #[cfg(feature = "futures")]
    #[doc(cfg(feature = "futures"))]
    async fn send_msg_async<M, F>(&self, msg: M, flags: F) -> Option<()>
    where
        M: Into<Message> + Clone + Send,
        F: Into<SendFlags> + Copy + Send;
}

#[cfg_attr(feature = "futures", async_trait)]
impl<T> Sender for Socket<T>
where
    T: sealed::SocketType + sealed::SenderFlag + Unpin,
    Socket<T>: Sync,
{
    fn send_msg<M, F>(&self, msg: M, flags: F) -> ZmqResult<()>
    where
        M: Into<Message>,
        F: Into<SendFlags> + Copy,
    {
        msg.into().send(self, flags.into().bits())
    }

    #[cfg(feature = "futures")]
    #[doc(cfg(feature = "futures"))]
    async fn send_msg_async<M, F>(&self, msg: M, flags: F) -> Option<()>
    where
        M: Into<Message> + Clone + Send,
        F: Into<SendFlags> + Copy + Send,
    {
        futures::MessageSendingFuture {
            receiver: self,
            message: msg,
            flags: flags.into(),
        }
        .now_or_never()
    }
}

#[cfg_attr(feature = "futures", async_trait)]
/// Trait for sending multipart messages
pub trait MultipartSender: Sender {
    fn send_multipart<M, F>(&self, iter: M, flags: F) -> ZmqResult<()>
    where
        M: Into<MultipartMessage>,
        F: Into<SendFlags> + Copy,
    {
        let mut last_part: Option<Message> = None;
        for part in iter.into() {
            let maybe_last = last_part.take();
            if let Some(last) = maybe_last {
                self.send_msg(last, flags.into() | SendFlags::SEND_MORE)?;
            }
            last_part = Some(part);
        }
        if let Some(last) = last_part {
            self.send_msg(last, flags)
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "futures")]
    #[doc(cfg(feature = "futures"))]
    async fn send_multipart_async<M, F>(&self, multipart: M, flags: F) -> Option<()>
    where
        M: Into<MultipartMessage> + Send,
        F: Into<SendFlags> + Copy + Send,
    {
        let mut last_part = None;
        for part in multipart.into() {
            let maybe_last = last_part.take();
            if let Some(last) = maybe_last {
                self.send_msg_async(last, flags.into() | SendFlags::SEND_MORE)
                    .await?;
            }
            last_part = Some(part);
        }
        if let Some(last) = last_part {
            self.send_msg_async(last, flags.into()).await
        } else {
            None
        }
    }
}

#[cfg(feature = "futures")]
mod futures {
    use core::{pin::Pin, task::Poll};

    use super::{RecvFlags, SendFlags, Socket};
    use crate::{
        message::{Message, Sendable},
        sealed,
    };

    pub(super) struct MessageSendingFuture<'a, T, M>
    where
        T: sealed::SocketType + sealed::SenderFlag + Unpin,
        M: Into<Message> + Clone + Send,
    {
        pub(super) receiver: &'a Socket<T>,
        pub(super) message: M,
        pub(super) flags: SendFlags,
    }

    impl<'a, T, M> Future for MessageSendingFuture<'a, T, M>
    where
        T: sealed::SocketType + sealed::SenderFlag + Unpin,
        M: Into<Message> + Clone + Send,
    {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _ctx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
            let message = self.message.clone().into();

            message
                .send(self.receiver, self.flags.bits())
                .map_or(Poll::Pending, Poll::Ready)
        }
    }

    pub(super) struct MessageReceivingFuture<'a, T>
    where
        T: sealed::SocketType + sealed::ReceiverFlag + Unpin,
    {
        pub(super) receiver: &'a Socket<T>,
    }

    impl<T> Future for MessageReceivingFuture<'_, T>
    where
        T: sealed::SocketType + sealed::ReceiverFlag + Unpin,
    {
        type Output = Message;

        fn poll(self: Pin<&mut Self>, _ctx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
            self.receiver
                .socket
                .recv(RecvFlags::DONT_WAIT.bits())
                .map(Message::from_raw_msg)
                .map_or(Poll::Pending, Poll::Ready)
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
/// Flags for poll operations on sockets
pub struct PollEvents(i16);

bitflags! {
    impl PollEvents: i16 {
        /// For 0MQ sockets, at least one message may be received from the `Socket` without
        /// blocking. For standard sockets this is equivalent to the `POLLIN` flag of the `poll()`
        /// system call and generally means that at least one byte of data may be read from `fd`
        /// without blocking.
        const POLL_IN = 0b0000_0001;
        /// For 0MQ sockets, at least one message may be sent to the `Socket` without blocking. For
        /// standard sockets this is equivalent to the `POLLOUT` flag of the `poll()` system call
        /// and generally means that at least one byte of data may be written to `fd` without
        /// blocking.
        const POLL_OUT = 0b0000_0010;
        /// For standard sockets, this flag is passed to the underlying `poll()` system call and
        /// generally means that some sort of error condition is present on the socket specified by
        /// `fd`. For 0MQ sockets this flag has no effect if set in `events`.
        const POLL_ERR = 0b0000_0100;
        /// For 0MQ sockets this flags is of no use. For standard sockets this means there isurgent data to read. Refer to the POLLPRI flag for more information. For filedescriptor, refer to your use case: as an example, GPIO interrupts are signaled througha POLLPRI event. This flag has no effect on Windows.
        const POLL_PRI = 0b0000_1000;
    }
}

#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
/// Flags for the [`ReconnectStop`] socket option
///
/// [`ReconnectStop`]: SocketOption::ReconnectStop
pub struct ReconnectStop(i32);

#[cfg(feature = "draft-api")]
bitflags! {
    impl ReconnectStop: i32 {
        /// The [`CONNECTION_REFUSED`] option will stop reconnection when 0MQ receives the
        /// [`ConnectionRefused`] return code from the connect. This indicates that there is no
        /// code bound to the specified endpoint.
        ///
        /// [`CONNECTION_REFUSED`]: ReconnectStop::CONNECTION_REFUSED
        /// [`ConnectionRefused`]: crate::ZmqError::ConnectionRefused
        const CONNECTION_REFUSED = zmq_sys_crate::ZMQ_RECONNECT_STOP_CONN_REFUSED as i32;
        /// The [`HANDSHAKE_FAILED`] option will stop reconnection if the 0MQ handshake fails. This
        /// can be used to detect and/or prevent errant connection attempts to non-0MQ sockets.
        /// Note that when specifying this option you may also want to set [`HandshakeInterval`]
        /// — the default handshake interval is 30000 (30 seconds), which is typically too large.
        ///
        /// [`HANDSHAKE_FAILED`]: ReconnectStop::HANDSHAKE_FAILED
        /// [`HandshakeInterval`]: SocketOption::HandshakeInterval
        const HANDSHAKE_FAILED = zmq_sys_crate::ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED as i32;
        /// The [`AFTER_DISCONNECT`] option will stop reconnection when `disconnect()` has been
        /// called. This can be useful when the user’s request failed (server not ready), as the
        /// socket does not need to continue to reconnect after user disconnect actively.
        ///
        /// [`AFTER_DISCONNECT`]: ReconnectStop::AFTER_DISCONNECT
        const AFTER_DISCONNECT = zmq_sys_crate::ZMQ_RECONNECT_STOP_AFTER_DISCONNECT as i32;
}}

#[cfg(test)]
mod socket_tests {
    use std::{thread, time::Duration};

    #[cfg(feature = "draft-api")]
    use rstest::*;

    #[cfg(feature = "draft-api")]
    use super::ReconnectStop;
    use super::{
        DealerSocket, MonitorFlags, MonitorSocketEvent, PairSocket, PollEvents, SendFlags,
    };
    use crate::{
        prelude::{Context, MonitorReceiver, Sender, ZmqResult},
        security::SecurityMechanism,
    };

    #[test]
    fn set_affinity_sets_affinity() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_affinity(42)?;

        assert_eq!(socket.affinity()?, 42);

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_backlog_sets_backlog() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_backlog(42)?;

        assert_eq!(socket.backlog()?, 42);

        Ok(())
    }

    #[test]
    fn set_connect_timeout_sets_connect_timeout() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_connect_timeout(42)?;

        assert_eq!(socket.connect_timeout()?, 42);

        Ok(())
    }

    #[test]
    fn events_when_no_events_available() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;

        assert_eq!(socket.events()?, PollEvents::empty());

        Ok(())
    }

    #[test]
    fn events_when_connected() -> ZmqResult<()> {
        let context = Context::new()?;

        let endpoint = "inproc://test";
        let server_socket = PairSocket::from_context(&context)?;
        server_socket.bind(endpoint)?;

        let client_socket = PairSocket::from_context(&context)?;
        client_socket.connect(endpoint)?;

        assert_eq!(client_socket.events()?, PollEvents::POLL_OUT);

        Ok(())
    }

    #[test]
    fn set_handshake_interval_sets_handshake_interval() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_handshake_interval(42)?;

        assert_eq!(socket.handshake_interval()?, 42);

        Ok(())
    }

    #[test]
    fn set_heartbeat_interval_sets_heartbeat_interval() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_heartbeat_interval(42)?;

        assert_eq!(socket.heartbeat_interval()?, 42);

        Ok(())
    }

    #[test]
    fn set_heartbeat_timeout_sets_heartbeat_timeout() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_heartbeat_timeout(42)?;

        assert_eq!(socket.heartbeat_timeout()?, 42);

        Ok(())
    }

    #[test]
    fn set_heartbeat_timetolive_sets_heartbeat_ttl() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_heartbeat_timetolive(42_000)?;

        assert_eq!(socket.heartbeat_timetolive()?, 42_000);

        Ok(())
    }

    #[test]
    fn set_immediate_sets_immediate() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_immediate(true)?;

        assert!(socket.immediate()?);

        Ok(())
    }

    #[test]
    fn set_ipv6_sets_ipv6() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_ipv6(true)?;

        assert!(socket.ipv6()?);

        Ok(())
    }

    #[test]
    fn set_linger_sets_linger() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_linger(42)?;

        assert_eq!(socket.linger()?, 42);

        Ok(())
    }

    #[test]
    fn last_endpoint_when_not_bound_or_connected() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;

        assert_eq!(socket.last_endpoint()?, "");

        Ok(())
    }

    #[test]
    fn last_endpoint_when_bound() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.bind("inproc://last-endpoint-test")?;

        assert_eq!(socket.last_endpoint()?, "inproc://last-endpoint-test");

        Ok(())
    }

    #[test]
    fn set_max_msg_size_sets_max_msg_size() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_max_message_size(42)?;

        assert_eq!(socket.max_message_size()?, 42);

        Ok(())
    }

    #[test]
    fn set_security_mechanism_set_security_mechanism() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_security_mechanism(&SecurityMechanism::Plain {
            username: "username".into(),
            password: "supersecret".into(),
        })?;

        assert_eq!(
            socket.security_mechanism()?,
            SecurityMechanism::Plain {
                username: "username".into(),
                password: "supersecret".into()
            }
        );

        Ok(())
    }

    #[test]
    fn set_multicast_hops_sets_multicast_hops() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_multicast_hops(42)?;

        assert_eq!(socket.multicast_hops()?, 42);

        Ok(())
    }

    #[test]
    fn set_rate_sets_rate() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_rate(42)?;

        assert_eq!(socket.rate()?, 42);

        Ok(())
    }

    #[test]
    fn set_receive_buffer_sets_receive_buffer() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_receive_buffer(42)?;

        assert_eq!(socket.receive_buffer()?, 42);

        Ok(())
    }

    #[test]
    fn set_receive_high_watermark_sets_receive_high_watermark() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_receive_highwater_mark(42)?;

        assert_eq!(socket.receive_highwater_mark()?, 42);

        Ok(())
    }

    #[test]
    fn set_receive_timeout_sets_receive_timeout() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_receive_timeout(42)?;

        assert_eq!(socket.receive_timeout()?, 42);

        Ok(())
    }

    #[test]
    fn set_reconnect_interval_sets_reconnect_interval() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_reconnect_interval(42)?;

        assert_eq!(socket.reconnect_interval()?, 42);

        Ok(())
    }

    #[test]
    fn set_reconnect_interval_max_sets_reconnect_interval_max() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_reconnect_interval_max(42)?;

        assert_eq!(socket.reconnect_interval_max()?, 42);

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_reconnect_stop_sets_reconnect_stop() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_reconnect_stop(
            ReconnectStop::AFTER_DISCONNECT | ReconnectStop::CONNECTION_REFUSED,
        )?;

        assert_eq!(
            socket.reconnect_stop()?,
            ReconnectStop::AFTER_DISCONNECT | ReconnectStop::CONNECTION_REFUSED
        );

        Ok(())
    }

    #[test]
    fn set_recoveery_interval_sets_recovery_interval() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_recovery_interval(42)?;

        assert_eq!(socket.recovery_interval()?, 42);

        Ok(())
    }

    #[test]
    fn set_send_buffer_sets_send_buffer() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_send_buffer(42)?;

        assert_eq!(socket.send_buffer()?, 42);

        Ok(())
    }

    #[test]
    fn set_send_high_watermark_sets_send_high_watermark() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_send_highwater_mark(42)?;

        assert_eq!(socket.send_highwater_mark()?, 42);

        Ok(())
    }

    #[test]
    fn set_send_timeout_sets_send_timeout() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_send_timeout(42)?;

        assert_eq!(socket.send_timeout()?, 42);

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[rstest]
    #[case(None)]
    #[case(Some("asdf"))]
    fn set_socks_proxy_sets_proxy_value(#[case] socks_proxy: Option<&str>) -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_socks_proxy(socks_proxy)?;

        assert_eq!(socket.socks_proxy()?, socks_proxy.unwrap_or(""));

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_socks_username_sets_proxy_username() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_socks_username("username")?;

        assert_eq!(socket.socks_username()?, "username");

        Ok(())
    }

    #[cfg(feature = "draft-api")]
    #[test]
    fn set_socks_password_sets_proxy_password() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_socks_password("password")?;

        assert_eq!(socket.socks_password()?, "password");

        Ok(())
    }

    #[test]
    fn set_tcp_keepalive_sets_tcp_keepalive() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_tcp_keepalive(1)?;

        assert_eq!(socket.tcp_keepalive()?, 1);

        Ok(())
    }

    #[test]
    fn set_tcp_keepalive_count_sets_tcp_keepalive_count() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_tcp_keepalive_count(42)?;

        assert_eq!(socket.tcp_keepalive_count()?, 42);

        Ok(())
    }

    #[test]
    fn set_tcp_keepalive_idle_sets_tcp_keepalive_idle() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_tcp_keepalive_idle(42)?;

        assert_eq!(socket.tcp_keepalive_idle()?, 42);

        Ok(())
    }

    #[test]
    fn set_tcp_keepalive_interval_sets_tcp_keepalive_interval() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_tcp_keepalive_interval(42)?;

        assert_eq!(socket.tcp_keepalive_interval()?, 42);

        Ok(())
    }

    #[test]
    fn set_tcp_max_retransmit_timout_set_retransmit_timeout() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_tcp_max_retransmit_timeout(42)?;

        assert_eq!(socket.tcp_max_retransmit_timeout()?, 42);

        Ok(())
    }

    #[test]
    fn set_type_of_service_sets_type_of_service() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_type_of_service(42)?;

        assert_eq!(socket.type_of_service()?, 42);

        Ok(())
    }

    #[test]
    fn set_zap_domain_sets_zap_domain() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;
        socket.set_zap_domain(&"zap".into())?;

        assert_eq!(socket.zap_domain()?, "zap".into());

        Ok(())
    }

    #[test]
    fn unbind_unbinds_endpoint() -> ZmqResult<()> {
        let context = Context::new()?;

        let endpoint = "inproc://unbind-test";

        let socket = PairSocket::from_context(&context)?;
        socket.bind(endpoint)?;

        assert!(socket.unbind(endpoint).is_ok());

        Ok(())
    }

    #[test]
    fn connect_connects_to_endpoint() -> ZmqResult<()> {
        let context = Context::new()?;

        let endpoint = "inproc://connect-test";

        let server_socket = PairSocket::from_context(&context)?;
        server_socket.bind(endpoint)?;

        let client_socket = PairSocket::from_context(&context)?;
        assert!(client_socket.connect(endpoint).is_ok());

        Ok(())
    }

    #[test]
    fn disconnect_disconnects_from_endpoint() -> ZmqResult<()> {
        let context = Context::new()?;

        let endpoint = "inproc://disconnect-test";
        let server_socket = PairSocket::from_context(&context)?;
        server_socket.bind(endpoint)?;

        let client_socket = PairSocket::from_context(&context)?;
        client_socket.connect(endpoint)?;
        assert!(client_socket.disconnect(endpoint).is_ok());

        Ok(())
    }

    #[test]
    fn monitor_sets_up_socket_monitor() -> ZmqResult<()> {
        let context = Context::new()?;

        let dealer_server = DealerSocket::from_context(&context)?;
        dealer_server.bind("tcp://127.0.0.1:*")?;
        let client_endpoint = dealer_server.last_endpoint()?;

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(10));
            }
        });

        let dealer_client = DealerSocket::from_context(&context)?;
        let dealer_monitor = dealer_client.monitor(MonitorFlags::Connected)?;

        dealer_client.connect(client_endpoint)?;

        loop {
            match dealer_monitor.recv_monitor_event() {
                Err(_) => continue,
                Ok(event) => {
                    assert_eq!(event, MonitorSocketEvent::Connected);
                    break;
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "futures")]
    #[test]
    fn monitor_sets_up_async_socket_monitor() -> ZmqResult<()> {
        let context = Context::new()?;

        let dealer_server = DealerSocket::from_context(&context)?;
        dealer_server.bind("tcp://127.0.0.1:*")?;
        let client_endpoint = dealer_server.last_endpoint()?;

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(10));
            }
        });

        futures::executor::block_on(async {
            let dealer_client = DealerSocket::from_context(&context)?;
            let dealer_monitor = dealer_client.monitor(MonitorFlags::Connected)?;

            dealer_client.connect(client_endpoint)?;

            loop {
                match dealer_monitor.recv_monitor_event_async().await {
                    Some(event) => {
                        assert_eq!(event, MonitorSocketEvent::Connected);
                        break;
                    }
                    _ => continue,
                }
            }

            Ok(())
        })
    }

    #[test]
    fn poll_on_socket_when_no_events_available() -> ZmqResult<()> {
        let context = Context::new()?;

        let socket = PairSocket::from_context(&context)?;

        assert_eq!(socket.poll(PollEvents::all(), 0)?, PollEvents::empty());

        Ok(())
    }

    #[test]
    fn poll_on_socket_when_event_available() -> ZmqResult<()> {
        let context = Context::new()?;

        let endpoint = "inproc://poll-test";
        let pair_server = PairSocket::from_context(&context)?;
        pair_server.bind(endpoint)?;

        let pair_client = PairSocket::from_context(&context)?;
        pair_client.connect(endpoint)?;

        pair_server.send_msg("msg1", SendFlags::empty())?;
        pair_server.send_msg("msg2", SendFlags::empty())?;
        pair_server.send_msg("msg3", SendFlags::empty())?;

        assert_eq!(pair_client.poll(PollEvents::all(), 0)?, PollEvents::POLL_IN);

        Ok(())
    }
}

#[cfg(feature = "builder")]
pub(crate) mod builder {
    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use crate::{
        ZmqResult, auth::ZapDomain, context::Context, sealed, security::SecurityMechanism,
        socket::Socket,
    };

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(
        pattern = "owned",
        name = "SocketBuilder",
        public,
        build_fn(skip, error = "ZmqError"),
        derive(PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)
    )]
    #[builder_struct_attr(doc = "Builder for [`Socket`].\n\n")]
    #[allow(dead_code)]
    struct SocketConfig {
        #[cfg(feature = "draft-api")]
        #[doc(cfg(feature = "draft-api"))]
        #[builder(default = false)]
        busy_poll: bool,
        #[builder(setter(into), default = 0)]
        connect_timeout: i32,
        #[builder(setter(into), default = 30_000)]
        handshake_interval: i32,
        #[builder(setter(into), default = 0)]
        heartbeat_interval: i32,
        #[builder(setter(into), default = 0)]
        heartbeat_timeout: i32,
        #[builder(setter(into), default = 0)]
        heartbeat_timetolive: i32,
        #[builder(default = false)]
        immediate: bool,
        #[builder(default = false)]
        ipv6: bool,
        #[builder(setter(into), default = 0)]
        linger: i32,
        #[builder(setter(into), default = -1)]
        max_message_size: i64,
        #[builder(setter(into), default = -1)]
        receive_buffer: i32,
        #[builder(setter(into), default = 1_000)]
        receive_highwater_mark: i32,
        #[builder(setter(into), default = -1)]
        receive_timeout: i32,
        #[builder(setter(into), default = 100)]
        reconnect_interval: i32,
        #[builder(setter(into), default = 0)]
        reconnect_interval_max: i32,
        #[builder(setter(into), default = -1)]
        send_buffer: i32,
        #[builder(setter(into), default = 1_000)]
        send_highwater_mark: i32,
        #[builder(setter(into), default = -1)]
        send_timeout: i32,
        #[builder(setter(into))]
        zap_domain: ZapDomain,
        #[builder(default = "SecurityMechanism::Null")]
        security_mechanism: SecurityMechanism,
    }

    impl SocketBuilder {
        /// Applies this builder to the provided socket
        pub fn apply<T>(self, socket: &Socket<T>) -> ZmqResult<()>
        where
            T: sealed::SocketType,
        {
            #[cfg(feature = "draft-api")]
            self.busy_poll
                .iter()
                .try_for_each(|busy_poll| socket.set_busy_poll(*busy_poll))?;

            self.connect_timeout
                .iter()
                .try_for_each(|connect_timeout| socket.set_connect_timeout(*connect_timeout))?;

            self.handshake_interval
                .iter()
                .try_for_each(|handshake_interval| {
                    socket.set_handshake_interval(*handshake_interval)
                })?;

            self.heartbeat_interval
                .iter()
                .try_for_each(|heartbeat_interval| {
                    socket.set_heartbeat_interval(*heartbeat_interval)
                })?;

            self.heartbeat_timeout
                .iter()
                .try_for_each(|heartbeat_timeout| {
                    socket.set_heartbeat_timeout(*heartbeat_timeout)
                })?;

            self.heartbeat_timetolive
                .iter()
                .try_for_each(|heartbeat_timetolive| {
                    socket.set_heartbeat_timetolive(*heartbeat_timetolive)
                })?;

            self.immediate
                .iter()
                .try_for_each(|immediate| socket.set_immediate(*immediate))?;

            self.ipv6
                .iter()
                .try_for_each(|ipv6| socket.set_ipv6(*ipv6))?;

            self.linger
                .iter()
                .try_for_each(|linger| socket.set_linger(*linger))?;

            self.max_message_size
                .iter()
                .try_for_each(|max_message_size| socket.set_max_message_size(*max_message_size))?;

            self.receive_buffer
                .iter()
                .try_for_each(|receive_buffer| socket.set_receive_buffer(*receive_buffer))?;

            self.receive_highwater_mark
                .iter()
                .try_for_each(|receive_highwater_mark| {
                    socket.set_receive_highwater_mark(*receive_highwater_mark)
                })?;

            self.receive_timeout
                .iter()
                .try_for_each(|receive_timeout| socket.set_receive_timeout(*receive_timeout))?;

            self.reconnect_interval
                .iter()
                .try_for_each(|reconnect_interval| {
                    socket.set_reconnect_interval(*reconnect_interval)
                })?;

            self.reconnect_interval_max
                .iter()
                .try_for_each(|reconnect_interval_max| {
                    socket.set_reconnect_interval_max(*reconnect_interval_max)
                })?;

            self.send_buffer
                .iter()
                .try_for_each(|send_buffer| socket.set_send_buffer(*send_buffer))?;

            self.send_highwater_mark
                .iter()
                .try_for_each(|send_highwater_mark| {
                    socket.set_send_highwater_mark(*send_highwater_mark)
                })?;

            self.send_timeout
                .iter()
                .try_for_each(|send_timeout| socket.set_send_timeout(*send_timeout))?;

            self.zap_domain
                .iter()
                .try_for_each(|zap_domain| socket.set_zap_domain(zap_domain))?;

            self.security_mechanism
                .iter()
                .try_for_each(|security_mechanism| {
                    socket.set_security_mechanism(security_mechanism)
                })?;

            Ok(())
        }

        pub fn build_from_context<T>(self, context: &Context) -> ZmqResult<Socket<T>>
        where
            T: sealed::SocketType,
        {
            let socket = Socket::<T>::from_context(context)?;

            self.apply(&socket)?;

            Ok(socket)
        }
    }

    #[cfg(test)]
    mod socket_builder_tests {
        use super::SocketBuilder;
        use crate::{
            auth::ZapDomain,
            prelude::{Context, PairSocket, ZmqResult},
            security::SecurityMechanism,
        };

        #[test]
        fn default_socket_builder() -> ZmqResult<()> {
            let context = Context::new()?;

            let builder = SocketBuilder::default();
            let socket = PairSocket::from_context(&context)?;

            builder.apply(&socket)?;

            assert_eq!(socket.connect_timeout()?, 0);
            assert_eq!(socket.handshake_interval()?, 30_000);
            assert_eq!(socket.heartbeat_interval()?, 0);
            assert_eq!(socket.heartbeat_timeout()?, -1);
            assert_eq!(socket.heartbeat_timetolive()?, 0);
            assert!(!socket.immediate()?);
            assert!(!socket.ipv6()?);
            assert_eq!(socket.linger()?, -1);
            assert_eq!(socket.max_message_size()?, -1);
            assert_eq!(socket.receive_buffer()?, -1);
            assert_eq!(socket.receive_highwater_mark()?, 1_000);
            assert_eq!(socket.receive_timeout()?, -1);
            assert_eq!(socket.reconnect_interval()?, 100);
            assert_eq!(socket.reconnect_interval_max()?, 0);
            assert_eq!(socket.send_buffer()?, -1);
            assert_eq!(socket.send_highwater_mark()?, 1_000);
            assert_eq!(socket.send_timeout()?, -1);
            assert_eq!(socket.zap_domain()?, ZapDomain::new("".into()));
            assert_eq!(socket.security_mechanism()?, SecurityMechanism::Null);

            Ok(())
        }

        #[test]
        fn builder_with_custom_setttings() -> ZmqResult<()> {
            let context = Context::new()?;

            let builder = SocketBuilder::default()
                .connect_timeout(42)
                .handshake_interval(21)
                .heartbeat_interval(666)
                .heartbeat_timeout(1337)
                .heartbeat_timetolive(420)
                .immediate(true)
                .ipv6(true)
                .linger(1337)
                .max_message_size(1337)
                .receive_buffer(1337)
                .receive_highwater_mark(1337)
                .receive_timeout(1337)
                .reconnect_interval(1337)
                .reconnect_interval_max(1337)
                .send_buffer(1337)
                .send_highwater_mark(1337)
                .send_timeout(1337)
                .zap_domain(ZapDomain::new("test".into()))
                .security_mechanism(SecurityMechanism::Plain {
                    username: "username".into(),
                    password: "supersecret".into(),
                });
            let socket = PairSocket::from_context(&context)?;

            builder.apply(&socket)?;

            assert_eq!(socket.connect_timeout()?, 42);
            assert_eq!(socket.handshake_interval()?, 21);
            assert_eq!(socket.heartbeat_interval()?, 666);
            assert_eq!(socket.heartbeat_timeout()?, 1337);
            assert_eq!(socket.heartbeat_timetolive()?, 400);
            assert!(socket.immediate()?);
            assert!(socket.ipv6()?);
            assert_eq!(socket.linger()?, 1337);
            assert_eq!(socket.max_message_size()?, 1337);
            assert_eq!(socket.receive_buffer()?, 1337);
            assert_eq!(socket.receive_highwater_mark()?, 1337);
            assert_eq!(socket.receive_timeout()?, 1337);
            assert_eq!(socket.reconnect_interval()?, 1337);
            assert_eq!(socket.reconnect_interval_max()?, 1337);
            assert_eq!(socket.send_buffer()?, 1337);
            assert_eq!(socket.send_highwater_mark()?, 1337);
            assert_eq!(socket.send_timeout()?, 1337);
            assert_eq!(socket.zap_domain()?, ZapDomain::new("test".into()));
            assert_eq!(
                socket.security_mechanism()?,
                SecurityMechanism::Plain {
                    username: "username".into(),
                    password: "supersecret".into()
                }
            );

            Ok(())
        }
    }
}
