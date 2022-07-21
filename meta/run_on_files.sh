#!/usr/bin/env bash

dir=$(realpath $1)

for file in $(find $dir -name "*.php")
do
    cargo run -- $file --lexer

    if [ $? -ne 0 ]
    then
        break
    fi
done
