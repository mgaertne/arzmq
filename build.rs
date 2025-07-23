fn main() {
    [
        c"ipc", c"pgm", c"tipc", c"vmci", c"norm", c"curve", c"gssapi", c"draft",
    ]
    .iter()
    .for_each(|capability| {
        println!(
            "cargo::rustc-check-cfg=cfg(zmq_have_{})",
            capability.to_string_lossy()
        );
        if unsafe { arzmq_sys::zmq_has(capability.as_ptr()) } != 0 {
            println!("cargo::rustc-cfg=zmq_have_{}", capability.to_string_lossy());
        }
    });
}
