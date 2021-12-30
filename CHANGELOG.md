# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [UNRELEASED]

## [2.0.1]

- Fix missing link statement for libcec when vendored libcec sources were used.
## [2.0.0]

- Add missing `links` declaration in crate manifest.
- By default, we try to link locally installed `libcec`. See README for details.


## 1.1.1

- CI improvements: updated cross docker images used in build from version 0.1.16 to 0.2.1. This updates the cmake version used to build libcec as well.

## 1.1.0

- CI improvements
- Generated `enums.rs` pass new clippy rules
- Generated `enums.rs` rely on `enum-repr-derive` 0.2.0 or higher
