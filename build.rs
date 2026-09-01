extern crate alloc;

use alloc::ffi::CString;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(cold_path)");
    if rustversion::cfg!(any(nightly, since(1.95))) {
        println!("cargo:rustc-cfg=cold_path");
    }

    let capabilities = [
        "ipc", "pgm", "tipc", "vmci", "norm", "curve", "gssapi", "draft",
    ];

    println!(
        "cargo::rustc-check-cfg=cfg(zmq_has, values(\"{}\"))",
        capabilities.join("\", \"")
    );

    capabilities
        .iter()
        .filter(|&&capability| {
            let c_str = CString::new(capability).unwrap();
            (unsafe { arzmq_sys::zmq_has(c_str.as_ptr()) } != 0)
        })
        .for_each(|capability| println!("cargo::rustc-cfg=zmq_has=\"{capability}\""));
}
