# V1.0 Universal Usefulness Expansion Checklist

Last updated: 2026-05-26  
Owner: Ruff core maintainers  
Status: Active (post-hardening expansion track)

## Goal

Use this checklist to drive Ruff from "pre-v1 hardened" to "production-polished and universally useful" with additive, backward-compatible work only.

This list is intentionally execution-oriented so another agent can take one item at a time, implement it, test it, check it off, and move to the next.

## Evidence Snapshot (Why This Exists)

- TODO debt still present in production tree:
  - `docs/generated/V1_CODE_TODO_TRIAGE.md` reports `30` markers, including medium-priority `src/type_checker.rs` clusters.
- Unsafe surface remains concentrated:
  - `docs/generated/UNSAFE_INVENTORY.md` reports `Executable matches: 55`, mostly JIT boundaries in `src/jit.rs`.
- Large complexity hotspots remain:
  - `src/interpreter/legacy_full.rs` has been removed from the active tree to eliminate mirror-runtime debt.
  - `src/jit.rs` (`9,288` LOC), `src/vm.rs` (`9,202` LOC), `src/interpreter/mod.rs` (`6,018` LOC).
- Dependency footprint remains broad in default build:
  - `Cargo.toml` compiles heavy stacks by default (`tokio`, `reqwest`, `mysql_async`, `postgres`, `rusqlite`, `image`, `zip`, `cranelift*`) without optional dependency partitioning.
- Root hygiene policy exists but local artifact creep is still easy:
  - `docs/REPO_HYGIENE_POLICY.md` + `tests/repo_hygiene_contract.rs` protect tracked root files, not all local root clutter workflows.

## Non-Negotiables

1. Additive and backward-compatible unless explicitly approved.
2. No runtime, parser, VM, interpreter, or diagnostics regressions.
3. Keep VM-first posture (`ruff run`) and avoid reintroducing interpreter dependency.
4. Every item closure requires:
   - code changes,
   - tests (success + failure + regression),
   - docs updates,
   - command evidence in `notes/`.
5. Release/tag/publication tasks remain frozen unless explicitly unblocked.

## Execution Model (for agents)

1. Pick exactly one unchecked item.
2. Create a short todo list before edits.
3. Add tests first where practical.
4. Implement minimal scoped change.
5. Run required tests for that item.
6. Update checklist row with dated evidence bullets.
7. Commit with item ID in message (example: `v1x(V1X-SEC-002): ...`).

## Priority Key

- `P0`: pre-v1 quality blocker for universal usefulness.
- `P1`: high-value launch polish.
- `P2`: post-v1 leverage improvements.

## Checklist

### P0 — Must Complete Before Calling Ruff "Enterprise-Ready"

- [x] **V1X-SEC-001 (P0)**: Replace panic-prone lock handling in production native/runtime paths.
  - Scope:
    - Eliminate `lock().unwrap()` on user-reachable runtime paths (network/process/db/concurrency/native collections) in:
      - `src/interpreter/mod.rs`
      - `src/interpreter/native_functions/*.rs`
      - `src/builtins.rs` (reachable paths only)
  - Acceptance:
    - Poisoned lock paths return deterministic Ruff errors (not host panic).
    - New regression tests cover poisoned/failed lock scenarios for representative APIs.
  - Minimum tests:
    - `cargo test --test runtime_security`
    - `cargo test --test native_api_security_boundaries`
    - focused suites for touched native modules
    - `cargo test --test vm_interpreter_parity_surfaces`
  - 2026-05-26 closure evidence:
    - Replaced panic-prone mutex locking in user-reachable async promise/task flows (`src/interpreter/native_functions/async_ops.rs`) with deterministic error propagation via `lock_or_async_error(...)`.
    - Removed remaining panic-prone seeded RNG lock sites in `src/builtins.rs` by moving to poison-recovery guard helper (`lock_seeded_rng`).
    - Hardened interpreter await/image/channel and cleanup paths (`src/interpreter/mod.rs`) to avoid `lock().unwrap()` panics on poisoned runtime state.
    - Added lock-poison regression tests:
      - `read_cached_promise_result_returns_error_when_lock_poisoned`
      - `cache_promise_result_returns_error_when_lock_poisoned`
    - Command evidence captured in `notes/2026-05-26_07-43_v1x-sec-001-lock-hardening.md`.

