$env:RUSTFLAGS="-Zfmt-debug=none -Zlocation-detail=none"
cargo +nightly build --release #-Z build-std=core,alloc -Z build-std-features="optimize_for_size"
Remove-Item env:RUSTFLAGS
# wasm-snip doesnt know about wasm-eh for now
# wasm-snip target/wasm32-unknown-unknown/release/neoweb.wasm -o pkg/snipped.wasm
wasm-opt --enable-bulk-memory-opt --enable-exception-handling -Oz --strip-dwarf --vacuum --strip-debug -n target/wasm32-unknown-unknown/release/neoweb.wasm -o pkg/neoweb.wasm
gzip pkg/neoweb.wasm -9 -f
