#!/bin/bash
set -e
cargo +nightly build
cp target/wasm32-unknown-unknown/debug/neoweb.wasm pkg/neoweb.wasm