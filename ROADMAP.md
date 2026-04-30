# Ruff Development Roadmap

This roadmap tracks work that is still current or upcoming. Completed features and implementation history belong in [CHANGELOG.md](CHANGELOG.md), not here.

> Current crate version: `0.11.0` in [Cargo.toml](Cargo.toml)
> Next planned release: `v0.12.0`
> Last audited: April 29, 2026

---

## Release Focus

`v0.11.0` is the completed throughput and release-hardening release.
Current roadmap focus shifts to `v0.12.0` developer-experience work.

Primary release theme:

- Native SSG read/render/write throughput through `bench-ssg`.
- VM cooperative scheduler reliability for high-volume async workloads.
- Benchmark measurement quality strong enough to make a release call.

Non-goals for `v0.11.0`:

- LSP, formatter, linter, package manager, and other developer tooling.
- Module-system completion beyond documenting current limitations.
- Broad language design changes unrelated to SSG throughput or release stability.
- Function-level JIT promotion unless it shows measurable `bench-ssg` improvement without correctness regressions.

Completed `v0.11.0` throughput slices are already recorded in [CHANGELOG.md](CHANGELOG.md), including Rayon SSG execution, cached metadata, reusable output-path buffers, opt-in stage profiling, throughput gates, benchmark warning signals, and scheduler timeout-budget support.

---

## v0.11.0 Release Checklist (Completed For Release Cut)

### P0: Release Blockers

These were used as the release-cut readiness checklist for `v0.11.0`.

1. **Capture final release-mode SSG gate evidence**

   Run the current code in release mode on an idle machine and record the output in the release notes.

   ```bash
   cargo build --release
   ./target/release/ruff bench-ssg --runs 7 --warmup-runs 2 --profile-async --throughput-gate-ms 10000 --tmp-dir tmp/ruff-v0.11-ssg-gate
   ```

   Required release decision:

   - PASS if Ruff median build time is `<= 10000 ms` and correctness checks are green.
   - FAIL or defer if Ruff median build time is above `10000 ms` unless an explicit release exception is made.
   - Preserve `RUFF_SSG_FILES`, `RUFF_SSG_BUILD_MS`, `RUFF_SSG_FILES_PER_SEC`, `RUFF_SSG_CHECKSUM`, `RUFF_SSG_READ_MS`, and `RUFF_SSG_RENDER_WRITE_MS` metric contracts.
   - Treat `read_ms` and `render_write_ms` as cumulative stage CPU-time signals from the Rayon path, not wall-clock phase durations.

   Latest local smoke evidence from this audit:

   - command: `cargo run --release -- bench-ssg --runs 3 --warmup-runs 1 --profile-async --throughput-gate-ms 10000`
   - result: PASS
   - Ruff median build time: `1114.421 ms`
   - Ruff median throughput: `8973.27 files/sec`
   - checksum: `946670`
   - stage medians: read `1119.445 ms`, render/write `11718.502 ms`
   - warning status: CV variability warnings emitted for build time, throughput, read stage, and render/write stage
   - release note: this is a useful smoke result, but the final gate should still use the longer idle-machine command above.

   Additional local smoke evidence from a busy-machine run:

   - date: `2026-04-28 22:06 EDT`
   - commit: `65cc08d`
   - command: `./target/release/ruff bench-ssg --runs 7 --warmup-runs 2 --profile-async --throughput-gate-ms 10000 --tmp-dir tmp/ruff-v0.11-ssg-gate`
   - result: PASS
   - Ruff median build time: `993.559 ms`
   - Ruff median throughput: `10064.83 files/sec`
   - checksum: `946670`
   - stage medians: read `1146.473 ms`, render/write `10471.032 ms`
   - warning status: trend drift, CV variability, mean/median drift, and range-spread warnings emitted across Ruff build/throughput and stage metrics
   - release note: the operator confirmed the machine was maxed out with other apps during this run, so this is local smoke evidence only and does not satisfy the final idle-machine release gate.

   Additional local release-mode evidence from this audit:

   - date: `2026-04-29 21:04 EDT`
   - commit: `15d1eaa`
   - command: `./target/release/ruff bench-ssg --runs 7 --warmup-runs 2 --profile-async --throughput-gate-ms 10000 --tmp-dir tmp/ruff-v0.11-ssg-gate`
   - result: PASS
   - Ruff median build time: `1804.098 ms`
   - Ruff median throughput: `5542.94 files/sec`
   - checksum: `946670`
   - stage medians: read `1929.052 ms`, render/write `18918.995 ms`
   - warning status: no benchmark warning sections emitted
   - host context: macOS `26.3.1` on Darwin `25.3.0`; load averages captured before the run were `3.95 4.71 3.75`
   - release note: this was a clean local release-mode PASS, but idle-machine status was not operator-confirmed, so it is local evidence only and does not satisfy the final idle-machine release gate.

   Additional local release-mode evidence from this audit:

   - date: `2026-04-29 21:25 EDT`
   - commit: `b70af59`
   - command: `./target/release/ruff bench-ssg --runs 7 --warmup-runs 2 --profile-async --throughput-gate-ms 10000 --tmp-dir tmp/ruff-v0.11-ssg-gate`
   - result: PASS
   - Ruff median build time: `1017.971 ms`
   - Ruff median throughput: `9823.46 files/sec`
   - checksum: `946670`
   - stage medians: read `1046.698 ms`, render/write `10095.369 ms`
   - warning status: CV variability warnings emitted for Ruff build time, throughput, read stage, and render/write stage; no trend-drift, mean/median drift, or range-spread warnings emitted
   - host context: macOS `26.3.1` on Darwin `25.3.0`; load averages captured before the run were `5.41 8.26 8.38`
   - release note: this run passes the configured gate but was captured on a loaded host and without explicit idle-machine confirmation, so it is local smoke evidence only and does not satisfy the final idle-machine release gate.

