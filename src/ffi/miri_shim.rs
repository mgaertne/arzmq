//! Rust stand-ins for the `zmq_msg_*` functions of libzmq.
//!
//! Miri cannot call into C, but it resolves foreign functions that are defined in Rust with
//! `#[unsafe(no_mangle)]`. With these, the message code and its raw pointer handling run under
//! Miri. Everything that needs a context or a socket still needs the real libzmq.
//!
//! The `zmq_msg_t` only holds the key of its state, because Miri does not preserve the provenance
//! of pointers that are stored in a byte array.

use alloc::boxed::Box;
use core::{
    cell::Cell,
    ffi::{CStr, c_char, c_int, c_void},
    ptr, slice,
};
use std::{
    collections::HashMap,
    ffi::CString,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::zmq_sys_crate::{errno, zmq_free_fn, zmq_msg_t};

const GROUP_MAX_LENGTH: usize = 255;

thread_local! {
    static ERRNO: Cell<c_int> = const { Cell::new(0) };
}

static NEXT_KEY: AtomicU64 = AtomicU64::new(1);
static STATES: Mutex<Option<HashMap<u64, State>>> = Mutex::new(None);

enum Storage {
    Owned(*mut [u8]),
    External {
        data: *mut u8,
        len: usize,
        free: zmq_free_fn,
        hint: *mut c_void,
    },
}

struct State {
    storage: Storage,
    routing_id: u32,
    group: Option<CString>,
}

unsafe impl Send for State {}

impl State {
    fn owned(bytes: Box<[u8]>) -> Self {
        Self {
            storage: Storage::Owned(Box::into_raw(bytes)),
            routing_id: 0,
            group: None,
        }
    }

    fn data(&self) -> *mut u8 {
        match self.storage {
            Storage::Owned(bytes) => bytes.cast::<u8>(),
            Storage::External { data, .. } => data,
        }
    }

    fn len(&self) -> usize {
        match self.storage {
            Storage::Owned(bytes) => bytes.len(),
            Storage::External { len, .. } => len,
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        match self.storage {
            Storage::Owned(bytes) => drop(unsafe { Box::from_raw(bytes) }),
            Storage::External {
                data,
                free: Some(free),
                hint,
                ..
            } => unsafe { free(data.cast::<c_void>(), hint) },
            Storage::External { free: None, .. } => (),
        }
    }
}

fn with_states<R>(operation: impl FnOnce(&mut HashMap<u64, State>) -> R) -> R {
    let mut states = STATES.lock().unwrap();
    operation(states.get_or_insert_with(HashMap::new))
}

unsafe fn key(msg: *const zmq_msg_t) -> u64 {
    unsafe { msg.cast::<u64>().read() }
}

fn insert(msg: *mut zmq_msg_t, state: State) {
    let key = NEXT_KEY.fetch_add(1, Ordering::Relaxed);
    with_states(|states| states.insert(key, state));
    unsafe { msg.cast::<u64>().write(key) };
}

fn with_state<R>(msg: *const zmq_msg_t, operation: impl FnOnce(&mut State) -> R) -> R {
    let key = unsafe { key(msg) };
    with_states(|states| operation(states.get_mut(&key).expect("message is not initialized")))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_errno() -> c_int {
    ERRNO.get()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_init(msg: *mut zmq_msg_t) -> c_int {
    insert(msg, State::owned(Box::default()));
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_init_size(msg: *mut zmq_msg_t, size: usize) -> c_int {
    insert(msg, State::owned(vec![0u8; size].into_boxed_slice()));
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_init_buffer(
    msg: *mut zmq_msg_t,
    buf: *const c_void,
    size: usize,
) -> c_int {
    let bytes = unsafe { slice::from_raw_parts(buf.cast::<u8>(), size) };
    insert(msg, State::owned(bytes.into()));
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_init_data(
    msg: *mut zmq_msg_t,
    data: *mut c_void,
    size: usize,
    free: zmq_free_fn,
    hint: *mut c_void,
) -> c_int {
    insert(
        msg,
        State {
            storage: Storage::External {
                data: data.cast::<u8>(),
                len: size,
                free,
                hint,
            },
            routing_id: 0,
            group: None,
        },
    );
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_close(msg: *mut zmq_msg_t) -> c_int {
    let key = unsafe { key(msg) };
    let state = with_states(|states| states.remove(&key));
    drop(state);
    unsafe { msg.cast::<u64>().write(0) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_copy(dest: *mut zmq_msg_t, src: *mut zmq_msg_t) -> c_int {
    let copy = with_state(src, |source| State {
        storage: Storage::Owned(Box::into_raw(
            unsafe { slice::from_raw_parts(source.data(), source.len()) }.into(),
        )),
        routing_id: source.routing_id,
        group: source.group.clone(),
    });
    unsafe { zmq_msg_close(dest) };
    insert(dest, copy);
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_data(msg: *mut zmq_msg_t) -> *mut c_void {
    with_state(msg, |state| state.data().cast::<c_void>())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_size(msg: *const zmq_msg_t) -> usize {
    with_state(msg, |state| state.len())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_more(_msg: *const zmq_msg_t) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_set_routing_id(msg: *mut zmq_msg_t, routing_id: u32) -> c_int {
    if routing_id == 0 {
        ERRNO.set(errno::EINVAL);
        return -1;
    }
    with_state(msg, |state| state.routing_id = routing_id);
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_routing_id(msg: *mut zmq_msg_t) -> u32 {
    with_state(msg, |state| state.routing_id)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_set_group(msg: *mut zmq_msg_t, group: *const c_char) -> c_int {
    let bytes = unsafe { CStr::from_ptr(group) }.to_bytes();
    let group = CString::new(&bytes[..bytes.len().min(GROUP_MAX_LENGTH)])
        .expect("the bytes of a C string contain no null byte");
    with_state(msg, |state| state.group = Some(group));
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zmq_msg_group(msg: *mut zmq_msg_t) -> *const c_char {
    with_state(msg, |state| match &state.group {
        Some(group) => group.as_ptr(),
        None => ptr::from_ref(c"").cast::<c_char>(),
    })
}
