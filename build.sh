#!/usr/bin/env bash
set -euo pipefail

rm -f module.wasm component.wasm

cargo build -p module --target wasm32-unknown-unknown --release
cp -f target/wasm32-unknown-unknown/release/module.wasm module.wasm

cargo build -p component --target wasm32-wasi --release
wasm-tools component new target/wasm32-wasi/release/component.wasm -o component.wasm
