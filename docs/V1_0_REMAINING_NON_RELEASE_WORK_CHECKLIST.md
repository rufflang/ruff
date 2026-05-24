# V1.0 Remaining Non-Release Work Checklist

Date: 2026-05-24
Owner: runtime/compiler/docs maintainers

## Scope

This checklist is the **post-hardening, non-release-flight** remainder for v1.0 readiness.

Explicitly excluded from this document:
- `V1U-OPEN-003` (release artifact tag-time sign-off)
- `V1U-FINAL-003` (final tag-time artifact completion)

Those two release-flight items remain governed by `docs/PRE_V1_MASTER_UNFINISHED_CHECKLIST.md` and require explicit `UNBLOCK_V1_RELEASE` to execute.

## Current Snapshot (2026-05-24)

- `docs/V1_0_HARDENING_AND_LEANNESS_CHECKLIST.md`: `0` unchecked.
- `docs/V1_0_TECH_READINESS_CHECKLIST.md`: `0` unchecked.
- `docs/VM_NO_INTERPRETER_UNIVERSALIZATION_CHECKLIST.md`: `1` unchecked (`V1VM-PAR-004`).
- `docs/PRE_V1_MASTER_UNFINISHED_CHECKLIST.md`: `2` unchecked, both release-flight (excluded above).

Non-release unchecked total: **1** primary blocker item (`V1VM-PAR-004`) plus supporting quality-gate follow-through below.

## A) Remaining Blockers / Stoppers

- [ ] **RNR-PAR-001**: Close `V1VM-PAR-004` by reducing non-intentional VM parity mismatches to intentional-divergence-only.
  - Why open: `docs/generated/VM_RUNTIME_MISMATCH_INVENTORY.md` still reports `runtime-parity-bug` fixtures.
  - Current mismatch set (16 fixtures):
    - `tests/env_and_args.ruff`
    - `tests/image_processing_test.ruff`
    - `tests/integer_types.ruff`
    - `tests/result_option.ruff`
    - `tests/simple_image_test.ruff`
    - `tests/stdlib_os_path_test.ruff`
    - `tests/stdlib_test.ruff`
    - `tests/test_assertions.ruff`
    - `tests/test_connection_pooling.ruff`
    - `tests/test_enhanced_collections.ruff`
    - `tests/test_func_loop_correct.ruff`
    - `tests/test_function_drop_fix.ruff`
    - `tests/test_generators.ruff`
    - `tests/test_loop_correct.ruff`
    - `tests/test_method_chaining.ruff`
    - `tests/test_unary_overload.ruff`
  - Exit criteria:
    - `runtime-parity-bug` bucket reaches zero **or** residual divergences are explicitly reclassified as intentional with rationale.
    - `V1VM-PAR-004` can be marked complete with dated evidence.

- [ ] **RNR-PAR-002**: Refresh universalization readiness verdict after parity closure.
  - Why open: `V1VM-FINAL-001` currently records `NO-GO` pending `V1VM-PAR-004` burn-down.
  - Exit criteria:
    - Update `docs/VM_NO_INTERPRETER_UNIVERSALIZATION_CHECKLIST.md` final-readiness evidence with current counts and explicit GO/NO-GO.

## B) In-Progress / Partial / Needs-Follow-Through

- [x] **RNR-DOC-001**: Resolve docs snippet contract failure in `docs/NATIVE_API_SECURITY_POSTURE.md` code block #1.
  - Current evidence (2026-05-24):
    - `cargo test --test docs_examples` fails at `docs_ruff_snippets_parse_or_expected_fail`.
    - Error: parse failure in `docs/NATIVE_API_SECURITY_POSTURE.md#1` (`Expected expression`).
  - Exit criteria:
    - Snippet updated to valid Ruff syntax (or intentionally expected-fail with reason, if policy allows).
    - `cargo test --test docs_examples` passes.
  - Completed (2026-05-24):
    - Updated snippet syntax in `docs/NATIVE_API_SECURITY_POSTURE.md` from legacy `fn`/`=` style to current Ruff `func`/`:=` style.
    - Validation: `cargo test --test docs_examples` (`5 passed, 0 failed`).

- [ ] **RNR-EVID-001**: Re-sync blocker evidence text in `V1VM-PAR-004` notes to current inventory values.
  - Why open: blocker note history still contains earlier larger counts; latest inventory is materially lower.
  - Exit criteria:
    - Add fresh dated blocker evidence line aligned to latest generated artifact.

## C) Structured Parity Burn-Down Plan (Recommended Workload Slices)

Use these as execution loops to close `RNR-PAR-001` predictably:

- [ ] **RNR-PAR-G1**: Method/operator semantics cluster
  - Targets: `dict_methods_test`, `spread_operator`, `test_method_chaining`, `test_unary_overload`.
  - Focus: method dispatch consistency, operator overload parity, chained-call evaluation order.
  - Progress (2026-05-24):
    - Closed VM parity for `dict_methods_test` by adding `FixedDict` support in collection natives (`merge`, `clear`, `remove`).
    - Closed VM parity for `spread_operator` by preserving source-order dict insertion in VM `MakeDict`/`MakeDictFromMarker` (later keys now override deterministically).
    - Remaining in this cluster: `test_method_chaining` (VM compile lacks `??`/`?.`/`|>` support), `test_unary_overload` (VM unary struct overload parity gap).

- [ ] **RNR-PAR-G2**: Loop/function/generator control-flow cluster
  - Targets: `test_loop_correct`, `test_func_loop_correct`, `test_function_drop_fix`, `test_generators`, `integer_types`.
  - Focus: frame/stack lifecycle, loop progression semantics, generator-next behavior consistency.

- [ ] **RNR-PAR-G3**: Stdlib/env/IO behavior cluster
  - Targets: `env_and_args`, `stdlib_test`, `stdlib_os_path_test`, `test_connection_pooling`, `image_processing_test`, `simple_image_test`.
  - Focus: deterministic native interop results, environment handling, stable output normalization.

- [ ] **RNR-PAR-G4**: Collections/assertion/result semantics cluster
  - Targets: `result_option`, `test_enhanced_collections`, `test_assertions`, `dict_methods_test` (if residual overlap).
  - Focus: container mutation/view parity, Result/Option behavior, assertion output consistency.

## D) Verification Matrix For Remaining Work

For each parity loop:
- `cargo test --test vm_interpreter_parity_surfaces`
- `cargo run -- test --runtime vm`
- `cargo run -- test --runtime dual`
- Regenerate inventory artifacts:
  - `bash scripts/generate_vm_runtime_mismatch_inventory.sh`
- Contract checks:
  - `cargo test --test vm_runtime_mismatch_inventory_contract`
  - `cargo test --test vm_runtime_mismatch_baseline_contract`

For docs-fix loop:
- `cargo test --test docs_examples`
- related docs contract tests touched by the change.

## E) Definition Of Done (Non-Release)

This checklist is done when all are true:
- `RNR-PAR-001` complete (`V1VM-PAR-004` closed with evidence).
- `RNR-PAR-002` complete (readiness verdict refreshed from current artifact state).
- `RNR-DOC-001` complete (`docs_examples` passing).
- `RNR-EVID-001` complete (fresh dated blocker evidence synced).
- No new regressions in required parity/doc contract suites.
