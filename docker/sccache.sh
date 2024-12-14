#!/bin/bash

# From Cross repository wiki (https://github.com/cross-rs/cross/wiki/Recipes)
# 2023-07-10
#
# wiki Recipes follow cross-rs repository license, see https://github.com/cross-rs/cross/discussions/1297
# 
# Cross repository (https://github.com/cross-rs/) is licensed as follows:
#
# Licensed under either of
#
#     Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
#     MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
#
# at your option.

set -x
set -euo pipefail

# shellcheck disable=SC1091
. lib.sh

main() {
    local triple
    local tag
    local td
    local url="https://github.com/mozilla/sccache"
    triple="${1}"
    
    install_packages unzip tar

    # Download our package, then install our binary.
    td="$(mktemp -d)"
    pushd "${td}"
    tag=$(git ls-remote --tags --refs --exit-code \
        "${url}" \
        | cut -d/ -f3 \
        | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' \
        | sort --version-sort \
        | tail -n1)
    curl -LSfs "${url}/releases/download/${tag}/sccache-${tag}-${triple}.tar.gz" \
        -o sccache.tar.gz
    tar -xvf sccache.tar.gz
    rm sccache.tar.gz
    cp "sccache-${tag}-${triple}/sccache" "/usr/bin/sccache"
    chmod +x "/usr/bin/sccache"

    # clean up our install
    purge_packages
    popd
    rm -rf "${td}"
    rm "${0}"
}

main "${@}"