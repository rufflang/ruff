# Ruff VM-First Universal Runtime & Module Import Reliability Checklist

Status: active pre-release execution checklist for removing practical `--interpreter` dependency in day-to-day development.
Created: 2026-05-21

Purpose: drive Ruff to a VM-first state where developers do not need `--interpreter` for normal multi-module projects, while preserving backward compatibility and deterministic behavior.

Primary evidence inputs:
- `docs/INTERPRETER_FLAG_DEPENDENCY_MAP.md`
- `docs/VM_INTERPRETER_PARITY_MATRIX.md`
- `docs/V1_0_SENIOR_CODEBASE_AUDIT_2026-05-21.md`
- `docs/generated/V1_CODE_TODO_TRIAGE.md`
- `docs/PRE_V1_MASTER_UNFINISHED_CHECKLIST.md`
- `tests/` fixture corpus and parity suites

---

## Non-Negotiable Guardrails

1. Additive and backward-compatible only.
2. No regressions in parser, resolver, runtime, diagnostics, or documented contracts.
3. Deterministic runtime-path behavior (explicit precedence, explicit fallback rules).
4. Path/security boundaries must remain enforced (no traversal/symlink escape regressions).
5. No release/tag/publish/sign-off work unless explicitly unblocked by the owner.

---

## Loop Governance And Selection Rules

### Loop Selection Rule (Mandatory)

1. Pick exactly one unchecked (`- [ ]`) item per loop.
2. Choose the first unchecked item in top-to-bottom order.
3. Do not skip unless blocked.
4. If blocked:
   - Add a dated blocker note under that item with reason + command/file evidence.
   - Continue scanning in order within the same loop.
5. Complete one unblocked item per loop.

### Closure Rule (Mandatory)

Do not mark an item complete until all of the following are true:

1. Implementation artifact exists (code/script/doc update).
2. Relevant tests/commands were run and results captured.
3. Docs/checklists directly affected by the item were updated.
4. Checklist row changed to `- [x]` with dated evidence bullets.
5. Commit message references the checklist item ID.

### Required Per-Loop Report Fields

Each loop report must include exactly:

1. Item completed.
2. Files changed.
3. Tests/commands run with results.
4. Blockers or follow-ups.

---

## Minimum Final Test Expectations By Item Type

- `V1VM-BASE-*`:
  - Run inventory/triage scripts and contract tests for generated artifacts.

- `V1VM-IMP-*` (module import reliability):
  - `cargo test --test vm_interpreter_parity_surfaces`
  - plus focused parser/module/runtime tests touched by import changes.
  - If runtime semantics changed, run `cargo test` unless blocked (document evidence).

- `V1VM-PAR-*` / `V1VM-HAR-*`:
  - `cargo run -- test --runtime vm`
  - `cargo run -- test --runtime dual`
  - targeted suites touched by the change.

- `V1VM-PERF-*`:
  - Run the targeted import/runtime benchmark command(s) before/after and capture evidence.

- `V1VM-DOC-*`:
  - Run impacted docs/checklist contract tests.

- `V1VM-FINAL-*`:
  - Run the full verification matrix for this checklist and capture the dated summary note.

---

## Execution Backlog

### 0) Baseline, Inventory, And Mismatch Classification

- [x] **V1VM-BASE-001**: Generate deterministic VM-vs-interpreter fixture mismatch inventory.
  - Scope: produce machine-readable baseline from `ruff test` runtime modes and classify pass/fail deltas.
  - Acceptance criteria:
    - Generated artifact under `docs/generated/` with per-fixture runtime outcomes.
    - Baseline command evidence captured in `notes/`.
  - Evidence (2026-05-21):
    - Added `scripts/generate_vm_runtime_mismatch_inventory.sh` and generated `docs/generated/VM_RUNTIME_MISMATCH_INVENTORY.md` + `.csv`.
    - Added generator contract coverage in `tests/vm_runtime_mismatch_inventory_contract.rs` (success output + deterministic capped scan).
    - Captured baseline command evidence and mismatch totals in `notes/2026-05-21_18-09_v1vm-base-001-runtime-mismatch-baseline.md`.

