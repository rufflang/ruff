# Ruff Language - Development Roadmap

This roadmap outlines **upcoming** planned features and improvements. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.8.0 (Released January 2026)  
> **Next Planned Release**: v0.9.0 (JIT Performance - Beat Python)  
> **Status**: Phase 7 In Progress - Recursive JIT Working, Performance Optimization Needed

---

## 🎯 What's Next (Priority Order)

**IMMEDIATE (v0.9.0 BLOCKERS)**:
1. **🔥 JIT Performance Optimization** - Make Ruff FASTER than Python (P0 - CRITICAL)
   - Recursive JIT working but 33x slower than Python due to runtime overhead
   - Must achieve 5-10x speedup over Python across ALL benchmarks
   - See Phase 7 details below

2. **v1.0 Release Preparation** - Finalize APIs, comprehensive documentation

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

**Status**: 🔥 IN PROGRESS - Correctness Done, Performance Optimization Needed  
**Current State**: Recursive JIT compiles and executes correctly, but too slow

#### Current Benchmark Results (2026-01-28):

| Benchmark | Ruff JIT | Python | Status |
|-----------|----------|--------|--------|
| fib(10) | Correct (55) | Correct | ✅ Works |
| fib(25) | 1.2s | 0.04s | ❌ 30x slower |
| Array Sum (1M) | 52ms | 52ms | ✅ Matches |
| Hash Map (100k) | 34ms | 34ms | ✅ Matches |

#### Root Cause Analysis (Completed):

The JIT is **correct** but **slow** due to runtime overhead:

1. ~~**HashMap Lookup per Variable** - Every `LoadVar`/`StoreVar` calls C function + HashMap lookup~~ ✅ FIXED (Step 7)
2. **HashMap Clone per Call** - `var_names` HashMap cloned on every function invocation
3. **Value Boxing/Unboxing** - Every operation wraps/unwraps `Value` enum
4. **No Register Allocation** - All values go through memory, not CPU registers
5. **Function Call Overhead** - Each recursive call goes through VM dispatch

#### Implementation Plan:

**Step 7: Register-Based Locals (P0) - ✅ COMPLETE**
- [x] Pre-allocate local variable slots at compile time
- [x] Map variable names to Cranelift stack slots (no HashMap)
- [x] Use direct memory access instead of function calls
- [x] Parameters initialized from HashMap at function entry
- [x] Fall back to runtime for globals and function references
- [x] Comprehensive test suite added
- Status: Local variable access now uses fast stack slots

**Step 8: Inline Caching (P0 - NEXT)**
- [ ] Cache resolved function pointers after first call
- [ ] Avoid function lookup on subsequent calls
- [ ] Target: 5-10x speedup on recursive functions

**Step 9: Value Unboxing (P1)**
- [ ] Keep integers as raw i64 in JIT code
- [ ] Only box when crossing JIT/interpreter boundary
- [ ] Target: 2-5x speedup on arithmetic

**Step 10: Loop Back-Edge Fix (P1)**
- [ ] Fix SSA block parameters for backward jumps
- [ ] Enable JIT for `while` and `for` loops
- [ ] Currently `test_compile_simple_loop` is ignored

#### Performance Targets (Non-Negotiable for v0.9.0):

```
TARGET:
- Fib Recursive (n=25):  <40ms   (match Python)
- Fib Recursive (n=30):  <300ms  (match Python)
- Array Sum (1M):        <10ms   (5x faster than Python)
- Hash Map (100k):       <10ms   (3x faster than Python)

GOAL: Ruff >= 5x faster than Python on compute-heavy benchmarks
```

#### Success Criteria (BLOCKING v0.9.0):

- [ ] Fibonacci faster than Python (minimum match, target 5x)
- [ ] All compute benchmarks show Ruff >= Python performance
- [ ] No regressions in correctness (198 tests passing)

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
