use std::{env, path::PathBuf};

fn main() {
    #[cfg(windows)]
    println!("cargo::rustc-link-lib=Advapi32");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");

    let maybe_libsodium = if cfg!(all(not(windows), feature = "libsodium")) {
        let lib_dir = env::var("DEP_SODIUM_LIB").expect("build metadata `DEP_SODIUM_LIB` required");
        let include_dir =
            env::var("DEP_SODIUM_INCLUDE").expect("build metadata `DEP_SODIUM_INCLUDE` required");

        Some(zeromq_src::LibLocation::new(lib_dir, include_dir))
    } else {
        None
    };

    let mut zmq_builder = zeromq_src::Build::new();
    zmq_builder.with_libsodium(maybe_libsodium);

    #[cfg(feature = "draft-api")]
    zmq_builder.enable_draft(true);

    zmq_builder.build();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include_dir = out_dir.join("source/include");

    let builder = bindgen::Builder::default()
        .header(include_dir.join("zmq.h").to_string_lossy())
        .size_t_is_usize(true)
        .derive_default(true)
        .derive_eq(true)
        .derive_partialeq(true)
        .derive_debug(true)
        .use_core()
        .allowlist_function("^zmq_.*")
        .allowlist_type("^zmq_.*")
        .allowlist_var("^ZMQ_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    #[cfg(feature = "draft-api")]
    let builder = builder.clang_args(vec!["-DZMQ_BUILD_DRAFT_API=1"]);

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