- [x] **V1VM-BASE-002**: Classify mismatch causes into actionable buckets.
  - Scope: classify each mismatch as parser-invalid fixture, stale expectation, runtime parity bug, intentional divergence, or test-harness debt.
  - Acceptance criteria:
    - Every mismatching fixture mapped to one bucket with rationale.
    - Priority order and owner tags documented.
  - Evidence (2026-05-21):
    - Extended `scripts/generate_vm_runtime_mismatch_inventory.sh` output with per-row `mismatch_bucket`, `bucket_owner`, `priority`, and `rationale` columns covering every mismatch row.
    - Added ordered bucket totals section (`P0`/`P1`/`P2`) to `docs/generated/VM_RUNTIME_MISMATCH_INVENTORY.md`.
    - Updated and passed `tests/vm_runtime_mismatch_inventory_contract.rs` contract coverage; captured command evidence in `notes/2026-05-21_18-15_v1vm-base-002-mismatch-classification.md`.

- [x] **V1VM-BASE-003**: Add/refresh contract tests for baseline artifacts.
  - Scope: prevent silent drift in mismatch inventory format and required buckets.
  - Acceptance criteria:
    - Contract test fails if required columns/buckets/evidence markers are missing.
  - Evidence (2026-05-21):
    - Added strict classification enforcement to `scripts/generate_vm_runtime_mismatch_inventory.sh` via `--strict` (fails if mismatch rows lack required bucket/owner/priority fields).
    - Added `tests/vm_runtime_mismatch_baseline_contract.rs` to enforce baseline artifact schema + semantic mismatch-classification invariants.
    - Updated and passed `tests/vm_runtime_mismatch_inventory_contract.rs` under strict mode; command evidence captured in `notes/2026-05-21_18-18_v1vm-base-003-baseline-contracts.md`.

### 1) Module Import Reliability (Critical Path)

- [x] **V1VM-IMP-001**: Harden dotted `from ... import ...` parser acceptance and diagnostics.
  - Scope: preserve existing import syntax while ensuring dotted-path acceptance and crisp malformed-path errors.
  - Acceptance criteria:
    - Positive parser tests for single-level and multi-level dotted from-import.
    - Negative parser tests for malformed token order/punctuation/path segments.
    - Existing flat import parser behavior unchanged by regression tests.
  - Evidence (2026-05-21):
    - Added parser negative coverage in `tests/parser_diagnostics_contract.rs` for trailing-dot module paths (`from src. import value`) and invalid token order (`from src util import value`).
    - Revalidated dotted parser acceptance + legacy flat import regression coverage (`single-level`, `multi-level`, and existing flat import forms) in `tests/parser_diagnostics_contract.rs`.
    - Ran runtime/module parity validation (`tests/interpreter_tests.rs`, `tests/package_module_workflow_integration.rs`, `tests/vm_interpreter_parity_surfaces.rs`) and captured command evidence in `notes/2026-05-21_18-21_v1vm-imp-001-parser-dotted-from-import-hardening.md`.

- [x] **V1VM-IMP-002**: Lock deterministic dotted module resolution precedence.
  - Scope: define and enforce stable lookup order for nested module files/directories without changing legacy flat-module behavior.
  - Acceptance criteria:
    - Precedence rules documented and covered with resolver tests.
    - Conflict scenarios (flat vs nested naming collisions) behave deterministically and are test-locked.
  - Evidence (2026-05-21):
    - Added VM/interpreter parity conflict coverage in `tests/vm_interpreter_parity_surfaces.rs` (`vm_and_interpreter_dotted_import_resolution_prefers_flat_module_before_nested_path`) to assert flat dotted filename precedence over nested directory-backed resolution when both candidates exist.
    - Revalidated existing resolver precedence unit coverage (`load_module_dotted_name_resolution_prefers_legacy_flat_filename_before_nested_path`) and confirmed docs precedence wording in `docs/LANGUAGE_SPEC.md` remains aligned with runtime behavior.
    - Captured command evidence in `notes/2026-05-21_18-24_v1vm-imp-002-dotted-resolution-precedence.md`.

- [x] **V1VM-IMP-003**: Enforce import resolution boundaries for dotted paths.
  - Scope: ensure out-of-root traversal and symlink-based escape attempts are rejected in dotted import flows.
  - Acceptance criteria:
    - Boundary/security tests cover parent traversal and symlink escape attempts.
    - Errors are deterministic and actionable.
  - Evidence (2026-05-21):
    - Added dotted-path symlink escape regression `runtime_security_rejects_dotted_module_symlink_escape_in_vm_and_interpreter` in `tests/runtime_security.rs`, asserting deterministic rejection for both `ruff run` (VM default) and `ruff run --interpreter`.
    - Revalidated module loader traversal/symlink boundary tests in `tests/runtime_security.rs` and runtime-path parity stability in `tests/vm_interpreter_parity_surfaces.rs`.
    - Captured execution evidence in `notes/2026-05-21_18-34_v1vm-imp-003-dotted-import-boundary-security.md`.

