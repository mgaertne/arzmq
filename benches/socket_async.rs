//! Baseline benchmarks for the `futures` based send and receive operations.

use std::hint::black_box;

use arzmq::prelude::{Context, PairSocket, Receiver, SendFlags, Sender, ZmqResult};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::executor::block_on;

const SIZES: [usize; 3] = [16, 1024, 65536];

fn pair(context: &Context, endpoint: &str) -> ZmqResult<(PairSocket, PairSocket)> {
    let server = PairSocket::from_context(context)?;
    server.bind(endpoint)?;
    let client = PairSocket::from_context(context)?;
    client.connect(endpoint)?;
    Ok((server, client))
}

fn recv_msg_async_empty(c: &mut Criterion) {
    let context = Context::new().unwrap();
    let (server, _client) = pair(&context, "inproc://bench-async-empty").unwrap();

    c.bench_function("async_recv_msg_empty", |b| {
        b.iter(|| block_on(server.recv_msg_async()));
    });
}

fn send_recv_async(c: &mut Criterion) {
    let context = Context::new().unwrap();
    let (server, client) = pair(&context, "inproc://bench-async-roundtrip").unwrap();

    let mut group = c.benchmark_group("async_pair_inproc_send_recv");
    for size in SIZES {
        let payload = vec![0xA5_u8; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &payload, |b, p| {
            b.iter(|| {
                block_on(async {
                    client
                        .send_msg_async(black_box(p.as_slice()), SendFlags::empty())
                        .await
                        .unwrap();
                    server.recv_msg_async().await.unwrap()
                })
            });
        });
    }
    group.finish();
}

fn send_recv_wait(c: &mut Criterion) {
    let context = Context::new().unwrap();
    let (server, client) = pair(&context, "inproc://bench-async-wait").unwrap();

    let mut group = c.benchmark_group("async_wait_pair_inproc_send_recv");
    for size in SIZES {
        let payload = vec![0xA5_u8; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &payload, |b, p| {
            b.iter(|| {
                client
                    .send_msg(black_box(p.as_slice()), SendFlags::empty())
                    .unwrap();
                block_on(server.recv_msg_wait()).unwrap()
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    recv_msg_async_empty,
    send_recv_async,
    send_recv_wait
);
criterion_main!(benches);
