# Ruff Language - Development Roadmap

This roadmap outlines **upcoming** planned features and improvements. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.8.0 (Released January 2026)  
> **Next Planned Release**: v0.9.0 (JIT Performance - Beat Python)  
> **Status**: Phase 7 - 95% Complete! Recursive JIT is **31-48x FASTER** than Python! Loop JIT remaining.

---

## 🎯 What's Next (Priority Order)

**IMMEDIATE (v0.9.0)**:
1. **✅ Recursive JIT Performance** - COMPLETE! Ruff is **31-48x FASTER** than Python!
   - fib(25): 0.5ms vs Python's 24ms (48x faster)
   - fib(30): 5-8ms vs Python's 253ms (31-50x faster)

2. **🔥 Loop JIT (Step 11)** - Enable JIT for while/for loops (P1 - CURRENT FOCUS)
   - Loops currently fall back to interpreter
   - Needed for array/hashmap benchmarks

3. **v1.0 Release Preparation** - Finalize APIs, comprehensive documentation

**AFTER v0.9.0**:
3. **Developer Experience** - LSP, Formatter, Linter, Package Manager (v0.11.0)
4. **Optional Static Typing** - Gradual typing for additional performance (v0.10.0, exploratory)

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

**Step 11: Loop Back-Edge Fix (P1) - NEXT PRIORITY**
- [ ] Fix SSA block parameters for backward jumps
- [ ] Enable JIT for `while` and `for` loops
- [ ] Currently `test_compile_simple_loop` is ignored
- Status: Loops fall back to interpreter. This is the main remaining work.

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
