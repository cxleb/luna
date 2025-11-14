#! /bin/sh

cargo test

cargo build

for filename in tests/*; do ./target/debug/luna-rs "$filename"; done