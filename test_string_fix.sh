#!/bin/bash
# Test if string constants no longer block JIT compilation

set -e

echo "Building Ruff..."
cd /Users/robertdevore/2026/ruff
cargo build --release 2>&1 | grep -E "(Compiling ruff|Finished|error)" || true

if [ ! -f target/release/ruff ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "Test 1: Simple loop WITH print (should now JIT-compile)"
cat > /tmp/test_with_print.ruff << 'EOF'
sum := 0
i := 0
while i < 100000 {
    sum := sum + i
    i := i + 1
}
print("Sum: ", sum)
EOF

echo "Running with DEBUG_JIT=1 to see JIT activity..."
DEBUG_JIT=1 ./target/release/ruff run /tmp/test_with_print.ruff 2>&1 | head -20

echo ""
echo "Test 2: Loop without print (baseline)"
cat > /tmp/test_no_print.ruff << 'EOF'
sum := 0
i := 0
while i < 100000 {
    sum := sum + i
    i := i + 1
}
# No print
EOF

DEBUG_JIT=1 ./target/release/ruff run /tmp/test_no_print.ruff 2>&1 | head -10

echo ""
echo "Done!"
