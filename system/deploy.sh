#!/bin/sh

MY_PATH=$(realpath $0)
BASE_PATH=$(dirname $MY_PATH)

scp -r "$BASE_PATH/home/root/.profile" "$BASE_PATH/home/root/bin" ampivalence:/home/root/

scp -r "$BASE_PATH/home/alfred/.profile" "$BASE_PATH/home/alfred/.config" ampivalence:/home/alfred/