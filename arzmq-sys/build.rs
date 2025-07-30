use std::{env, path::PathBuf};

fn main() {
    #[cfg(windows)]
    println!("cargo::rustc-link-lib=Advapi32");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");

    let mut zmq_builder = zeromq_src::Build::new();

    #[cfg(all(feature = "curve", not(windows)))]
    {
        let mut libsodium_dir =
            PathBuf::from(env::var("DEP_SODIUM_INCLUDE").expect("DEP_SODIUM_INCLUDE not set"));
        libsodium_dir.pop();
        libsodium_dir.push("libsodium");

        let lib_dir = libsodium_dir.join("x64\\Debug\\v143\\static");
        let include_dir = libsodium_dir.join("include");

        zmq_builder.with_libsodium(Some(zeromq_src::LibLocation::new(lib_dir, include_dir)));
    }
    #[cfg(any(not(feature = "curve"), windows))]
    zmq_builder.with_libsodium(None);

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
