use fs_extra::dir::copy as copy_dir;
use fs_extra::dir::CopyOptions;
use reqwest::StatusCode;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use target_lexicon::OperatingSystem;

#[cfg(not(target_os = "windows"))]
const P8_PLATFORM_ROOT_ENV: &str = "p8-platform_ROOT";
const LIBCEC_BUILD: &str = "libcec_build";
#[cfg(not(target_os = "windows"))]
const PLATFORM_BUILD: &str = "platform_build";
const LIBCEC_SRC: &str = "vendor";

#[cfg(target_os = "windows")]
const ARCHITECTURE: &str = if cfg!(target_pointer_width = "64") {
    "amd64"
} else {
    "x86"
};

enum CecVersion {
    V4,
    V5,
    V6,
    V7,
}

impl CecVersion {
    fn major(&self) -> u32 {
        match *self {
            Self::V4 => 4,
            Self::V5 => 5,
            Self::V6 => 6,
            Self::V7 => 7,
        }
    }
}

enum BuildMode {
    Vendored,
    DownloadStaticPrebuilt,
    Dynamic,
}

// libcec versions that are supported when linking dynamically. In preference order
const CEC_MAJOR_VERSIONS: [CecVersion; 4] = [
    CecVersion::V7,
    CecVersion::V6,
    CecVersion::V5,
    CecVersion::V4,
];

