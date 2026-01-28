#!/bin/bash
# Quick benchmark test script

set -e

echo "Building Ruff in release mode..."
cargo build --release 2>&1 | tail -5

echo ""
echo "==== Test 1: Simple Loop (Should JIT well) ===="
cat > /tmp/test_loop.ruff << 'EOF'
sum := 0
i := 0
while i < 1000000 {
    sum := sum + i
    i := i + 1
}
print("Sum: ", sum)
EOF
time ./target/release/ruff run /tmp/test_loop.ruff

echo ""
echo "==== Test 2: Fibonacci Iterative (n=30) ===="
cat > /tmp/test_fib_iter.ruff << 'EOF'
func fib(n) {
    if n <= 1 {
        return n
    }
    a := 0
    b := 1
    i := 2
    while i <= n {
        temp := a + b
        a := b
        b := temp
        i := i + 1
    }
    return b
}
result := fib(30)
print("Fib(30) = ", result)
EOF
time ./target/release/ruff run /tmp/test_fib_iter.ruff

echo ""
echo "==== Test 3: Fibonacci Recursive (n=20) ===="
cat > /tmp/test_fib_rec.ruff << 'EOF'
func fib(n) {
    if n <= 1 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}
result := fib(20)
print("Fib(20) = ", result)
EOF
time ./target/release/ruff run /tmp/test_fib_rec.ruff

echo ""
echo "==== Comparison with Python ===="
echo "Python Fibonacci Recursive (n=20):"
time python3 -c "
def fib(n):
    if n <= 1:
        return n
    return fib(n-1) + fib(n-2)
print('Fib(20) =', fib(20))
"

echo ""
echo "Done!"
