use copy_dir::copy_dir;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;

const P8_PLATFORM_DIR_ENV: &str = "p8-platform_DIR";
const LIBCEC_BUILD: &str = "libcec_build";
const PLATFORM_BUILD: &str = "platform_build";
const LIBCEC_SRC: &str = "vendor";

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

// libcec versions that are supported when linking dynamically. In preferce order
const CEC_MAJOR_VERSIONS: [CecVersion; 3] = [CecVersion::V6, CecVersion::V5, CecVersion::V4];

fn prepare_vendored_build(dst: &Path) {
    let dst_src = dst.join(LIBCEC_SRC);
    if dst_src.exists() && dst_src.is_dir() {
        fs::remove_dir_all(&dst_src).expect("Failed to remove build dir");
    }
    copy_dir(LIBCEC_SRC, &dst_src).unwrap();

    // libcec build tries to embed git revision and other details
    // in LIB_INFO variable. This makes the build fail in certain cases.
    // Let's disable the complex logic by overriding the variable with a constant
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
        .write_all(b"set(LIB_INFO \"\")")
        .unwrap_or_else(|_| panic!("Error writing {}", &set_build_info_path.to_string_lossy()));
}

fn compile_vendored_platform(dst: &Path) {
    let platform_build = dst.join(PLATFORM_BUILD);
    // let tmp_libcec_src = dst.join(LIBCEC_SRC);
    fs::create_dir_all(&platform_build).unwrap();
    println!("cmake platform");
    cmake::Config::new(dst.join(LIBCEC_SRC).join("src").join("platform"))
        .out_dir(&platform_build)
        .env(P8_PLATFORM_DIR_ENV, &platform_build)
        .build();

    println!("make platform");
    Command::new("make")
        .current_dir(&platform_build)
        .env(P8_PLATFORM_DIR_ENV, &platform_build)
        .status()
        .expect("failed to make libcec platform!");
}

fn compile_vendored_libcec(dst: &Path) {
    let platform_build = dst.join(PLATFORM_BUILD);
    let libcec_build = dst.join(LIBCEC_BUILD);
    fs::create_dir_all(&libcec_build).unwrap();
    println!("cmake libcec");
    cmake::Config::new(&dst.join(LIBCEC_SRC))
        .out_dir(&libcec_build)
        .env(P8_PLATFORM_DIR_ENV, &platform_build)
        .build();

    println!("make libcec");
    Command::new("make")
        .current_dir(&libcec_build)
        .env(P8_PLATFORM_DIR_ENV, &platform_build)
        .status()
        .expect("failed to make libcec!");
}

fn libcec_installed_smoke_test() -> Result<CecVersion, ()> {
    let compiler = cc::Build::new().get_compiler();
    let compiler_path = compiler.path();
    let mut cc_cmd = Command::new(compiler_path);
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    for abi in CEC_MAJOR_VERSIONS {
        cc_cmd
            .arg(format!("build/smoke_abi{}.c", abi.major()))
            .arg("-o")
            .arg(dst.join("smoke_out"))
            .arg("-lcec");
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
    for abi in CEC_MAJOR_VERSIONS {
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
    println!("cargo:lib_vendored=true");

    let cmakelists = format!("{}/CMakeLists.txt", LIBCEC_SRC);
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
    println!("cargo:libcec_version_major={}", abi);
    println!("cargo:rustc-cfg=abi{}", abi);

    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("Building libcec from local source");
    prepare_vendored_build(&dst);
    compile_vendored_platform(&dst);
    compile_vendored_libcec(&dst);
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join(LIBCEC_BUILD).join("lib").display()
    );
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
    println!("cargo:rerun-if-changed=build.rs");
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