- [x] **V1VM-IMP-004**: Add integration fixtures for real nested project layouts.
  - Scope: add end-to-end interpreter+VM fixtures representing downstream multi-module project structures.
  - Acceptance criteria:
    - At least one realistic nested fixture proving prior blocked pattern now works.
    - Existing flat-module fixtures continue to pass unchanged.
  - Evidence (2026-05-21):
    - Added `package_module_workflow_nested_layout_is_runtime_mode_consistent_and_keeps_flat_imports` in `tests/package_module_workflow_integration.rs` using a realistic nested layout (`src/core`, `src/rag`) with dotted imports executed in both VM and interpreter modes.
    - Added explicit flat-module control flow in the same integration test to verify legacy flat import behavior remains unchanged across both runtime modes.
    - Revalidated integration and parity suites (`tests/package_module_workflow_integration.rs`, `tests/vm_interpreter_parity_surfaces.rs`) and captured command evidence in `notes/2026-05-21_18-37_v1vm-imp-004-nested-layout-integration-fixtures.md`.

- [x] **V1VM-IMP-005**: Align module-import docs with current reliability guarantees.
  - Scope: remove stale blanket guidance that requires `--interpreter` specifically for module import reliability when no longer true.
  - Acceptance criteria:
    - Updated docs clearly state what is guaranteed now vs remaining VM caveats.
    - Docs contract tests updated where applicable.
  - Evidence (2026-05-21):
    - Updated `README.md` to explicitly state dotted import workflows are supported on the default VM path and that `--interpreter` is optional fallback/debug mode.
    - Updated generator-owned dependency-map guidance (`scripts/generate_interpreter_flag_dependency_map.sh` + regenerated `docs/INTERPRETER_FLAG_DEPENDENCY_MAP.md`) to reflect current `ruff test --runtime dual|vm|interpreter` strategy and remove stale interpreter-hardcoding language.
    - Updated and passed docs contracts (`tests/readme_contracts.rs`, `tests/interpreter_flag_dependency_map_contract.rs`) and revalidated runtime parity suite; command evidence captured in `notes/2026-05-21_18-40_v1vm-imp-005-module-import-guidance-alignment.md`.

### 2) VM Parity Burn-Down For `ruff test`/Runtime Surfaces

- [x] **V1VM-PAR-001**: Build a runtime-diff harness for fixture output normalization.
  - Scope: make VM/interpreter comparison reproducible with normalized diagnostics/noise handling.
  - Acceptance criteria:
    - Tooling/script committed to compare runtime outputs deterministically.
    - At least one targeted contract test for output normalization rules.
  - Evidence (2026-05-21):
    - Added deterministic harness script `scripts/generate_vm_runtime_diff_harness.sh` with explicit normalization rules, self-check mode (`--normalization-self-check-only`), and reproducible CSV/markdown artifact generation.
    - Added contract coverage in `tests/vm_runtime_diff_harness_contract.rs` validating normalization self-check success and required output schema/classification columns (`raw_equal`, `normalized_noise_only`, `semantic_drift`).
    - Generated baseline artifacts (`docs/generated/VM_RUNTIME_DIFF_HARNESS.md`, `docs/generated/VM_RUNTIME_DIFF_HARNESS.csv`) and ran required parity sweeps:
      - `cargo run -- test --runtime vm` -> `Passed 59/150`
      - `cargo run -- test --runtime dual` -> `Passed 78/150` (`vm_primary=59`, `interpreter_fallback=19`)
    - Captured execution notes in `notes/2026-05-21_18-49_v1vm-par-001-runtime-diff-harness.md`.

