#!/usr/bin/env bash

file = $1

cargo run --bin trunk_rs -- $1
rustc ./build/input.rs -O