fn prepare_vendored_build(dst: &Path) {
    let dst_src = dst.join(LIBCEC_SRC);
    if dst_src.exists() && dst_src.is_dir() {
        fs::remove_dir_all(&dst_src).expect("Failed to remove build dir");
    }
    let copy_opts = CopyOptions::new().overwrite(true).copy_inside(true);
    copy_dir(LIBCEC_SRC, &dst_src, &copy_opts).unwrap();

    // libcec build tries to embed git revision and other details
    // in LIB_INFO variable. This makes the build fail in certain cases.
    // Let's disable the complex logic by overriding the variable with a constant
    //
    // In addition, we disable building of python wrappers, not needed
    let set_build_info_path = dst_src
        .join("src")
        .join("libcec")
        .join("cmake")
        .join("SetBuildInfo.cmake");
    let mut build_info_file = OpenOptions::new()
        .write(true)
        .open(&set_build_info_path)
        .unwrap_or_else(|_| panic!("Error opening {}", &set_build_info_path.to_string_lossy()));
    build_info_file
        .set_len(0)
        .expect("Error truncacting SetBuildInfo.cmake");
    build_info_file
        .write_all(
            b"
            set(LIB_INFO \"\")",
        )
        .unwrap_or_else(|_| panic!("Error writing {}", &set_build_info_path.to_string_lossy()));

    #[cfg(target_os = "windows")]
    prepare_windows_libcec_cmake_opts(&dst_src);
}

#[cfg(not(target_os = "windows"))]
fn compile_vendored_platform(dst: &Path) {
    let platform_build = dst.join(PLATFORM_BUILD);
    // let tmp_libcec_src = dst.join(LIBCEC_SRC);
    fs::create_dir_all(&platform_build).unwrap();
    println!("==============================================================\ncmake platform\n==============================================================");
    cmake::Config::new(dst.join(LIBCEC_SRC).join("src").join("platform"))
        .out_dir(&platform_build)
        .env(P8_PLATFORM_ROOT_ENV, &platform_build)
        .build();

    println!("==============================================================\nmake platform\n==============================================================");
    Command::new("make")
        .current_dir(&platform_build)
        .env(P8_PLATFORM_ROOT_ENV, &platform_build)
        .status()
        .expect("failed to make libcec platform!");
}

#[cfg(not(target_os = "windows"))]
fn compile_vendored_libcec(dst: &Path) {
    let platform_build = dst.join(PLATFORM_BUILD);
    let libcec_build = dst.join(LIBCEC_BUILD);
    fs::create_dir_all(&libcec_build).unwrap();
    println!("==============================================================\ncmake libcec\n==============================================================");
    let mut cmake_builder = cmake::Config::new(dst.join(LIBCEC_SRC));
    dbg!(&platform_build);
    cmake_builder
        .very_verbose(true)
        .out_dir(&libcec_build)
        .define("SKIP_PYTHON_WRAPPER", "1")
        // For some reason, with arm architectures we need to manually define
        // - p8-platform_DIR (folder with p8-platform-config.cmake),
        // - p8-platform_INCLUDE_DIRS (folder with include),
        // - p8-platform_LIBRARY (location to p8-platform .a archive file)
        //
        // Otherwise we get error that p8-platform-config.cmake.
        //
        // With x86_64-unknown-linux-gnu platform this work without hassle
        // using the p8-platform_ROOT hint
        .define("p8-platform_DIR", platform_build.join("build"))
        .define("p8-platform_INCLUDE_DIRS", platform_build.join("include"))
        .define(
            "p8-platform_LIBRARY",
            platform_build.join("build").join("libp8-platform.a"),
        )
        .env(P8_PLATFORM_ROOT_ENV, &platform_build);
    cmake_builder.build();

    println!("==============================================================\nmake libcec\n==============================================================");
    Command::new("make")
        .current_dir(&libcec_build)
        .env(P8_PLATFORM_ROOT_ENV, &platform_build)
        .status()
        .expect("failed to make libcec!");
}

#[cfg(target_os = "windows")]
fn compile_vendored_platform(dst: &Path) {
    let libcec_build = dst.join(LIBCEC_BUILD);
    Command::new("cmd")
        .current_dir(&dst.join(LIBCEC_SRC).join("project"))
        .arg("/C")
        .arg(
            dst.join(LIBCEC_SRC)
                .join("src")
                .join("platform")
                .join("windows")
                .join("build-lib.cmd"),
        )
        .arg(ARCHITECTURE)
        .arg(if cfg!(debug_assertions) {
            "Debug"
        } else {
            "Release"
        })
        .arg("2019")
        .arg(&libcec_build)
        .arg("nmake")
        .status()
        .expect("failed to build p8 platform!");
    // Remove build target of the p8 platform build
    // aka "BUILDTARGET" in windows\build-lib.cmd
    fs::remove_dir_all(libcec_build.join("cmake").join(ARCHITECTURE))
        .expect("Could not remove built target of p8 build");
}

#[cfg(target_os = "windows")]
fn prepare_windows_libcec_cmake_opts(dst_src: &Path) {
    //
    // We disable Python wrapper builds with vendored builds
    // It is not needed for the purposes of rust interfacing
    // and slows the build down.
    //
    let windows_cmake_gen_path = dst_src
        .join("support")
        .join("windows")
        .join("cmake")
        .join("generate.cmd");

    let contents =
        fs::read_to_string(&windows_cmake_gen_path).expect("Could not read cmake/generate.cmd");
    let new = contents.replace(
        "-DCMAKE_BUILD_TYPE=%BUILDTYPE% ^",
        &format!("-DCMAKE_BUILD_TYPE=%BUILDTYPE% -DSKIP_PYTHON_WRAPPER=1 ^"),
    );
    println!("==============================================================\n--- generate.cmd start ---\n==============================================================\n{new}\n==============================================================\n--- generate.cmd end ---\n==============================================================\n");
    // Content should have changed
    assert!(new.contains(" -DSKIP_PYTHON_WRAPPER=1 "));
    assert_ne!(new, contents);
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&windows_cmake_gen_path)
        .expect("Could not open cmake/generate.cmd for writing");
    file.write_all(new.as_bytes())
        .expect("Could not write cmake/generate.cmd");
}