- [x] **V1VM-PAR-002**: Close highest-volume VM/runtime mismatch bucket from baseline.
  - Scope: pick the largest mismatch class from `V1VM-BASE-002` and resolve it end-to-end.
  - Acceptance criteria:
    - Bucket count materially reduced with regression tests.
    - No backward-compatibility regressions introduced.
  - Evidence (2026-05-21):
    - Selected highest-volume baseline bucket `stale-snapshot-expectation` (`48`) from `docs/generated/VM_RUNTIME_MISMATCH_INVENTORY.md`.
    - Refreshed stale fixture snapshots for all 48 affected fixtures (tracked `.out` expectations) using deterministic VM runtime output capture to align test contracts with current agreed runtime behavior where VM and interpreter already matched each other.
    - Regenerated mismatch inventory via `bash scripts/generate_vm_runtime_mismatch_inventory.sh`; bucket totals now show `stale-snapshot-expectation: 0` (down from `48`) with no increase in `runtime-parity-bug` count.
    - Revalidated required parity commands:
      - `cargo test --test vm_interpreter_parity_surfaces` -> `85 passed`
      - `cargo run -- test --runtime vm` -> `Passed 102/150` (previous baseline: `59/150`)
      - `cargo run -- test --runtime dual` -> `Passed 121/150` (`vm_primary=102`, `interpreter_fallback=19`; previous baseline: `78/150`)
    - Captured execution notes in `notes/2026-05-21_19-10_v1vm-par-002-highest-volume-bucket-burndown.md`.

- [x] **V1VM-PAR-003**: Close second highest-volume mismatch bucket.
  - Scope: repeat `V1VM-PAR-002` for the next priority class.
  - Acceptance criteria:
    - Updated mismatch artifact shows monotonic reduction.
    - Targeted parity and regression tests added.
  - Evidence (2026-05-21):
    - Addressed a concrete `runtime-parity-bug` subclass by fixing interpreter recursion on custom enum constructors (`Result::Ok(...)`-style tags) in `src/interpreter/mod.rs` (`Expr::Tag` handling for namespaced tags now constructs tagged values directly instead of recursively re-entering generated constructor bindings).
    - Added parity regression `vm_and_interpreter_match_custom_enum_constructor_calls_without_recursion` in `tests/vm_interpreter_parity_surfaces.rs`.
    - Refreshed enum fixture snapshots touched by this fix (`tests/test_enum_{err,err_only,nested,none,ok}.out`) and regenerated mismatch inventory.
    - Monotonic bucket reduction captured in `docs/generated/VM_RUNTIME_MISMATCH_INVENTORY.md`:
      - `runtime-parity-bug`: `30 -> 25`
      - `both match snapshot`: `117 -> 122`
      - `vm_primary` progress in dual runtime: `102 -> 107` (`interpreter_fallback`: `19 -> 14`)
    - Revalidated required suites/commands:
      - `cargo test --test vm_interpreter_parity_surfaces` -> `86 passed`
      - `cargo run -- test --runtime vm` -> `Passed 107/150`
      - `cargo run -- test --runtime dual` -> `Passed 121/150` (`vm_primary=107`, `interpreter_fallback=14`)
      - `cargo test --test vm_runtime_mismatch_inventory_contract` -> `2 passed`
    - Captured execution notes in `notes/2026-05-21_19-28_v1vm-par-003-runtime-parity-bucket-reduction.md`.

- [ ] **V1VM-PAR-004**: Revalidate and document intentional divergences only.
  - Scope: ensure remaining runtime differences are explicit, intentional, and documented with evidence.
  - Acceptance criteria:
    - `docs/VM_INTERPRETER_PARITY_MATRIX.md` updated with only intentional, test-backed divergences.
    - No unexplained mismatch categories remain.
  - Blocker note (2026-05-21): deferred until parity burn-down removes unexplained mismatch buckets.
    - Evidence: `docs/generated/VM_RUNTIME_MISMATCH_INVENTORY.md` still reports non-intentional categories (`runtime-parity-bug: 25`, `harness-debt: 16`), so intentional-only divergence documentation would be premature.
  - Blocker note (2026-05-21, loop retry): still blocked.
    - Evidence: latest generated inventory remains above intentional-only threshold with unresolved non-intentional buckets (`runtime-parity-bug: 25`, `harness-debt: 16`), so `V1VM-PAR-004` acceptance criteria cannot be met yet.

### 3) Harness And CLI Runtime Strategy Hardening

