#!/bin/bash
set -e

echo "Building Ruff with JIT execution fix..."
cargo build --release 2>&1 | tee build.log

if [ $? -eq 0 ]; then
    echo ""
    echo "=== BUILD SUCCESSFUL ==="
    echo ""
    echo "Testing with simple loop..."
    cat > test_jit_simple.ruff << 'EOF'
// Simple loop test for JIT
sum := 0
i := 0
while i < 1000000 {
    sum := sum + i
    i := i + 1
}
print("Sum:")
print(sum)
EOF
    
    echo "Running with DEBUG_JIT enabled..."
    DEBUG_JIT=1 ./target/release/ruff run test_jit_simple.ruff 2>&1 | head -30
    
    echo ""
    echo "=== Running Full Benchmark Suite ==="
    cd benchmarks/cross-language
    ./run_benchmarks.sh
else
    echo ""
    echo "=== BUILD FAILED ==="
    tail -50 build.log
    exit 1
fi
