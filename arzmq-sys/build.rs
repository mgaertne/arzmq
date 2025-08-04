#[cfg(windows)]
use std::fs;
use std::{env, path::PathBuf};

fn compile_zmq() {
    let libraries = system_deps::Config::new().probe().unwrap();

    let mut libzmq_config = cmake::Config::new("vendor");
    libzmq_config
        .no_build_target(true)
        .pic(true)
        .define("ZMQ_OUTPUT_BASENAME", "rust_zmq")
        .define("CMAKE_POLICY_VERSION_MINIMUM", "4.0")
        .define("BUILD_TESTS", "OFF")
        .define("WITH_DOC", "OFF")
        .define("BUILD_SHARED", "OFF")
        .define("BUILD_STATIC", "ON");

    #[cfg(feature = "draft-api")]
    libzmq_config.define("ENABLE_DRAFTS", "ON");
    #[cfg(not(feature = "draft-api"))]
    libzmq_config
        .define("ENABLE_DRAFTS", "OFF")
        .define("ZMQ_BUILD_DRAFT_API", "OFF");

    #[cfg(feature = "curve")]
    libzmq_config
        .define("WITH_LIBSODIUM", "ON")
        .define("ENABLE_CURVE", "ON");
    #[cfg(feature = "gssapi")]
    {
        libzmq_config.cxxflag("-DHAVE_LIBGSSAPI_KRB5=1");
        libraries
            .iter()
            .iter()
            .filter(|(name, _lib)| name.contains("gssapi"))
            .for_each(|(_name, lib)| {
                lib.include_paths.iter().for_each(|include| {
                    libzmq_config.cxxflag(format!("-I{}", include.display()));
                });
            });
    }
    #[cfg(feature = "pgm")]
    {
        libzmq_config.define("WITH_OPENPGM", "ON");
        libraries
            .iter()
            .iter()
            .filter(|(name, _lib)| name.starts_with("openpgm"))
            .for_each(|(_name, lib)| {
                libzmq_config.define("OPENPGM_PKGCONFIG_NAME", lib.name.clone());
            });
        #[cfg(target_os = "macos")]
        libzmq_config.cxxflag("-Drestrict=__restrict__");
    }
    #[cfg(feature = "norm")]
    libzmq_config.define("WITH_NORM", "ON");
    #[cfg(feature = "vmci")]
    libzmq_config.define("WITH_VMCI", "ON");

    #[cfg(target_os = "macos")]
    libzmq_config.define("CMAKE_OSX_DEPLOYMENT_TARGET", "10.12");
    #[cfg(windows)]
    libzmq_config.static_crt(true);

    let libzmq = libzmq_config.build();

    #[cfg(not(windows))]
    {
        println!(
            "cargo:rustc-link-search=all={}",
            libzmq.join("build").join("lib").display()
        );
    }
    #[cfg(windows)]
    {
        let dir = libzmq.join("build").join("lib");
        let artifacts = walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(|entry| {
                entry.ok().filter(|dir_entry| {
                    let path = dir_entry.path();
                    path.is_file()
                        && path.extension().is_some_and(|ext| ext == "lib")
                        && path.file_name().is_some_and(|file_name| {
                            file_name.to_string_lossy().contains("rust_zmq")
                        })
                })
            })
            .map(|e| e.path().to_owned())
            .collect::<Vec<_>>();

        for artifact in artifacts {
            fs::copy(artifact, dir.join("rust_zmq.lib")).unwrap();
        }
        println!("cargo:rustc-link-search=all={}", dir.display());
        println!("VARS: {:?}", env::vars());
    }
    println!("cargo:rustc-link-lib=static=rust_zmq");

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=c++");
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=bsd");
    }
    #[cfg(windows)]
    {
        println!("cargo::rustc-link-lib=Advapi32");
        println!("cargo::rustc-link-lib=wsock32");
        println!("cargo::rustc-link-lib=ws2_32");
        println!("cargo::rustc-link-lib=Iphlpapi");
    }
}

fn generate_bindings() {
    let vendor_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("vendor");
    let include_dir = vendor_dir.join("include");

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

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_FEATURE");

    compile_zmq();

    generate_bindings();
}
