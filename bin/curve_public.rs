#![cfg(zmq_has = "curve")]

extern crate alloc;

use alloc::ffi::CString;
use core::ffi::{CStr, c_char};

use arzmq_sys as zmq_sys_crate;

#[allow(dead_code)]
pub fn main() -> Result<(), &'static str> {
    println!(
        "This tool generates a CurveZMQ public key from a secret key, as printable string you can \
use in configuration files or source code. The encoding uses Z85, which is a base-85 format that \
is described in 0MQ RFC 32, and which has an implementation in the z85_codec.h source used by this \
tool. The keypair always works with the secret key held by one party and the public key \
distributed (securely!) to peers wishing to connect to it."
    );

    let mut public_key: [u8; 41] = [0; 41];
    let Some(arg) = std::env::args().nth(1) else {
        return Err("Please provide a secret key.");
    };
    let secret_key = CString::new(arg).unwrap();

    if unsafe {
        zmq_sys_crate::zmq_curve_public(
            public_key.as_mut_ptr() as *mut c_char,
            secret_key.as_bytes_with_nul().as_ptr() as *const c_char,
        )
    } == -1
    {
        match unsafe { zmq_sys_crate::zmq_errno() } {
            zmq_sys_crate::errno::ENOTSUP => {
                return Err(
                    "To use curve_keygen, please install libsodium and then rebuild libzmq.",
                );
            }
            zmq_sys_crate::errno::EINVAL => {
                return Err("Invalid secret key.");
            }
            _ => unreachable!(),
        }
    }

    println!(
        "\n== CURVE PUBLIC KEY == {:?}",
        CStr::from_bytes_until_nul(&public_key).unwrap()
    );

    Ok(())
}
