#!/bin/bash -e
cd $(dirname $0)/sysroot
export SYSROOT=$(pwd)
cd ../corerust
./build.sh
cd ..
./test.sh
