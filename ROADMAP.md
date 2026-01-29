# Ruff Language - Development Roadmap

This roadmap outlines **upcoming** planned features and improvements. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.8.0 (Released January 2026)  
> **Next Planned Release**: v0.9.0 (JIT Performance - Beat Python)  
> **Status**: Phase 7 - 98% Complete! All JIT infrastructure working. Loops in functions now JIT-compile!

---

## 🎯 What's Next (Priority Order)

**IMMEDIATE (v0.9.0)**:
1. **✅ Recursive JIT Performance** - COMPLETE! Ruff is **31-48x FASTER** than Python!
   - fib(25): 0.5ms vs Python's 24ms (48x faster)
   - fib(30): 5-8ms vs Python's 253ms (31-50x faster)

2. **✅ Loop JIT (Step 11)** - COMPLETE! Loops in JIT-compiled functions now work!
   - Added late sealing for loop header blocks
   - JumpBack correctly passes stack values to loop headers
   - All 31 JIT tests passing

3. **🔥 Parallel Processing / Concurrency (P0)** - CRITICAL for Real-World Performance
   - Current Bottleneck: SSG benchmark shows 49s loop overhead for 10K iterations (~5ms/iteration)
   - Python SSG does 10K builds in 0.05s using `ProcessPoolExecutor` (multicore parallelism)
   - **Goal**: Add goroutine-style concurrency or async/await for I/O-bound workloads
   - **Impact**: Would enable 8-10x speedup on multicore systems for file processing
   - **Priority**: P0 - Without this, Ruff appears slow on real-world workloads despite fast JIT

4. **v1.0 Release Preparation** - Finalize APIs, comprehensive documentation

**AFTER v0.9.0**:
5. **Developer Experience** - LSP, Formatter, Linter, Package Manager (v0.11.0)
6. **Optional Static Typing** - Gradual typing for additional performance (v0.10.0, exploratory)

---

## Priority Levels

- **P0 (Critical)**: Blocking v0.9.0 release
- **P1 (High)**: Core features needed for v1.0 production readiness
- **P2 (Medium)**: Quality-of-life improvements and developer experience
- **P3 (Low)**: Nice-to-have features for advanced use cases

---

## v0.9.0 - JIT Performance (IN PROGRESS)

**Focus**: Achieve 5-10x faster than Python performance  
**Timeline**: Q1 2026 (2-4 weeks remaining)  
**Priority**: P0 - CRITICAL - Blocking Release

---

### Phase 7: JIT Performance Critical Path - Beat Python!

**Status**: ✅ SUCCESS - Recursive JIT is 30-50x FASTER than Python!  
**Current State**: JIT for recursive functions COMPLETE. Loop JIT optimization remaining.

#### Current Benchmark Results (2026-01-28 UPDATED):

| Benchmark | Ruff JIT | Python | Speedup | Status |
|-----------|----------|--------|---------|--------|
| fib(25) | 0.5ms | 24ms | **48x faster** | ✅ EXCEEDS TARGET |
| fib(30) | 5-8ms | 253ms | **31-50x faster** | ✅ EXCEEDS TARGET |
| fib(35) | ~60ms | ~2.8s | **~47x faster** | ✅ EXCEEDS TARGET |
| Array Sum (1M) | ~4s | 52ms | ❌ Slower | ⚠️ Loop JIT needed |
| Hash Map (100k) | TBD | 34ms | TBD | ⚠️ Loop JIT needed |

**Note**: Recursive function JIT is complete and exceeds all targets. Loop operations still use interpreter, which is why array/hash benchmarks are slow. Step 11 (Loop Back-Edge Fix) will address this.

#### Root Cause Analysis (Resolved for Recursive Functions):

Previous issues that have been **FIXED**:

1. ~~**HashMap Lookup per Variable**~~ ✅ FIXED (Step 7) - Stack slots now used
2. ~~**HashMap Clone per Call**~~ ✅ FIXED (Step 9) - Inline caching eliminates clones
3. ~~**Value Boxing/Unboxing**~~ ✅ FIXED (Step 8) - Direct i64 returns
4. ~~**Function Call Overhead**~~ ✅ FIXED (Step 10/12) - Direct JIT recursion works

