#!/bin/bash
set -e
RUSTFLAGS="-Zfmt-debug=none -Zlocation-detail=none" cargo +nightly build --release
wasm-opt --enable-bulk-memory-opt --enable-exception-handling -Oz --strip-dwarf --vacuum --strip-debug -n target/wasm32-unknown-unknown/release/neoweb.wasm -o pkg/neoweb.wasm
gzip pkg/neoweb.wasm -9 -f
