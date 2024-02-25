alias b := build
alias r := run

# build and optimize wasm blob
build: build-without-opt
    wasm-opt -all -Oz target/wasm32-unknown-unknown/release/raycaster.wasm \
        -o target/wasm32-unknown-unknown/release/raycaster.wasm

# build wasm blob without optimizing through wasm-opt
build-without-opt:
    cargo build --release

# run the wasm block in wasm-4's native window
run: build
    w4 run-native target/wasm32-unknown-unknown/release/raycaster.wasm

# clean the build directory
clean:
    cargo clean
