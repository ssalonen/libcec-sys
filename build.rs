use copy_dir::copy_dir;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

const MIN_LIBCEC_VERSION: &str = "4.0.0";

const P8_PLATFORM_DIR_ENV: &str = "p8-platform_DIR";
const LIBCEC_BUILD: &str = "libcec_build";
const PLATFORM_BUILD: &str = "platform_build";
const LIBCEC_SRC: &str = "vendor";

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
    cmake::Config::new(dst.join(LIBCEC_SRC).join("src").join("platform"))
        .out_dir(&platform_build)
        .env(P8_PLATFORM_DIR_ENV, &platform_build)
        .build();
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
    cmake::Config::new(&dst.join(LIBCEC_SRC))
        .out_dir(&libcec_build)
        .env(P8_PLATFORM_DIR_ENV, &platform_build)
        .build();

    Command::new("make")
        .current_dir(&libcec_build)
        .env(P8_PLATFORM_DIR_ENV, &platform_build)
        .status()
        .expect("failed to make libcec!");
}

fn libcec_installed_smoke_test() -> bool {
    let compiler = cc::Build::new().get_compiler();
    let compiler_path = compiler.path();
    let mut cc_cmd = Command::new(compiler_path);
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    cc_cmd
        .arg("src/smoke.c")
        .arg("-o")
        .arg(dst.join("smoke_out"))
        .arg("-lcec");
    if let Ok(status) = cc_cmd.status() {
        if status.success() {
            return true;
        }
    }
    false
}

fn compile_vendored() {
    let cmakelists = format!("{}/CMakeLists.txt", LIBCEC_SRC);
    let libcec_git_dir = Path::new(&cmakelists);
    if !libcec_git_dir.exists() {
        panic!(
            "git submodules (tested {}, working dir {}) are not properly initialized! Aborting.",
            libcec_git_dir.display(),
            env::current_dir()
                .expect("Unknown working directory")
                .display()
        )
    }
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

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if cfg!(feature = "vendored") {
        // vendored build explicitly requested. Build vendored sources
        compile_vendored();
    }
    // Try discovery using pkg-config
    else if pkg_config::Config::new()
        .atleast_version(MIN_LIBCEC_VERSION)
        .probe("libcec")
        .is_ok()
    {
        // pkg-config found the package and the parameters will be used for linking
    }
    // Try smoke-test build using -lcec. If unsuccessful, revert to vendored sources
    else if libcec_installed_smoke_test() {
        println!("cargo:rustc-link-lib=cec");
    } else {
        compile_vendored();
    }
}
