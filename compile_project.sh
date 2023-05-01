#!/bin/bash -x

# clean up build directory
if [ -d "build" ] 
then
    rm -rf build/*
else 
    mkdir build
fi

# copy OS files in build dir
#cp nand2tetris/tools/OS/*.vm build
# copy any pre-compiled vm files from target into the build folder
cp $1/* build
target=`basename $1`

echo $target
# compile jack files into vm files in the build directory
cargo run --bin compiler $1 build
# compile vm files in build file into ASM and HACK files
cargo run --bin vmtranslator build $target