**Remaining Issues (Loop Performance)**:
- Loop bytecode fails JIT compilation due to SSA block parameter issues
- Loops fall back to bytecode interpreter (still correct, but slower)
- Step 11 (Loop Back-Edge Fix) will address this

#### Implementation Plan:

**Step 7: Register-Based Locals (P0) - ✅ COMPLETE**
- [x] Pre-allocate local variable slots at compile time
- [x] Map variable names to Cranelift stack slots (no HashMap)
- [x] Use direct memory access instead of function calls
- [x] Parameters initialized from HashMap at function entry
- [x] Fall back to runtime for globals and function references
- [x] Comprehensive test suite added
- Status: Local variable access now uses fast stack slots

**Step 8: Return Value Optimization (P0) - ✅ COMPLETE**
- [x] Added `return_value` and `has_return_value` fields to VMContext
- [x] Implemented `jit_set_return_int()` fast return helper
- [x] Return opcode uses optimized path (avoids stack push)
- [x] VM reads return value from VMContext when available
- [x] Comprehensive test for return value optimization
- Status: Integer returns now bypass VM stack operations

**Step 9: Inline Caching (P0) - ✅ COMPLETE**
- [x] Cache resolved function pointers after first call
- [x] Avoid function lookup on subsequent calls
- [x] Added `CallSiteId` and `InlineCacheEntry` structures
- [x] Eliminated var_names HashMap clone in inline cache fast path
- [x] Added hit/miss counters for profiling
- [x] Comprehensive test suite added (`tests/jit_inline_cache.ruff`)
- Status: Inline cache reduces per-call overhead for JIT functions
- Note: Discovered JIT limitation with higher-order functions (functions as args)

**Step 10: Fast Argument Passing (P1) - PARTIAL**
- [x] Added VMContext.argN fields (arg0-arg3, arg_count)
- [x] Added `jit_get_arg()` runtime helper
- [x] JIT reads parameters from VMContext.argN for ≤4 int args
- [x] Eliminated var_names HashMap clone on recursive calls
- [x] Skip HashMap population for simple integer functions
- [x] Direct JIT recursion fully working (see Step 12)
- Status: ✅ COMPLETE - 48x faster than Python for fib(25)!

**Step 11: Loop Back-Edge Fix (P1) - ✅ COMPLETE**
- [x] Added `analyze_loop_headers()` for pre-detecting backward jump targets
- [x] Added `stack_effect()` to calculate stack depth changes per opcode
- [x] Added `loop_header_pcs` HashSet to track loop header blocks
- [x] Loop headers created with proper Cranelift block parameters
- [x] Implemented late sealing - loop headers sealed after back-edges processed
- [x] JumpBack now passes stack values as block arguments
- [x] Test `test_compile_simple_loop` passing
- [x] All 31 JIT tests passing
- Status: ✅ COMPLETE - Loops in JIT-compiled functions now work!
- Note: Loops in top-level scripts still run interpreted (functions-only JIT design)

**Step 12: Direct JIT Recursion (P0) - ✅ COMPLETE**
- [x] Added `CompiledFnWithArg` type for direct-arg functions
- [x] Added `CompiledFnInfo` struct to track both variants
- [x] Implemented `function_has_self_recursion()` detection
- [x] Implemented `compile_function_with_direct_arg()` method
- [x] Implemented `translate_direct_arg_instruction()` for direct-arg mode
- [x] Updated interpreter to prefer direct-arg variant for 1-int-arg calls
- [x] Self-recursion detection correctly identifies recursive calls
- [x] **VERIFIED**: fib(25)=0.5ms, fib(30)=5-8ms (31-48x faster than Python!)
- Status: ✅ COMPLETE - Recursive JIT performance exceeds all targets!

#### Performance Targets (v0.9.0):

