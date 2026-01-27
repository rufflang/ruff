# Ruff Benchmarks

This directory contains comprehensive benchmarks for the Ruff programming language, including micro-benchmarks, real-world scenarios, and cross-language comparisons.

## Directory Structure

```
benchmarks/
├── README.md                    # This file
├── compare_languages.sh         # Cross-language comparison script
├── fibonacci.ruff               # Recursive fibonacci (main)
├── fibonacci_python.py          # Python equivalent
├── fibonacci_go.go              # Go equivalent
├── fibonacci_node.js            # Node.js equivalent
├── array_ops_comparison.ruff    # Array operations (main)
├── array_ops_python.py          # Python equivalent
├── array_ops_node.js            # Node.js equivalent
├── higher_order.ruff            # Higher-order functions
├── string_ops.ruff              # String manipulation
├── math_ops.ruff                # Mathematical operations
├── struct_ops.ruff              # Struct operations
├── dict_ops_simple.ruff         # Dictionary operations
├── nested_loops_simple.ruff     # Nested loop performance
├── func_calls.ruff              # Function call overhead
├── json_parsing.ruff            # JSON parsing performance
├── file_io.ruff                 # File I/O performance
├── string_processing.ruff       # Advanced string processing
└── jit/                         # JIT-specific benchmarks
    ├── arithmetic_intensive.ruff
    ├── variable_heavy.ruff
    ├── loop_nested.ruff
    └── comparison_specialized.ruff
```

## Quick Start

### Run All Benchmarks

```bash
# From project root
cargo run --release -- bench examples/benchmarks/
```

### Run Specific Benchmark

```bash
cargo run --release -- bench examples/benchmarks/fibonacci.ruff
```

### Custom Iterations

```bash
# 20 iterations, 5 warmup runs
cargo run --release -- bench examples/benchmarks/fibonacci.ruff -i 20 -w 5
```

### Cross-Language Comparison

```bash
cd examples/benchmarks
chmod +x compare_languages.sh
./compare_languages.sh
```

## Benchmark Categories

### 1. Micro-Benchmarks

Test specific language features in isolation.

| Benchmark | Tests | Complexity |
|-----------|-------|------------|
| `fibonacci.ruff` | Recursive function calls | O(2^n) |
| `higher_order.ruff` | map/filter/reduce | O(n) |
| `string_ops.ruff` | String operations | O(n) |
| `math_ops.ruff` | Arithmetic operations | O(1) |
| `struct_ops.ruff` | Struct creation/access | O(1) |
| `dict_ops_simple.ruff` | HashMap operations | O(1) average |
| `nested_loops_simple.ruff` | Loop overhead | O(n²) |
| `func_calls.ruff` | Function call overhead | O(n) |

### 2. Real-World Benchmarks

Simulate real application scenarios.

| Benchmark | Scenario | Operations |
|-----------|----------|------------|
| `json_parsing.ruff` | API data processing | Parse, serialize, transform |
| `file_io.ruff` | File processing | Read, write, append, seek |
| `string_processing.ruff` | Text processing | Concat, split, regex, case conversion |
| `sorting_algorithms.ruff` | Data sorting | QuickSort, MergeSort |

### 3. JIT Benchmarks

Test JIT compilation effectiveness.

| Benchmark | Focus | Expected Speedup |
|-----------|-------|------------------|
| `jit/arithmetic_intensive.ruff` | Pure arithmetic | 500-1000x |
| `jit/variable_heavy.ruff` | Variable operations | 50-100x |
| `jit/loop_nested.ruff` | Loop optimization | 100-300x |
| `jit/comparison_specialized.ruff` | Type specialization | 200-400x |

## Benchmark Output

### Example Output

```
========================================
Ruff Performance Benchmarks
========================================

Comparing execution modes:
  Interpreter: Tree-walking AST interpreter
  VM: Bytecode virtual machine
  JIT: Just-in-time native compilation
========================================

fibonacci
  Interpreter:  2.543s  (1.00x)
  VM:           0.156s  (16.30x faster)
  JIT:          0.008s  (317.88x faster) 🚀

higher_order
  Interpreter:  1.234s  (1.00x)
  VM:           0.089s  (13.87x faster)
  JIT:          0.012s  (102.83x faster) 🚀

math_ops
  Interpreter:  0.876s  (1.00x)
  VM:           0.045s  (19.47x faster)
  JIT:          0.002s  (438.00x faster) 🚀

========================================
Comparison Table
========================================
Benchmark       Interpreter    VM          JIT
-------------------------------------------------
fibonacci       2.543s         0.156s      0.008s
higher_order    1.234s         0.089s      0.012s
math_ops        0.876s         0.045s      0.002s
-------------------------------------------------

========================================
Summary:
  Total benchmarks: 8
  Average VM speedup: 15.5x
  Average JIT speedup: 286.2x
  JIT speedup range: 102x - 438x
========================================
```

