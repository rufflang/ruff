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
  - Blocker (2026-05-22): Revalidated this loop; `unsafe`/`SAFETY:` ratio remains unchanged, and a full annotation pass is still high-churn without automation-backed enforcement.
    Evidence: repeated `rg` counts during loop setup remained `51` unsafe markers and `3` `SAFETY:` comments.
  - Blocker (2026-05-22): Revalidated in loop 4; annotation scope is still broad (`src/jit.rs` executable boundaries span FFI functions plus inline unsafe blocks) and remains deferred behind a dedicated enforcement pass.
    Evidence: `rg -n "\bunsafe\b" src/jit.rs` still reports 51 markers concentrated in one file.
  - Blocker (2026-05-23): Revalidated before `V1H-SIZE-005`; unsafe boundary annotation gap remains materially unchanged and still too broad for a single scoped hardening loop.
    Evidence: `rg -n "\bunsafe\b" src/jit.rs | wc -l` -> `51`; `rg -n "SAFETY:" src/jit.rs | wc -l` -> `3`.
  - Blocker (2026-05-23): Revalidated before `V1H-SEC-001`; unsafe/SAFETY annotation ratio remains unchanged and still requires a dedicated invariant-enforcement pass.
    Evidence: `rg -n "\bunsafe\b" src/jit.rs | wc -l` -> `51`; `rg -n "SAFETY:" src/jit.rs | wc -l` -> `3`.
  - Blocker (2026-05-23): Revalidated before `V1H-SEC-002`; invariant annotation gap is unchanged and still requires a dedicated unsafe-boundary documentation loop.
    Evidence: `rg -n "\bunsafe\b" src/jit.rs | wc -l` -> `51`; `rg -n "SAFETY:" src/jit.rs | wc -l` -> `3`.
  - Blocker (2026-05-23): Revalidated before `V1H-SEC-004`; unsafe-boundary invariant coverage is still unchanged and remains scoped for a dedicated unsafe pass.
    Evidence: `rg -n "\bunsafe\b" src/jit.rs | wc -l` -> `51`; `rg -n "SAFETY:" src/jit.rs | wc -l` -> `3`.
  - Blocker (2026-05-23): Revalidated before `V1H-FEAT-004`; unsafe-boundary invariant coverage remains unchanged and still requires dedicated unsafe-loop closure.
    Evidence: `rg -n "\bunsafe\b" src/jit.rs | wc -l` -> `51`; `rg -n "SAFETY:" src/jit.rs | wc -l` -> `3`.

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
  - Blocker (2026-05-22): Revalidated after adding `scripts/unsafe_safety_gate.sh`; wrapper reduction still deferred until `V1H-UNSAFE-002` documentation sweep is completed to avoid moving unsafe callsites without updated boundary invariants.
    Evidence: `docs/generated/UNSAFE_INVENTORY.md` still reports `49` executable matches concentrated in `src/jit.rs`.
  - Blocker (2026-05-22): Revalidated in loop 4; callsite-reduction work remains coupled to unresolved unsafe-boundary invariant documentation.
    Evidence: generated inventory still reports `49` executable unsafe matches in JIT pathways.
  - Blocker (2026-05-23): Revalidated before `V1H-SIZE-005`; wrapper-reduction remains coupled to unresolved `V1H-UNSAFE-002` invariant documentation and is deferred to avoid unaudited unsafe-motion churn.
    Evidence: `docs/generated/UNSAFE_INVENTORY.md` still reports concentrated executable unsafe rows in `src/jit.rs`.
  - Blocker (2026-05-23): Revalidated before `V1H-SEC-001`; executable unsafe reduction remains blocked on unresolved unsafe-boundary invariant documentation sequencing.
    Evidence: `docs/generated/UNSAFE_INVENTORY.md` still reports concentrated executable unsafe rows in `src/jit.rs`.
  - Blocker (2026-05-23): Revalidated before `V1H-SEC-002`; executable unsafe reduction remains sequenced after `V1H-UNSAFE-002` invariant standardization.
    Evidence: `docs/generated/UNSAFE_INVENTORY.md` still reports concentrated executable unsafe rows in `src/jit.rs`.
  - Blocker (2026-05-23): Revalidated before `V1H-SEC-004`; executable unsafe reduction remains blocked on unresolved `V1H-UNSAFE-002` invariant standardization.
    Evidence: `docs/generated/UNSAFE_INVENTORY.md` still reports concentrated executable unsafe rows in `src/jit.rs`.
  - Blocker (2026-05-23): Revalidated before `V1H-FEAT-004`; executable unsafe reduction remains sequenced after unresolved invariant-standardization work.
    Evidence: `docs/generated/UNSAFE_INVENTORY.md` still reports concentrated executable unsafe rows in `src/jit.rs`.

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