2. **Capture final cross-language context**

   Run the Python comparison on the same machine after the release-gate run. This is not the primary gate, but it gives release-note context and catches checksum drift against the Python baseline.

   ```bash
   ./target/release/ruff bench-ssg --runs 5 --warmup-runs 1 --compare-python --profile-async --tmp-dir tmp/ruff-v0.11-ssg-python
   ```

   Required release decision:

   - Ruff and Python checksums must match.
   - Record median Ruff build time, median Python build time, median speedup, and warning sections if emitted.
   - If warnings fire, decide whether the run is noisy and should be repeated or whether the warning reflects a real release risk.

   Latest local cross-language evidence from this audit:

   - date: `2026-04-29 21:52 EDT`
   - commit: `ed77acd`
   - command: `./target/release/ruff bench-ssg --runs 5 --warmup-runs 1 --compare-python --profile-async --tmp-dir tmp/ruff-v0.11-ssg-python`
   - result: PASS
   - Ruff median build time: `984.872 ms`
   - Python median build time: `3575.723 ms`
   - median speedup: `3.94x`
   - checksums: no mismatch detected (benchmark command completed successfully with Ruff checksum `946670`)
   - warning status: trend-drift warnings for Python build time/throughput and Ruff-vs-Python speedup; measurement-quality variability warnings for Ruff/Python metrics plus Ruff read-stage range spread
   - host context: load averages captured before this run were `9.89 7.86 6.69`
   - release note: this provides local cross-language context and checksum verification, but warning-heavy output on a loaded host should be treated as local smoke evidence.

3. **Run focused SSG and benchmark-harness regression tests**

   ```bash
   cargo test ssg
   cargo test bench_ssg
   cargo test run_ssg_benchmark
   ```

   Required release decision:

   - All focused tests pass.
   - Any environment-specific failure is documented with repro steps and a release decision.

   Latest local test evidence from this audit:

   - `cargo test ssg` => PASS (`163` tests passed in `src/lib.rs`, mirrored `163` pass in `src/main.rs`)
   - `cargo test bench_ssg` => PASS (filter matched no tests; command completed cleanly)
   - `cargo test run_ssg_benchmark` => PASS (`10` tests passed in `src/lib.rs`, mirrored `10` pass in `src/main.rs`)
   - release note: focused SSG and benchmark-harness regression commands are green on current local environment.

