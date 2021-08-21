#!/bin/bash
cargo build --target wasm32-unknown-unknown --release &&
cp ./target/wasm32-unknown-unknown/release/maze_walk.wasm ./pkg/maze_walk.wasm