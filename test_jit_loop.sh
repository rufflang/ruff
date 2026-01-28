#!/bin/bash
echo "=== Building with JIT loop start fix ==="
cargo build --release 2>&1 > build.log
if [ $? -ne 0 ]; then
    echo "BUILD FAILED!"
    tail -30 build.log
    exit 1
fi

echo "=== Testing with simple array sum ==="
cat > test_array_sum.ruff << 'EOF'
sum := 0
i := 0
while i < 1000000 {
    sum := sum + i
    i := i + 1
}
print("Sum of 0..999999:")
print(sum)
print("Expected: 499999500000")
EOF

DEBUG_JIT=1 ./target/release/ruff run test_array_sum.ruff 2>&1 | head -20

echo ""
echo "=== Running full benchmark suite ==="
cd benchmarks/cross-language
./run_benchmarks.sh
