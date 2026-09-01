extern crate core;

use core::error::Error;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use cc::Build;

static SOURCES: &[&str] = &[
    "common/protoAddress.cpp",
    "common//protoApp.cpp",
    "common/protoBase64.cpp",
    "common/protoBitmask.cpp",
    "common/protoCap.cpp",
    "common/protoChannel.cpp",
    "common/protoDebug.cpp",
    "common/protoDispatcher.cpp",
    "common/protoEvent.cpp",
    "common/protoFile.cpp",
    "common/protoFlow.cpp",
    "common/protoGraph.cpp",
    "common/protoJson.cpp",
    "common/protoLFSR.cpp",
    "common/protoList.cpp",
    "common/protoNet.cpp",
    "common/protoPipe.cpp",
    "common/protoPkt.cpp",
    "common/protoPktARP.cpp",
    "common/protoPktETH.cpp",
    "common/protoPktGRE.cpp",
    "common/protoPktIGMP.cpp",
    "common/protoPktIP.cpp",
    "common/protoPktRTP.cpp",
    "common/protoQueue.cpp",
    "common/protoRouteMgr.cpp",
    "common/protoRouteTable.cpp",
    "common/protoSerial.cpp",
    "common/protoSocket.cpp",
    "common/protoSpace.cpp",
    "common/protoString.cpp",
    "common/protoTime.cpp",
    "common/protoTimer.cpp",
    "common/protoTree.cpp",
    "common/protoVif.cpp",
];

static HEADERS: &[&str] = &[
    "include/manetGraph.h",
    "include/manetGraphML.h",
    "include/manetMsg.h",
    "include/ns3ProtoSimAgent.h",
    "include/protoAddress.h",
    "include/protoApp.h",
    "include/protoAverage.h",
    "include/protoBase64.h",
    "include/protoBitmask.h",
    "include/protoCap.h",
    "include/protoChannel.h",
    "include/protoCheck.h",
    "include/protoDebug.h",
    "include/protoDefs.h",
    "include/protoDetour.h",
    "include/protoDispatcher.h",
    "include/protoEvent.h",
    "include/protoFile.h",
    "include/protoFlow.h",
    "include/protoGraph.h",
    "include/protoJson.h",
    "include/protoLFSR.h",
    "include/protoList.h",
    "include/protoNet.h",
    "include/protoNotify.h",
    "include/protoPipe.h",
    "include/protoPkt.h",
    "include/protoPktARP.h",
    "include/protoPktETH.h",
    "include/protoPktGRE.h",
    "include/protoPktIGMP.h",
    "include/protoPktIP.h",
    "include/protoPktRIP.h",
    "include/protoPktRTP.h",
    "include/protoPktTCP.h",
    "include/protoQueue.h",
    "include/protoRouteMgr.h",
    "include/protoRouteTable.h",
    "include/protoSerial.h",
    "include/protoSimAgent.h",
    "include/protoSocket.h",
    "include/protoSpace.h",
    "include/protoString.h",
    "include/protoTime.h",
    "include/protoTimer.h",
    "include/protoTree.h",
    "include/protoVersion.h",
    "include/protoVif.h",
    "include/protokit.h",
    "include/testFuncs.h",
    "include/wxProtoApp.h",
    "include/unix/zebraRouteMgr.h",
];

