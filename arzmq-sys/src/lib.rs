#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use crate::unix::RawFd;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::RawFd;

pub mod errno;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use crate::bindings::*;
