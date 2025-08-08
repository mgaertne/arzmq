use std::ffi::CString;

fn main() {
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
