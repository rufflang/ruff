#!/bin/bash

echo "=== Building Ruff with Hash Map Fix ==="
cargo build --release 2>&1 | tee build.log | tail -30

if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo ""
    echo "BUILD FAILED!"
    exit 1
fi

echo ""
echo "=== Testing Hash Map Integer Keys ==="
./target/release/ruff run test_hashmap.ruff

echo ""
echo "=== Running Full Benchmark Suite ==="
cd benchmarks/cross-language
./run_benchmarks.sh
