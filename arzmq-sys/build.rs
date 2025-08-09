use core::error::Error;
#[cfg(target_env = "msvc")]
use std::fs;
use std::{
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use cc::Build;
use system_deps::{Config, Dependencies};
#[cfg(target_env = "msvc")]
use tap::TapFallible;
use tap::TapOptional;

static DEFAULT_SOURCES: &[&str] = &[
    "address",
    "channel",
    "client",
    "clock",
    "ctx",
    "curve_client",
    "curve_mechanism_base",
    "curve_server",
    "dealer",
    "decoder_allocators",
    "devpoll",
    "dgram",
    "dish",
    "dist",
    "endpoint",
    "epoll",
    "err",
    "fq",
    "gather",
    "gssapi_client",
    "gssapi_mechanism_base",
    "gssapi_server",
    "io_object",
    "io_thread",
    "ip_resolver",
    "ip",
    "ipc_address",
    "ipc_connecter",
    "ipc_listener",
    "kqueue",
    "lb",
    "mailbox_safe",
    "mailbox",
    "mechanism_base",
    "mechanism",
    "metadata",
    "msg",
    "mtrie",
    "norm_engine",
    "null_mechanism",
    "object",
    "options",
    "own",
    "pair",
    "peer",
    "pgm_receiver",
    "pgm_sender",
    "pgm_socket",
    "pipe",
    "plain_client",
    "plain_server",
    "poll",
    "poller_base",
    "polling_util",
    "pollset",
    "precompiled",
    "proxy",
    "pub",
    "pull",
    "push",
    "radio",
    "radix_tree",
    "random",
    "raw_decoder",
    "raw_encoder",
    "raw_engine",
    "reaper",
    "rep",
    "req",
    "router",
    "scatter",
    "select",
    "server",
    "session_base",
    "signaler",
    "socket_base",
    "socket_poller",
    "socks_connecter",
    "socks",
    "stream_connecter_base",
    "stream_engine_base",
    "stream_listener_base",
    "stream",
    "sub",
    "tcp_address",
    "tcp_connecter",
    "tcp_listener",
    "tcp",
    "thread",
    "timers",
    "tipc_address",
    "tipc_connecter",
    "tipc_listener",
    "trie",
    "udp_address",
    "udp_engine",
    "v1_decoder",
    "v1_encoder",
    "v2_decoder",
    "v2_encoder",
    "v3_1_encoder",
    "vmci_address",
    "vmci_connecter",
    "vmci_listener",
    "vmci",
    "ws_address",
    "ws_connecter",
    "ws_decoder",
    "ws_encoder",
    "ws_engine",
    "ws_listener",
    "xpub",
    "xsub",
    "zap_client",
    "zmq_utils",
    "zmq",
    "zmtp_engine",
];

fn add_cpp_sources(build: &mut Build, root: impl AsRef<Path>, files: &[&str]) {
    build.cpp(true);
    let root = root.as_ref();
    build.files(files.iter().map(|src| {
        let mut p = root.join(src);
        p.set_extension("cpp");
        p
    }));

    build.include(root);
}

fn add_c_sources(build: &mut Build, root: impl AsRef<Path>, files: &[&str]) {
    let root = root.as_ref();
    // Temporarily use c instead of c++.
    build.cpp(false);
    build.files(files.iter().map(|src| {
        let mut p = root.join(src);
        p.set_extension("c");
        p
    }));

    build.include(root);
}

#[cfg(target_env = "msvc")]
fn rename_libzmq_in_dir<D, N>(dir: D, new_name: N) -> Result<(), ()>
where
    D: AsRef<Path>,
    N: AsRef<Path>,
{
    let dir = dir.as_ref();
    let new_name = new_name.as_ref();

    let artifacts = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| {
            entry.ok().filter(|dir_entry| {
                let path = dir_entry.path();
                path.is_file()
                    && path.extension().is_some_and(|ext| ext == "lib")
                    && path
                        .file_name()
                        .is_some_and(|file_name| file_name.to_string_lossy().contains("rust_zmq"))
            })
        })
        .map(|e| e.path().to_owned())
        .collect::<Vec<_>>();

    for artifact in artifacts {
        fs::copy(artifact, dir.join(new_name)).map_err(|_| ())?;
    }

    Ok(())
}

