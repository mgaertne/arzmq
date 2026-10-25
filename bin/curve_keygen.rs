#![cfg(feature = "curve")]

use core::ffi::{CStr, c_char};

use arzmq_sys as zmq_sys_crate;

const MISSING_CURVE: &str =
    "To use curve_keygen, please install libsodium and then rebuild libzmq.";

type Key = [u8; 41];

fn keypair() -> Result<(Key, Key), &'static str> {
    if !cfg!(zmq_has = "curve") {
        return Err(MISSING_CURVE);
    }

    let mut public_key: Key = [0; 41];
    let mut secret_key: Key = [0; 41];

    if unsafe {
        zmq_sys_crate::zmq_curve_keypair(
            public_key.as_mut_ptr() as *mut c_char,
            secret_key.as_mut_ptr() as *mut c_char,
        )
    } == -1
    {
        match unsafe { zmq_sys_crate::zmq_errno() } {
            zmq_sys_crate::errno::ENOTSUP => return Err(MISSING_CURVE),
            _ => unreachable!(),
        }
    }

    Ok((public_key, secret_key))
}

pub fn main() -> Result<(), &'static str> {
    println!(
        "This tool generates a CurveZMQ keypair, as two printable strings you can use in \
configuration files or source code. The encoding uses Z85, which is a base-85 format that \
is described in 0MQ RFC 32, and which has an implementation in the z85_codec.h source used \
by this tool. The keypair always works with the secret key held by one party and the public \
key distributed (securely!) to peers wishing to connect to it."
    );

    let (public_key, secret_key) = keypair()?;

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

#[cfg(test)]
mod curve_keygen_tests {
    use core::ffi::CStr;

    use super::{keypair, main};

    #[test]
    fn keypair_consists_of_two_distinct_z85_keys() {
        let (public_key, secret_key) = keypair().unwrap();

        let public_key = CStr::from_bytes_until_nul(&public_key).unwrap();
        let secret_key = CStr::from_bytes_until_nul(&secret_key).unwrap();

        assert_eq!(public_key.to_bytes().len(), 40);
        assert_eq!(secret_key.to_bytes().len(), 40);
        assert_ne!(public_key, secret_key);
    }

    #[test]
    fn keypairs_differ_between_calls() {
        assert_ne!(keypair().unwrap(), keypair().unwrap());
    }

    #[test]
    fn main_prints_keypair_successfully() {
        assert_eq!(main(), Ok(()));
    }
}