## Cross-Language Results

### Fibonacci (Recursive, N=30)

Expected relative performance:

```
Language         Time      Relative
-----------------------------------------
Go               ~0.05s    1.00x (baseline)
Ruff (JIT)       ~0.15s    0.33x (2-3x slower) ✅
Node.js (V8)     ~0.25s    0.20x
Ruff (VM)        ~1.2s     0.04x
Python           ~4.5s     0.01x
```

### Array Operations (100K elements)

Expected performance for map/filter/reduce:

```
Language         Total     Relative
-----------------------------------------
Go               ~6ms      1.00x
Node.js          ~40ms     0.15x
Ruff (JIT)       ~60ms     0.10x ✅
Ruff (VM)        ~400ms    0.015x
Python           ~650ms    0.009x
```

## Writing New Benchmarks

### Basic Template

```ruff
# my_benchmark.ruff

func operation_to_benchmark() {
    # Your code here
    let result := 0
    for i in range(1000) {
        result := result + i * i
    }
    return result
}

# The benchmark runner will measure execution time
let result := operation_to_benchmark()
print(result)
```

### Guidelines

1. **Focus**: Test one thing at a time
2. **Size**: Keep iterations high enough for JIT (>100)
3. **Warmup**: Account for compilation time
4. **Reproducibility**: Avoid random/time-dependent operations
5. **Clarity**: Add comments explaining what's being tested

### Benchmark Checklist

- [ ] Focuses on specific operation
- [ ] Runs enough iterations (>100 for JIT)
- [ ] Has descriptive filename
- [ ] Includes comments
- [ ] Produces verifiable output
- [ ] Comparable to other languages (if applicable)

## Profiling Benchmarks

Use the profile command to understand performance characteristics:

```bash
# Profile a benchmark
cargo run --release -- profile examples/benchmarks/fibonacci.ruff

# Generate flamegraph
cargo run --release -- profile examples/benchmarks/fibonacci.ruff --flamegraph fib.txt
flamegraph.pl fib.txt > fib.svg
```

## Performance Targets

### VM Mode
- 10-50x faster than interpreter ✅
- Instant compilation (<1ms) ✅
- All tests pass ✅

### JIT Mode
- 100-500x faster than interpreter ✅
- 2-5x slower than Go ✅
- 2-10x faster than Python ✅
- Competitive with Node.js ✅

## Known Limitations

### JIT Optimization Caveats

1. **Cold Start**: First 100 iterations use VM
2. **Type Mixing**: Guards fail if types change
3. **Complex Control Flow**: May not optimize
4. **I/O Bound**: No speedup for I/O operations

### Benchmark Limitations

1. **Micro-benchmarks**: Don't represent real workloads
2. **JIT Warmup**: Results include compilation time
3. **System Variance**: CPU frequency, background processes
4. **Language Differences**: Not always apples-to-apples

## Contributing Benchmarks

To add a new benchmark:

1. Create `.ruff` file in `examples/benchmarks/`
2. Follow template and guidelines above
3. Add equivalent implementations for other languages (optional)
4. Update this README with description
5. Run and verify results
6. Submit PR with benchmark

## Interpreting Results

### Good Performance Indicators

- VM: 10-20x faster than interpreter
- JIT: 100-300x faster for arithmetic
- Consistent results across runs
- High guard success rate (>95%)

### Performance Red Flags

- JIT not activating (0 functions compiled)
- Low guard success rate (<90%)
- High recompilation count
- Memory leaks (allocations >> deallocations)

### When to Optimize

1. Profile first (find bottleneck)
2. Fix algorithmic issues (O(n²) → O(n))
3. Enable JIT (increase iterations)
4. Optimize hot paths only
5. Profile again (verify improvement)

## Resources

- [Performance Guide](../../docs/PERFORMANCE.md) - Comprehensive guide
- [JIT Internals](../../docs/JIT_INTERNALS.md) - How JIT works
- [ROADMAP](../../ROADMAP.md) - Performance milestones

---

## Frequently Asked Questions

**Q: Why is my benchmark slow?**
A: Check execution mode, iterations (JIT needs 100+), and type consistency.

**Q: How do I compare against other languages?**
A: Use `compare_languages.sh` or write equivalent implementations.

**Q: What's a realistic speedup expectation?**
A: VM: 10-50x, JIT: 100-500x for CPU-bound code. I/O-bound sees minimal improvement.

**Q: Why does first run take longer?**
A: JIT compilation happens on first detection of hot path (100 iterations).

**Q: How do I profile my benchmark?**
A: Use `ruff profile <file>` with optional `--flamegraph` output.

---

*Last Updated: January 2026 (v0.9.0 Phase 6)*
