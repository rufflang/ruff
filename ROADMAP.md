# Ruff Language - Development Roadmap

This roadmap outlines **upcoming** planned features and improvements. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: Latest stable release (see [CHANGELOG.md](CHANGELOG.md))  
> **Next Planned Release**: v0.11.0  
> **Status**: Roadmap tracks upcoming work only (for completed items see CHANGELOG).

---

## 🎯 What's Next (Priority Order)

**IMMEDIATE (v0.11.0)**:
1. **🔥 Parallel Processing / Concurrency (P0)** - SSG and async runtime throughput focus
2. **Developer Experience Foundations (P1)** - LSP, formatter, linter planning/initial implementation (non-blocking for v0.11 throughput gate)

### v0.11 Scope Lock (March 2026)

- **Gate first**: v0.11 ships on throughput progress (`bench-ssg` target trajectory) and correctness stability.
- **JIT policy**: function-level JIT is a supporting/parallel workstream in v0.11 and only advances release priority when benchmark-proven to improve `bench-ssg` / `--profile-async` metrics without regressions.
- **Execution rule**: if an optimization does not materially improve benchmark gates, defer it behind throughput-critical work.

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

- **Benchmark Stability: Repeat-Run + Median Reporting (✅ Complete, March 2026)**
    - Added `ruff bench-ssg --runs <N>` support for repeated benchmark execution.
    - Added aggregate benchmark reporting (median/mean/min/max/stddev) for Ruff build time and throughput.
    - Added aggregate comparison reporting for Python baseline runs and median speedup output.
    - Added median-based stage bottleneck reporting for `--profile-async` output.
    - Added comprehensive unit coverage for statistical aggregation and consistency validation.

- **Benchmark Stability: Warmup-Run Support for Measurement Quality (✅ Complete, March 2026)**
    - Added `ruff bench-ssg --warmup-runs <N>` so pre-measurement warmup runs can be executed and excluded from measured summary statistics.
    - Added shared benchmark harness series orchestration for warmup + measured phases (`run_ssg_benchmark_series(...)`) with consistent validation/error contracts.
    - Added comprehensive harness coverage for warmup exclusion behavior plus warmup/measured failure surfacing.

- **SSG Native Bulk Helper: Output Path Generation (✅ Complete, March 2026)**
    - Added `ssg_build_output_paths(output_dir, file_count, extension?)` as a native helper for indexed SSG output path construction.
    - Updated benchmark pipeline to use native path generation instead of script-level path loops in the timed render/write path.
    - Added comprehensive native-function behavior and contract validation coverage.

- **SSG Native Bulk Helper: Async Render+Write Fusion (✅ Complete, March 2026)**
    - Added `ssg_render_and_write_pages(source_pages, output_dir, concurrency_limit?)` to render and write SSG pages in one bounded-concurrency native async operation.
    - Updated benchmark render/write stage to use the fused native helper instead of separate render + write orchestration.
    - Added comprehensive success/error contract coverage and dispatcher-level hardening tests.

- **SSG Throughput Follow-Through: Render/Write Pipeline Optimization (✅ Complete, March 2026)**
    - Optimized `ssg_render_and_write_pages(...)` to eliminate serial pre-render buffering and render HTML inside bounded async write workers.
    - Preserved benchmark checksum/file-count equivalence contracts while reducing render/write orchestration overhead.
    - Added comprehensive regression coverage for checksum integrity and empty-input summary behavior.

- **SSG Throughput Follow-Through: Read/Render/Write Pipeline Fusion (✅ Complete, March 2026)**
    - Added `ssg_read_render_and_write_pages(source_paths, output_dir, concurrency_limit?)` to fuse read + render + write into one bounded-concurrency async native operation.
    - Updated the timed `bench-ssg` Ruff path to call the fused helper and consume stage timings (`read_ms`, `render_write_ms`) from the native summary.
    - Added comprehensive success/error contract coverage, including checksum/file-count equivalence and read/write failure propagation.

