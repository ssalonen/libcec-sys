# libcec-sys

[![Crates.io](https://img.shields.io/crates/v/libcec-sys.svg)](https://crates.io/crates/libcec-sys)
[![Docs.rs](https://docs.rs/libcec-sys/badge.svg)](https://docs.rs/libcec-sys)
[![CI](https://github.com/ssalonen/libcec-sys/workflows/Continuous%20Integration/badge.svg)](https://github.com/ssalonen/libcec-sys/actions)

FFI bindings for the libcec

## Finding libcec

This crate works with `libcec` v4.x, v5.x, v6.x and v7.x(latest version as time of writing). During the build we try to find `libcec` system library installation using `pkg-config` and compilation using default C compiler (`cc` crate). 

As a fallback, static pre-built `libcec` (v7.1.1) is downloaded from [ssalonen/libcec-static-builds](https://github.com/ssalonen/libcec-static-builds/releases/tag/libcec-v7.1.1-202504-1). Most common targets are supported.

There are `vendored` and `static` feature to allow more explicit control. There are also `LIBCEC_VENDORED` and `LIBCEC_STATIC` environment variables, just set them to value `1`.

The crate is tested mainly with Linux and Windows but could work with other platforms as well. PRs welcome.

### Linux (general)

On Linux, for most convenient build process, it is recommended to install `pkg-config`, `libcec-dev` (headers and pkg-config configuration), `libcec6` or `libcec7` (dynamic library), `libp8-platform-dev` and `libp8-platform2` from your package distribution before installing this crate. Exact package names vary between distributions and package managers.

In addition `libudev-dev` might be needed.

With debian based distributions, you can simply

```
sudo apt-get install libudev-dev libp8-platform2 libp8-platform-dev libcec-dev pkg-config libcec6
```

If your environment lacks the needed dependencies, most easy option might be to fallback to `static` build.

### Raspberry Pi OS

**NOTE:** new versions of Raspberry Pi OS should support standard linux API for CEC control. Static builds used by this library have the linux API enabled.

### Windows

On Windows, probably easiest is to let the `libcec-sys` fallback to statically pre-built library. One can request this explicitly by using `LIBCEC_STATIC=1` environment variable or by using `static` feature.

For dynamic linking: On Windows, it is recommended to install `libcec` via the [installer](https://github.com/Pulse-Eight/libcec/releases/latest) and add `cec.dll` to the `PATH` environment variable.

For a vendored build: `libcec-sys` will dynamically link to the compiled `cec.dll`. This means you must package your standalone executable with the compiled dynamic library.

#### Vendored Build Prerequisites:
- Visual Studio 2022 w/ `Desktop Development with C++` and `Universal Windows Platform development`
- CMake 3.12+
- Python 3.6+ with Debug Binaries

## Static build of libcec

Static build has been adapted from great work from @opeik in https://github.com/ssalonen/cec-rs/issues/52

See https://github.com/ssalonen/libcec-static-builds

## License

This repo contains content distributed under three different licenses.

1. Main package, licensed under GNU General Public License version 2, ([LICENSE](LICENSE) or [https://opensource.org/licenses/GPL-2.0](https://opensource.org/licenses/GPL-2.0))

2. The CI/CD setup in `.github/` is based on [rust-github/template](https://github.com/rust-github/template), and therefore licensed under either of

   * Apache License, Version 2.0
      ([LICENSE-CI-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
   * MIT license
      ([LICENSE-CI-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

   at your option.

3. The CI uses sccache build cache tooling as shared in [Cross repository wiki](https://github.com/cross-rs/cross/wiki/Recipes). The Cross repo itself is licensed under either of

   * Apache License, Version 2.0
      ([LICENSE-CI-docker-sscache-APACHE](LICENSE-CI-docker-sscache-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
   * MIT license
      ([LICENSE-CI-docker-sscache-MIT](LICENSE-CI-docker-sscache-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Releasing

```cargo release --no-publish --dev-version --execute``` and let the github CD pipeline do the rest.
