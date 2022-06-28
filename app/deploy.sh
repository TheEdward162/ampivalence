#!/bin/sh

MY_PATH=$(realpath $0)
BASE_PATH=$(dirname $MY_PATH)

cargo build --release --target aarch64-unknown-linux-musl
scp "$BASE_PATH/target/aarch64-unknown-linux-musl/release/ambient_display" ampivalence:/home/alfred
