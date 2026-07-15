#!/bin/bash
set -e
echo "Building HLLSet Redis module from crates/hllset-module/..."
cd "$(dirname "$0")/.."
cargo build -p hllset-module --release
echo "Module built: target/release/libredis_hllset.so"