```
TARGET:                          ACTUAL:                     STATUS:
- Fib Recursive (n=25): <40ms    0.5ms (with warmup)         ✅ 80x EXCEEDED
- Fib Recursive (n=30): <300ms   5-8ms                       ✅ 37-60x EXCEEDED
- Array Sum (1M): <10ms          ~4s (interpreter)           ⚠️ Needs Loop JIT
- Hash Map (100k): <10ms         TBD                         ⚠️ Needs Loop JIT
- Hash Map (100k):       <10ms   (3x faster than Python)

GOAL: Ruff >= 5x faster than Python on compute-heavy benchmarks ✅ ACHIEVED FOR RECURSIVE FUNCTIONS!
```

#### Success Criteria (v0.9.0):

- [x] Fibonacci faster than Python (minimum match, target 5x) ✅ **31-48x FASTER!**
- [ ] All compute benchmarks show Ruff >= Python performance (Loop JIT needed for arrays/hashmaps)
- [x] No regressions in correctness (198 tests passing) ✅

**v0.9.0 Release Blocker Status**: Fibonacci benchmark targets EXCEEDED. Loop-based benchmarks (array sum, hash map) blocked on Step 11 (Loop Back-Edge Fix). Once loops compile to JIT, all targets expected to be met.

---

## v0.9.0 - Architecture Cleanup Tasks (P2)

These run in parallel with JIT work and don't block v0.9.0:

### Fix LeakyFunctionBody (P2)

**Status**: Planned  
**Estimated Effort**: Medium (1-2 weeks)

**Problem**: Memory leak from recursive drop on deeply nested function bodies.

**Solution**: Implement iterative drop traversal or arena allocation.

---

### Separate AST from Runtime Values (P2)

**Status**: Planned  
**Estimated Effort**: Large (3-4 weeks)

**Problem**: Runtime `Value::Function` contains raw AST (`Vec<Stmt>`).

**Solution**: Compile functions to IR/bytecode, don't store AST in runtime values.

---

## v0.9.0 - Parallel Processing & Concurrency (P0)

**Focus**: Enable multicore parallelism for real-world I/O-bound workloads  
**Timeline**: Q1 2026 (Before v1.0)  
**Priority**: P0 - CRITICAL for production performance perception

### Problem Statement

**Current Situation**: 
- JIT makes compute-heavy code 30-50x faster than Python ✅
- Ruff has `async`/`await` **syntax** but it's not truly concurrent (functions execute synchronously)
- Real-world file processing workloads show poor performance:
  - SSG benchmark: 10,000 files processed in ~55 seconds
  - Python equivalent: 0.05 seconds (1000x faster!)
  - Profiling shows 49s "loop overhead" (~5ms per iteration)

**Root Cause**:
- Single-threaded execution on I/O-bound workloads (async functions run synchronously!)
- Python's `ProcessPoolExecutor` uses all CPU cores (8-10x parallel speedup)
- Ruff processes files sequentially, despite having async/await syntax

**Impact**: Without real concurrency, Ruff appears "slow" on practical workloads even though JIT is fast.

### Proposed Solutions (Choose One)

#### Option 1: Goroutine-Style Concurrency (Recommended)

**Inspired by**: Go, Erlang  
**API Design**:

```ruff
# Spawn lightweight concurrent tasks
spawn {
    process_file(file1)
}

spawn {
    process_file(file2)
}

# Wait for all spawned tasks
await_all()

# Or with channels for communication
let ch := channel()

spawn {
    ch.send(process_file(file1))
}

let result := ch.receive()
```

**Advantages**:
- Lightweight green threads (thousands possible)
- Familiar to developers from Go/Erlang
- Great for I/O-bound workloads
- No data race concerns with proper channel semantics

**Implementation**:
- Use tokio or smol runtime for async execution
- M:N threading model (many green threads on few OS threads)
- Work-stealing scheduler

#### Option 2: True Async/Await (Make it Real)

**Current State**: Ruff has `async func` and `await` **syntax** but functions execute synchronously  
**Inspired by**: JavaScript, Python, Rust  
**What's Needed**: Make async actually asynchronous!

**API Design** (syntax already exists, just need to make it work):

```ruff
async func process_files(files) {
    let tasks := []
    for file in files {
        push(tasks, process_file(file))  # Start async task
    }
    return await_all(tasks)  # Wait for all to complete
}
```

