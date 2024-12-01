#!/bin/bash

# let's remove libudev, it causes only trouble in CI
# when building libcec
#apt-get install -y libclang-dev:$CROSS_DEB_ARCH
apt-get update && apt-get install --assume-yes --no-install-recommends libclang-10-dev clang-10
apt-get remove --allow-remove-essential -y libudev1 udev libudev-dev || :