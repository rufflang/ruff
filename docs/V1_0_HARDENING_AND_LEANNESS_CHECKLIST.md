# Ruff v1.0 Hardening And Leanness Checklist

Status: active deep-audit checklist (pre-release hardening/optimization pass)  
Created: 2026-05-22

Purpose: capture additive, non-breaking work that can improve safety, maintainability, VM reliability, and binary footprint before final release.

---

## Evidence Snapshot (2026-05-22)

- Unsafe inventory currently reports `53` total `unsafe` matches, `49` executable, concentrated in `src/jit.rs` (`docs/generated/UNSAFE_INVENTORY.md`).
- VM parity still has `25` `runtime-parity-bug` mismatches and `16` `harness-debt` mismatches (`docs/generated/VM_RUNTIME_MISMATCH_INVENTORY.md`).
- Large dependency surface is always compiled in current default build (`Cargo.toml`), including heavy stacks: `tokio`, `reqwest`, `mysql_async`, `postgres`, `rusqlite` (`bundled`), `image`, `zip`, and `cranelift*`.
- DRY duplication exists in runtime logic, for example HTTP route/query parsing duplicated in both `src/interpreter/mod.rs` and `src/vm.rs`.
- Outbound network is capability-gated, but there is no built-in destination class policy (private/loopback/link-local deny-by-default) in `src/network_policy.rs` for HTTP/TCP client calls.
- Runtime native code still contains many `lock().unwrap()` sites in production paths (not just tests), which can panic on poisoned locks:
  - `src/interpreter/native_functions/network.rs`
  - `src/interpreter/native_functions/database.rs`
  - `src/interpreter/native_functions/concurrency.rs`

---

## Guardrails

1. Additive and backward compatible only.
2. No syntax/behavior regression for existing valid Ruff programs.
3. Deterministic runtime behavior and diagnostics.
4. No expansion of unsafe boundaries without explicit invariant docs and tests.
5. Any size optimization that removes functionality must be opt-in via feature/build profile, not a default breaking change.

---

## A) Unsafe Boundary Hardening

- [x] **V1H-UNSAFE-001**: Reconcile unsafe truth set and retire stale audit narratives.
  - Scope: align `docs/UNSAFE_CODE_AUDIT.md` with generated inventory contracts and current code reality.
  - Acceptance criteria:
    - No conflicting unsafe counts/docs across audit artifacts.
    - Clear executable vs non-executable classification reference.
  - Validation:
    - `bash scripts/generate_unsafe_inventory.sh`
    - `cargo test --test unsafe_inventory_contract`
  - Evidence (2026-05-22):
    - Rewrote `docs/UNSAFE_CODE_AUDIT.md` to align with generated source-of-truth artifacts (`docs/generated/UNSAFE_INVENTORY.md` and `.csv`) and removed stale static count narratives.
    - Captured loop evidence in `notes/2026-05-22_11-25_v1h-unsafe-001-unsafe-truth-set-reconciliation.md`.
    - Validation: `bash scripts/generate_unsafe_inventory.sh`, `cargo test --test unsafe_inventory_contract` (2 passed), and `cargo test --test vm_interpreter_parity_surfaces` (86 passed).

- [ ] **V1H-UNSAFE-002**: Add/standardize `SAFETY:` invariant comments at JIT FFI/pointer boundaries.
  - Scope: every executable unsafe boundary in `src/jit.rs` has concise precondition/postcondition notes.
  - Acceptance criteria:
    - No undocumented executable unsafe blocks/functions in `src/jit.rs`.
    - Invariants reference ownership/lifetime/ABI assumptions.
  - Validation:
    - focused JIT tests + `cargo test --test vm_interpreter_parity_surfaces`
  - Blocker (2026-05-22): Current `src/jit.rs` has 51 `unsafe` markers and only 3 existing `SAFETY:` comments; a one-loop manual annotation sweep would be high-churn and error-prone without a preparatory enforcement/checker pass.
    Evidence: `rg -n "\bunsafe\b" src/jit.rs | wc -l` -> `51`; `rg -n "SAFETY:" src/jit.rs` -> `3`.

