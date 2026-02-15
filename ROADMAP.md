# Ruff Language - Development Roadmap

This roadmap outlines **upcoming** planned features and improvements. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.9.0 (Released February 2026)  
> **Next Planned Release**: v0.10.0  
> **Status**: Roadmap tracks post-v0.9.0 work only (for completed items see CHANGELOG).

---

## 🎯 What's Next (Priority Order)

**IMMEDIATE (v0.10.0)**:
1. **📦 Release Hardening (P1)** - stabilize APIs and prepare v1.0 trajectory
2. **🧪 Optional Static Typing Design (Exploratory)** - narrow design surface and implementation plan

**AFTER v0.10.0**:
3. **🔥 Parallel Processing / Concurrency (P0)** - SSG and async runtime throughput focus for v0.11.0
4. **Developer Experience** - LSP, formatter, linter, package management

---

## Priority Levels

- **P0 (Critical)**: Highest-priority next release blockers
- **P1 (High)**: Core features needed for v1.0 production readiness
- **P2 (Medium)**: Quality-of-life improvements and developer experience
- **P3 (Low)**: Nice-to-have features for advanced use cases

---

## v0.9.0 Release Status

v0.9.0 work is complete and archived in [CHANGELOG.md](CHANGELOG.md).  
This roadmap intentionally tracks only upcoming items.

---

## v0.10.0 - Release Hardening (P1)

### Completed Milestones

- **Builtin API Parity Contract (✅ Complete, February 2026)**
    - Synchronized VM builtin name registry (`get_builtin_names`) with interpreter-native registrations for release stability
    - Added regression coverage to detect missing and duplicate builtin API entries

- **Alias + OS/Path API Compatibility Contract (✅ Complete, February 2026)**
    - Restored modular runtime support for declared OS/path builtins and path aliases
    - Added queue/stack size API parity (`queue_size`, `stack_size`) in modular native handlers
    - Added integration tests for alias equivalence and path/collection API behavior

- **API Argument-Shape Compatibility Contract (✅ Complete, February 2026)**
    - Hardened filesystem alias argument-shape behavior for `join_path(...)` and `path_join(...)`
    - Hardened collection size API argument validation for `queue_size(...)` and `stack_size(...)`
    - Added integration regression coverage for async/filesystem/collection argument-shape parity (`promise_all`/`await_all`, `join_path`/`path_join`, `queue_size`/`stack_size`)

- **Unknown Native Dispatch Contract (✅ Complete, February 2026)**
    - Removed silent unknown-native fallback in modular native dispatcher
    - Unknown native names now return explicit runtime errors for faster API drift detection
    - Added dispatcher-level regression coverage to ensure newly introduced high-risk builtins do not regress into unknown-native fallback

### Remaining Focus

- Stabilize external-facing runtime APIs and aliases ahead of v1.0
- Continue expanding API compatibility regression coverage for VM/interpreter parity on newly introduced builtins

---

## v0.11.0 - Parallel Processing & Concurrency (P0)

**Focus**: Deliver production-grade async throughput for large I/O-bound workloads (SSG priority)  
**Timeline**: Q2-Q3 2026  
**Priority**: P0 - CRITICAL for production performance perception  
**Status**: Planned (execution focus shifted from v0.10.0)

### Scope (Forward Work Only)

Existing async/runtime groundwork is tracked in [CHANGELOG.md](CHANGELOG.md).
v0.11.0 tracks only the remaining performance and architecture work.

### Remaining High-Priority Workstreams

1. **Async VM Integration Completion**
     - Make cooperative suspend/resume path the default execution model for async-heavy workflows.
     - Remove remaining `block_on()` bottlenecks from VM await execution paths.
     - Ensure user-defined async functions can execute with true concurrency semantics.

2. **SSG Throughput Focus (Primary Benchmark Gate)**
     - Continue reducing render/write overhead in `bench-ssg` execution path.
     - Add additional native bulk helpers where script-level hot loops dominate.
     - Keep checksum/file-count equivalence validation intact for all benchmark path changes.

3. **Benchmark Stability & Measurement Quality**
     - Add repeat-run/median reporting to reduce one-off benchmark noise.
     - Keep Ruff-only stage profiling (`--profile-async`) as the optimization signal.
     - Keep cross-language runs (`--compare-python`) for directional trend tracking.

### Success Criteria

- `bench-ssg` Ruff build time: **<10 seconds** (phase target)
- Stretch target: **<5 seconds**
- No regressions in correctness (`cargo test` remains green)
- Async API surface remains stable and documented

### Performance Snapshot (Tracking)

- Baseline (synchronous): ~55 seconds
- Current (`ruff bench-ssg --compare-python`): ~36 seconds (Ruff local sample)
- Target (v0.11.0): <10 seconds

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

## v0.12.0 - Developer Experience

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

- **v0.10.0**: Architecture cleanup + release hardening + typing exploration
- **v0.11.0**: Parallel processing / concurrency performance (SSG focus)
- **v0.12.0**: Developer experience (LSP, package manager)
- **v1.0.0**: Production-ready 🎉

**See Also**:
- [CHANGELOG.md](CHANGELOG.md) - Completed features
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md) - Performance guide
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - System architecture

---

*Last Updated: February 15, 2026*