#[cfg(target_os = "windows")]
fn compile_vendored_libcec(dst: &Path) {
    let libcec_build = dst.join(LIBCEC_BUILD);
    let build_target = libcec_build.join("cmake").join(ARCHITECTURE);
    Command::new("cmd")
        .current_dir(&dst.join(LIBCEC_SRC).join("project"))
        .arg("/C")
        .arg(
            dst.join(LIBCEC_SRC)
                .join("support")
                .join("windows")
                .join("cmake")
                .join("generate.cmd"),
        )
        .arg(ARCHITECTURE)
        .arg("nmake")
        .arg(dst.join(LIBCEC_SRC))
        .arg(&build_target) // aka "BUILDTARGET" in windows\build-lib.cmd
        .arg(libcec_build.join(ARCHITECTURE)) // aka "TARGET" in windows\build-lib.cmd
        .arg(if cfg!(debug_assertions) {
            "Debug"
        } else {
            "Release"
        })
        .arg("2019")
        .arg(&libcec_build)
        .status()
        .expect("failed to generate libcec build files!");

    // println!(
    //     "MAKEFILE: {}",
    //     fs::read_to_string(build_target.join("makefile")).expect("could not read makefile")
    // );
    // println!(
    //     "CMakeCache.txt: {}",
    //     fs::read_to_string(build_target.join("CMakeCache.txt"))
    //         .expect("could not read CMakeCache.txt")
    // );
    // println!(
    //     "cmake_install.cmake: {}",
    //     fs::read_to_string(build_target.join("cmake_install.cmake"))
    //         .expect("could not read cmake_install.cmake")
    // );

    Command::new("cmd")
        .current_dir(&dst.join(LIBCEC_SRC).join("project"))
        .arg("/C")
        .arg(
            dst.join(LIBCEC_SRC)
                .join("support")
                .join("windows")
                .join("cmake")
                .join("build.cmd"),
        )
        .arg(ARCHITECTURE)
        .arg(&build_target) // aka "BUILDTARGET" in windows\build-lib.cmd
        .arg("2019")
        .status()
        .expect("failed to build libcec!");
}

#[cfg(not(target_os = "windows"))]
fn link_libcec(dst: &Path) {
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join(LIBCEC_BUILD).join("lib").display()
    );
}

#[cfg(target_os = "windows")]
fn link_libcec(dst: &Path) {
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join(LIBCEC_BUILD).join(ARCHITECTURE).display()
    );
}

fn libcec_installed_smoke_test() -> Result<CecVersion, ()> {
    let compiler = cc::Build::new().get_compiler();
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("\n\n==============================================================\nUsing 'smoke test' to find out if libcec is installed\n==============================================================");
    for abi in CEC_MAJOR_VERSIONS {
        let mut cc_cmd = compiler.to_command();
        println!("\n\n==============================================================\nSmoke testing with libcec major {}\n==============================================================", abi.major());
        cc_cmd.arg(format!("build/smoke_abi{}.c", abi.major()));
        if cfg!(windows) {
            cc_cmd
                .arg("/Fe:")
                .arg(dst.join(format!("smoke_abi{}_out.exe", abi.major())));
        } else {
            cc_cmd
                .arg("-o")
                .arg(dst.join(format!("smoke_abi{}_out", abi.major())))
                .arg("-lcec");
        }
        if let Ok(status) = cc_cmd.status() {
            if status.success() {
                println!("smoke_abi{} -> ok", abi.major());
                return Ok(abi);
            }
        }
        println!("==============================================================\nsmoke_abi{} -> fail: {:?}\n==============================================================\n", abi.major(), cc_cmd.output().is_err());
    }
    Err(())
}

fn libcec_installed_pkg_config() -> Result<CecVersion, ()> {
    println!("\n\n==============================================================\nUsing pkg-config to find out if libcec is installed\n==============================================================");
    for abi in CEC_MAJOR_VERSIONS {
        println!("\n\npkg-config with libcec major {}", abi.major());
        let major = format!("{}.0.0", abi.major()); // inclusive
        let next_major = format!("{}.0.0", abi.major() + 1); // exclusive
        let pkg_config_result = pkg_config::Config::new()
            .range_version(major.as_str()..next_major.as_str())
            .probe("libcec");
        if pkg_config_result.is_ok() {
            println!("pkg_config(>={}) -> found", abi.major());
            return Ok(abi);
        } else {
            println!(
                "pkg_config(>={}) -> fail: {:?}",
                abi.major(),
                pkg_config_result
            )
        }
    }
    Err(())
}

fn compile_vendored() {
    println!("\n\n==============================================================\nBuilding vendored libcec\n==============================================================");
    println!("cargo:lib_vendored=true");

    let cmakelists = format!("{LIBCEC_SRC}/CMakeLists.txt");
    let cmakelists = Path::new(&cmakelists);
    if !cmakelists.exists() {
        panic!(
            "git submodules (tested {}, working dir {}) are not properly initialized! Aborting.",
            cmakelists.display(),
            env::current_dir()
                .expect("Unknown working directory")
                .display()
        )
    }
    let abi = parse_vendored_libcec_major_version(cmakelists);
    println!("cargo:libcec_version_major={abi}");
    println!("cargo:rustc-cfg=abi{abi}");

    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    prepare_vendored_build(&dst);
    compile_vendored_platform(&dst);
    compile_vendored_libcec(&dst);
    link_libcec(&dst);
    println!("cargo:rustc-link-lib=cec");
}

