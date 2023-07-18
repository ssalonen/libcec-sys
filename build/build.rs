use fs_extra::dir::copy as copy_dir;
use fs_extra::dir::CopyOptions;
use std::env;
use std::fs;
use std::fs::remove_dir_all;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;

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
}

impl CecVersion {
    fn major(&self) -> u32 {
        match *self {
            Self::V4 => 4,
            Self::V5 => 5,
            Self::V6 => 6,
        }
    }
}

// libcec versions that are supported when linking dynamically. In preference order
const CEC_MAJOR_VERSIONS: [CecVersion; 3] = [CecVersion::V6, CecVersion::V5, CecVersion::V4];

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
            set(LIB_INFO \"\")
            set(SKIP_PYTHON_WRAPPER \"1\")",
        )
        .unwrap_or_else(|_| panic!("Error writing {}", &set_build_info_path.to_string_lossy()));
}

#[cfg(not(target_os = "windows"))]
fn compile_vendored_platform(dst: &Path) {
    let platform_build = dst.join(PLATFORM_BUILD);
    // let tmp_libcec_src = dst.join(LIBCEC_SRC);
    fs::create_dir_all(&platform_build).unwrap();
    println!("cmake platform");
    cmake::Config::new(dst.join(LIBCEC_SRC).join("src").join("platform"))
        .out_dir(&platform_build)
        .env(P8_PLATFORM_ROOT_ENV, &platform_build)
        .build();

    println!("make platform");
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
    println!("cmake libcec");
    cmake::Config::new(dst.join(LIBCEC_SRC))
        .out_dir(&libcec_build)
        .env(P8_PLATFORM_ROOT_ENV, &platform_build)
        .build();

    println!("make libcec");
    Command::new("make")
        .current_dir(&libcec_build)
        .env(P8_PLATFORM_ROOT_ENV, &platform_build)
        .status()
        .expect("failed to make libcec!");
}

#[cfg(target_os = "windows")]
fn compile_vendored_libcec(dst: &Path) {
    // All the compilation steps are combined into one command.
    println!("build libcec");
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
    remove_dir_all(libcec_build.join("cmake").join(ARCHITECTURE));

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
        .arg(libcec_build.join("cmake").join(ARCHITECTURE)) // aka "BUILDTARGET" in windows\build-lib.cmd
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
        .arg(libcec_build.join("cmake").join(ARCHITECTURE)) // aka "BUILDTARGET" in windows\build-lib.cmd
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
    println!("\n\nUsing 'smoke test' to find out if libcec is installed");
    for abi in CEC_MAJOR_VERSIONS {
        let mut cc_cmd = compiler.to_command();
        println!("\n\nSmoke testing with libcec major {}", abi.major());
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
        println!("smoke_abi{} -> fail: {:?}", abi.major(), cc_cmd.output());
    }
    Err(())
}

fn libcec_installed_pkg_config() -> Result<CecVersion, ()> {
    println!("\n\nUsing pkg-config to find out if libcec is installed");
    for abi in CEC_MAJOR_VERSIONS {
        println!("\n\npkg-config with libcec major {}", abi.major());
        let pkg_config_result = pkg_config::Config::new()
            .atleast_version(&abi.major().to_string())
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
    println!("\n\nBuilding vendored libcec");
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
    println!("Building libcec from local source");
    prepare_vendored_build(&dst);
    #[cfg(not(target_os = "windows"))]
    {
        compile_vendored_platform(&dst);
    }
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

    // Try discovery using pkg-config
    if !cfg!(feature = "vendored") {
        let version = libcec_installed_pkg_config();
        if let Ok(version) = version {
            // pkg-config found the package and the parameters will be used for linking
            println!("cargo:libcec_version_major={}", version.major());
            println!("cargo:rustc-cfg=abi{}", version.major());
            return;
        }
        // Try smoke-test build using -lcec. If unsuccessful, revert to vendored sources
        let version = libcec_installed_smoke_test();
        if let Ok(version) = version {
            println!("cargo:rustc-link-lib=cec");
            println!("cargo:libcec_version_major={}", version.major());
            println!("cargo:rustc-cfg=abi{}", version.major());
            return;
        }
    }
    // Either vendored build has been explicitly requested (feature=vendored)
    // or we could not detect system-installed libcec
    // => fallback to compiling vendored
    compile_vendored();
}