- **SSG Throughput Follow-Through: Read-to-Write Streaming Pipeline (✅ Complete, March 2026)**
    - Optimized `ssg_read_render_and_write_pages(...)` to stream completed async reads directly into bounded render/write workers.
    - Removed full read-stage source-body buffering in the fused helper path while preserving checksum/file-count equivalence contracts.
    - Added regression coverage for empty-input summary contracts and single-worker output-contract preservation.

- **SSG Throughput Follow-Through: Fused Write-Backpressure Hardening (✅ Complete, March 2026)**
    - Hardened `ssg_read_render_and_write_pages(...)` to enforce bounded write in-flight concurrency with explicit pending-write backpressure during streaming read/render/write execution.
    - Preserved stage-metric contracts (`read_ms`, `render_write_ms`) and checksum/file-count equivalence while removing unbounded write-task growth risk under read-heavy batches.
    - Added high-volume regression coverage for large-batch single-worker and low-concurrency output-contract preservation.

- **SSG Throughput Follow-Through: Output-Path Precompute in Fused Write Pipelines (✅ Complete, March 2026)**
    - Optimized `ssg_render_and_write_pages(...)` and `ssg_read_render_and_write_pages(...)` to precompute indexed output paths once per batch and reuse them across async write workers.
    - Removed per-write output-path string construction overhead while preserving checksum/file-count and stage-metric contracts in the timed benchmark path.
    - Added regression coverage for batch output-path generation and high-volume low-concurrency render/write output-contract preservation.

- **SSG Throughput Follow-Through: Direct Read-to-Write Dispatch + Path-Clone Elimination (✅ Complete, March 2026)**
    - Optimized `ssg_read_render_and_write_pages(...)` to dispatch completed reads directly into available bounded write slots before queueing, reducing intermediate write-buffer churn under high-concurrency runs.
    - Optimized fused and render/write-only write futures to reuse precomputed output paths without per-task path cloning while preserving checksum/file-count and stage-metric contracts.
    - Added regression coverage for high-concurrency output/checksum contract preservation in both `ssg_render_and_write_pages(...)` and `ssg_read_render_and_write_pages(...)`.

- **SSG Throughput Follow-Through: Fused Read-Ahead Overlap Window (✅ Complete, March 2026)**
    - Optimized `ssg_read_render_and_write_pages(...)` with a bounded read-ahead scheduling window (`2x` write concurrency, capped by file count) to improve overlap between read completions and bounded write dispatch.
    - Preserved checksum/file-count equivalence and stage-metric contracts (`read_ms`, `render_write_ms`) in the timed benchmark path while reducing read-lane starvation risk.
    - Added comprehensive helper + integration regression coverage, including extreme-concurrency contract validation.

- **SSG Throughput Follow-Through: Streamed HTML Writes in Fused Pipelines (✅ Complete, March 2026)**
    - Optimized `ssg_render_and_write_pages(...)` and `ssg_read_render_and_write_pages(...)` to stream rendered HTML segments directly to async file writes instead of allocating full per-page HTML strings during write futures.
    - Preserved checksum/file-count equivalence and stage-metric contracts (`read_ms`, `render_write_ms`) while reducing render/write allocation pressure in timed benchmark execution.
    - Added direct streamed-writer contract coverage for exact-output and write-failure behavior.

- **SSG Throughput Follow-Through: Precomputed HTML Prefixes in Streamed Write Workers (✅ Complete, March 2026)**
    - Optimized `ssg_render_and_write_pages(...)` and `ssg_read_render_and_write_pages(...)` to precompute per-index HTML prefixes once per batch and reuse them across streamed async write futures.
    - Removed per-write index formatting overhead while preserving checksum/file-count equivalence and stage-metric contracts (`read_ms`, `render_write_ms`).
    - Added direct prefix-helper + large-index heading regression coverage to lock rendered output naming/content contracts.