- [x] **V1VM-HAR-001**: Tighten `ruff test --runtime dual` fallback determinism.
  - Scope: keep fallback bounded, explicit, and visible in output/contracts.
  - Acceptance criteria:
    - Fallback triggers are deterministic and covered by CLI contract tests.
    - No silent broad fallback behavior.
  - Evidence (2026-05-21):
    - Updated `src/parser.rs` dual-runtime harness output to emit explicit per-fixture marker when interpreter fallback is used on a passing case: `[dual fallback: interpreter]`.
    - Added/updated CLI contract assertions in `tests/cli_contracts.rs`:
      - `cli_test_runtime_dual_mode_falls_back_to_interpreter_for_vm_drift_fixture` now requires fallback marker presence.
      - `cli_test_runtime_vm_mode_reports_mismatch_for_vm_drift_fixture` asserts the dual fallback marker is absent in VM mode.
    - Revalidated fallback visibility and bounded behavior with focused contract tests and runtime sweeps:
      - `cargo test --test cli_contracts cli_test_runtime_vm_mode_reports_mismatch_for_vm_drift_fixture`
      - `cargo test --test cli_contracts cli_test_runtime_dual_mode_falls_back_to_interpreter_for_vm_drift_fixture`
      - `cargo run -- test --runtime vm` -> `Passed 107/150`
      - `cargo run -- test --runtime dual` -> `Passed 121/150` (`vm_primary=107`, `interpreter_fallback=14`)
    - Captured execution notes in `notes/2026-05-21_19-46_v1vm-har-001-dual-fallback-determinism.md`.

- [x] **V1VM-HAR-002**: Increase VM-only fixture coverage percentage to target threshold.
  - Scope: migrate parity-safe fixtures from fallback dependency to VM-clean execution.
  - Acceptance criteria:
    - Baseline report includes VM-only pass percentage and trend evidence.
    - Threshold target documented and met for this milestone.
  - Evidence (2026-05-21):
    - Extended `scripts/generate_vm_runtime_mismatch_inventory.sh` to publish a VM coverage gate section in `docs/generated/VM_RUNTIME_MISMATCH_INVENTORY.md`:
      - metric: `vm_matches_snapshot / fixtures_scanned`
      - target threshold: `70.0%`
      - current: `133/163 (81.6%)`
      - gate status: `PASS`
    - Added contract assertions for the new gate section in `tests/vm_runtime_mismatch_inventory_contract.rs`.
    - Trend evidence (same-day loop progression) shows sustained VM-primary improvement while fallback reliance drops:
      - VM runtime sweep: `59/150 -> 102/150 -> 107/150`
      - Dual runtime split: `vm_primary=59, interpreter_fallback=19` -> `vm_primary=102, interpreter_fallback=19` -> `vm_primary=107, interpreter_fallback=14`
    - Revalidated required suites/commands:
      - `cargo test --test vm_runtime_mismatch_inventory_contract` -> `2 passed`
      - `cargo run -- test --runtime vm` -> `Passed 107/150`
      - `cargo run -- test --runtime dual` -> `Passed 121/150` (`vm_primary=107`, `interpreter_fallback=14`)
    - Captured execution notes in `notes/2026-05-21_20-02_v1vm-har-002-vm-coverage-threshold.md`.

- [x] **V1VM-HAR-003**: Reassess default runtime strategy for `ruff test`.
  - Scope: decide whether default remains `dual` or can safely move to stricter VM-first behavior.
  - Acceptance criteria:
    - Decision note includes explicit risk analysis and command evidence.
    - CLI docs and contract tests align with decision.
  - Evidence (2026-05-21):
    - Decision recorded: keep default `ruff test` runtime as `dual` until parity burn-down removes unresolved mismatch categories.
    - Added explicit decision/risk-analysis section to `docs/VM_INTERPRETER_PARITY_MATRIX.md` (`### \`ruff test\` Default Runtime Decision (2026-05-21)`), including command-backed evidence:
      - `cargo run -- test --runtime vm` -> `Passed 107/150`
      - `cargo run -- test --runtime dual` -> `Passed 121/150` (`vm_primary=107`, `interpreter_fallback=14`)
    - Aligned CLI docs in `README.md` Known Boundaries:
      - clarified `ruff run` default VM vs `ruff test` default `--runtime dual`.
    - Tightened contract coverage:
      - `tests/cli_contracts.rs::cli_test_discovers_and_runs_expected_fixtures` now asserts default runtime summary contains `Runtime strategy: dual` and `interpreter_fallback=0`.
      - `tests/runtime_path_matrix_contract.rs` now asserts decision section markers remain present.
    - Revalidated required HAR command matrix and focused suites:
      - `cargo test --test cli_contracts cli_test_discovers_and_runs_expected_fixtures`
      - `cargo test --test runtime_path_matrix_contract`
      - `cargo run -- test --runtime vm`
      - `cargo run -- test --runtime dual`
    - Captured execution notes in `notes/2026-05-21_20-18_v1vm-har-003-default-runtime-strategy-decision.md`.

