extern crate core;

use core::error::Error;
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use cc::Build;
use chrono::prelude::*;

static SOURCES: &[&str] = &[
    "cpu.c",
    "thread.c",
    "mem.c",
    "string.c",
    "list.c",
    "slist.c",
    "queue.c",
    "hashtable.c",
    "messages.c",
    "error.c",
    "math.c",
    "packet_parse.c",
    "packet_test.c",
    "sockaddr.c",
    "time.c",
    "if.c",
    "inet_lnaof.c",
    "getifaddrs.c",
    "get_nprocs.c",
    "getnetbyname.c",
    "getnodeaddr.c",
    "getprotobyname.c",
    "indextoaddr.c",
    "indextoname.c",
    "nametoindex.c",
    "inet_network.c",
    "md5.c",
    "rand.c",
    "gsi.c",
    "tsi.c",
    "txw.c",
    "rxw.c",
    "skbuff.c",
    "socket.c",
    "source.c",
    "receiver.c",
    "recv.c",
    "engine.c",
    "timer.c",
    "net.c",
    "rate_control.c",
    "checksum.c",
    "reed_solomon.c",
    "wsastrerror.c",
    "histogram.c",
    "galois_tables.c",
];

static HEADERS: &[&str] = &[
    "include/pgm/atomic.h",
    "include/pgm/engine.h",
    "include/pgm/error.h",
    "include/pgm/gsi.h",
    "include/pgm/if.h",
    "include/pgm/in.h",
    "include/pgm/list.h",
    "include/pgm/macros.h",
    "include/pgm/mem.h",
    "include/pgm/messages.h",
    "include/pgm/msgv.h",
    "include/pgm/packet.h",
    "include/pgm/pgm.h",
    "include/pgm/skbuff.h",
    "include/pgm/socket.h",
    "include/pgm/time.h",
    "include/pgm/tsi.h",
    "include/pgm/types.h",
    "include/pgm/version.h",
    "include/pgm/winint.h",
    "include/pgm/wininttypes.h",
    "include/pgm/zinttypes.h",
];

fn configure(build: &mut Build, root: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    #[cfg(not(target_env = "msvc"))]
    build.std("gnu99");

    build
        .flag_if_supported("-march=native")
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wextra")
        .flag_if_supported("-Wfloat-equal")
        .flag_if_supported("-Wshadow")
        .flag_if_supported("-Wpointer-arith")
        .flag_if_supported("-Wbad-function-cast")
        .flag_if_supported("-Wcast-qual")
        .flag_if_supported("-Wcast-align")
        .flag_if_supported("-Wwrite-strings")
        .flag_if_supported("-Waggregate-return")
        .flag_if_supported("-Wstrict-prototypes")
        .flag_if_supported("-Wold-style-definition")
        .flag_if_supported("-Wmissing-prototypes")
        .flag_if_supported("-Wmissing-declarations")
        .flag_if_supported("-Wmissing-noreturn")
        .flag_if_supported("-Wmissing-format-atrributes")
        .flag_if_supported("-Wredundant-decls")
        .flag_if_supported("-Wnested-externs")
        .flag_if_supported("-Winline")
        .flag_if_supported("-Wno-inline")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-cast-align")
        .flag_if_supported("-Wno-keyword-macro")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-value")
        .flag_if_supported("-pedantic")
        .flag_if_supported("-Wno-deprecated")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-incompatible-pointer-types")
        .flag_if_supported("-Wno-incompatible-pointer-types-discards-qualifiers")
        .flag_if_supported("-Wno-gnu-folding-constant")
        .flag_if_supported("-Wno-pointer-bool-conversion")
        .flag_if_supported("-Wno-maybe-uninitialized")
        .flag_if_supported("-Wno-address")
        .flag_if_supported("-Wno-discarded-qualifiers")
        .flag_if_supported("-Wno-aggregate-return")
        .flag_if_supported("/LTCG")
        .flag_if_supported("/MP")
        .flag_if_supported("/arch:AVX2");

    build
        .define("GETTEXT_PACKAGE", "pgm")
        .define("HAVE_ISO_VARARGS", None)
        .define("HAVE_DSO_VISIBILITY", None)
        .define("USE_BIND_INADDR_ANY", None)
        .define("USE_TICKET_SPINLOCK", None)
        .define("USE_DUMB_RWSPINLOCK", None)
        .define("USE_GALOIS_MUL_LUT", None);
    #[cfg(target_os = "windows")]
    build
        .define("HAVE_FTIME", None)
        .define("HAVE_RDTSC", None)
        .define("HAVE_WSACMSGHDR", None);
    #[cfg(all(target_os = "windows", not(target_env = "msvc")))]
    build
        .define("_BSD_SOURCE", None)
        .define("_WIN32_WINNT", "0x0501")
        .define("_REENTRANT", None)
        .define("HAVE_STRUCT_IP_MSFILTER", None);
    #[cfg(target_env = "msvc")]
    build
        .define("WIN32", None)
        .define("_CRT_SECURE_NO_WARNINGS", None)
        .define("_WINSOCK_DEPRECATED_NO_WARNINGS", None);
    #[cfg(target_os = "macos")]
    build
        .define("__APPLE_USE_RFC_3542", None)
        .define("_REENTRANT", None)
        .define("HAVE_GETNETENT", None)
        .define("HAVE_FTIME", None)
        .define("HAVE_GETTIMEOFDAY", None)
        .define("HAVE_CLOCKGETTIME", None)
        .define("HAVE_PSELECT", None)
        .define("HAVE_POLL", None)
        .define("HAVE_GETIFADDRS", None)
        .define("HAVE_STRUCT_IFADDRS_IFR_NETMASK", None)
        .define("HAVE_SPRINTF_GROUPING", None)
        .define("HAVE_VASPRINTF", None)
        .define("HAVE_HOST_ORDER_IP_LEN", None)
        .define("HAVE_HOST_ORDER_IP_OFF", None)
        .define("HAVE_GETOPT", None);
    #[cfg(target_env = "gnu")]
    build
        .define("_DEFAULT_SOURCE", None)
        .define("_REENTRANT", None)
        .define("HAVE_PTHREAD_SPINLOCK", None)
        .define("HAVE_GETPROTOBYNAME_R", None)
        .define("HAVE_GETNETENT", None)
        .define("HAVE_ALLOCA_H", None)
        .define("HAVE_PROC_CPUINFO", None)
        .define("HAVE_BACKTRACE", None)
        .define("HAVE_GETTIMEOFDAY", None)
        .define("HAVE_CLOCKGETTIME", None)
        .define("HAVE_PSELECT", None)
        .define("HAVE_DEV_RTC", None)
        .define("HAVE_RDTSC", None)
        .define("HAVE_POLL", None)
        .define("HAVE_EPOLL_CTL", None)
        .define("HAVE_GETIFADDRS", None)
        .define("HAVE_STRUCT_IFADDRS_IFR_NETMASK", None)
        .define("HAVE_STRUCT_GROUP_REQ", None)
        .define("HAVE_STRUCT_IP_MREQN", None)
        .define("HAVE_STRUCT_IP_MSFILTER", None)
        .define("HAVE_SPRINTF_GROUPING", None)
        .define("HAVE_VASPRINTF", None)
        .define("HAVE_GETOPT", None);

    build.files(SOURCES.iter().map(|src| {
        let file = root.as_ref().join(src);
        println!("cargo:rerun-if-changed={}", file.display());

        file
    }));

    let version_path = generate_version_src()?;
    build.file(&version_path);

    build.include(root.as_ref().join("include"));

    Ok(())
}