- [ ] **V1H-UNSAFE-003**: Reduce executable unsafe callsites via safe wrappers where behavior is unchanged.
  - Scope: trim ad hoc unsafe deref/transmute callsites without broad rewrites.
  - Acceptance criteria:
    - Executable unsafe count reduced or centralized with equivalent behavior.
    - Regression coverage added for touched JIT/VM call paths.
  - Validation:
    - `bash scripts/generate_unsafe_inventory.sh`
    - focused JIT tests + `cargo test --test vm_interpreter_parity_surfaces`
  - Blocker (2026-05-22): Wrapper-reduction pass is blocked this loop pending an explicit safety-gate/checker workflow so callsite reductions can be validated deterministically as invariants move.
    Evidence: Unsafe boundary concentration remains high (`docs/generated/UNSAFE_INVENTORY.md`: 49 executable matches), and no optional nightly sanitizer/Miri gate existed before this loop.

- [x] **V1H-UNSAFE-004**: Add optional sanitizer/Miri-oriented safety gate for CI/nightly verification.
  - Scope: machine-verifiable unsafe regression signal beyond unit tests.
  - Acceptance criteria:
    - Repeatable command/script documented under `scripts/` or CI config.
    - Failure mode documented for triage.
  - Evidence (2026-05-22):
    - Added `scripts/unsafe_safety_gate.sh` with deterministic base gate commands plus optional `--with-miri` probe and explicit failure-mode exits (`2` for bad args, `3` for missing nightly/Miri prerequisites).
    - Added contract coverage in `tests/unsafe_safety_gate_contract.rs` for help output, dry-run command emission (including Miri probe), and unknown-argument failure path.
    - Validation: `cargo test --test unsafe_safety_gate_contract` (3 passed) and `bash scripts/unsafe_safety_gate.sh` (unsafe inventory generation + `unsafe_inventory_contract` + `vm_interpreter_parity_surfaces` all passed).

---

## B) DRY/Modularity And Binary Size

- [ ] **V1H-SIZE-001**: Establish reproducible binary size baseline matrix.
  - Scope: record size for `debug`, `release`, and stripped release artifacts; include host/target/toolchain metadata.
  - Acceptance criteria:
    - Dated artifact note in `notes/` with exact commands and byte sizes.
    - Repeatable measurement command block committed.
  - Validation:
    - `cargo build --release`
    - `ls -lh target/release/ruff` (plus stripped variant if used)

- [ ] **V1H-SIZE-002**: Add non-breaking feature gates for heavyweight optional subsystems.
  - Scope: make DB/image/archive/JIT-heavy stacks opt-out for smaller binaries while keeping current full behavior available.
  - Acceptance criteria:
    - Default behavior compatibility explicitly preserved, or migration note provided if default changes.
    - `--no-default-features`/targeted feature combinations build and test cleanly.
  - Validation:
    - feature-matrix `cargo check` / targeted tests

- [ ] **V1H-SIZE-003**: Consolidate duplicated runtime helpers shared by VM and interpreter.
  - Scope: extract shared HTTP path/query parsing and similar duplicated helpers into shared module(s).
  - Acceptance criteria:
    - Duplicate helper implementations removed from one-off runtime copies.
    - Parity tests prove no behavior drift.
  - Validation:
    - `cargo test --test vm_interpreter_parity_surfaces`
    - focused HTTP/runtime suites

- [ ] **V1H-SIZE-004**: Audit `#[allow(dead_code)]` hotspots for removable production bloat.
  - Scope: classify dead-code allowances into keep/remove/feature-gate buckets.
  - Acceptance criteria:
    - inventory note with rationale per major hotspot (`src/builtins.rs`, `src/jit.rs`, `src/vm.rs`, etc.).
    - remove low-risk dead paths or gate them behind features where practical.

- [ ] **V1H-SIZE-005**: Add release profile tuning for smaller binaries.
  - Scope: evaluate `lto`, `codegen-units`, `panic=abort`, and strip strategy.
  - Acceptance criteria:
    - before/after size + runtime sanity evidence.
    - no functional regression in release smoke tests.

---

## C) Security Hardening (SSRF/XSS/Runtime Safety)

