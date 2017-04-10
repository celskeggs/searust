#!/bin/bash -e
cd $(dirname $0)
mkdir -p sysroot
cd sysroot
export SYSROOT=$(pwd)
cd ../corerust
./build.sh
cd ..
./test.sh
