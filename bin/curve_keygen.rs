#![cfg(zmq_has = "curve")]

use core::ffi::{CStr, c_char};

use arzmq_sys as zmq_sys_crate;

pub fn main() -> Result<(), &'static str> {
    println!(
        "This tool generates a CurveZMQ keypair, as two printable strings you can use in \
configuration files or source code. The encoding uses Z85, which is a base-85 format that \
is described in 0MQ RFC 32, and which has an implementation in the z85_codec.h source used \
by this tool. The keypair always works with the secret key held by one party and the public \
key distributed (securely!) to peers wishing to connect to it."
    );

    let mut public_key: [u8; 41] = [0; 41];
    let mut secret_key: [u8; 41] = [0; 41];

    if unsafe {
        zmq_sys_crate::zmq_curve_keypair(
            public_key.as_mut_ptr() as *mut c_char,
            secret_key.as_mut_ptr() as *mut c_char,
        )
    } == -1
    {
        match unsafe { zmq_sys_crate::zmq_errno() } {
            zmq_sys_crate::errno::ENOTSUP => {
                return Err(
                    "To use curve_keygen, please install libsodium and then rebuild libzmq.",
                );
            }
            _ => unreachable!(),
        }
    }

    println!(
        "\n== CURVE PUBLIC KEY == {:?}",
        CStr::from_bytes_until_nul(&public_key).unwrap()
    );
    println!(
        "== CURVE SECRET KEY == {:?}",
        CStr::from_bytes_until_nul(&secret_key).unwrap()
    );

    Ok(())
}