- [x] **V1H-SIZE-001**: Establish reproducible binary size baseline matrix.
  - Scope: record size for `debug`, `release`, and stripped release artifacts; include host/target/toolchain metadata.
  - Acceptance criteria:
    - Dated artifact note in `notes/` with exact commands and byte sizes.
    - Repeatable measurement command block committed.
  - Validation:
    - `cargo build --release`
    - `ls -lh target/release/ruff` (plus stripped variant if used)
  - Evidence (2026-05-22):
    - Added reproducible measurement script `scripts/measure_binary_size.sh` with metadata output, dry-run support, and deterministic byte-count reporting for debug/release/stripped artifacts.
    - Added contract tests in `tests/binary_size_baseline_contract.rs` (help, dry-run emission, unknown-arg failure path).
    - Captured measured baseline evidence in `notes/2026-05-22_11-55_v1h-size-001-binary-size-baseline.md`:
      - debug `91597784` bytes
      - release `31006832` bytes
      - release_stripped `26557120` bytes

- [ ] **V1H-SIZE-002**: Add non-breaking feature gates for heavyweight optional subsystems.
  - Scope: make DB/image/archive/JIT-heavy stacks opt-out for smaller binaries while keeping current full behavior available.
  - Acceptance criteria:
    - Default behavior compatibility explicitly preserved, or migration note provided if default changes.
    - `--no-default-features`/targeted feature combinations build and test cleanly.
  - Validation:
    - feature-matrix `cargo check` / targeted tests
  - Blocker (2026-05-22): This requires a crate-feature matrix design decision (default-vs-optional subsystem partitioning for DB/image/archive/JIT stacks) that risks broad behavior/build-surface churn beyond a single scoped loop.
    Evidence: `Cargo.toml` currently declares heavyweight runtime dependencies directly in always-on `[dependencies]`, with no existing feature partition to extend incrementally.
  - Blocker (2026-05-23): Revalidated before `V1H-SIZE-005`; feature-gate partitioning remains a multi-surface design change and is still deferred to avoid broad default behavior/build-matrix churn in a single loop.
    Evidence: `Cargo.toml` still has heavyweight runtime subsystems in always-on `[dependencies]` with no existing incremental feature partition.
  - Blocker (2026-05-23): Revalidated before `V1H-SEC-001`; feature-partitioning remains a broader build-surface decision and is still deferred outside this single security-hardening loop.
    Evidence: `Cargo.toml` still has heavyweight runtime subsystems in always-on `[dependencies]` without an incremental feature matrix scaffold.
  - Blocker (2026-05-23): Revalidated before `V1H-SEC-002`; dependency feature-partitioning remains a multi-surface build-matrix change and is deferred outside this scoped URL-hardening loop.
    Evidence: `Cargo.toml` still has heavyweight runtime subsystems in always-on `[dependencies]` without an incremental feature matrix scaffold.
  - Blocker (2026-05-23): Revalidated before `V1H-SEC-004`; feature-gate partitioning remains a broader build-matrix decision outside this docs-only hardening loop.
    Evidence: `Cargo.toml` still has heavyweight runtime subsystems in always-on `[dependencies]` without an incremental feature matrix scaffold.
  - Blocker (2026-05-23): Revalidated before `V1H-FEAT-004`; feature-gate partitioning remains a multi-surface build-matrix change beyond this downstream-doc loop.
    Evidence: `Cargo.toml` still has heavyweight runtime subsystems in always-on `[dependencies]` without an incremental feature matrix scaffold.