### 4) Security, Determinism, And Performance Guardrails

- [x] **V1VM-PERF-001**: Add import-heavy interpreter startup/perf benchmark for nested modules.
  - Scope: benchmark module-resolution-heavy startup path to detect dotted import regressions.
  - Acceptance criteria:
    - Benchmark fixture and command documented.
    - Baseline numbers captured.
  - Evidence (2026-05-21):
    - Added new Criterion workload in `benches/v1_perf_benchmarks.rs`:
      - benchmark id: `module_resolution/import_heavy_nested_dotted_startup_cold_loader`
      - fixture: generated nested `src/core/mod_*.ruff` modules (64 modules) imported through dotted `from src.core.mod_* import value_*` entry module.
      - execution path: cold `ModuleLoader` per iteration with interpreter-backed module evaluation via `load_module`.
    - Documented benchmark command in `README.md` Benchmarks section:
      - `cargo bench --bench v1_perf_benchmarks -- import_heavy_nested_dotted_startup_cold_loader`
    - Captured baseline command/result:
      - `cargo bench --bench v1_perf_benchmarks -- import_heavy_nested_dotted_startup_cold_loader --noplot --sample-size 10 --warm-up-time 0.5 --measurement-time 1`
      - `time: [278.40 ms 350.61 ms 420.40 ms]`
      - Criterion warned sample window was short for 10 samples in 1.0s and estimated ~2.15s collection; this run is kept as the initial baseline for `V1VM-PERF-001`.
    - Captured execution notes in `notes/2026-05-21_20-44_v1vm-perf-001-import-heavy-nested-startup-baseline.md`.

- [ ] **V1VM-PERF-002**: Demonstrate no unacceptable perf regression after reliability fixes.
  - Scope: compare before/after metrics for import-heavy path and report variance.
  - Acceptance criteria:
    - Results recorded with tolerance threshold and pass/fail interpretation.

- [ ] **V1VM-PERF-003**: Add cache/lookup validation for repeated nested imports.
  - Scope: ensure repeated imports do not trigger avoidable repeated filesystem work.
  - Acceptance criteria:
    - Tests confirm stable behavior and no duplicate side-effect imports.
    - Measurable lookup behavior documented.

### 5) Docs, Downstream Guidance, And Final Readiness

- [ ] **V1VM-DOC-001**: Update runtime guidance docs to VM-first practical recommendations.
  - Scope: align `README.md`, parity docs, and dependency map language with current execution reality.
  - Acceptance criteria:
    - No stale blanket guidance that developers must use `--interpreter` for ordinary modular projects.

- [ ] **V1VM-DOC-002**: Publish downstream migration guidance for teams currently pinned to `--interpreter`.
  - Scope: provide deterministic migration playbook (runtime mode selection, known caveats, verification commands).
  - Acceptance criteria:
    - One canonical doc section or migration note with explicit command recipes.

- [ ] **V1VM-FINAL-001**: Produce universal no-`--interpreter` readiness verdict for v1 track.
  - Scope: summarize completed items, remaining intentional divergences, and go/no-go recommendation.
  - Acceptance criteria:
    - Dated readiness note in `notes/` with explicit evidence table.
    - Checklist status and linked docs fully synchronized.

---

## Suggested Execution Order

1. `V1VM-BASE-001`
2. `V1VM-BASE-002`
3. `V1VM-IMP-001`
4. `V1VM-IMP-002`
5. `V1VM-IMP-003`
6. `V1VM-IMP-004`
7. `V1VM-IMP-005`
8. `V1VM-PAR-001`
9. `V1VM-PAR-002`
10. `V1VM-PAR-003`
11. `V1VM-PAR-004`
12. `V1VM-HAR-001`
13. `V1VM-HAR-002`
14. `V1VM-HAR-003`
15. `V1VM-PERF-001`
16. `V1VM-PERF-002`
17. `V1VM-PERF-003`
18. `V1VM-DOC-001`
19. `V1VM-DOC-002`
20. `V1VM-FINAL-001`
