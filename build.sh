#!/usr/bin/env bash
# Build bili-core (Rust) and bili-mini (Qt6/QML).
set -euo pipefail
cd "$(dirname "$0")"

echo "==> cargo build (bili-core)"
cargo build --manifest-path core/Cargo.toml "$@"

echo "==> cmake build (bili-mini)"
cmake -S app -B build -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo
cmake --build build

# The app looks for bili-core next to its own binary.
cp -f core/target/debug/bili-core build/bili-core
echo "==> done: $(pwd)/build/bili-mini"