4. **Run native builtin dispatch and release-hardening coverage**

   ```bash
   cargo test release_hardening_builtin_dispatch_coverage
   cargo test test_release_hardening_ssg_render_pages_dispatch_contracts
   ```

   Required release decision:

   - `expected_known_legacy_dispatch_gaps` remains empty.
   - No declared builtin regresses to unknown-native fallback.
   - SSG native helper argument/error-shape contracts remain stable.

   Latest local dispatch-hardening evidence from this audit:

   - `cargo test release_hardening_builtin_dispatch_coverage` => PASS (`2` tests passed in `src/lib.rs`, mirrored `2` pass in `src/main.rs`)
   - `cargo test test_release_hardening_ssg_render_pages_dispatch_contracts` => PASS (`1` test passed in `src/lib.rs`, mirrored `1` pass in `src/main.rs`)
   - release note: declared builtin dispatch and SSG helper dispatch contracts are stable in current local validation.

5. **Cut release metadata**

   Required edits at release time:

   - Bump [Cargo.toml](Cargo.toml) from `0.10.0` to `0.11.0`.
   - Move relevant [CHANGELOG.md](CHANGELOG.md) `Unreleased` entries under a dated `v0.11.0` heading.
   - Update README status text if it still describes `0.11.0` as in progress.
   - Confirm `ruff --version` reports `0.11.0` after the version bump.

   Current release-exception stance:

   - Do not block `v0.11.0` solely on final idle-machine benchmark evidence when deterministic correctness and dispatch-hardening checks are green.
   - Keep existing local benchmark runs clearly labeled as smoke/local evidence in release notes.
   - Track one post-release follow-up capture on an operator-confirmed idle machine for canonical benchmark evidence.

### P1: Release Evidence And Documentation

These should be completed before release unless you explicitly decide to defer them.

1. **Publish one canonical SSG benchmark snapshot**

   Add or update a benchmark-results note with:

   - exact command
   - date
   - machine/OS summary
   - commit SHA
   - release/debug build mode
   - warmup count and measured run count
   - median/mean/p90/p95/min/max/stddev for Ruff build time and throughput
   - stage medians when `--profile-async` is enabled
   - warning sections emitted by the benchmark harness

2. **Calibrate benchmark warning thresholds**

   Current defaults are implemented as:

   - variability warning: `5.0%`
   - trend warning: `10.0%`
   - mean/median drift warning: `7.5%`
   - range-spread warning: `42.0%`

   Remaining release question:

   - Run enough repeated samples to decide whether these defaults are useful for local release-gate runs.
   - If they are too noisy or too quiet, adjust before release and update tests/docs.

