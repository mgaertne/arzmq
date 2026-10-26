extern crate core;

use core::error::Error;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use cc::Build;

static SOURCES: &[&str] = &[
    "common/galois.cpp",
    "common/normApi.cpp",
    "common/normEncoder.cpp",
    "common/normEncoderMDP.cpp",
    "common/normEncoderRS16.cpp",
    "common/normEncoderRS8.cpp",
    "common/normFile.cpp",
    "common/normMessage.cpp",
    "common/normNode.cpp",
    "common/normObject.cpp",
    "common/normSegment.cpp",
    "common/normSession.cpp",
];

static HEADERS: &[&str] = &[
    "include/galois.h",
    "include/normApi.h",
    "include/normEncoder.h",
    "include/normEncoderMDP.h",
    "include/normEncoderRS16.h",
    "include/normEncoderRS8.h",
    "include/normFile.h",
    "include/normMessage.h",
    "include/normNode.h",
    "include/normObject.h",
    "include/normPostProcess.h",
    "include/normSegment.h",
    "include/normSession.h",
    "include/normSimAgent.h",
    "include/normVersion.h",
];

fn configure(build: &mut Build, root: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    build.cpp(true).pic(true).cargo_metadata(true);

    #[cfg(feature = "native")]
    build.flag_if_supported("-march=native");

    build
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wno-extra")
        .flag_if_supported("-Wno-gnu-zero-variadic-macro-arguments")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-misleading-indentation")
        .flag_if_supported("-Wno-unused-private-field")
        .flag_if_supported("-Wno-ignored-attributes")
        .flag_if_supported("-pedantic")
        .flag_if_supported("/EHsc")
        .flag_if_supported("/MP")
        .flag_if_supported("/Qspectre");

    #[cfg(target_env = "msvc")]
    {
        build
            .define("_CRT_SECURE_NO_WARNINGS", None)
            .define("WIN32", None)
            .define("_WINDOWS", None)
            .define("NDEBUG", None)
            .define("HAVE_ASSERT", None)
            .define("NO_SCM_RIGHTS", None);
    }
    #[cfg(target_family = "unix")]
    build.define("UNIX", None).define("_FILE_OFFSET_BITS", "64");
    #[cfg(target_os = "macos")]
    {
        build
            .define("MACOOSX", None)
            .define("HAVE_DIRFD", None)
            .define("HAVE_ASSERT", None)
            .define("HAVE_LOCKF", None)
            .define("HAVE_FLOCK", None)
            .define("HAVE_IPV6", None)
            .define("HAVE_PSELECT", None)
            .define("NO_SCM_RIGHTS", None)
            .define("USE_SELECT", None);
    }
    #[cfg(target_os = "linux")]
    {
        build
            .define("LINUX", None)
            .define("HAVE_ASSERT", None)
            .define("HAVE_DIRFD", None)
            .define("HAVE_FLOCK", None)
            .define("HAVE_IPV6", None)
            .define("HAVE_LOCKF", None)
            .define("HAVE_PSELECT", None)
            .define("HAVE_SCHED", None)
            .define("NO_SCM_RIGHTS", None)
            .define("USE_EPOLL", None)
            .define("USE_EVENTFD", None)
            .define("USE_TIMERFD", None);
    }

    build.files(SOURCES.iter().map(|src| {
        let file = root.as_ref().join("src").join(src);
        println!("cargo:rerun-if-changed={}", file.display());

        file
    }));

    build.include(root.as_ref().join("include"));
    build.include(env::var("DEP_PROTOKIT_INCLUDE")?);

    Ok(())
}

fn provide_includes() -> Result<(), Box<dyn Error>> {
    let vendor_base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let include_dir = out_dir.join("include");
    if !include_dir.is_dir() {
        fs::create_dir_all(&include_dir)?;
    }

    HEADERS.iter().try_for_each(|header| {
        let header_file = vendor_base.join(header);

        if let Some(file_name) = header_file.file_name() {
            fs::copy(header_file.as_path(), include_dir.join(file_name).as_path())?;
        }
        Ok::<_, Box<dyn Error>>(())
    })?;

    println!("cargo:include={}", include_dir.display(),);

    Ok(())
}

fn compile() -> Result<(), Box<dyn Error>> {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let vendor = base_dir.join("vendor");

    let mut build = Build::new();
    configure(&mut build, &vendor)?;

    build.out_dir(out_dir.join("lib"));
    build.compile("norm");

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=User32",);
    }
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=dl",);
        println!("cargo:rustc-link-lib=rt",);
    }
    println!("cargo:lib={}", out_dir.join("lib").display());
    println!("cargo:out={}", out_dir.display());

    provide_includes()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");

    compile()
}
