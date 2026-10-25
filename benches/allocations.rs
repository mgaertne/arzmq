//! Counts the heap allocations of the calling thread per operation.
//!
//! The counts do not depend on the load of the machine, so they allow comparing implementations
//! where timings are too noisy. Run with `cargo bench --bench allocations`.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    hint::black_box,
    thread,
    time::Duration,
};

use arzmq::{
    prelude::{
        Context, Message, MonitorFlags, MonitorReceiver, PairSocket, Receiver, RecvFlags,
        SendFlags, Sender,
    },
    socket::PollEvents,
};

struct CountingAllocator;

thread_local! {
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn allocations_of<R>(operation: impl FnOnce() -> R) -> usize {
    let before = ALLOCATIONS.get();
    black_box(operation());
    ALLOCATIONS.get() - before
}

fn report(name: &str, operations: usize, allocations: usize) {
    println!(
        "{name:<32} {:>6.2} allocations per operation",
        allocations as f64 / operations as f64
    );
}

fn main() {
    let context = Context::new().unwrap();
    let server = PairSocket::from_context(&context).unwrap();
    server.bind("inproc://allocations").unwrap();
    let client = PairSocket::from_context(&context).unwrap();
    client.connect("inproc://allocations").unwrap();

    const REPEATS: usize = 1000;
    let payload = vec![0xA5_u8; 64];

    report(
        "send_msg + recv_msg (64 B)",
        REPEATS,
        allocations_of(|| {
            for _ in 0..REPEATS {
                client
                    .send_msg(payload.as_slice(), SendFlags::empty())
                    .unwrap();
                server.recv_msg(RecvFlags::empty()).unwrap();
            }
        }),
    );
    report(
        "last_endpoint",
        REPEATS,
        allocations_of(|| {
            for _ in 0..REPEATS {
                server.last_endpoint().unwrap();
            }
        }),
    );
    report(
        "poll",
        REPEATS,
        allocations_of(|| {
            for _ in 0..REPEATS {
                server.poll(PollEvents::POLL_IN, 0).unwrap();
            }
        }),
    );

    let message = Message::from(payload.as_slice());
    report(
        "Message::bytes",
        REPEATS,
        allocations_of(|| {
            for _ in 0..REPEATS {
                message.bytes();
            }
        }),
    );
    report(
        "Message::as_bytes",
        REPEATS,
        allocations_of(|| {
            for _ in 0..REPEATS {
                black_box(message.as_bytes().len());
            }
        }),
    );

    const CYCLES: usize = 50;
    let monitored = PairSocket::from_context(&context).unwrap();
    let monitor = monitored
        .monitor(MonitorFlags::Listening | MonitorFlags::Closed)
        .unwrap();
    for _ in 0..CYCLES {
        monitored
            .bind_endpoint("tcp://127.0.0.1:*")
            .unwrap()
            .unbind()
            .unwrap();
    }
    thread::sleep(Duration::from_millis(200));

    let mut events = 0;
    let allocations = allocations_of(|| {
        while monitor.recv_monitor_event().is_ok() {
            events += 1;
        }
    });
    assert!(
        events + 2 >= 2 * CYCLES,
        "too few monitor events have arrived"
    );
    report("recv_monitor_event", events, allocations);
}