3. **Clean stale performance claims outside the README**

   Some older docs still describe older execution-mode status or old benchmark claims. Before release, review at least:

   - [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
   - [docs/PERFORMANCE.md](docs/PERFORMANCE.md)
   - [benchmarks/cross-language/BENCHMARK_RESULTS.md](benchmarks/cross-language/BENCHMARK_RESULTS.md)
   - [benchmarks/cross-language/README.md](benchmarks/cross-language/README.md)

   Remaining release question:

   - Either update stale claims or mark the documents as historical so they do not contradict the `v0.11.0` README and release notes.

4. **Document current runtime boundaries**

   The README now calls out current limitations. Keep the same truth in release notes:

   - VM is default; interpreter remains a fallback.
   - Module loader syntax exists, but module execution/export collection is incomplete.
   - Struct methods, spread/destructuring, and richer `Result`/`Option` pattern behavior should be verified per runtime path.
   - Static typing is optional and not a VM-enforced release gate.

### P2: Optional Follow-Ups Before Release

These are useful, but should not block `v0.11.0` unless the P0/P1 evidence reveals a problem.

1. **Add CLI-output snapshot tests for `bench-ssg`**

   Candidate coverage:

   - warmup banner and measured-run summary
   - throughput gate PASS/FAIL text
   - trend warning section
   - measurement-quality warning section
   - `--profile-async` stage output presence/absence

2. **Run larger benchmark series for warning readability**

   Suggested command:

   ```bash
   ./target/release/ruff bench-ssg --runs 10 --warmup-runs 2 --profile-async --tmp-dir tmp/ruff-v0.11-warning-calibration
   ```

   Use this only for threshold/readability calibration, not as the primary release gate.

3. **Evaluate remaining SSG micro-optimizations only if gate fails**

   Candidate areas from field notes:

   - file pre-allocation in the sync write path
   - matching reusable output-path-buffer follow-through for `ssg_render_and_write_pages(...)`
   - bounded eviction for SSG metadata caches if non-benchmark workloads use many `file_count` values
   - profiling residual render/write overhead after reusable output-path-buffer work

4. **Consider command-level success-path integration tests**

   The benchmark harness has strong unit/error coverage. A real success-path CLI integration test would improve release confidence, but may be too expensive for default CI.

---

## v0.11.0 Done And No Longer Roadmap Work

The following areas were previously listed as future work but are now implemented and should be tracked through the changelog/tests instead of roadmap tasks:

- async runtime full-suite timing-flake stabilization for `interpreter::async_runtime::tests::test_concurrent_tasks` (migrated to relative timing assertions plus timeout-budget coverage).
- `bench-ssg --runs`, median/mean/min/max/stddev aggregation.
- `bench-ssg` p90/p95 percentile reporting.
- `bench-ssg --warmup-runs`.
- `bench-ssg --tmp-dir`.
- `bench-ssg --throughput-gate-ms`.
- variability, trend, mean/median drift, and range-spread warning signals.
- threshold override flags for benchmark warnings.
- opt-in `--profile-async` stage metrics.
- fused SSG native helpers: `ssg_render_and_write_pages(...)` and `ssg_read_render_and_write_pages(...)`.
- Rayon single-pass read/render/write pipeline.
- CPU-bounded and cached Rayon pools.
- sync vectored write path.
- byte-path source reads.
- in-iterator checksum/timing aggregation.
- cached render-prefix and output-suffix metadata.
- reusable output-path buffers in the hot SSG path.
- timeout-budget cooperative scheduler completion.
- `ruff run --scheduler-timeout-ms <MILLISECONDS>` CLI override support with deterministic precedence over `RUFF_SCHEDULER_TIMEOUT_MS` and default timeout.
- JSON, crypto, database, and network native dispatch coverage. These are implemented modules, not future stub-module work.

---

## v0.12.0: Developer Experience

`v0.12.0` should begin after the `v0.11.0` performance release is tagged.

Priority work:

1. **Language Server Protocol**

   Planned features:

   - autocomplete for builtins, variables, and functions
   - go to definition
   - find references
   - hover documentation
   - real-time diagnostics
   - rename refactoring
   - code actions

2. **Formatter**

   Planned features:

   - opinionated formatting
   - configurable indentation
   - line-length policy
   - import ordering once module semantics are stable

3. **Linter**

   Planned rules:

   - unused variables
   - unreachable code
   - obvious type mismatches
   - missing error-handling patterns
   - auto-fix for safe rules

4. **Package/project workflow**

   Planned features:

   - `ruff.toml`
   - dependency metadata
   - `ruff init`
   - package install/add/publish workflow

5. **REPL improvements**

   Planned features:

   - tab completion
   - syntax highlighting
   - stronger multi-line editing
   - `.help <function>` documentation

6. **Documentation generator**

   Planned features:

   - HTML docs from `///` comments
   - examples extracted from doc comments
   - builtin/native API reference generation

---

## v1.0.0 Readiness

`v1.0.0` should not be planned in detail until `v0.11.0` and `v0.12.0` are complete.

Required before `v1.0.0`:

- `v0.11.0` performance release complete.
- `v0.12.0` developer tooling substantially complete.
- Stable language/runtime API policy.
- Current, accurate user documentation.
- Clear compatibility policy for native builtins and CLI output contracts.

Possible post-`v0.12.0` design tracks:

- generic types
- union types
- enum methods
- macros/metaprogramming
- FFI
- WebAssembly target
- ML/AI libraries

---

## Version Strategy

- `v0.11.0`: SSG throughput, async scheduler reliability, benchmark release evidence.
- `v0.12.0`: developer experience and project tooling.
- `v1.0.0`: stabilization, documentation, compatibility policy, ecosystem polish.

See also:

- [CHANGELOG.md](CHANGELOG.md): completed changes.
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md): performance guide.
- [docs/CONCURRENCY.md](docs/CONCURRENCY.md): concurrency notes.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): architecture notes. Some sections may be stale and should be reviewed before release.