- [x] **V1X-SEC-002 (P0)**: Harden process execution surfaces to minimize shell-injection blast radius.
  - Scope:
    - Audit `Command::new(... "sh" "-c" ...)` and `cmd /C` usage in:
      - `src/builtins.rs`
      - `src/interpreter/native_functions/system.rs`
    - Ensure risky shell-string execution is either:
      - capability-gated with explicit docs/warnings, or
      - replaced by structured argv APIs where possible.
  - Acceptance:
    - Clear deterministic error/guardrail behavior for untrusted mode.
    - Tests verify blocked/disallowed paths and safe structured execution.
  - Minimum tests:
    - `cargo test --test native_api_security_boundaries`
    - focused system/process native function tests
    - `cargo test --test cli_contracts`
  - 2026-05-26 closure evidence:
    - Added deterministic shell command input validation for `execute()` / `execute_status()` in `src/interpreter/native_functions/system.rs`:
      - reject empty command text,
      - reject embedded NUL bytes,
      - reject newline/carriage-return command payloads with explicit guidance to use `spawn_process([...])`.
    - Applied aligned command-text validation in legacy builtin shell helper `execute_command(...)` (`src/builtins.rs`) to avoid unconstrained shell payload execution on direct helper use.
    - Added regression boundary tests:
      - `process_execute_rejects_empty_shell_command`
      - `process_execute_status_rejects_newline_shell_command`
      in `tests/native_api_security_boundaries.rs`.
    - Command evidence captured in `notes/2026-05-26_08-01_v1x-sec-002-shell-surface-hardening.md`.

- [x] **V1X-DRY-001 (P0)**: Resolve `src/interpreter/legacy_full.rs` duplication risk.
  - Scope:
    - Decide one path:
      - remove/archive it with policy evidence, or
      - integrate/maintain it with explicit ownership + synchronization tests.
    - Eliminate ambiguous "shadow runtime" state.
  - Acceptance:
    - No orphan runtime mirror file without enforcement plan.
    - Docs and TODO triage reflect final decision.
  - Minimum tests:
    - `cargo test --test v1_code_todo_triage_contract`
    - `cargo test --test vm_interpreter_parity_surfaces`
    - `cargo test`
  - 2026-05-26 closure evidence:
    - Removed `src/interpreter/legacy_full.rs` from the active source tree to eliminate ambiguous shadow-runtime maintenance risk.
    - Updated TODO triage generation logic (`scripts/generate_v1_code_todo_triage.sh`) to remove stale `legacy_full` special-case handling.
    - Regenerated triage artifacts:
      - `docs/generated/V1_CODE_TODO_TRIAGE.md`
      - `docs/generated/V1_CODE_TODO_TRIAGE.csv`
      now reporting `30` classified markers with `0` unclassified.
    - Verified required suites:
      - `cargo test --test v1_code_todo_triage_contract`
      - `cargo test --test vm_interpreter_parity_surfaces`
      - `cargo test` (full workspace pass).
    - Command evidence captured in `notes/2026-05-26_09-02_v1x-dry-001-legacy-full-removal.md`.

- [ ] **V1X-SIZE-001 (P0)**: Convert heavy runtime dependencies to true optional features.
  - Scope:
    - Update `Cargo.toml` so large surfaces (`db`, `image`, `archive`, possibly `jit`) use optional dependencies and feature wiring.
    - Keep default behavior compatible (or document exact additive profile plan).
  - Acceptance:
    - Reproducible binary-size table before/after.
    - No feature/profile regression in default build.
  - Minimum tests:
    - `cargo build --release`
    - `cargo test --test cli_contracts`
    - `cargo test --test vm_interpreter_parity_surfaces`
    - `cargo test` (if feature matrix is changed broadly)

- [ ] **V1X-VM-001 (P0)**: Add external-project VM smoke gates (no interpreter fallback).
  - Scope:
    - Add reproducible VM smoke workflow for real modular projects (for example Ruff Eval style command surfaces).
    - Ensure imported function/module call paths are validated on VM by default.
  - Acceptance:
    - CI/local gate catches regression class: imported symbol appears callable but fails at call site.
    - Explicit pass/fail evidence for VM path.
  - Minimum tests:
    - `cargo test --test vm_interpreter_parity_surfaces`
    - new integration test(s) for imported call execution in VM
    - `cargo run -- test --runtime vm`

### P1 — High-Value Launch Polish

- [ ] **V1X-TYPE-001 (P1)**: Close next medium-priority type checker TODO cluster.
  - Scope:
    - `src/type_checker.rs` TODOs around:
      - destructuring inference,
      - module existence checks,
      - struct field type lookup,
      - Promise unwrap typing.
  - Acceptance:
    - At least one full medium cluster closed with regression tests.
    - `docs/OPTIONAL_TYPING_DESIGN.md` updated for changed behavior/boundaries.
  - Minimum tests:
    - `cargo test type_checker::tests::`
    - `cargo test --test v1_code_todo_triage_contract`

- [ ] **V1X-DRY-002 (P1)**: Reduce dead-code/`#[allow(dead_code)]` sprawl on production paths.
  - Scope:
    - Audit `#[allow(dead_code)]` in:
      - `src/builtins.rs`
      - `src/vm.rs`
      - `src/interpreter/value.rs`
      - `src/jit.rs`
    - Remove or gate unused surfaces; keep only justified suppressions with comments.
  - Acceptance:
    - Fewer suppression points with rationale.
    - No behavior regressions from cleanup.
  - Minimum tests:
    - `cargo clippy --all-targets --all-features -- -D warnings`
    - focused touched suites
    - `cargo test --test vm_interpreter_parity_surfaces`