fn parse_vendored_libcec_major_version(cmakelists: &Path) -> u32 {
    let file = File::open(cmakelists).expect("Error opening cmakelists");
    let reader = BufReader::new(file);
    // Parse major version from line similar to    set(LIBCEC_VERSION_MAJOR 4)
    for line in reader.lines() {
        let line = line.expect("Error reading cmakelists");
        let mut numbers = String::new();
        if line.trim().starts_with("set(LIBCEC_VERSION_MAJOR ") {
            for char in line.chars() {
                if let '0'..='9' = char {
                    numbers.push(char)
                }
            }
            return numbers.parse().expect("major version parse failed");
        }
    }
    panic!("Could not parse LIBCEC_VERSION_MAJOR from cmakelists");
}

pub fn fetch_static_libcec<P: AsRef<Path>>(path: P, debug_build: bool) {
    println!("\n\n==============================================================\nFetching pre-built static libcec\n==============================================================");
    println!("cargo:lib_static=true");
    println!("cargo:libcec_version_major=7");
    println!("cargo:rustc-cfg=abi7");

    let target = env::var("TARGET").expect("Must have TARGET env variable in build.rs");
    let kind = if debug_build { "debug" } else { "release" };
    let url = format!("https://github.com/ssalonen/libcec-static-builds/releases/download/libcec-v7.0.0-202504-1/libcec-v7.0.0-static-{target}-{kind}.zip");
    dbg!(&target, kind, &url);

    let response = reqwest::blocking::get(&url)
        .unwrap_or_else(|_| panic!("failed to download libcec from {url}"));
    if response.status() == StatusCode::NOT_FOUND {
        panic!("Could not find pre-built static libcec for {}", &target);
    }
    response
        .error_for_status_ref()
        .unwrap_or_else(|e| panic!("Error downloading pre-built static libcec: {}", e));
    let file = response
        .bytes()
        .unwrap_or_else(|_| panic!("failed to download libcec from {url}"));
    zip_extract::extract(Cursor::new(file), path.as_ref(), true).unwrap_or_else(|e| {
        panic!(
            "failed to extract libcec archive to `{}`: {}",
            path.as_ref().to_string_lossy(),
            e
        )
    });
    let paths = std::fs::read_dir(path).unwrap();
    for path in paths {
        println!("Extracted: {}", path.unwrap().path().display())
    }
}

