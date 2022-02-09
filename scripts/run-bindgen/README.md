# Scripts to generate FFI bindings

## Prerequisites

-   following folder structure
-   `<root>/cec-rs`
-   `<root>/libcec-sys`

## How to use

Run bindgen:

```bash
./bindgen.sh
```

The script generates raw FFI bindings in `<root>/libcec-sys/src/lib_abi<ABI>.rs`. 
`lib.rs` imports the relevant definitions based on detected `libcec` version.

In addition, enums are generated for `cec-rs` in `<root>/cec-rs/src/enums<ABI>.rs`.

## Updating libcec version

Check the versions specified in `LIBCEC_VERSIONS` of `bindgen.sh`.

Note that vendored sources in `<root>/libcec-sys/vendor` have no relation how the FFI bindings and enums are generated.