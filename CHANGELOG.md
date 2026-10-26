# Changelog

## 0.7.0

Contains breaking changes for downstream code, hence the minor bump of the `0.y.z` version.

### Breaking

- `Sender`, `Receiver`, `MonitorReceiver` are sealed (supertrait `sealed::Sealed`). They can still be used as bounds and
  called, but no longer be implemented outside of `arzmq`. This allows adding methods without further breaking changes.
- `ZmqError` is `#[non_exhaustive]` and now implements `Copy`, `Hash` and `From<ZmqError> for io::Error`.
- `get_sockopt_bytes` returns the option value without the trailing NUL byte. Use `get_sockopt_string` for string
  options. `routing_id()` getters read binary-safe.
- `Socket::poll` returns the occurred `PollEvents` instead of silently mapping the event count.
- The sys crates no longer compile with `-march=native` by default. Enable the new feature `native` to get it back.
- `protokit-sys`, `libnorm-sys` and `openpgm-sys` no longer have the `bindings` feature. The C++ headers of the first two
  cannot be processed by `bindgen`, and nothing used the generated bindings.
- `tokio`, `smol` and `smol-macros` are dev-dependencies. The features `examples-tokio` and `examples-smol` imply
  `futures`.

### Added

- Endpoint handles `BoundEndpoint` and `ConnectedEndpoint`, additive to `bind`/`connect`.
- `recv_msg_wait`/`send_msg_wait`/`send_multipart_wait` (`futures`, unix): wait until the operation can complete,
  safe with several concurrent waiters. `recv_msg_async` keeps returning immediately with `Again`.
- `MultipartStream`/`OwnedMultipartStream` (`Stream<Item = ZmqResult<MultipartMessage>>`), `MultipartSink` (`Sink`),
  `MonitorEventStream` (`futures`, unix).
- `Poller` to wait for events on several sockets at once.
- `routing_id_bytes`/`set_routing_id_bytes` (and `set_connect_routing_id_bytes`) for binary routing ids.
- `Message::as_bytes`, indexing and iteration of `MultipartMessage`.

### Fixed

- `libnorm-sys` no longer scans the self-referencing `vendor/norp` directory when packaging.
- Lost wake-ups with several tasks waiting on the same socket.
- Panics (`unreachable!`) on unexpected errno values in the ffi layer now surface as `ZmqError`.
- `proxy()` returns the error of `zmq_proxy` instead of panicking.
