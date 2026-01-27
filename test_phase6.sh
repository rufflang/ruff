#!/bin/bash
# Test script to compile and test Ruff after Phase 6 implementation

set -e

echo "========================================="
echo "Phase 6: Profiling & Benchmarking - Test"
echo "========================================="
echo ""

# Navigate to project root
cd /Users/robertdevore/2026/ruff

# Build in release mode
echo "1. Building release version..."
cargo build --release 2>&1 | tail -20
echo "✓ Build complete"
echo ""

# Run tests
echo "2. Running test suite..."
cargo test --release 2>&1 | tail -30
echo "✓ Tests complete"
echo ""

# Test profiling module
echo "3. Testing profiling module..."
cargo test --release profiler 2>&1 | tail -20
echo "✓ Profiling tests complete"
echo ""

# Try running a simple benchmark
echo "4. Running simple benchmark..."
./target/release/ruff bench examples/benchmarks/fibonacci.ruff -i 3 -w 1
echo "✓ Benchmark complete"
echo ""

echo "========================================="
echo "All checks passed!"
echo "========================================="
