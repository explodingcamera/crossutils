#!/bin/sh
cargo build -Zbuild-std=libc,panic_abort,std -Zbuild-std-features="" --target=./x86_64-unknown-linux-cosmo.json