fn check_low_level_compilation<S, F>(c_src: S, configure_build: F) -> Result<bool, Box<dyn Error>>
where
    S: AsRef<str>,
    F: FnOnce(&mut Build) -> &mut Build,
{
    let out_dir = env::var("OUT_DIR")?;
    let out_dir = Path::new(&out_dir);
    let check_compile = tempfile::Builder::new()
        .prefix("check_compile")
        .tempdir_in(out_dir)?;

    let src_path = check_compile.path().join("check_compile.c");
    {
        let mut src_file = File::create(&src_path)?;
        src_file.write_all(c_src.as_ref().as_bytes())?;
        src_file.flush()?;
    }

    let mut builder = Build::new();
    let mut compile_command = configure_build(&mut builder).get_compiler().to_command();

    compile_command.arg(src_path);

    #[cfg(not(target_env = "msvc"))]
    compile_command
        .arg("-o")
        .arg(check_compile.path().join("check_compile"));

    #[cfg(target_env = "msvc")]
    compile_command.arg("/c").arg(format!(
        "/Fo{}",
        check_compile.path().join("check_compile").display()
    ));

    Ok(compile_command.status().map(|status| status.success())?)
}

#[cfg(target_env = "gnu")]
fn check_strlcpy() -> Result<bool, Box<dyn Error>> {
    check_low_level_compilation(
        r#"
#include <string.h>

int main() {
    char buf[1];
    (void)strlcpy(buf, "a", 1);
    return 0;
}
"#,
        |build| build.warnings(false),
    )
}

#[cfg(all(target_os = "windows", not(target_vendor = "uwp")))]
fn check_ipc_headers() -> Result<bool, Box<dyn Error>> {
    check_low_level_compilation(
        r#"
#include <winsock2.h>
#include <afunix.h>

int main() {
    SOCKET sock = INVALID_SOCKET;
    int family = AF_UNIX;
    return 0;
}
"#,
        |build| build.warnings(false),
    )
}

#[cfg(not(target_env = "msvc"))]
fn check_cxx11() -> Result<bool, Box<dyn Error>> {
    check_low_level_compilation(
        r#"
int main(void) {
    return 0;
}
"#,
        |build| {
            build
                .cpp(true)
                .warnings(true)
                .warnings_into_errors(true)
                .std("c++11")
        },
    )
}

