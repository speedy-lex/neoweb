cargo +nightly build
copy target/wasm32-unknown-unknown/debug/neoweb.wasm pkg/neoweb.wasm
gzip pkg/neoweb.wasm -1 -f
