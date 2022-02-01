#[cfg(abi4)]
mod lib_abi4;
#[cfg(abi4)]
pub use crate::lib_abi4::*;
#[cfg(abi5)]
mod lib_abi5;
#[cfg(abi5)]
pub use crate::lib_abi5::*;
#[cfg(abi6)]
mod lib_abi6;
#[cfg(abi6)]
pub use crate::lib_abi6::*;
#[cfg(not(any(abi4, abi5, abi6)))]
compile_error!("BUG: libcec abi not detected");

#[cfg(test)]
mod tests {

    use crate::CEC_LIB_VERSION_MAJOR;

    #[test]
    fn test_abi_ci() {
        let expected_abi = option_env!("EXPECTED_LIBCEC_VERSION_MAJOR");
        if let Some(expected_abi) = expected_abi {
            assert_eq!(
                CEC_LIB_VERSION_MAJOR + 1,
                expected_abi
                    .parse()
                    .expect("Invalid EXPECTED_LIBCEC_VERSION_MAJOR: could not parse to number")
            );
        }
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