- [ ] **V1X-TEST-001 (P1)**: Strengthen generated-artifact freshness contracts.
  - Scope:
    - Ensure key generated docs (`docs/generated/*`) are not stale relative to code/checklist claims.
    - Add explicit recency/content contract checks for mismatch, unsafe, and TODO triage artifacts.
  - Acceptance:
    - Contracts fail on stale or contradictory generated artifacts.
  - Minimum tests:
    - `cargo test --test vm_runtime_mismatch_inventory_contract`
    - `cargo test --test unsafe_inventory_contract`
    - `cargo test --test v1_code_todo_triage_contract`

- [ ] **V1X-SEC-003 (P1)**: Security negative-path expansion for HTTP/network/process.
  - Scope:
    - Add hostile-input tests for:
      - malformed URLs/hosts/ports,
      - boundary edge cases for destination policy,
      - process env allow/deny corner cases.
  - Acceptance:
    - New failure-path coverage proves deterministic guarded errors.
  - Minimum tests:
    - `cargo test --test native_api_security_boundaries`
    - `cargo test --test runtime_security`

- [ ] **V1X-PERF-001 (P1)**: Expand import-heavy and startup performance guardrails.
  - Scope:
    - Add/extend reproducible startup/import-heavy perf measurements.
    - Track and enforce tolerance in generated perf artifacts/contracts.
  - Acceptance:
    - Before/after evidence with explicit tolerance interpretation.
    - Contract tests lock expected artifact schema and PASS/FAIL status.
  - Minimum tests:
    - `cargo bench --bench v1_perf_benchmarks`
    - `cargo test --test vm_import_heavy_perf_comparison_contract`
    - `cargo test --test vm_import_heavy_cache_lookup_contract`

- [ ] **V1X-REPO-001 (P1)**: Tighten root-local artifact hygiene beyond tracked files.
  - Scope:
    - Extend hygiene policy and tooling for local clutter classes (`*.db`, ad-hoc backup zips, extracted temp dirs).
    - Keep developer workflows practical (allow local temp under designated dirs).
  - Acceptance:
    - Clear policy and automated check for disallowed root clutter patterns.
    - No conflict with normal dev/test flows.
  - Minimum tests:
    - `cargo test --test repo_hygiene_contract`
    - any new hygiene contract tests

### P2 — Post-v1 Leverage Improvements

- [ ] **V1X-DOC-001 (P2)**: Final architecture narrative refresh for forward-facing clarity.
  - Scope:
    - Ensure `README.md`, `docs/ARCHITECTURE.md`, `docs/LANGUAGE_SPEC.md`, and runtime-parity docs present one consistent "how Ruff works" model.
  - Acceptance:
    - No stale architecture claims.
    - Contract tests updated/passing.
  - Minimum tests:
    - `cargo test --test readme_contracts`
    - `cargo test --test architecture_docs_contract`
    - `cargo test --test docs_policy_consistency_contract`

- [ ] **V1X-FEAT-001 (P2)**: Package/module DX improvements for large codebases.
  - Scope:
    - Expand module/package workflow docs and tests for nested project layouts, lockfiles, and deterministic imports.
  - Acceptance:
    - Additional end-to-end examples pass on VM default path.
  - Minimum tests:
    - `cargo test --test package_module_workflow_integration`
    - `cargo test --test vm_interpreter_parity_surfaces`

- [ ] **V1X-FEAT-002 (P2)**: Improve diagnostics UX for high-friction runtime failures.
  - Scope:
    - Add actionable remediation hints for common user blockers (module resolution, capability denials, callable/type confusion).
  - Acceptance:
    - Golden diagnostics updated with deterministic IDs/messages.
  - Minimum tests:
    - `cargo test --test diagnostics_golden`
    - `cargo test --test cli_json_contracts`

## Suggested Execution Order

1. `V1X-SEC-001`
2. `V1X-SEC-002`
3. `V1X-DRY-001`
4. `V1X-SIZE-001`
5. `V1X-VM-001`
6. `V1X-TYPE-001`
7. `V1X-DRY-002`
8. `V1X-TEST-001`
9. `V1X-SEC-003`
10. `V1X-PERF-001`
11. `V1X-REPO-001`
12. `V1X-DOC-001`
13. `V1X-FEAT-001`
14. `V1X-FEAT-002`

## Definition of Done for This Checklist

- All `P0` items are completed with evidence.
- Any deferred `P1`/`P2` items have explicit owner + rationale + timeline.
- No unresolved regression in:
  - `cargo test`
  - `cargo run -- test --runtime vm`
  - `cargo run -- test --runtime dual`
  - security and docs contract suites touched by completed items.
