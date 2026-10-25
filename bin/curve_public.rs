#![cfg(feature = "curve")]

extern crate alloc;

use alloc::ffi::CString;
use core::ffi::{CStr, c_char};

use arzmq_sys as zmq_sys_crate;

const MISSING_CURVE: &str =
    "To use curve_public, please install libsodium and then rebuild libzmq.";
const INVALID_SECRET_KEY: &str = "Invalid secret key.";

fn public_key(secret_key: &str) -> Result<[u8; 41], &'static str> {
    if !cfg!(zmq_has = "curve") {
        return Err(MISSING_CURVE);
    }

    let mut public_key: [u8; 41] = [0; 41];
    let secret_key = CString::new(secret_key).map_err(|_| INVALID_SECRET_KEY)?;

    if unsafe {
        zmq_sys_crate::zmq_curve_public(
            public_key.as_mut_ptr() as *mut c_char,
            secret_key.as_bytes_with_nul().as_ptr() as *const c_char,
        )
    } == -1
    {
        match unsafe { zmq_sys_crate::zmq_errno() } {
            zmq_sys_crate::errno::ENOTSUP => return Err(MISSING_CURVE),
            zmq_sys_crate::errno::EINVAL => return Err(INVALID_SECRET_KEY),
            _ => unreachable!(),
        }
    }

    Ok(public_key)
}

fn run(secret_key: Option<String>) -> Result<(), &'static str> {
    println!(
        "This tool generates a CurveZMQ public key from a secret key, as printable string you can \
use in configuration files or source code. The encoding uses Z85, which is a base-85 format that \
is described in 0MQ RFC 32, and which has an implementation in the z85_codec.h source used by this \
tool. The keypair always works with the secret key held by one party and the public key \
distributed (securely!) to peers wishing to connect to it."
    );

    if !cfg!(zmq_has = "curve") {
        return Err(MISSING_CURVE);
    }

    let Some(secret_key) = secret_key else {
        return Err("Please provide a secret key.");
    };
    let public_key = public_key(&secret_key)?;

    println!(
        "\n== CURVE PUBLIC KEY == {:?}",
        CStr::from_bytes_until_nul(&public_key).unwrap()
    );

    Ok(())
}

pub fn main() -> Result<(), &'static str> {
    run(std::env::args().nth(1))
}

#[cfg(test)]
mod curve_public_tests {
    use core::ffi::{CStr, c_char};

    use arzmq_sys as zmq_sys_crate;

    use super::{INVALID_SECRET_KEY, public_key, run};

    fn generated_keypair() -> (String, String) {
        let mut public_key = [0u8; 41];
        let mut secret_key = [0u8; 41];
        let rc = unsafe {
            zmq_sys_crate::zmq_curve_keypair(
                public_key.as_mut_ptr() as *mut c_char,
                secret_key.as_mut_ptr() as *mut c_char,
            )
        };
        assert_eq!(rc, 0);
        let to_string = |key: &[u8]| {
            CStr::from_bytes_until_nul(key)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        };
        (to_string(&public_key), to_string(&secret_key))
    }

    #[test]
    fn public_key_matches_the_one_of_the_generated_keypair() {
        let (expected, secret_key) = generated_keypair();

        let derived = public_key(&secret_key).unwrap();

        assert_eq!(
            CStr::from_bytes_until_nul(&derived).unwrap().to_str(),
            Ok(&*expected)
        );
    }

    #[test]
    fn invalid_secret_key_is_rejected() {
        assert_eq!(public_key("too short"), Err(INVALID_SECRET_KEY));
    }

    #[test]
    fn secret_key_with_interior_nul_is_rejected() {
        assert_eq!(public_key("abc\0def"), Err(INVALID_SECRET_KEY));
    }

    #[test]
    fn run_requires_a_secret_key() {
        assert_eq!(run(None), Err("Please provide a secret key."));
    }

    #[test]
    fn run_prints_public_key_for_valid_secret_key() {
        let (_, secret_key) = generated_keypair();

        assert_eq!(run(Some(secret_key)), Ok(()));
    }
}
