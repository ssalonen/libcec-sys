# Contribution guidelines

> **This repository is deprecated.** New applications should use the official
> [Pulse-Eight `libcec` crate](https://crates.io/crates/libcec), with Rust
> sources in [Pulse-Eight/libcec](https://github.com/Pulse-Eight/libcec/tree/master/src/rust).
> It replaces both the safe wrapper and raw FFI needs for libCEC 8+.
> Please direct Rust binding contributions and issues upstream to
> [Pulse-Eight/libcec](https://github.com/Pulse-Eight/libcec).

This document is retained for historical releases of `libcec-sys`.

## Reporting issues

Please report Rust binding issues upstream at
[Pulse-Eight/libcec issues](https://github.com/Pulse-Eight/libcec/issues).

## Pull requests

Please contribute Rust binding changes upstream to
[Pulse-Eight/libcec](https://github.com/Pulse-Eight/libcec).

### Updating the changelog

Update the changes you have made in
[CHANGELOG](https://github.com/ssalonen/libcec-sys/blob/master/CHANGELOG.md)
file under the **Unreleased** section.

Add the changes of your pull request to one of the following subsections,
depending on the types of changes defined by
[Keep a changelog](https://keepachangelog.com/en/1.0.0/):

- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Removed` for now removed features.
- `Fixed` for any bug fixes.
- `Security` in case of vulnerabilities.

If the required subsection does not exist yet under **Unreleased**, create it!

## Developing

### Set up

This is no different than other Rust projects.

```shell
git clone https://github.com/ssalonen/libcec-sys
cd libcec-sys
cargo build
```

### Useful Commands

- Build and run release version:

  ```shell
  cargo build --release && cargo run --release
  ```

- Run Clippy:

  ```shell
  cargo clippy --all
  ```

- Run all tests:

  ```shell
  cargo test --all
  ```

- Check to see if there are code formatting issues

  ```shell
  cargo fmt --all -- --check
  ```

- Format the code in the project

  ```shell
  cargo fmt --all
  ```