- [x] **V1H-SIZE-003**: Consolidate duplicated runtime helpers shared by VM and interpreter.
  - Scope: extract shared HTTP path/query parsing and similar duplicated helpers into shared module(s).
  - Acceptance criteria:
    - Duplicate helper implementations removed from one-off runtime copies.
    - Parity tests prove no behavior drift.
  - Validation:
    - `cargo test --test vm_interpreter_parity_surfaces`
    - focused HTTP/runtime suites
  - Evidence (2026-05-22):
    - Added shared helper module `src/http_request_utils.rs` with lexical query parsing utilities and unit tests.
    - Removed duplicated `split_http_path_and_query` / `parse_http_query_params` implementations from `src/interpreter/mod.rs` and `src/vm.rs`, wiring both runtimes to `http_request_utils::split_http_path_and_query`.
    - Validation:
      - `cargo test split_http_path_and_query` (new helper tests passed in lib+main test binaries)
      - `cargo test vm_http_server_route_method_returns_updated_server`
      - `cargo test vm_http_handler_wrapper_executes_lambda_response_correctly`
      - `cargo test --test vm_interpreter_parity_surfaces` (86 passed)

- [x] **V1H-SIZE-004**: Audit `#[allow(dead_code)]` hotspots for removable production bloat.
  - Scope: classify dead-code allowances into keep/remove/feature-gate buckets.
  - Acceptance criteria:
    - inventory note with rationale per major hotspot (`src/builtins.rs`, `src/jit.rs`, `src/vm.rs`, etc.).
    - remove low-risk dead paths or gate them behind features where practical.
  - Evidence (2026-05-22):
    - Added hotspot inventory and classification note: `docs/generated/DEAD_CODE_ALLOW_HOTSPOT_AUDIT.md`.
    - Classified major hotspots (`src/builtins.rs`, `src/interpreter/value.rs`, `src/jit.rs`, `src/ast.rs`, `src/vm.rs`, `src/module.rs`) into keep/defer buckets with rationale, and identified `src/path_security.rs` as safe immediate reduction.
    - Applied low-risk removal/gating by converting `reject_url_encoded_parent_traversal` in `src/path_security.rs` from runtime `#[allow(dead_code)]` to `#[cfg(test)] pub(crate)` since it is test-only call-path code.
    - Validation:
      - `cargo test reject_url_encoded_parent_traversal`
      - `cargo test path_security`
      - `cargo test --test vm_interpreter_parity_surfaces`

- [x] **V1H-SIZE-005**: Add release profile tuning for smaller binaries.
  - Scope: evaluate `lto`, `codegen-units`, `panic=abort`, and strip strategy.
  - Acceptance criteria:
    - before/after size + runtime sanity evidence.
    - no functional regression in release smoke tests.
  - Evidence (2026-05-23):
    - Added additive release-profile tuning in `Cargo.toml`:
      - `[profile.release] lto = "thin"`
      - `[profile.release] codegen-units = 1`
      - `[profile.release] strip = "symbols"`
    - Before baseline (prior measured matrix from `notes/2026-05-22_11-55_v1h-size-001-binary-size-baseline.md`):
      - debug `91597784`
      - release `31006832`
      - release_stripped `26557120`
    - Post-change measurement:
      - `cargo build --release`
      - `wc -c target/debug/ruff target/release/ruff`
      - `cp target/release/ruff target/release/ruff.stripped && strip target/release/ruff.stripped && wc -c target/release/ruff.stripped`
      - debug `91593328`
      - release `24067240`
      - release_stripped-copy `24067320`
    - Runtime sanity/regression verification: `cargo test` (full suite passed; no failures).

