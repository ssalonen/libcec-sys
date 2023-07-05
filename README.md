# libcec-sys

[![Crates.io](https://img.shields.io/crates/v/libcec-sys.svg)](https://crates.io/crates/libcec-sys)
[![Docs.rs](https://docs.rs/libcec-sys/badge.svg)](https://docs.rs/libcec-sys)
[![CI](https://github.com/ssalonen/libcec-sys/workflows/Continuous%20Integration/badge.svg)](https://github.com/ssalonen/libcec-sys/actions)

FFI bindings for the libcec
## Linking to system libcec

This crate works with `libcec` v4.x, v5.x and v6.x (latest version as time of writing). During the build we try to find `libcec` system library installation using `pkg-config` and compilation using default C compiler (`cc` crate). As a fallback, vendored `libcec` (v6.x) is used during the build.

Alternatively, one can the decide to skip logic above and force the use of vendored sources by enabling `vendored` feature.

The crate is tested mainly with Linux and Windows but could work with other platforms as well. PRs welcome.

### Linux (general)
On Linux, for most convenient build process, it is recommended to install `pkg-config`, `libcec-dev` (headers and pkg-config configuration), `libcec6` (dynamic library), `libp8-platform-dev` and `libp8-platform2` from your package distribution before installing this crate. Exact package names vary between distributions and package managers.

In addition `libudev-dev` might be needed.

With debian based distributions, you can simply

```
sudo apt-get install libudev-dev libp8-platform2 libp8-platform-dev libcec-dev pkg-config libcec6
```

### Raspberry Pi OS

If you are using Raspberry Pi OS and want to use the built-in HDMI port CEC, you might need to build the libcec yourself, since the libcec as packaged by debian is not providing the driver (as of 2022)

Adapted from [libcec documentation](https://github.com/Pulse-Eight/libcec/blob/master/docs/README.raspberrypi.md):

```
# Remove libcec (since we are going to build it ourselves)
apt-get remove libcec6

# Install libcec build dependencies, but not libcec itself
sudo apt-get install libp8 platform-dev libp8-platform cmake libudev-dev libxrandr-dev python3-dev swig git

# Build libcec 6.0.2 with RPI CEC driver enabled
sudo rmdir -rf /tmp/libcec-build-tmp
sudo mkdir /tmp/libcec-build-tmp
sudo cd /tmp/libcec-build-tmp
sudo git clone --recursive https://github.com/Pulse-Eight/libcec.git
sudo cd libcec
sudo git checkout libcec-6.0.2
sudo mkdir build
sudo cd build
sudo cmake -DRPI_INCLUDE_DIR=/opt/vc/include -DRPI_LIB_DIR=/opt/vc/lib ..
sudo make -j4
sudo make install
sudo ldconfig

```

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

```cargo release --no-publish --dev-version --execute``` and let the github CD pipeline do the rest.