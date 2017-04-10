#!/bin/bash -e
cd $(dirname $0)/sysroot
export SYSROOT=$(pwd)
cd ../sel4-build
./build.sh
cd ../corerust
rustup override add nightly
./build.sh