fn configure(build: &mut Build, root: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    build.cpp(true).pic(true).cargo_metadata(true);

    build
        .flag_if_supported("-march=native")
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wno-extra")
        .flag_if_supported("-Wno-gnu-zero-variadic-macro-arguments")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-nontrivial-memcall")
        .flag_if_supported("-Wno-overloaded-virtual")
        .flag_if_supported("-pedantic")
        .flag_if_supported("/EHsc")
        .flag_if_supported("/MP");

    #[cfg(target_env = "msvc")]
    {
        build
            .define("_CRT_SECURE_NO_WARNINGS", None)
            .define("WIN32", None)
            .define("_WINDOWS", None)
            .define("NDEBUG", None)
            .define("HAVE_ASSERT", None)
            .define("NO_SCM_RIGHTS", None);
        build.files(
            ["win32/win32Net.cpp", "win32/win32RouteMgr.cpp"]
                .iter()
                .map(|src| {
                    let file = root.as_ref().join("src").join(src);
                    println!("cargo:rerun-if-changed={}", file.display());

                    file
                }),
        );
    }
    #[cfg(target_family = "unix")]
    {
        build.define("UNIX", None).define("_FILE_OFFSET_BITS", "64");
        build.files(
            [
                "unix/unixNet.cpp",
                "unix/unixSerial.cpp",
                "unix/unixVif.cpp",
                "manet/manetGraph.cpp",
                "manet/manetMsg.cpp",
            ]
            .iter()
            .map(|src| {
                let file = root.as_ref().join("src").join(src);
                println!("cargo:rerun-if-changed={}", file.display());

                file
            }),
        );
    }
    #[cfg(target_os = "macos")]
    {
        build
            .define("MACOOSX", None)
            .define("HAVE_ASSERT", None)
            .define("HAVE_LOCKF", None)
            .define("HAVE_FLOCK", None)
            .define("HAVE_PSELECT", None)
            .define("NO_SCM_RIGHTS", None)
            .define("USE_SELECT", None);
        build.files(
            [
                "bsd/bsdDetour.cpp",
                "bsd/bsdRouteMgr.cpp",
                "bsd/bsdNet.cpp",
                "unix/bpfCap.cpp",
            ]
            .iter()
            .map(|src| {
                let file = root.as_ref().join("src").join(src);
                println!("cargo:rerun-if-changed={}", file.display());

                file
            }),
        );
    }
    #[cfg(target_os = "linux")]
    {
        build
            .define("LINUX", None)
            .define("HAVE_ASSERT", None)
            .define("HAVE_FLOCK", None)
            .define("HAVE_IPV6", None)
            .define("HAVE_LOCKF", None)
            .define("HAVE_PSELECT", None)
            .define("HAVE_SCHED", None)
            .define("NO_SCM_RIGHTS", None)
            .define("USE_EPOLL", None)
            .define("USE_EVENTFD", None)
            .define("USE_TIMERFD", None);
        build.files(
            [
                "linux/linuxCap.cpp",
                "linux/linuxNet.cpp",
                "linux/linuxRouteMgr.cpp",
                "unix/zebraRouteMgr.cpp",
            ]
            .iter()
            .map(|src| {
                let file = root.as_ref().join("src").join(src);
                println!("cargo:rerun-if-changed={}", file.display());

                file
            }),
        );
    }

    build.files(SOURCES.iter().map(|src| {
        let file = root.as_ref().join("src").join(src);
        println!("cargo:rerun-if-changed={}", file.display());

        file
    }));

    build.include(root.as_ref().join("include"));

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

    println!("cargo:include={}", include_dir.display());

    Ok(())
}

fn compile() -> Result<(), Box<dyn Error>> {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let vendor = base_dir.join("vendor");

    let mut build = Build::new();
    configure(&mut build, &vendor)?;

    build.out_dir(out_dir.join("lib"));
    build.compile("protokit");

    #[cfg(target_env = "msvc")]
    {
        println!("cargo:rustc-link-lib=ws2_32",);
        println!("cargo:rustc-link-lib=iphlpapi",);
        println!("cargo:rustc-link-lib=user32",);
        println!("cargo:rustc-link-lib=gdi32",);
        println!("cargo:rustc-link-lib=Advapi32",);
        println!("cargo:rustc-link-lib=ntdll",);
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

#[cfg(feature = "bindings")]
fn generate_bindings() -> Result<(), Box<dyn Error>> {
    let vendor_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor");
    let include_dir = vendor_dir.join("include");

    let builder = bindgen::Builder::default()
        .header(include_dir.join("protokit.h").to_string_lossy())
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
