#!/usr/bin/env bash

set -xe

dir=$(realpath $1)

cargo build --release

for file in $(find $dir -name "*.php")
do
    ./target/release/phpast $file

    if [ $? -ne 0 ]
    then
        break
    fi
done
