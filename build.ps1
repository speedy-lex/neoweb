$env:RUSTFLAGS="-Zfmt-debug=none -Zlocation-detail=none"
cargo +nightly build --release #-Z build-std=core,alloc -Z build-std-features="optimize_for_size"
Remove-Item env:RUSTFLAGS
wasm-snip target/wasm32-unknown-unknown/release/neoweb.wasm -o pkg/snipped.wasm
wasm-opt --enable-bulk-memory-opt -Oz --strip-dwarf --vacuum --strip-debug -n pkg/snipped.wasm -o pkg/neoweb.wasm
