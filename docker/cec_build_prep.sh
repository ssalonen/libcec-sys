#!/bin/bash

apt-get update
apt-get install -yq libssl-dev
# let's remove libudev, it causes only trouble in CI
# when building libcec
apt-get remove --allow-remove-essential -y libudev1 udev libudev-dev || :