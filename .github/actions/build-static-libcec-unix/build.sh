#! /usr/bin/env bash

set -euxo pipefail

path="$1"
build_type="$2"

case "$build_type" in
  'debug')
    cmake_build_type='Debug'
    ;;

  'release')
    cmake_build_type='RelWithDebInfo'
    ;;

  *)
    echo "invalid build type"
    exit 1
    ;;
esac

cd "$path"
cmake -S . -B build -G Ninja -D "CMAKE_BUILD_TYPE=$cmake_build_type" -D BUILD_STATIC_LIB=True -D CMAKE_CXX_STANDARD=11 -Wno-dev
cmake --build build

mkdir -p dist/include
ls -R build
find build \( -name '*.a' -o -name '*.so' -o -name '*.dylib' \) -print -exec cp {} dist \;
find include -name '*.h' -print -exec cp --parents {} dist \;