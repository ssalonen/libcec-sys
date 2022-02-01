#!/usr/bin/env bash
set -xe

# libcec git tags to use to generate bindings
LIBCEC_VERSIONS="4.0.5 5.0.0 6.0.2"


OUT=lib.rs
DEST_DIR=../../src/
CEC_REGEX='(libcec|cec|CEC|LIBCEC)_.*'
VENDOR_TMP=../../vendor_tmp

function generate() {
    # bindgen layout tests are disabled for cross-arch compatibility
    # See https://kornel.ski/rust-sys-crate Stable API guidance
    # and https://users.rust-lang.org/t/what-to-do-when-bindgen-tests-fail/23822
    bindgen wrapper.h -o ${OUT}.tmp \
    --whitelist-type "$CEC_REGEX" \
    --whitelist-function "$CEC_REGEX" \
    --whitelist-var "$CEC_REGEX" \
    --blacklist-type cec_boolean \
    --no-layout-tests \
    --no-prepend-enum-name \
    --rustfmt-bindings \
    --raw-line='#![allow(non_upper_case_globals)]' \
    --raw-line='#![allow(non_camel_case_types)]' \
    --raw-line='#![allow(non_snake_case)]' \
    --raw-line='#![allow(dead_code)]' \
    --raw-line='#![allow(' \
    --raw-line='    clippy::redundant_static_lifetimes,' \
    --raw-line='    clippy::unreadable_literal,' \
    --raw-line='    clippy::cognitive_complexity' \
    --raw-line=')]' \
    "$@" \
    -- \
    -I vendor_tmp/include
}


source ../../abis.env
git clone --recursive git@github.com:Pulse-Eight/libcec.git $VENDOR_TMP

cp -a ../../vendor $VENDOR_TMP
for LIBCEC_VERSION in $LIBCEC_VERSIONS; do
    echo "Generating bindings for ABI=$ABI"
    ABI_MAJOR=$(echo "$LIBCEC_VERSION"|cut -d '.' -f1)

    git -C $VENDOR_TMP checkout "libcec-$LIBCEC_VERSION"
    cp $VENDOR_TMP/include/version.h.in $VENDOR_TMP/include/version.h

    LIBCEC_VERSION_MAJOR=$(grep -E -o 'set\(LIBCEC_VERSION_MAJOR [^)]' $VENDOR_TMP/CMakeLists.txt|cut -d ' ' -f2)
    LIBCEC_VERSION_MINOR=$(grep -E -o 'set\(LIBCEC_VERSION_MINOR [^)]' $VENDOR_TMP/CMakeLists.txt|cut -d ' ' -f2)
    LIBCEC_VERSION_PATCH=$(grep -E -o 'set\(LIBCEC_VERSION_PATCH [^)]' $VENDOR_TMP/CMakeLists.txt|cut -d ' ' -f2)

    if [[ "$LIBCEC_VERSION_MAJOR" != "$ABI_MAJOR" ]]; then
        echo "LIBCEC_VERSION_MAJOR ($LIBCEC_VERSION_MAJOR) did not match expected ABI_MAJOR ($ABI_MAJOR)"
        exit 1
    fi

    sed -i s/@LIBCEC_VERSION_MAJOR@/"$LIBCEC_VERSION_MAJOR"/ $VENDOR_TMP/include/version.h
    sed -i s/@LIBCEC_VERSION_MINOR@/"$LIBCEC_VERSION_MINOR"/ $VENDOR_TMP/include/version.h
    sed -i s/@LIBCEC_VERSION_PATCH@/"$LIBCEC_VERSION_PATCH"/ $VENDOR_TMP/include/version.h

    # Generate version with enums, and capture the enum definitions
    generate --rustified-enum "$CEC_REGEX"
    ./sed_bindings.py ${OUT}.tmp ${OUT} --outfile_enum ${OUT}.enum
    # Generate (safer) version without enums
    generate --constified-enum "$CEC_REGEX"
    ./sed_bindings.py ${OUT}.tmp ${OUT}

    # Copy enums to cec-rs/src/ crate
    cp ${OUT}.enum "../../../cec-rs/src/enums$ABI_MAJOR.rs"
    # libcec ABI
    mv ${OUT} "${DEST_DIR}/lib_abi$ABI_MAJOR.rs"


    # Cleanup        
    rm ${OUT}.tmp ${OUT}.enum

done

rm -rf $VENDOR_TMP
