# Ruff Field Notes — SSG Rayon CPU-Bounded Thread-Pool Sizing

**Date:** 2026-03-18
**Branch/Commits:** main / bc8a0ef (impl), 6478e48 (tests)
**Scope:** Capped the Rayon `ThreadPoolBuilder` thread count in
`ssg_run_rayon_read_render_write` at `min(concurrency_limit, available_parallelism).max(1)`
to prevent thread over-subscription when `concurrency_limit` >> CPU count.

---

## What I Changed

- **`src/interpreter/native_functions/async_ops.rs`** — new helper:
  - Added `ssg_rayon_cpu_cap() -> usize` (before `ssg_run_rayon_read_render_write`)
  - Calls `std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)`
  - Requires no new crate dependency — stable std since Rust 1.59

- **`src/interpreter/native_functions/async_ops.rs`** — `ssg_run_rayon_read_render_write`:
  - Replaced `ThreadPoolBuilder::new().num_threads(concurrency_limit.max(1))`
    with `ThreadPoolBuilder::new().num_threads(concurrency_limit.min(cpu_cap).max(1))`
    where `cpu_cap = ssg_rayon_cpu_cap()`
  - Added inline comment explaining the capping rationale

- **New regression tests** (4 tests, lib count 346 → 350):
  - `test_ssg_rayon_cpu_cap_returns_at_least_one` — floor guarantee
  - `test_ssg_run_rayon_oversized_concurrency_limit_clamps_to_cpu_count` — `concurrency_limit=256` still produces correct checksums
  - `test_ssg_run_rayon_small_concurrency_limit_respected_below_cpu_count` — `concurrency_limit=1` still produces correct checksums
  - `test_ssg_run_rayon_cpu_cap_equal_concurrency_limit_produces_correct_output` — boundary branch when limit == cpu_cap

---

## Why This Matters

The `bench_ssg.ruff` benchmark uses `batch_size = 256` (the default
`DEFAULT_ASYNC_TASK_POOL_SIZE`). Before this change, `ssg_run_rayon_read_render_write`
would call `ThreadPoolBuilder::new().num_threads(256)` on a machine with, say, 10
logical cores. Rayon would then create 256 worker threads for work-stealing execution
— all of which execute synchronous I/O and compute tasks. On an 10-core machine, 256
threads means:

- 246 threads sit idle waiting for work-stealing slots
- OS context-switches between 256 threads add latency to each I/O call
- Thread creation and teardown overhead for 246 extra threads (Rayon recreates the
  pool on each call to `ssg_run_rayon_read_render_write` via `spawn_blocking`)

After the cap, on a 10-core machine with `batch_size=256`, the pool will have 10
threads — exactly matching the available parallelism.

---

## Gotchas (Read This Next Time)

- **Gotcha:** `std::thread::available_parallelism()` returns `NonZeroUsize`, not
  plain `usize`. You need `.map(|n| n.get())` to extract the inner value, then
  `.unwrap_or(1)` for the fallback.
  - **Symptom:** If you write `available_parallelism().unwrap_or(1)`, the compiler
    will reject it because `NonZeroUsize` does not implement `From<usize>`.
  - **Fix:** Use `.map(|n| n.get()).unwrap_or(1)` — exactly as coded.

- **Gotcha:** `ThreadPoolBuilder` builds a new pool on every call to
  `ssg_run_rayon_read_render_write`. This is intentional — the function is called from
  `tokio::task::spawn_blocking`, which runs it in a blocking thread pool. The Rayon
  pool is scoped to the SSG batch operation only.
  - **Implication:** The CPU cap is re-evaluated on every call via `ssg_rayon_cpu_cap()`.
    This is a tiny `available_parallelism()` syscall — negligible overhead at SSG scale.

- **Gotcha:** The `parallel_map` path (`try_parallel_map_with_rayon_native_mapper`)
  at `src/interpreter/native_functions/async_ops.rs` line ~371 also calls
  `ThreadPoolBuilder::new().num_threads(concurrency_limit.max(1))` without the CPU
  cap. It was intentionally left out of scope for this session — it is a separate
  feature with different caller semantics. Do NOT apply the SSG-specific cap to
  `parallel_map` without careful analysis of that function's concurrency-limit contract.

- **Gotcha:** `concurrency_limit` values below CPU count must pass through unchanged.
  The formula `concurrency_limit.min(cpu_cap).max(1)` achieves this:
  - `concurrency_limit=1` on a 10-core machine → `1.min(10).max(1)` = 1 ✅
  - `concurrency_limit=4` on a 10-core machine → `4.min(10).max(1)` = 4 ✅
  - `concurrency_limit=256` on a 10-core machine → `256.min(10).max(1)` = 10 ✅
  - `concurrency_limit=0` (guarded upstream, but) → `0.min(10).max(1)` = 1 ✅

---

## Open Follow-Ups

- `[ ]` Run `cargo bench --bench bench-ssg` to measure wall-clock impact of CPU cap
  vs uncapped pool on local hardware (bench_ssg uses batch_size=256).
- `[ ]` Consider whether `parallel_map`'s `ThreadPoolBuilder` sizing warrants a
  similar cap — it's a different use case (user-controlled, not SSG-specific) and
  probably needs a separate ROADMAP item.
- `[ ]` Continue v0.11.0 P0 residual-overhead slices per ROADMAP.md.
