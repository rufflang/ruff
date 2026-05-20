# Interpreter Flag Dependency Map

- Generated: 2026-05-20 14:01:13 EDT
- Command: `rg -n -- "--interpreter" src tests docs README.md ROADMAP.md examples notes .github`

Reason tags:
- `harness-legacy`: Existing harness behavior still forces interpreter mode.
- `security-test-choice`: Security-boundary regression intentionally exercises interpreter path.
- `diagnostics-diff`: Diagnostic contract coverage currently pins interpreter output shape.
- `docs-smoke`: Docs/example smoke harness runs interpreter as canonical execution path.
- `package-workflow`: Package/module workflow integration still validated via interpreter runs.
- `docs-contract`: User-facing docs explicitly describe interpreter mode behavior.
- `benchmark-baseline`: Example/benchmark docs keep interpreter as baseline comparator.
- `archive-note`: Historical field notes mentioning interpreter usage.

| File | Category | Reason Tags | Usage Count | Line References |
| --- | --- | --- | --- | --- |
| `README.md` | documentation | `docs-contract` | 2 | 11,121 |
| `ROADMAP.md` | documentation | `docs-contract` | 1 | 1334 |
| `docs/IMAGE_CONVERSION_AGENT_HANDOFF.md` | documentation | `docs-contract` | 1 | 52 |
| `docs/NATIVE_API_SECURITY_POSTURE.md` | documentation | `docs-contract` | 3 | 193,199,211 |
| `docs/PERFORMANCE.md` | documentation | `docs-contract` | 3 | 46,495,500 |
| `docs/PRE_V1_MASTER_UNFINISHED_CHECKLIST.md` | documentation | `docs-contract` | 2 | 191,197 |
| `docs/RUFF_FEATURE_INVENTORY.md` | documentation | `docs-contract` | 2 | 28,34 |
| `examples/benchmarks/README_REAL_WORLD.md` | example-doc | `benchmark-baseline` | 1 | 150 |
| `notes/2026-01-27_20-54_phase5-tokio-async-runtime.md` | notes-history | `archive-note` | 2 | 78,83 |
| `notes/2026-04-29_17-02_image-method-dispatch-parity.md` | notes-history | `archive-note` | 1 | 63 |
| `notes/2026-05-06_10-09_cli-serve-command-holistic-preview.md` | notes-history | `archive-note` | 3 | 24,42,45 |
| `notes/2026-05-13_01-13_v1-mod-001-module-import-hardening.md` | notes-history | `archive-note` | 1 | 24 |
| `notes/2026-05-16_16-12_v1-perf-003-resource-exhaustion-safeguards.md` | notes-history | `archive-note` | 1 | 39 |
| `notes/2026-05-16_16-59_v1-test-003-runtime-native-security-regressions.md` | notes-history | `archive-note` | 1 | 18 |
| `notes/2026-05-16_17-10_v1-test-005-docs-examples-smoke-suite.md` | notes-history | `archive-note` | 1 | 20 |
| `notes/2026-05-20_11-44_pre-v1-master-unfinished-checklist-audit.md` | notes-history | `archive-note` | 1 | 31 |
| `notes/2026-05-20_16-10_v1u-run-002_ruff-test-interpreter-hardcoding-analysis.md` | notes-history | `archive-note` | 2 | 8,21 |
| `notes/vm_performance.md` | notes-history | `archive-note` | 1 | 21 |
| `src/main.rs` | other | `manual-review` | 1 | 125 |
| `src/parser.rs` | cli-harness | `harness-legacy` | 1 | 2190 |
| `tests/diagnostics_golden.rs` | integration-test | `diagnostics-diff,harness-legacy` | 1 | 60 |
| `tests/docs_examples.rs` | integration-test | `docs-smoke,harness-legacy` | 1 | 256 |
| `tests/interpreter_flag_dependency_map_contract.rs` | integration-test | `harness-legacy` | 1 | 84 |
| `tests/native_api_security_boundaries.rs` | integration-test | `security-test-choice` | 34 | 134,211,307,327,371,398,407,421,450,459,468,477,500,544,573,582,591,609,643,682,700,736,776,807,842,882,918,954,987,996,1029,1038,1074,1083 |
| `tests/package_module_workflow_integration.rs` | integration-test | `harness-legacy,package-workflow` | 4 | 124,316,347,366 |
| `tests/runtime_security.rs` | integration-test | `security-test-choice` | 5 | 128,146,175,206,261 |

## V1U-RUN-002: `ruff test` Interpreter Hardcoding Decision

Current state (`src/parser.rs::run_all_tests`): each fixture is executed via `ruff run <fixture> --interpreter`.

Root-cause evidence for keeping interpreter-pinned today:

- Snapshot corpus compatibility: `ruff test` compares fixture stdout against existing `tests/*.out` snapshots created around interpreter-first behavior.
- Runtime-path drift is still material: a local comparison sweep (`ruff run` vs `ruff run --interpreter`) found 15 mismatches in the first 21 fixtures scanned, including `tests/array_methods_test.ruff`, `tests/net_test.ruff`, `tests/error_call_stack_test.ruff`, and `tests/image_processing_test.ruff`.
- Divergence is not one class of issue: differences include runtime diagnostic code/subsystem shape (`[RUFVM001]` vs `[RUFRUN001]`), optimizer banner output, and builtin availability/behavior differences in legacy fixtures.

Decision (2026-05-20): keep `ruff test` interpreter-pinned for now, and close migration work under `V1U-RUN-003`.

Removal criteria for this hardcoding:

1. Add an explicit runtime-path strategy for `ruff test` (VM-first or dual-engine with deterministic fallback policy).
2. Normalize or rebaseline fixture expectations so runtime-path-specific diagnostics/noise do not create accidental false failures.
3. Add parity coverage for currently divergent fixture classes, then prove the selected strategy with focused command-level tests.
