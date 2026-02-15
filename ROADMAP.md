# Ruff Language - Development Roadmap

This roadmap outlines **upcoming** planned features and improvements. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.9.0 (Released February 2026)  
> **Next Planned Release**: v0.10.0  
> **Status**: Roadmap tracks post-v0.9.0 work only (for completed items see CHANGELOG).

---

## 🎯 What's Next (Priority Order)

**IMMEDIATE (v0.10.0)**:
1. **🔥 Parallel Processing / Concurrency (P0)** - critical real-world throughput work
2. **🏗️ Architecture Cleanup (P1/P2)** - isolate runtime from AST and remove leaky internals
3. **📦 Release Hardening (P1)** - stabilize APIs and prepare v1.0 trajectory

**AFTER v0.10.0**:
4. **Developer Experience** - LSP, formatter, linter, package management
5. **Optional Static Typing** - gradual typing exploration and optimization paths

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

## v0.10.0 - Architecture Cleanup Tasks (P2)

These are planned post-v0.9.0 and are candidates for v0.10.0 scope.

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

## v0.10.0 - Parallel Processing & Concurrency (P0)

**Focus**: Enable multicore parallelism for real-world I/O-bound workloads  
**Timeline**: Q1 2026 (Before v1.0)  
**Priority**: P0 - CRITICAL for production performance perception  
**Status**: ⚠️ IN PROGRESS - Async I/O infrastructure complete, but architecture limits prevent full performance gains

### Current Implementation Status (2026-01-29)

**✅ COMPLETED**:
- Tokio runtime integration (tokio 1.x)
- VM stores tokio::runtime::Handle for spawning async tasks
- Promise Value type includes task_handle for true async execution
- File I/O operations now truly async using tokio::fs:
  - `read_file()` - async file reading
  - `write_file()` - async file writing
  - `list_dir()` - async directory listing
- `await_all()` utility function for concurrent promise execution
- `await_all(promises, concurrency_limit)` / `promise_all(promises, concurrency_limit)` batching support
- `parallel_map(array, func, concurrency_limit?)` for bounded concurrent mapping workflows
- `par_map(array, func, concurrency_limit?)` alias for concise concurrent mapping syntax
- `par_each(array, func, concurrency_limit?)` for concurrent side-effect workflows
- Rayon-backed parallel mapping fast path for supported native mappers (`len`, `upper`/`to_upper`, `lower`/`to_lower`)
- VM/JIT execution lane for `BytecodeFunction` closures passed to `parallel_map` / `par_map`
- Configurable default async task pool sizing via `set_task_pool_size(size)` / `get_task_pool_size()`
- Promises work correctly with await syntax
- Small-scale concurrency performs well (10 files in 1.26ms = 126μs/file)

**❌ LIMITATIONS DISCOVERED**:
- **VM Architecture Bottleneck**: Each `await` blocks the entire VM with `block_on()` (synchronous execution model)
- **Large-scale concurrency overhead**: Spawning 10K+ tokio tasks has excessive overhead
  - 10,001 files: 91 seconds (worse than 55s synchronous baseline!)
  - Batching helps per-batch speed but batches process serially (196s total)
- **Async functions still synchronous**: User-defined async functions don't spawn concurrent tasks
- **No true parallelism**: Cannot utilize multiple CPU cores for parallel execution

### Problem Statement

**Current Situation**: 
- JIT makes compute-heavy code 30-50x faster than Python ✅
- File I/O operations are now truly async with tokio ✅
- Small concurrent workloads perform well (10 files in 1.26ms) ✅
- BUT: Large-scale concurrency is slower than sequential execution ❌
  - SSG benchmark: 10,000 files processed in 91-196 seconds (async)
  - SSG baseline: 55 seconds (synchronous)
  - Python equivalent: 0.05 seconds (1000x faster!)

**Root Cause**:
- VM's synchronous execution model: `block_on()` serializes all awaits
- Each await opcode blocks the entire VM thread
- Spawning 10K+ tasks simultaneously creates massive overhead
- Python's `ProcessPoolExecutor` uses all CPU cores (8-10x parallel speedup)
- Async functions in Ruff code still execute synchronously (not spawned as tasks)

