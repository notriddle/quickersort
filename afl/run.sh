#!/bin/sh
/usr/local/bin/afl-fuzz -i input/ -o output/ -t 500 ./target/release/quickersort-afl

