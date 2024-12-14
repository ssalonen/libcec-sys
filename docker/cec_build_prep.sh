#!/bin/bash
apt-get update \
    && apt-get install --assume-yes --no-install-recommends \
       libclang-10-dev clang-10
#apt-cache search clang 1>&2

#dpkg -L clang-10 1>&2
dpkg -L libclang-10-dev |grep -F .so 1>&2
echo finding libclang START 1>&2
find /usr/lib -type f -name 'libclang-*.so' 1>&2
echo finding libclang END 1>&2

# let's remove libudev, it causes only trouble in CI
# when building libcec
apt-get remove --allow-remove-essential -y libudev1 udev libudev-dev || :