- [ ] **V1H-SEC-001**: Implement outbound destination policy layer for network client APIs.
  - Scope: classify and gate target addresses for HTTP/TCP client calls (loopback/private/link-local/multicast handling).
  - Acceptance criteria:
    - deterministic allow/deny rules documented.
    - explicit override switch or policy mode for trusted local workflows.
  - Validation:
    - new security tests for allowed public targets + denied local/private targets
    - `cargo test --test native_api_security_boundaries`

- [ ] **V1H-SEC-002**: Enforce URL scheme/host validation for HTTP native calls.
  - Scope: reject unsupported schemes and malformed targets before request execution.
  - Acceptance criteria:
    - stable actionable error messages for invalid scheme/host.
    - no regressions for current valid `http`/`https` flows.
  - Validation:
    - focused HTTP native tests + parity checks

- [ ] **V1H-SEC-003**: Replace panic-prone `lock().unwrap()` in production native surfaces.
  - Scope: network/database/concurrency native functions return controlled `ErrorObject` on poisoned lock instead of panicking process-wide.
  - Acceptance criteria:
    - no `lock().unwrap()` remaining in production runtime paths.
    - poisoned lock behavior is deterministic and test-covered.
  - Validation:
    - focused native function tests
    - `cargo test --test native_api_security_boundaries`

- [ ] **V1H-SEC-004**: Add explicit threat-model documentation for script-generated HTML responses.
  - Scope: clarify that `html_response` can propagate unescaped content; provide safe usage guidance and helper recommendations.
  - Acceptance criteria:
    - docs include actionable XSS-safe patterns for Ruff HTTP handlers.
    - no behavior break to existing response helpers.

---

## D) Reliability And Feature Enhancements

- [ ] **V1H-FEAT-001**: Burn down P0 VM parity mismatches from current baseline.
  - Scope: close `runtime-parity-bug` cases in `docs/generated/VM_RUNTIME_MISMATCH_INVENTORY.md` (current: `25`).
  - Acceptance criteria:
    - monotonic reduction across loops with updated generated evidence.
    - no regression in already-passing fixtures.
  - Validation:
    - `cargo run -- test --runtime vm`
    - `cargo run -- test --runtime dual`
    - `cargo test --test vm_interpreter_parity_surfaces`

- [ ] **V1H-FEAT-002**: Resolve P2 harness debt to improve signal quality.
  - Scope: normalize/refresh fixture expectations where both runtimes are correct but contract snapshots are stale/noisy.
  - Acceptance criteria:
    - `harness-debt` count reduced from current baseline (`16`).
    - contract rationale documented per touched fixture family.

- [ ] **V1H-FEAT-003**: Tighten type-checker ergonomics for high-impact TODOs.
  - Scope: targeted improvements from `src/type_checker.rs` medium-severity TODO cluster (module checks, struct field inference, generic collection inference).
  - Acceptance criteria:
    - at least one medium-severity TODO cluster closed with tests.
    - no parser/runtime behavior regression.

- [ ] **V1H-FEAT-004**: Remove stale `--interpreter` preference language from downstream-facing docs and examples.
  - Scope: ensure docs present VM-first guidance with explicit caveats tied to current parity state.
  - Acceptance criteria:
    - README/docs examples default to VM path unless feature explicitly requires interpreter fallback.
    - references to fallback include concrete limitation rationale.

---

## Suggested Execution Order

1. `V1H-SEC-001` and `V1H-SEC-003` (highest hardening impact)
2. `V1H-FEAT-001` (VM parity P0 burn-down)
3. `V1H-SIZE-001` then `V1H-SIZE-005` (measure, then tune)
4. `V1H-SIZE-002`/`V1H-SIZE-003` (structural leanness)
5. Remaining unsafe + docs follow-through

---

## Definition Of Done

- Security hardening items produce deterministic tests and docs.
- VM parity P0 count reduced to an agreed threshold (or zero) with generated evidence.
- Binary-size work includes before/after measurements and reproducible commands.
- Unsafe inventory remains machine-verifiable and trend-improving.
- All checklist closures include commit-linked evidence and command outputs.