---

## C) Security Hardening (SSRF/XSS/Runtime Safety)

- [x] **V1H-SEC-001**: Implement outbound destination policy layer for network client APIs.
  - Scope: classify and gate target addresses for HTTP/TCP client calls (loopback/private/link-local/multicast handling).
  - Acceptance criteria:
    - deterministic allow/deny rules documented.
    - explicit override switch or policy mode for trusted local workflows.
  - Validation:
    - new security tests for allowed public targets + denied local/private targets
    - `cargo test --test native_api_security_boundaries`
  - Completed (2026-05-23):
    - Added deterministic destination-policy enforcement in `src/network_policy.rs` with:
      - `RUFF_NET_DESTINATION_POLICY=allow_all|deny_private`
      - `RUFF_ALLOW_PRIVATE_NETWORK_DESTINATIONS=true` trusted override
    - Wired policy enforcement into outbound HTTP/TCP/UDP client surfaces across builtin and interpreter-native paths.
    - Added/updated tests for strict deny, public allow, and override behavior.
    - Validation:
      - `cargo test --test native_api_security_boundaries` (46 passed)
      - `cargo test --test runtime_security` (9 passed)
      - `cargo test` (full suite passed; no failures)

- [x] **V1H-SEC-002**: Enforce URL scheme/host validation for HTTP native calls.
  - Scope: reject unsupported schemes and malformed targets before request execution.
  - Acceptance criteria:
    - stable actionable error messages for invalid scheme/host.
    - no regressions for current valid `http`/`https` flows.
  - Validation:
    - focused HTTP native tests + parity checks
  - Completed (2026-05-23):
    - Added deterministic HTTP URL pre-validation in `src/network_policy.rs`:
      - allowlist: `http`, `https`
      - unsupported scheme rejection before request execution
      - malformed URL rejection with stable error prefix
    - Reused the shared pre-validation path across builtin and interpreter-native HTTP call surfaces via existing `enforce_http_url_destination_policy`.
    - Added/updated tests:
      - unit: unsupported scheme rejection (`network_policy` module tests)
      - integration: unsupported scheme + malformed URL rejection contracts in `tests/native_api_security_boundaries.rs`
    - Validation:
      - `cargo test network_policy::tests::outbound_policy_http_url_evaluation_rejects_unsupported_scheme` (passed)
      - `cargo test --test native_api_security_boundaries` (48 passed)
      - `cargo test --test runtime_security` (9 passed)
      - `cargo test` (blocked by unrelated guardrail: `tests/lsp_latency_guardrails.rs` exceeded diagnostics average latency threshold at ~156-158ms; no failures in touched network/security suites)

- [ ] **V1H-SEC-003**: Replace panic-prone `lock().unwrap()` in production native surfaces.
  - Scope: network/database/concurrency native functions return controlled `ErrorObject` on poisoned lock instead of panicking process-wide.
  - Acceptance criteria:
    - no `lock().unwrap()` remaining in production runtime paths.
    - poisoned lock behavior is deterministic and test-covered.
  - Validation:
    - focused native function tests
    - `cargo test --test native_api_security_boundaries`
  - Blocker (2026-05-23): Revalidated and deferred; current production native surfaces still contain many `lock().unwrap()` sites across network/database/concurrency paths and require a dedicated staged conversion plan to avoid broad runtime regression risk in a single loop.
    Evidence: `rg -n "lock\\(\\)\\.unwrap\\(\\)" src/interpreter/native_functions src/builtins.rs src/main.rs` reports extensive occurrences across `network.rs`, `database.rs`, `concurrency.rs`, and shared runtime surfaces.
  - Blocker (2026-05-23): Revalidated before `V1H-FEAT-004`; `lock().unwrap()` conversion scope remains broad and still requires a dedicated staged runtime-hardening loop with poison-lock contract tests.
    Evidence: `rg -n "lock\\(\\)\\.unwrap\\(\\)" src/interpreter/native_functions src/builtins.rs src/main.rs` still reports extensive occurrences across production runtime surfaces.

