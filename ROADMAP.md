# Ruff Language - Development Roadmap

This roadmap outlines **upcoming** planned features and improvements. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: Latest stable release (see [CHANGELOG.md](CHANGELOG.md))  
> **Next Planned Release**: v0.11.0  
> **Status**: Roadmap tracks upcoming work only (for completed items see CHANGELOG).

---

## 🎯 What's Next (Priority Order)

**IMMEDIATE (v0.11.0)**:
1. **🔥 Parallel Processing / Concurrency (P0)** - SSG and async runtime throughput focus
2. **Developer Experience Foundations (P1)** - LSP, formatter, linter planning/initial implementation

**AFTER v0.11.0**:
3. **📦 v0.12.0 Developer Experience Expansion** - LSP, formatter, linter, package management
4. **v1.0 Readiness** - stabilization, docs completeness, ecosystem polish

---

## Priority Levels

- **P0 (Critical)**: Highest-priority next release blockers
- **P1 (High)**: Core features needed for v1.0 production readiness
- **P2 (Medium)**: Quality-of-life improvements and developer experience
- **P3 (Low)**: Nice-to-have features for advanced use cases

---

Completed release work is archived in [CHANGELOG.md](CHANGELOG.md).

---

## v0.11.0 - Parallel Processing & Concurrency (P0)

**Focus**: Deliver production-grade async throughput for large I/O-bound workloads (SSG priority)  
**Timeline**: Q2-Q3 2026  
**Priority**: P0 - CRITICAL for production performance perception  
**Status**: In Progress

### Completed Milestones

- **Async VM Integration Completion (✅ Complete, February 2026)**
  - Made cooperative suspend/resume the default execution model for async-heavy workloads
  - Enabled `cooperative_suspend_enabled: true` in VM constructor to activate non-blocking await semantics by default
  - Integrated cooperative scheduler loop into main execution path (`execute_until_suspend()` + `run_scheduler_until_complete()`)
  - Replaced blocking `vm.execute()` with cooperative scheduler in VM execution entry points
  - User-defined async functions now execute with true concurrency semantics
  - Eliminated remaining blocking `block_on()` bottleneck in critical VM await paths
  - Added comprehensive integration tests for cooperative default behavior (7 new tests, all passing)
  - Result: Production-ready non-blocking async VM ready for SSG and I/O-bound workloads

### Scope (Forward Work Only)

Existing async/runtime groundwork is tracked in [CHANGELOG.md](CHANGELOG.md).
v0.11.0 tracks only the remaining performance and architecture work.

### Remaining High-Priority Workstreams

1. **SSG Throughput Focus (Primary Benchmark Gate)**
     - Continue reducing render/write overhead in `bench-ssg` execution path.
     - Add additional native bulk helpers where script-level hot loops dominate.
     - Keep checksum/file-count equivalence validation intact for all benchmark path changes.

2. **Benchmark Stability & Measurement Quality**
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
- All v0.11.0 performance targets met
- All v0.12.0 tooling milestones complete
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

- **v0.11.0**: Parallel processing / concurrency performance (SSG focus)
- **v0.12.0**: Developer experience (LSP, package manager)
- **v1.0.0**: Production-ready 🎉

**See Also**:
- [CHANGELOG.md](CHANGELOG.md) - Completed features
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md) - Performance guide
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - System architecture

---

*Last Updated: February 18, 2026*