fn link_to_static() {
    let lib_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("libcec");
    let lib_path_str = lib_path.to_string_lossy();
    let debug_build = cfg!(debug_assertions);
    let target_triple = target_lexicon::Triple::from_str(
        &env::var("TARGET").expect("Must have TARGET env variable in build.rs"),
    )
    .expect("Failed to parse TARGET env variable");
    let target_os = target_triple.operating_system;

    dbg!(&lib_path, target_triple, debug_build);
    println!("cargo:rustc-link-search=native={lib_path_str}");
    println!("cargo:rustc-link-lib=static=cec-static");
    println!("cargo:rustc-link-lib=static=p8-platform");

    match (target_os, debug_build) {
        (OperatingSystem::Windows, true) => {
            println!("cargo:rustc-link-lib=dylib=msvcrtd");
        }
        (OperatingSystem::Windows, false) => {
            println!("cargo:rustc-link-lib=dylib=msvcrt");
        }
        (OperatingSystem::Darwin, _) => {
            println!("cargo:rustc-link-search=framework=/Library/Frameworks");
            println!("cargo:rustc-link-lib=dylib=c++");
            println!("cargo:rustc-link-lib=framework=CoreVideo");
            println!("cargo:rustc-link-lib=framework=IOKit");
        }
        (OperatingSystem::Linux, _) => {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
        _ => panic!("unsupported target"),
    };

    // Building libcec from source is _painful_, so we don't!
    fetch_static_libcec(&lib_path, debug_build);
}

fn find_using_pkg_config() -> bool {
    let version = libcec_installed_pkg_config();
    if let Ok(version) = version {
        // pkg-config found the package and the parameters will be used for linking
        println!("cargo:libcec_version_major={}", version.major());
        println!("cargo:rustc-cfg=abi{}", version.major());
        true
    } else {
        false
    }
}

fn find_using_smoke_test() -> bool {
    // Try smoke-test build using -lcec. If unsuccessful, revert to vendored sources
    let version = libcec_installed_smoke_test();
    if let Ok(version) = version {
        println!("cargo:rustc-link-lib=cec");
        println!("cargo:libcec_version_major={}", version.major());
        println!("cargo:rustc-cfg=abi{}", version.major());
        true
    } else {
        false
    }
}

fn determine_mode() -> BuildMode {
    let vendored_explicitly_via_env =
        env::var("LIBCEC_VENDORED").is_ok_and(|s| s != "0" && !s.is_empty());
    let vendored_forbidden_explicitly_via_env =
        env::var("LIBCEC_NO_VENDOR").is_ok_and(|s| s != "0" && !s.is_empty());
    let static_explicitly_via_env =
        env::var("LIBCEC_STATIC").is_ok_and(|s| s != "0" && !s.is_empty());

    if (cfg!(feature = "vendored") || vendored_explicitly_via_env)
        && !vendored_forbidden_explicitly_via_env
    {
        println!("Build mode: 'vendored' asked via feature or LIBCEC_VENDORED={:?} env, and not explicitly disabled via LIBCEC_NO_VENDOR={:?} env", env::var("LIBCEC_VENDORED"), env::var("LIBCEC_NO_VENDOR"));
        BuildMode::Vendored
    } else if cfg!(feature = "static") || static_explicitly_via_env {
        println!(
            "Build mode: 'static' asked via feature or LIBCEC_STATIC={:?} env",
            env::var("LIBCEC_STATIC")
        );
        BuildMode::DownloadStaticPrebuilt
    } else if find_using_pkg_config() {
        println!("Build mode: dynamic, found via pkg-config");
        // Found using pkg-config
        BuildMode::Dynamic
    } else if find_using_smoke_test() {
        // Found the library using smoke-test build using -lcec
        println!("Build mode: dynamic, found via smoke test");
        BuildMode::Dynamic
    } else {
        // => fallback to compiling static
        println!("Build mode: static (fallback). LIBCEC_VENDORED={:?}, LIBCEC_NO_VENDOR={:?}, LIBCEC_STATIC={:?}", env::var("LIBCEC_VENDORED"), env::var("LIBCEC_NO_VENDOR"), env::var("LIBCEC_STATIC"));
        BuildMode::DownloadStaticPrebuilt
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build");
    println!("cargo:rerun-if-changed=vendor");
    println!("cargo:rerun-if-env-changed=LD_LIBRARY_PATH");
    println!("cargo:rerun-if-env-changed=LDFLAGS");
    println!("cargo:rerun-if-env-changed=INCLUDE");
    println!("cargo:rerun-if-env-changed=PATH");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=CXXFLAGS");
    println!("cargo:rerun-if-env-changed=LIB");
    println!("cargo:rerun-if-env-changed=CL");
    println!("cargo:rerun-if-env-changed=_CL_");
    println!("cargo:rerun-if-env-changed=CMAKE_C_COMPILER_LAUNCHER");
    println!("cargo:rerun-if-env-changed=CMAKE_CXX_COMPILER_LAUNCHER");
    println!("cargo:rerun-if-env-changed=LIBCEC_VENDORED");
    println!("cargo:rerun-if-env-changed=LIBCEC_NO_VENDOR");
    println!("cargo:rerun-if-env-changed=LIBCEC_STATIC");

    let build_mode = determine_mode();

    // add hint to link to libudev if found
    let _ = pkg_config::find_library("libudev");

    match build_mode {
        BuildMode::Vendored => compile_vendored(),
        BuildMode::DownloadStaticPrebuilt => link_to_static(),
        BuildMode::Dynamic =>
            /* no building needed */
            {}
    }
}
