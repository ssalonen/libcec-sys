mod bindings {

    #![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
    cfg_if::cfg_if! {
        if #[cfg(all(abi4, target_os = "windows", target_arch = "x86_64", target_env = "msvc"))] {
            include!("lib_abi4_x86_64-pc-windows-msvc.rs");
        } else if #[cfg(all(abi5, target_os = "windows", target_arch = "x86_64", target_env = "msvc"))] {
            include!("lib_abi5_x86_64-pc-windows-msvc.rs");
        } else if #[cfg(all(abi6, target_os = "windows", target_arch = "x86_64", target_env = "msvc"))] {
            include!("lib_abi6_x86_64-pc-windows-msvc.rs");
        }
        // Mac not tested/supported
        // else if #[cfg(all(abi4, target_os = "macos", target_arch = "aarch64"))] {
        //     include!("lib_abi4_aarch64-apple-darwin.rs");
        // } else if #[cfg(all(abi5, target_os = "macos", target_arch = "aarch64"))] {
        //     include!("lib_abi5_aarch64-apple-darwin.rs");
        // } else if #[cfg(all(abi6, target_os = "macos", target_arch = "aarch64"))] {
        //     include!("lib_abi6_aarch64-apple-darwin.rs");
        // }
        else if #[cfg(all(abi4, target_os = "linux", target_arch = "x86_64", target_env = "gnu"))] {
            include!("lib_abi4_x86_64-unknown-linux-gnu.rs");
        } else if #[cfg(all(abi5, target_os = "linux", target_arch = "x86_64", target_env = "gnu"))] {
            include!("lib_abi5_x86_64-unknown-linux-gnu.rs");
        } else if #[cfg(all(abi6, target_os = "linux", target_arch = "x86_64", target_env = "gnu"))] {
            include!("lib_abi6_x86_64-unknown-linux-gnu.rs");
        } else if #[cfg(all(abi4, target_os = "linux", target_arch = "armv7", target_env = "gnueabihf"))] {
            include!("lib_abi4_armv7-unknown-linux-gnueabihf.rs");
        } else if #[cfg(all(abi5, target_os = "linux", target_arch = "armv7", target_env = "gnueabihf"))] {
            include!("lib_abi5_armv7-unknown-linux-gnueabihf.rs");
        } else if #[cfg(all(abi6, target_os = "linux", target_arch = "armv7", target_env = "gnueabihf"))] {
            include!("lib_abi6_armv7-unknown-linux-gnueabihf.rs");
        } else if #[cfg(all(abi4, target_os = "linux", target_arch = "arm", target_env = "gnueabi"))] {
            include!("lib_abi4_arm-unknown-linux-gnueabi.rs");
        } else if #[cfg(all(abi5, target_os = "linux", target_arch = "arm", target_env = "gnueabi"))] {
            include!("lib_abi5_arm-unknown-linux-gnueabi.rs");
        } else if #[cfg(all(abi6, target_os = "linux", target_arch = "arm", target_env = "gnueabi"))] {
            include!("lib_abi6_arm-unknown-linux-gnueabi.rs");
        }

        else {
            compile_error!("unsupported platform");
        }
    }
}

pub use crate::bindings::*;

#[cfg(test)]
mod tests {

    use crate::CEC_LIB_VERSION_MAJOR;
    use std::env;

    #[test]
    fn test_abi_ci() {
        if env::var("CI").is_err() {
            // Not running in CI
            return;
        }
        let expected_abi = env::var("EXPECTED_LIBCEC_VERSION_MAJOR")
            .expect("CI needs to specify EXPECTED_LIBCEC_VERSION_MAJOR");

        assert_eq!(
            CEC_LIB_VERSION_MAJOR,
            expected_abi
                .parse()
                .expect("Invalid EXPECTED_LIBCEC_VERSION_MAJOR: could not parse to number")
        );
    }

    #[cfg(abi4)]
    #[test]
    fn test_abi4() {
        assert_eq!(CEC_LIB_VERSION_MAJOR, 4);
    }

    #[cfg(abi5)]
    #[test]
    fn test_abi5() {
        assert_eq!(CEC_LIB_VERSION_MAJOR, 5);
    }

    #[cfg(abi6)]
    #[test]
    fn test_abi6() {
        assert_eq!(CEC_LIB_VERSION_MAJOR, 6);
    }
}