fn configure(build: &mut Build) -> Result<(), Box<dyn Error>> {
    let libraries = Config::new().probe()?;

    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor");

    #[cfg(not(target_env = "msvc"))]
    build.flags(&[
        "-Wno-unused-function",
        "-Wno-deprecated",
        "-Wno-unused-parameter",
        "-Wno-ignored-qualifiers",
        "-Wno-implicit-fallthrough",
        "-Wno-missing-field-initializers",
        "-Wno-missing-braces",
    ]);

    build
        .define("ZMQ_BUILD_TESTS", "OFF")
        .include(vendor.join("include"));

    add_cpp_sources(build, vendor.join("src"), DEFAULT_SOURCES);

    libraries
        .iter()
        .iter()
        .filter(|(name, _lib)| *name == "gnutls")
        .for_each(|(_name, lib)| {
            add_cpp_sources(build, vendor.join("src"), &["wss_address", "wss_engine"]);
            build.includes(&lib.include_paths);
        });

    add_c_sources(build, vendor.join("external/sha1"), &["sha1.c"]);

    build.define("ZMQ_USE_CV_IMPL_STL11", "1");
    build.define("ZMQ_STATIC", "1");
    build.define("ZMQ_USE_BUILTIN_SHA1", "1");

    build.define("ZMQ_HAVE_WS", "1");

    #[cfg(not(windows))]
    let create_platform_hpp_shim = |build: &mut cc::Build| {
        let out_includes = PathBuf::from(env::var("OUT_DIR").unwrap());

        let mut f = File::create(out_includes.join("platform.hpp")).unwrap();
        f.write_all(b"").unwrap();
        f.sync_all().unwrap();

        build.include(out_includes);
    };

    #[cfg(target_os = "windows")]
    {
        #[cfg(not(target_env = "gnu"))]
        add_c_sources(build, vendor.join("external/wepoll"), &["wepoll.c"]);

        build.define("ZMQ_HAVE_WINDOWS", "1");
        build.define("ZMQ_IOTHREAD_POLLER_USE_EPOLL", "1");
        build.define("ZMQ_POLL_BASED_ON_POLL", "1");
        build.define("_WIN32_WINNT", "0x0600"); // vista
        build.define("ZMQ_HAVE_STRUCT_SOCKADDR_UN", "1");

        println!("cargo::rustc-link-lib=Advapi32");
        println!("cargo::rustc-link-lib=wsock32");
        println!("cargo::rustc-link-lib=ws2_32");
        println!("cargo::rustc-link-lib=Iphlpapi");

        #[cfg(target_env = "msvc")]
        {
            build.include(vendor.join("builds/deprecated-msvc"));
            build.flag("/GL-");

            build.flag("/EHsc");
        }
        #[cfg(not(target_env = "msvc"))]
        {
            create_platform_hpp_shim(build);
            build.define("HAVE_STRNLEN", "1");
        }

        #[cfg(not(target_vendor = "uwp"))]
        if check_ipc_headers().unwrap_or(false) {
            build.define("ZMQ_HAVE_IPC", "1");
        }
    }

    #[cfg(target_os = "linux")]
    {
        create_platform_hpp_shim(build);
        build.define("ZMQ_HAVE_LINUX", "1");
        build.define("ZMQ_IOTHREAD_POLLER_USE_EPOLL", "1");
        build.define("ZMQ_POLL_BASED_ON_POLL", "1");
        build.define("ZMQ_HAVE_IPC", "1");

        build.define("HAVE_STRNLEN", "1");
        build.define("ZMQ_HAVE_UIO", "1");
        build.define("ZMQ_HAVE_STRUCT_SOCKADDR_UN", "1");

        #[cfg(any(target_os = "android", target_env = "musl"))]
        build.define("ZMQ_HAVE_STRLCPY", "1");
    }

    #[cfg(any(target_os = "macos", target_os = "freebsd"))]
    {
        create_platform_hpp_shim(build);
        build.define("ZMQ_IOTHREAD_POLLER_USE_KQUEUE", "1");
        build.define("ZMQ_POLL_BASED_ON_POLL", "1");
        build.define("HAVE_STRNLEN", "1");
        build.define("ZMQ_HAVE_UIO", "1");
        build.define("ZMQ_HAVE_IPC", "1");
        build.define("ZMQ_HAVE_STRUCT_SOCKADDR_UN", "1");
        build.define("ZMQ_HAVE_STRLCPY", "1");
    }

    #[cfg(target_env = "gnu")]
    if check_strlcpy().unwrap_or(false) {
        build.define("ZMQ_HAVE_STRLCPY", "1");
    }

    #[cfg(not(target_env = "msvc"))]
    if check_cxx11().unwrap_or(false) {
        build.std("c++11");
    }

    #[cfg(feature = "draft-api")]
    build.define("ZMQ_BUILD_DRAFT_API", "1");

    check_curve_config(build, &libraries);
    check_gssapi_config(build, &libraries);
    check_pgm_config(build, &libraries);
    check_norm_config(build, &libraries);
    check_vmci_config(build)
}