- **SSG Throughput Follow-Through: Adaptive Read-Refill Backpressure in Fused Pipeline (✅ Complete, March 2026)**
    - Optimized `ssg_read_render_and_write_pages(...)` with adaptive read-refill targeting that scales read in-flight scheduling based on current bounded write backlog budget.
    - Added write-completion-path read refill so fused scheduling can restore read overlap earlier as write backlog drains, reducing queued source-body churn while preserving bounded write concurrency.
    - Added focused policy regression coverage for backlog-driven read-target behavior while preserving checksum/file-count and stage-metric contracts (`read_ms`, `render_write_ms`).

- **SSG Throughput Follow-Through: Precomputed Rendered-Length Scheduling (✅ Complete, March 2026)**
    - Optimized `ssg_render_and_write_pages(...)` and `ssg_read_render_and_write_pages(...)` to precompute rendered HTML lengths at scheduling time and reuse them during write completion accounting.
    - Removed per-write rendered-length recomputation in async write futures while preserving checksum/file-count and stage-metric contracts (`read_ms`, `render_write_ms`).
    - Added focused rendered-length helper regression coverage to lock checksum accounting inputs for empty-body and representative prefix/body scenarios.

- **SSG Throughput Follow-Through: Adaptive Write-Priority Refill Ordering (✅ Complete, March 2026)**
    - Optimized `ssg_read_render_and_write_pages(...)` read-completion refill ordering to prioritize draining queued writes first when bounded write slots are available.
    - Preserved adaptive read-refill targeting after write-priority refill to keep read/write overlap while reducing pending-write backlog growth under bursty read completions.
    - Added focused policy regression coverage for write-priority decision contracts across backlog/no-backlog/saturated-lane scenarios.

- **SSG Throughput Follow-Through: Vectored Streamed HTML Writes (✅ Complete, March 2026)**
    - Optimized `ssg_write_rendered_html_page(...)` to emit pre-segmented HTML via a vectored async write loop with explicit flush completion.
    - Reduced per-page write-call overhead in both `ssg_render_and_write_pages(...)` and `ssg_read_render_and_write_pages(...)` while preserving checksum/file-count and stage-metric contracts.
    - Added streamed-writer regression coverage for empty-body and large-body output/length contract preservation.

- **SSG Throughput Follow-Through: Write-Result Checksum Accounting (✅ Complete, March 2026)**
    - Optimized `ssg_render_and_write_pages(...)` to remove pre-write checksum precomputation and account checksum from actual async write-result byte counts.
    - Optimized `ssg_read_render_and_write_pages(...)` to remove precomputed rendered-length queueing during read completions and account checksum from actual write-result byte counts.
    - Added focused Unicode checksum regression coverage to preserve byte-accurate checksum/file-count contracts under multibyte content.

- **SSG Throughput Follow-Through: Direct Vectored Write-Byte Accounting (✅ Complete, March 2026)**
    - Optimized `ssg_write_rendered_html_page(...)` to return accumulated byte totals directly from vectored async write results instead of post-write rendered-length recomputation.
    - Preserved checksum/file-count and stage-metric contracts in `ssg_render_and_write_pages(...)` and `ssg_read_render_and_write_pages(...)` while tightening byte-accounting to observed write totals.
    - Added focused streamed-writer regression coverage for ASCII and UTF-8 byte-count return contract preservation.

- **SSG Throughput Follow-Through: Single-Worker Fast Path for Write Pipelines (✅ Complete, March 2026)**
    - Optimized `ssg_render_and_write_pages(...)` with a dedicated `concurrency_limit=1` lane to bypass multi-future scheduling overhead while preserving checksum/file-count output contracts.
    - Optimized `ssg_read_render_and_write_pages(...)` with a dedicated single-worker read→write lane to reduce queue/select overhead while preserving checksum/file-count and stage-metric keys (`read_ms`, `render_write_ms`).
    - Added focused regression coverage for single-worker `ssg_render_and_write_pages(...)` output/checksum contract preservation.

- **SSG Throughput Follow-Through: Combined Output-Path and Prefix Precompute Pass (✅ Complete, March 2026)**
    - Optimized `ssg_render_and_write_pages(...)` and `ssg_read_render_and_write_pages(...)` to precompute output paths and HTML prefixes in one shared batch pass.
    - Removed duplicate per-index string conversion work across separate precompute loops while preserving checksum/file-count and stage-metric contracts.
    - Added focused regression coverage for combined path/prefix generation behavior, including empty-input contracts.

