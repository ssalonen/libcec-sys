# libcec-sys

[![Crates.io](https://img.shields.io/crates/v/libcec-sys.svg)](https://crates.io/crates/libcec-sys)
[![Docs.rs](https://docs.rs/libcec-sys/badge.svg)](https://docs.rs/libcec-sys)
[![CI](https://github.com/ssalonen/libcec-sys/workflows/Continuous%20Integration/badge.svg)](https://github.com/ssalonen/libcec-sys/actions)

FFI bindings for the libcec
## Linking to system libcec

This crate works with `libcec` v4.x, v5.x and v6.x (latest version as time of writing). During the build we try to find `libcec` system library installation using `pkg-config` and compilation using default C compiler (`cc` crate). As a fallback, vendored `libcec` (v6.x) is used during the build.

Alternatively, one can the decide to skip logic above and force the use of vendored sources by enabling `vendored` feature.

The crate is tested mainly with Linux and Windows but could work with other platforms as well. PRs welcome.

### Linux
On Linux, for most convenient build process, it is recommended to install `pkg-config`, `libcec-dev` (headers and pkg-config configuration), `libcec6` (dynamic library), `p8-platform-dev` and `p8-platform` from your package distribution before installing this crate. Exact package names vary between distributions and package managers.

### Windows
On Windows, it is recommended to install `libcec` via the [installer](https://github.com/Pulse-Eight/libcec/releases/latest) and add `cec.dll` to the `PATH` environment variable.

For a vendored build, `libcec-sys` will dynamically link to the compiled `cec.dll`. This means you must package your standalone executable with the compiled dynamic library.

#### Vendored Build Prerequisites:
- Visual Studio 2019 w/ `Desktop Development with C++` and `Universal Windows Platform development`
- CMake 3.12+
- Python 3.6+ with Debug Binaries

## License

Licensed under GNU General Public License version 2, ([LICENSE](LICENSE) or [https://opensource.org/licenses/GPL-2.0](https://opensource.org/licenses/GPL-2.0))

The CI/CD setup in `.github/` is based on [rust-github/template](https://github.com/rust-github/template), and therefore licensed under either of

* Apache License, Version 2.0
   ([LICENSE-CI-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license
   ([LICENSE-CI-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Releasing

```cargo release --skip-publish``` and let the github CD pipeline do the rest.