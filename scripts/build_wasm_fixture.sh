#!/usr/bin/env bash
set -euo pipefail

# Run from the repository root. Binaryen lowers LLVM's memory.copy/fill to
# operations supported by the pinned CosmWasm execution profile.
cargo build --manifest-path contracts/examples/counter/Cargo.toml \
  --target wasm32-unknown-unknown --release --locked
fixture="contracts/examples/counter/target/wasm32-unknown-unknown/release/sprax_counter.wasm"
wasm-opt "$fixture" -Oz --llvm-memory-copy-fill-lowering \
  --strip-debug --strip-producers -o "${fixture}.optimized"
mv "${fixture}.optimized" "$fixture"