fn check_curve_config(build: &mut Build, libraries: &Dependencies) {
    if cfg!(not(feature = "curve")) {
        return;
    }

    libraries.get_by_name("libsodium").tap_some(|lib| {
        build.define("ZMQ_USE_LIBSODIUM", "1");
        build.define("ZMQ_HAVE_CURVE", "1");

        build.includes(&lib.include_paths);
    });

    #[cfg(target_env = "msvc")]
    let _ = vcpkg::find_package("libsodium").tap_ok(|lib| {
        build.define("ZMQ_USE_LIBSODIUM", "1");
        build.define("ZMQ_HAVE_CURVE", "1");

        build.includes(&lib.include_paths);
    });
}

fn check_gssapi_config(build: &mut Build, libraries: &Dependencies) {
    if cfg!(not(feature = "gssapi")) {
        return;
    }

    libraries.get_by_name("gssapi").tap_some(|lib| {
        build.define("HAVE_LIBGSSAPI_KRB5", "1");
        build.includes(&lib.include_paths);
    });

    #[cfg(target_env = "msvc")]
    {
        unsafe {
            env::set_var("VCPKGRS_DYNAMIC", "1");
        }
        let _ = vcpkg::Config::new()
            .target_triplet("x64-windows")
            .find_package("krb5")
            .tap_ok(|lib| {
                build.define("HAVE_LIBGSSAPI_KRB5", "1");
                build.includes(&lib.include_paths);
            });
        unsafe {
            env::remove_var("VCPKGRS_DYNAMIC");
        }
    }
}

fn check_pgm_config(build: &mut Build, libraries: &Dependencies) {
    if cfg!(not(feature = "pgm")) {
        return;
    }

    libraries.get_by_name("openpgm").tap_some(|lib| {
        build.define("ZMQ_HAVE_OPENPGM", "1");
        build.includes(&lib.include_paths);

        #[cfg(target_os = "macos")]
        build.define("restrict", "__restrict__");
    });
}

fn check_norm_config(build: &mut Build, libraries: &Dependencies) {
    if cfg!(not(feature = "norm")) {
        return;
    }

    libraries.get_by_name("norm").tap_some(|lib| {
        build.define("ZMQ_HAVE_NORM", "1");
        build.includes(&lib.include_paths);

        #[cfg(not(target_os = "linux"))]
        println!("cargo:rustc-link-lib=static=protokit");

        #[cfg(target_os = "windows")]
        println!("cargo:rustc-link-lib=user32");
    });
}

fn check_vmci_config(build: &mut Build) -> Result<(), Box<dyn Error>> {
    if cfg!(not(feature = "vmci")) {
        return Ok(());
    }

    let vmci = Path::new(env!("CARGO_MANIFEST_DIR")).join("vmci");

    build.define("ZMQ_HAVE_VMCI", "1");

    build.cpp(false);
    build.include(vmci);

    Ok(())
}

fn build_zmq() -> Result<(), Box<dyn Error>> {
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor");

    let mut build = Build::new();
    configure(&mut build)?;

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let lib_dir = out_dir.join("lib");

    build.out_dir(&lib_dir).cpp(true);

    build.compile("zmq");

    #[cfg(target_env = "msvc")]
    if rename_libzmq_in_dir(&lib_dir, "zmq.lib").is_err() {
        panic!("unable to find compiled `libzmq` lib");
    }

    let source_dir = out_dir.join("source");
    let include_dir = source_dir.join("include");

    dircpy::copy_dir(vendor.join("include"), &include_dir).expect("unable to copy include dir");
    dircpy::copy_dir(vendor.join("src"), source_dir.join("src")).expect("unable to copy src dir");
    dircpy::copy_dir(vendor.join("external"), source_dir.join("external"))
        .expect("unable to copy external dir");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=zmq");
    println!("cargo:include={}", include_dir.display());
    println!("cargo:lib={}", lib_dir.display());
    println!("cargo:out={}", out_dir.display());

    Ok(())
}

fn generate_bindings() -> Result<(), Box<dyn Error>> {
    let vendor_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("vendor");
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

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_FEATURE");

    build_zmq()?;

    generate_bindings()
}