fn generate_version_src() -> Result<PathBuf, Box<dyn Error>> {
    let version_path = PathBuf::from(env::var("OUT_DIR")?).join("version.c");
    let mut version_file = fs::File::create(&version_path)?;

    let build_dt = Local::now();
    let version_text = format!(
        r#"
#ifdef HAVE_CONFIG_H
#	include <config.h>
#endif
#include <impl/framework.h>
#include <pgm/version.h>

const unsigned pgm_major_version = 5;
const unsigned pgm_minor_version = 3;
const unsigned pgm_micro_version = 128;
const char* pgm_build_date = "{}";
const char* pgm_build_time = "{}";
const char* pgm_build_system = "{}";
const char* pgm_build_machine = "{}";
const char* pgm_build_revision = "";
"#,
        build_dt.format("%Y-%m-%d"),
        build_dt.format("%H:%M:%S"),
        env::var("CARGO_CFG_TARGET_OS")?,
        env::var("CARGO_CFG_TARGET_ARCH")?
    );
    version_file
        .write_all(version_text.as_bytes())
        .expect("could not generate version file");
    println!("cargo:rerun-if-changed={}", version_path.display());

    Ok(version_path)
}

fn provide_includes() -> Result<(), Box<dyn Error>> {
    let vendor_base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor/openpgm/pgm");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let include_dir = out_dir.join("include/pgm");
    if !include_dir.is_dir() {
        fs::create_dir_all(&include_dir)?;
    }

    HEADERS
        .iter()
        .try_for_each(|header| {
            let header_file = vendor_base.join(header);

            if let Some(file_name) = header_file.file_name() {
                fs::copy(header_file.as_path(), include_dir.join(file_name).as_path())?;
            }
            Ok::<_, Box<dyn Error>>(())
        })
        .expect("unable to copy include dir");

    println!(
        "cargo:include={}",
        include_dir
            .parent()
            .expect("include dir has no parent")
            .display()
    );

    Ok(())
}

fn compile() -> Result<(), Box<dyn Error>> {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let vendor = base_dir.join("vendor");
    let vendor_base = vendor.join("openpgm/pgm");

    let mut build = Build::new();
    configure(&mut build, &vendor_base)?;

    build.out_dir(out_dir.join("lib"));
    build.compile("pgm");

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=pgm",);
    println!("cargo:lib={}", out_dir.join("lib").display());
    println!("cargo:out={}", out_dir.display());

    provide_includes()?;

    Ok(())
}

#[cfg(feature = "bindings")]
fn generate_bindings() -> Result<(), Box<dyn Error>> {
    let vendor_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor");
    let include_dir = vendor_dir.join("openpgm").join("pgm").join("include");

    let builder = bindgen::Builder::default()
        .header(include_dir.join("pgm").join("pgm.h").to_string_lossy())
        .clang_arg(format!("-I{}", include_dir.display()))
        .size_t_is_usize(true)
        .derive_default(true)
        .derive_debug(true)
        .use_core();

    let bindings = builder.generate()?;

    bindings.write_to_file(PathBuf::from(env::var("OUT_DIR")?).join("bindings.rs"))?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");

    compile()?;
    #[cfg(feature = "bindings")]
    generate_bindings()?;

    Ok(())
}