- **Benchmark Stability: Configurable Artifact Root (✅ Complete, March 2026)**
    - Added `ruff bench-ssg --tmp-dir <PATH>` to route benchmark artifacts to an explicit directory root.
    - Updated Ruff and Python benchmark scripts to honor shared `RUFF_BENCH_SSG_TMP_DIR` override semantics.
    - Added unit coverage for tmp-dir override contract handling in SSG benchmark harness code.

- **Benchmark Stability: Command-Level Failure Validation (✅ Complete, March 2026)**
    - Added deterministic preflight missing-script checks in SSG benchmark harness execution path.
    - Added command-level benchmark harness tests for missing required metric outputs.
    - Added command-level benchmark harness tests for Ruff/Python checksum mismatch rejection.

- **Benchmark Stability: Variability Warning Signals (✅ Complete, March 2026)**
    - Added coefficient-of-variation (CV) analysis for measured-run aggregate statistics in SSG benchmark reporting.
    - Added `bench-ssg` measurement-quality warning output when high-variance run distributions are detected (threshold: `5%`).
    - Added comprehensive regression coverage for warning emission and suppression contracts across high/low-variance and low-run-count scenarios.

- **Benchmark Stability: Measured-Run Trend Tracking (✅ Complete, March 2026)**
    - Added first-to-last measured-run trend analysis and reporting for Ruff build-time and throughput metrics in `bench-ssg` summaries.
    - Added optional Python build-time/throughput and Ruff-vs-Python speedup trend reporting when cross-language comparison data is available.
    - Added comprehensive regression coverage for trend contracts, including single-run suppression, Python-comparison consistency validation, and zero-baseline percent-delta handling.

- **Benchmark Stability: Trend Drift Warning Signals (✅ Complete, March 2026)**
    - Added trend-drift warning analysis for first-to-last measured-run percent deltas in `bench-ssg` trend reports.
    - Added warning output when drift magnitude crosses threshold (`10%`) for Ruff metrics and optional Python/speedup metrics.
    - Added comprehensive regression coverage for warning emission/suppression and measured-run-count gating behavior.

- **Benchmark Stability: Mean/Median Drift Warning Signals (✅ Complete, March 2026)**
    - Added mean-vs-median drift warning analysis for measured-run aggregate statistics in `bench-ssg` measurement-quality reporting.
    - Added warning output when drift magnitude crosses threshold (`7.5%`) for Ruff metrics and optional Python/speedup metrics.
    - Added comprehensive regression coverage for drift calculation, warning emission/suppression behavior, and measured-run-count gating.

### Scope (Forward Work Only)

Existing async/runtime groundwork is tracked in [CHANGELOG.md](CHANGELOG.md).
v0.11.0 tracks only the remaining performance and architecture work.

**Scope lock**: Throughput gate first. Function-level JIT remains supporting/parallel unless benchmark-proven against v0.11 performance gates.

### Remaining High-Priority Workstreams

1. **SSG Throughput Focus (Primary Benchmark Gate)**
    - Continue reducing residual render/write overhead in `bench-ssg` execution path after direct dispatch, path-clone elimination, read-ahead overlap, streamed-write, combined path/prefix precompute, write-result checksum-accounting, and single-worker fast-path follow-through.
    - Keep checksum/file-count equivalence validation intact for all benchmark path changes.
    - Profile additional overlap opportunities between read completion handling and bounded write dispatch without changing stage-metric key contracts.

2. **Benchmark Stability & Measurement Quality**
     - Keep Ruff-only stage profiling (`--profile-async`) as the optimization signal.
     - Keep cross-language runs (`--compare-python`) for directional trend tracking.
    - Continue refining warning thresholds/presentation for measurement-quality interpretation without changing benchmark metric contracts.

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

*Last Updated: March 15, 2026*