**Advantages**:
- **Syntax already exists** - just need to implement real concurrency
- Clear async boundaries
- Explicit await points
- Good ecosystem support (tokio)
- Familiar to JavaScript/Python/Rust developers

**Disadvantages**:
- Function coloring problem (async functions can't call sync)
- More verbose than goroutines
- Requires refactoring existing async code

**Implementation Changes Needed**:
- Replace synchronous Promise execution with tokio runtime
- Make `await` actually poll futures instead of just unwrapping
- Add task spawning mechanism
- Make I/O operations truly async (non-blocking)

#### Option 3: Parallel Map/Reduce

**Inspired by**: Rayon (Rust)  
**API Design**:

```ruff
# Parallel map over collection
let results := files.par_map(func(f) {
    return process_file(f)
})

# Process collection in parallel
files.par_each(func(f) {
    process_file(f)
})
```

**Advantages**:
- Simple API for common patterns
- Automatic work distribution
- No explicit threading

**Disadvantages**:
- Less flexible than spawn/goroutines
- Limited to data-parallel workloads

### Implementation Plan

**Phase 1: Core Concurrency Primitives (2-3 weeks)**
- [ ] Choose concurrency model (Recommendation: Option 1 - Goroutines)
- [ ] Implement `spawn` keyword and scheduler
- [ ] Add `channel()` for message passing
- [ ] Implement `await_all()` for synchronization
- [ ] Thread-safe Value type operations

**Phase 2: Runtime Integration (1-2 weeks)**
- [ ] Integrate async runtime (tokio/smol)
- [ ] Make file I/O operations async-aware
- [ ] Add thread pool for blocking operations
- [ ] Ensure JIT-compiled code can run on any thread

**Phase 3: Testing & Benchmarks (1 week)**
- [ ] Add concurrency test suite
- [ ] Re-run SSG benchmark with parallelism
- [ ] Target: 10K file SSG build in <1 second (using all cores)
- [ ] Ensure no race conditions or deadlocks

### Success Criteria

- [ ] SSG benchmark: 10,000 files in <1 second (vs current 55s)
- [ ] Parallel overhead: <100ms for spawning 10,000 tasks
- [ ] No data races or deadlocks in test suite
- [ ] Clean API that's easy to understand and use

### Performance Targets

**SSG Benchmark**:
- Current: 55 seconds (single-threaded)
- With 8 cores: Target <7 seconds (8x speedup)
- With I/O overlap: Target <1 second (optimal)

**Microbenchmarks**:
- Spawn 10,000 goroutines: <100ms
- Channel send/receive: <1μs per message
- Context switching: <10μs

---

## v0.10.0 - Optional Static Typing (Exploratory)

**Status**: Research & Design Phase  
**Timeline**: TBD (After v0.9.0)  
**Priority**: Exploratory

**Key Question**: Should Ruff adopt optional static typing?

### Stage 1: Type Annotations (Documentation Only)

```ruff
func calculate(x: int, y: float) -> float {
    return x * y
}

let name: string := "Alice"
let scores: Array<int> := [95, 87, 92]
```

### Stage 2: Optional Runtime Type Checking

```ruff
@type_check
func calculate(x: int, y: float) -> float {
    return x * y
}
```

### Stage 3: JIT Optimization for Typed Code

Typed code could enable 10-50x performance improvements through:
- Unboxed arithmetic
- Stack allocation
- SIMD vectorization

**Status**: 🔬 EXPLORATORY - Not committed for v1.0

---

## v0.11.0 - Developer Experience

**Focus**: World-class tooling for productivity  
**Timeline**: Q3 2026  
**Priority**: P1

### Language Server Protocol (LSP) (P1)

**Estimated Effort**: Large (4-6 weeks)

**Features**:
- Autocomplete (built-ins, variables, functions)
- Go to definition
- Find references
- Hover documentation
- Real-time diagnostics
- Rename refactoring
- Code actions

**Implementation**: Use `tower-lsp` Rust framework

---

### Code Formatter (ruff-fmt) (P1)

**Estimated Effort**: Medium (2-3 weeks)

**Features**:
- Opinionated formatting (like gofmt, black)
- Configurable indentation
- Line length limits
- Import sorting

---

### Linter (ruff-lint) (P1)

**Estimated Effort**: Medium (3-4 weeks)

**Rules**:
- Unused variables
- Unreachable code
- Type mismatches
- Missing error handling
- Auto-fix for simple issues

---

### Package Manager (P1)

**Estimated Effort**: Large (8-12 weeks)

**Features**:
- `ruff.toml` project configuration
- Dependency management with semver
- Package registry
- CLI: `ruff init`, `ruff add`, `ruff install`, `ruff publish`

---

### REPL Improvements (P2)

**Estimated Effort**: Medium (1-2 weeks)

**Features**:
- Tab completion
- Syntax highlighting
- Multi-line editing
- `.help <function>` documentation

---

### Documentation Generator (P2)

**Estimated Effort**: Medium (2-3 weeks)

Generate HTML documentation from doc comments:

```ruff
/// Calculates the square of a number.
/// 
/// # Examples
/// ```ruff
/// result := square(5)  # 25
/// ```
func square(n) {
    return n * n
}
```

---

## v0.11.0+ - Stub Module Completion

**Status**: Planned  
**Priority**: P2 - Implement on-demand

### JSON Module (json.rs)
- `parse_json()`, `to_json()`, `json_get()`, `json_merge()`
- **Trigger**: When users need JSON API integration

### Crypto Module (crypto.rs)
- Hashing: MD5, SHA256, SHA512
- Encryption: AES, RSA
- Digital signatures
- **Trigger**: When users need secure authentication

### Database Module (database.rs)
- MySQL, PostgreSQL, SQLite connections
- Query execution, transactions
- Connection pooling
- **Trigger**: When users need persistent storage

### Network Module (network.rs)
- TCP/UDP socket operations
- **Trigger**: When users need low-level networking

---

## v1.0.0 - Production Ready

**Focus**: Polish, documentation, community  
**Timeline**: Q4 2026  
**Goal**: Production-ready language competitive with Python/Go

**Requirements**:
- All v0.9.0 performance targets met
- All v0.11.0 tooling complete
- Comprehensive documentation
- Stable API (no breaking changes)

---

## Future Versions (v1.0.0+)

### Generic Types (P2)
```ruff
func first<T>(arr: Array<T>) -> Option<T> {
    if len(arr) > 0 { return Some(arr[0]) }
    return None
}
```

### Union Types (P2)
```ruff
func process(value: int | string | null) {
    match type(value) {
        case "int": print("Number")
        case "string": print("Text")
    }
}
```

### Enums with Methods (P2)
```ruff
enum Status {
    Pending,
    Active { user_id: int },
    Completed { result: string }
    
    func is_done(self) {
        return match self {
            case Status::Completed: true
            case _: false
        }
    }
}
```

### Macros & Metaprogramming (P3)
```ruff
macro debug_print(expr) {
    print("${expr} = ${eval(expr)}")
}
```

### Foreign Function Interface (FFI) (P3)
```ruff
lib := load_library("libmath.so")
extern func cos(x: float) -> float from lib
```

### WebAssembly Compilation (P3)
```bash
ruff build --target wasm script.ruff
```

### AI/ML Built-in (P3)
```ruff
import ml
model := ml.linear_regression()
model.train(x_train, y_train)
```

---

## 🤝 Contributing

**Good First Issues**:
- String utility functions
- Array utility functions
- Test coverage improvements

**Medium Complexity**:
- JIT opcode coverage expansion
- Error message improvements
- Standard library modules

**Advanced Projects**:
- LSP implementation
- Package manager
- JIT performance optimization
- Debugger implementation

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## Version Strategy

- **v0.8.0**: VM + JIT foundation ✅
- **v0.9.0**: JIT performance (beat Python) ← CURRENT
- **v0.10.0**: Optional static typing (exploratory)
- **v0.11.0**: Developer experience (LSP, package manager)
- **v1.0.0**: Production-ready 🎉

**See Also**:
- [CHANGELOG.md](CHANGELOG.md) - Completed features
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md) - Performance guide
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - System architecture

---

*Last Updated: January 28, 2026*