- [x] **V1H-SEC-004**: Add explicit threat-model documentation for script-generated HTML responses.
  - Scope: clarify that `html_response` can propagate unescaped content; provide safe usage guidance and helper recommendations.
  - Acceptance criteria:
    - docs include actionable XSS-safe patterns for Ruff HTTP handlers.
    - no behavior break to existing response helpers.
  - Completed (2026-05-23):
    - Added explicit `html_response` threat-model boundary docs, including XSS risk statement and safe usage patterns in `docs/NATIVE_API_SECURITY_POSTURE.md`.
    - Added a concrete script-level `escape_html` example for defensive output encoding.
    - Added README callout that `html_response` is raw output and requires caller-side escaping for untrusted content.
    - Validation:
      - `cargo test --test security_posture_docs_contract` (2 passed)
      - `cargo test --test readme_contracts` (1 passed)
      - `cargo test --test native_api_security_boundaries` (48 passed)
      - `cargo test --test runtime_security` (9 passed)

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
  - Blocker (2026-05-23): Revalidated and deferred; P0 parity burn-down remains a multi-fixture runtime track that requires a dedicated parity loop with fixture-by-fixture closure evidence.
    Evidence: `docs/generated/VM_RUNTIME_MISMATCH_INVENTORY.md` still reports `P0 runtime-parity-bug (runtime-owner): 25`.

- [ ] **V1H-FEAT-002**: Resolve P2 harness debt to improve signal quality.
  - Scope: normalize/refresh fixture expectations where both runtimes are correct but contract snapshots are stale/noisy.
  - Acceptance criteria:
    - `harness-debt` count reduced from current baseline (`16`).
    - contract rationale documented per touched fixture family.
  - Blocker (2026-05-23): Revalidated and deferred; harness-debt normalization remains a dedicated fixture-contract burn-down track outside this docs loop.
    Evidence: `docs/generated/VM_RUNTIME_MISMATCH_INVENTORY.md` still reports `P2 harness-debt (harness-owner): 16`.

- [ ] **V1H-FEAT-003**: Tighten type-checker ergonomics for high-impact TODOs.
  - Scope: targeted improvements from `src/type_checker.rs` medium-severity TODO cluster (module checks, struct field inference, generic collection inference).
  - Acceptance criteria:
    - at least one medium-severity TODO cluster closed with tests.
    - no parser/runtime behavior regression.
  - Blocker (2026-05-23): Revalidated and deferred; type-checker TODO clusters require dedicated semantic design and regression coverage outside this downstream-doc loop.
    Evidence: `rg -n "TODO|FIXME" src/type_checker.rs` still reports unresolved medium-scope TODO clusters (module checks, struct fields, generic collection inference, method inference).

- [x] **V1H-FEAT-004**: Remove stale `--interpreter` preference language from downstream-facing docs and examples.
  - Scope: ensure docs present VM-first guidance with explicit caveats tied to current parity state.
  - Acceptance criteria:
    - README/docs examples default to VM path unless feature explicitly requires interpreter fallback.
    - references to fallback include concrete limitation rationale.
  - Completed (2026-05-23):
    - Updated downstream-facing secure-usage examples in `docs/NATIVE_API_SECURITY_POSTURE.md` from `ruff run --interpreter ...` to VM-default `ruff run ...`.
    - Added explicit caveat that `--interpreter` remains an optional compatibility/debug isolation mode, not default workflow guidance.
    - Retained explicit fallback references only where they represent intentional compatibility/runtime-path rationale.
    - Validation:
      - `cargo test --test security_posture_docs_contract` (2 passed)
      - `cargo test --test readme_contracts` (1 passed)
      - `cargo test --test native_api_security_boundaries` (48 passed)
      - `cargo test --test runtime_security` (9 passed)

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
