#!/bin/bash
echo "Building..."
cargo build --release 2>&1 > build.log
echo "Build done. Running benchmark..."
cd benchmarks/cross-language && ./run_benchmarks.sh
echo "Check latest results file!"
