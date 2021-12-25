# libcec-sys

[![Crates.io](https://img.shields.io/crates/v/libcec-sys.svg)](https://crates.io/crates/libcec-sys)
[![Docs.rs](https://docs.rs/libcec-sys/badge.svg)](https://docs.rs/libcec-sys)
[![CI](https://github.com/ssalonen/libcec-sys/workflows/Continuous%20Integration/badge.svg)](https://github.com/ssalonen/libcec-sys/actions)

FFI bindings for the libcec

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install libcec-sys`

### Linking of libcec

By default, this crate compiles `libcec` from vendored sources. The vendored sources also facilitate re-generating FFI bindings.

Downstream crates can opt to link to system `libcec` by overriding build script as discussed in [cargo documentation](https://doc.rust-lang.org/cargo/reference/build-scripts.html). Minimally, placing the following in workspace's `.cargo/config` will make the resulting crate link to system `cec` library

```toml
# Replace target-triplet accordingly
[target.x86_64-unknown-linux-gnu.cec]
rustc-link-lib = ["cec"]
```

This will mean that compilation of vendored `libcec` sources in `libcec-sys` is skipped, and build will be generally faster.

## License

Licensed under GNU General Public License version 2, ([LICENSE](LICENSE) or [https://opensource.org/licenses/GPL-2.0](https://opensource.org/licenses/GPL-2.0))

The CI/CD setup in `.github/` is based on [rust-github/template](https://github.com/rust-github/template), and therefore licensed under  either of

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