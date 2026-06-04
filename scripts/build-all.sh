#!/bin/bash
set -euo pipefail
echo "=== Static checks ==="
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
echo "=== Build ==="
cargo build --workspace
echo "=== FFI cdylib ==="
cargo build -p warehouse_ffi
echo "=== Tests ==="
cargo test --workspace
echo "=== All OK ==="