**Impact**: Without true async VM execution, Ruff appears "slow" on I/O-bound workloads despite having async infrastructure.

### Next Steps to Complete Async Implementation

The async infrastructure is in place but the VM architecture prevents full performance gains. Here are the options to proceed:

#### ⭐ Recommended: Option 1 - Async VM Execution Model

**Goal**: Remove `block_on()` from the VM's Await opcode to allow true concurrent execution.

**What's Working Now**:
- ✅ File I/O operations spawn tokio tasks and return Promises immediately
- ✅ `await_all()` can wait for multiple promises concurrently
- ✅ Small-scale concurrency performs well

**The Bottleneck**:
```rust
// Current approach in vm.rs Await opcode (BLOCKS THE VM!)
Value::Promise { receiver, .. } => {
    self.runtime_handle.block_on(async {
        receiver.recv().await  // This blocks the entire VM thread!
    })
}
```

**Solution**:
1. **Make the VM itself async**: The entire VM execution loop needs to be async-aware
2. **Replace `block_on()` with polling**: When hitting an await, save VM state and yield control
3. **Implement cooperative scheduling**: VM can execute other tasks while waiting for I/O
4. **Use tokio's select!/join! patterns**: Multiple VM contexts can run concurrently

**Implementation Steps**:
- [x] Refactor VM to support suspendable execution (save/restore VM state)
- [x] Implement VM context switching for concurrent execution
- [x] Change Await opcode to yield instead of block
- [x] Add VM scheduler to manage multiple concurrent VM contexts
- [x] Add and run reproducible SSG benchmark harness (`ruff bench-ssg`) for 10K-file async workload validation

**Estimated Effort**: 2-3 weeks  
**Complexity**: High (requires VM architecture changes)  
**Impact**: Would enable true async/await performance

#### Option 2 - Task Batching & Concurrency Limits

**Goal**: Work within current VM constraints but optimize task spawning.

**Approach**:
- Implement semaphore-based concurrency limiting (e.g., max 100 concurrent tasks)
- Batch large operations to avoid spawning 10K+ tasks at once
- Add native functions for controlled parallel execution

**Implementation Steps**:
- [x] Add `parallel_map(array, func, concurrency_limit)` native function
- [x] Implement batching-based task limiting in `promise_all` / `await_all` (optional `concurrency_limit`)
- [x] Add configurable task pool sizing
- [x] Optimize Promise.all for large arrays (removed per-promise await-task spawning overhead)

**Estimated Effort**: 1 week  
**Complexity**: Medium  
**Impact**: Would improve large-scale performance but still limited by VM blocking

#### Option 3 - Hybrid JIT + Parallel Execution

**Goal**: Combine JIT compilation with parallel task execution.

**Approach**:
- Use Rayon-style parallel iterators for compute-heavy loops
- JIT-compile loop bodies, execute in parallel across threads
- Keep async I/O for truly async operations

**Implementation Steps**:
- [x] Integrate rayon for parallel iteration
- [x] JIT-compile closures passed to parallel iterators
- [x] Benchmark against Python's ProcessPoolExecutor via `ruff bench-cross` and cross-language benchmark artifacts

**Estimated Effort**: 2-3 weeks  
**Complexity**: High (JIT + threading)  
**Impact**: Best of both worlds (JIT speed + parallelism)

### Implementation Priority

**RECOMMENDED PATH**: Start with **Option 2** (Task Batching) as a quick win, then pursue **Option 1** (Async VM) for full async/await performance.

**Rationale**:
- Option 2 can be completed in 1 week and provide immediate improvements
- Option 2 doesn't require VM architecture changes
- Option 1 is the "correct" long-term solution but requires significant refactoring
- Option 3 is interesting but doesn't address async I/O bottleneck

