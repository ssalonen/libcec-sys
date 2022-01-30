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
