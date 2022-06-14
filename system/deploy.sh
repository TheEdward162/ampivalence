#!/bin/sh

scp etc/local.d/fuckup-udev-database.start ampivalence:/etc/local.d/

scp -r home/root/.profile home/root/bin ampivalence:/home/root/

scp -r home/alfred/.profile home/alfred/.config ampivalence:/home/alfred/