**Phase 1: Quick Wins (1 week) - Task Batching**
- [x] Choose concurrency model (Implemented: Basic goroutine-style spawn)
- [x] Implement `spawn` keyword and scheduler (Basic implementation complete)
- [x] Add `channel()` for message passing (Implemented with send/receive methods)
- [x] Fix VM/JIT support for channels (Bug fix: FieldGet now supports Channel methods)
- [x] Implement `await_all()` for synchronization (Already implemented as alias for Promise.all)
- [x] Thread-safe Value type operations (Implemented via shared-value builtins: `shared_set/get/has/delete/add_int`)
- [x] Shared-state concurrency (Spawn now captures transferable parent binding snapshots for worker visibility)

**Status Note**: Spawn/channel infrastructure now supports parent-binding snapshot visibility:
- `spawn` workers receive transferable parent binding snapshots (readable in spawned code)
- Shared state coordination is available through thread-safe shared-value builtins (`shared_set/get/has/delete/add_int`)
- Parent scope write-back remains isolated by design (no implicit cross-thread mutation)
- Future work can extend richer shared-state APIs beyond key-based shared-value coordination

**Phase 2: Runtime Integration (1-2 weeks)**
- [x] Integrate async runtime (tokio/smol) (Tokio already integrated)
- [x] Make file I/O operations async-aware (Done: read_file, write_file, list_dir use tokio::fs)
- [x] Add thread pool for blocking operations (Done: configurable task pool sizing)
- [x] Ensure JIT-compiled code can run on any thread (Done: VM execution is thread-compatible)

**Phase 3: Testing & Benchmarks (1 week)**
- [x] Add concurrency test suite (Basic tests exist, expanded with scalability tests)
- [x] Re-run SSG benchmark with parallelism (`ruff bench-ssg --compare-python`)
- [ ] Target: 10K file SSG build in <1 second (using all cores)
- [x] Add `parallel_map(array, func, limit)` with concurrency control
- [x] Implement task limiting in `await_all` / `promise_all` with optional `concurrency_limit`
- [x] Add configurable default task pool size controls (`set_task_pool_size` / `get_task_pool_size`)
- [ ] Meet SSG benchmark phase target (<10 seconds)

**Phase 2: Async VM (2-3 weeks) - True Async/Await**
- [x] Design VM state save/restore mechanism
- [x] Implement VM context for suspendable execution
- [x] Change Await opcode from block_on() to yield/resume
- [x] Add VM scheduler for managing concurrent contexts
- [x] Test with SSG benchmark harness (`ruff bench-ssg`)

**Phase 3: Optimization (1 week)**
- [x] Profile async execution to find bottlenecks (`ruff bench-ssg --profile-async` stage breakdown + bottleneck summary)
- [x] Optimize Promise creation/resolution overhead (`parallel_map` mixed-result fast path + reduced Promise.all allocation churn)
- [x] Add caching for frequently-awaited operations (`Promise.all` / `parallel_map` now reuse cached promise results)
- [x] Add bulk async file operations (`async_read_files` / `async_write_files`) and migrate SSG harness to reduce per-file promise overhead
- [x] Offload SSG page wrapping to native bulk rendering (`ssg_render_pages`) to reduce interpreter-side render-loop overhead
- [x] Test scalability with 10K+ concurrent operations (comprehensive tests added - all pass in <3s)

### Success Criteria

- [ ] SSG benchmark: 10,000 files in <5 seconds (vs current 91s async, 55s sync)
- [x] Small concurrent operations maintain good performance (10 files in ~1ms) ✅
- [x] No regressions in correctness (all tests passing) ✅
- [x] Clean API that's easy to understand and use ✅
- [x] 10K+ scalability verified (tests pass in <3 seconds) ✅

### Performance Targets

**SSG Benchmark**:
- Baseline (synchronous): 55 seconds
- Current (`ruff bench-ssg --compare-python`): 36.26 seconds (Ruff), 7.71 seconds (Python baseline) ❌
- Phase 1 target (batching): <10 seconds
- Phase 2 target (async VM): <5 seconds
- Stretch goal (optimal): <1 second

**Microbenchmarks**:
- Small concurrency (10 files): ~1.26ms ✅
- Spawn overhead: <10ms per 100 tasks
- Promise resolution: <100μs per promise

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

*Last Updated: February 15, 2026*
