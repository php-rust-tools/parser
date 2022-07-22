#!/usr/bin/env bash

set -xe

dir=$(realpath $1)

for file in $(find $dir -name "*.php")
do
    cargo run -q -- $file

    if [ $? -ne 0 ]
    then
        break
    fi
done
