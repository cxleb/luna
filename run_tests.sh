#! /bin/sh

cargo test

cargo build

#for filename in tests/fail_cases/*; do echo "$filename"; done
for filename in tests/*; do 
    echo "Running test on file: $filename";
    ./target/debug/luna-rs "$filename";
done