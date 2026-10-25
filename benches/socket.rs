//! Baseline benchmarks for the hot paths of the socket API.
//!
//! Run with `cargo bench` or `cargo criterion`; compare against a saved baseline with
//! `cargo bench -- --save-baseline before` and `cargo bench -- --baseline before`.

use std::{hint::black_box, thread};

use arzmq::{
    prelude::{
        Context, Message, MultipartMessage, MultipartReceiver, MultipartSender, PairSocket,
        Receiver, RecvFlags, SendFlags, Sender, ZmqResult,
    },
    socket::PollEvents,
};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

const SIZES: [usize; 3] = [16, 1024, 65536];

fn pair(context: &Context, endpoint: &str) -> ZmqResult<(PairSocket, PairSocket)> {
    let server = PairSocket::from_context(context)?;
    server.bind(endpoint)?;
    let client = PairSocket::from_context(context)?;
    client.connect(endpoint)?;
    Ok((server, client))
}

fn message(c: &mut Criterion) {
    let mut group = c.benchmark_group("message");
    for size in SIZES {
        let payload = vec![0xA5_u8; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("from_slice", size), &payload, |b, p| {
            b.iter(|| Message::from(black_box(p.as_slice())));
        });

        let msg = Message::from(payload.as_slice());
        group.bench_with_input(BenchmarkId::new("clone", size), &msg, |b, m| {
            b.iter(|| black_box(m).clone());
        });
    }
    group.finish();
}

fn send_recv_same_thread(c: &mut Criterion) {
    let context = Context::new().unwrap();
    let (server, client) = pair(&context, "inproc://bench-same-thread").unwrap();

    let mut group = c.benchmark_group("pair_inproc_send_recv");
    for size in SIZES {
        let payload = vec![0xA5_u8; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &payload, |b, p| {
            b.iter(|| {
                client
                    .send_msg(black_box(p.as_slice()), SendFlags::empty())
                    .unwrap();
                server.recv_msg(RecvFlags::empty()).unwrap()
            });
        });
    }
    group.finish();
}

fn send_recv_multipart(c: &mut Criterion) {
    let context = Context::new().unwrap();
    let (server, client) = pair(&context, "inproc://bench-multipart").unwrap();
    let payload = vec![0xA5_u8; 256];

    let mut group = c.benchmark_group("pair_inproc_multipart");
    group.throughput(Throughput::Bytes(3 * payload.len() as u64));
    group.bench_function("three_parts", |b| {
        b.iter(|| {
            let parts: Vec<Message> = (0..3).map(|_| Message::from(payload.as_slice())).collect();
            client
                .send_multipart(MultipartMessage::from(parts), SendFlags::empty())
                .unwrap();
            server.recv_multipart(RecvFlags::empty()).unwrap()
        });
    });
    group.finish();
}

fn ping_pong_threads(c: &mut Criterion) {
    let context = Context::new().unwrap();
    let (server, client) = pair(&context, "inproc://bench-ping-pong").unwrap();

    let echo = thread::spawn(move || {
        loop {
            let msg = server.recv_msg(RecvFlags::empty()).unwrap();
            let stop = msg.is_empty();
            server.send_msg(msg, SendFlags::empty()).unwrap();
            if stop {
                break;
            }
        }
    });

    let mut group = c.benchmark_group("pair_inproc_ping_pong");
    for size in SIZES {
        let payload = vec![0xA5_u8; size];
        group.throughput(Throughput::Bytes(2 * size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &payload, |b, p| {
            b.iter(|| {
                client
                    .send_msg(black_box(p.as_slice()), SendFlags::empty())
                    .unwrap();
                client.recv_msg(RecvFlags::empty()).unwrap()
            });
        });
    }
    group.finish();

    client.send_msg(&[] as &[u8], SendFlags::empty()).unwrap();
    client.recv_msg(RecvFlags::empty()).unwrap();
    echo.join().unwrap();
}

fn socket_operations(c: &mut Criterion) {
    let context = Context::new().unwrap();
    let (server, client) = pair(&context, "inproc://bench-operations").unwrap();

    let mut group = c.benchmark_group("socket");
    group.bench_function("poll_no_events", |b| {
        b.iter(|| server.poll(PollEvents::POLL_IN, 0).unwrap());
    });
    group.bench_function("events", |b| {
        b.iter(|| server.events().unwrap());
    });
    group.bench_function("get_linger", |b| {
        b.iter(|| server.linger().unwrap());
    });
    group.bench_function("set_linger", |b| {
        b.iter(|| server.set_linger(black_box(100)).unwrap());
    });
    group.bench_function("last_endpoint", |b| {
        b.iter(|| server.last_endpoint().unwrap());
    });

    client.send_msg("Hello", SendFlags::empty()).unwrap();
    group.bench_function("poll_event_available", |b| {
        b.iter(|| server.poll(PollEvents::POLL_IN, 0).unwrap());
    });
    group.finish();
}

fn endpoints(c: &mut Criterion) {
    let context = Context::new().unwrap();
    let socket = PairSocket::from_context(&context).unwrap();
    socket.set_linger(0).unwrap();

    let mut group = c.benchmark_group("endpoint");
    group.bench_function("connect_disconnect", |b| {
        b.iter(|| {
            socket.connect("tcp://127.0.0.1:1").unwrap();
            socket.disconnect("tcp://127.0.0.1:1").unwrap();
        });
    });
    group.bench_function("connect_endpoint_disconnect", |b| {
        b.iter(|| {
            socket
                .connect_endpoint("tcp://127.0.0.1:1")
                .unwrap()
                .disconnect()
                .unwrap();
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    message,
    send_recv_same_thread,
    send_recv_multipart,
    ping_pong_threads,
    socket_operations,
    endpoints
);
criterion_main!(benches);
