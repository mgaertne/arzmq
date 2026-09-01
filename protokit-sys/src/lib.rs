#[cfg(feature = "bindings")]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::ptr_offset_with_cast)]
#[allow(clippy::useless_transmute)]
#[allow(clippy::too_many_arguments)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(feature = "bindings")]
pub use crate::bindings::*;
