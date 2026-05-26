# V1.0 Enterprise Readiness Enhancement Checklist

Last updated: 2026-05-25
Owner: Ruff core/runtime
Status: Active (post-parity hardening pass)

## Goal

Drive Ruff from "near release-ready" to "enterprise-grade and universally useful" by closing remaining hardening, performance, security, and product-presentation gaps without regressions.

## Operating Rules

1. All changes must be additive and backward-compatible unless explicitly approved.
2. No reduction in existing test coverage or contract guarantees.
3. Every checklist closure must include:
   - code + tests + docs updates,
   - command evidence,
   - a clear risk note.
4. Release publication/tagging remains frozen unless explicitly unblocked.

## Priority Summary

- P0: Must complete before claiming enterprise-grade readiness.
- P1: Strongly recommended for v1 launch quality.
- P2: Post-v1 improvements that materially improve adoption/operability.

## Checklist

- [ ] **ER-P0-001**: Run and archive full verification matrix on latest `main`.
  - Scope: `cargo test`, VM/dual sweeps, security boundaries, docs contracts, release gate scripts.
  - Acceptance:
    - all required suites pass,
    - failures triaged with owner and fix plan,
    - evidence captured in a dated note under `notes/`.
  - 2026-05-25 blocker: verification matrix is still red because `cargo test --test unsafe_inventory_contract` fails on executable-unsafe budget (`expected <=55, got 59`); VM/dual fixture sweeps remain at `137/150` with known parser-fixture debt (`tests/test_module_syntax.ruff`, `tests/testing_framework.ruff`, `tests/generators_test.ruff`, `tests/destructuring.ruff`, etc.). Evidence note: `notes/2026-05-25_23-06_er-p0-001-verification-matrix-triage.md`.

- [ ] **ER-P0-002**: Complete unsafe boundary tightening follow-through (JIT focus).
  - Scope: maintain machine-verifiable `SAFETY:` contracts and close remaining high-risk unsafe review gaps.
  - Acceptance:
    - unsafe inventory regenerated,
    - unsafe contract/checker tests pass,
    - residual unsafe sites categorized by risk and ownership.

- [ ] **ER-P0-003**: Finish runtime parity burn-down to zero known high-impact mismatches.
  - Scope: resolve remaining VM/interpreter mismatch inventory items and rebaseline artifacts.
  - Acceptance:
    - mismatch inventory regenerated,
    - no open high-severity parity mismatches,
    - `cargo run -- test --runtime vm` and `dual` pass.

- [x] **ER-P0-004**: Harden network/process/file capability defaults and docs alignment.
  - Scope: revalidate `--untrusted` guardrails against SSRF-style destinations, process execution limits, and path boundaries.
  - Acceptance:
    - `native_api_security_boundaries` and `runtime_security` green,
    - any policy drift resolved with explicit diagnostics and docs updates.
  - 2026-05-25 evidence:
    - `cargo test --test native_api_security_boundaries` passed (48/48).
    - `cargo test --test runtime_security` passed (11/11).
    - `cargo test --test docs_policy_consistency_contract` passed.
    - Updated `docs/NATIVE_API_SECURITY_POSTURE.md` outbound-policy diagnostics text to explicitly document deterministic invalid-policy and blocked-destination error strings.

- [ ] **ER-P0-005**: Repository hygiene cleanup for production-facing presentation.
  - Scope: remove/archive root-level non-source artifacts that do not belong in the main repo surface (ad-hoc backups, transient DBs, temporary directories) with explicit retention policy.
  - Acceptance:
    - root directory contains only intentional product/repo assets,
    - cleanup policy documented (what is generated vs versioned).

- [ ] **ER-P0-006**: Reduce executable `unsafe` budget back under gate threshold with evidence.
  - Scope: current strict inventory reports `Executable matches: 59` vs contract budget `<= 55`.
  - Acceptance:
    - `cargo test --test unsafe_inventory_contract` passes without raising budget ceilings,
    - reduction work is classified and documented in `docs/generated/UNSAFE_INVENTORY.md` + dated `notes/` entry.

- [ ] **ER-P1-001**: Binary size optimization pass with reproducible measurements.
  - Scope: benchmark release binary size and evaluate feature/profile/link-time optimizations without behavior regressions.
  - Acceptance:
    - before/after size table committed,
    - no contract test regressions,
    - tradeoff note for startup/runtime impact.

- [ ] **ER-P1-002**: Performance hot-path audit and micro-benchmark stabilization.
  - Scope: VM call dispatch, module loading/import-heavy startup, dict/index hot paths, and JIT/VM crossover.
  - Acceptance:
    - targeted bench commands + results committed,
    - identified regressions fixed or documented with owner/timeline.

- [ ] **ER-P1-003**: Type-checker ergonomics and diagnostics hardening for high-signal gaps.
  - Scope: resolve remaining medium/high-value TODO clusters in `src/type_checker.rs` and improve actionable messaging.
  - Acceptance:
    - targeted type-checker tests added,
    - TODO triage artifacts updated,
    - no misleading "supported" behavior in docs.

- [x] **ER-P1-004**: Production CLI UX polish for machine + human operators.
  - Scope: ensure deterministic JSON diagnostics, consistent exit-code semantics, and crisp remediation hints.
  - Acceptance:
    - CLI contract tests updated and passing,
    - docs/examples match actual behavior.
  - 2026-05-25 evidence:
    - `cargo test --test cli_contracts` passed (15/15), including dual/vm runtime summary and fallback-marker expectations.
    - `cargo test --test cli_json_contracts` passed (13/13), preserving machine-readable JSON surfaces and negative-path failure semantics.
    - Updated `docs/CLI_MACHINE_READABLE_CONTRACTS.md` with explicit deterministic `ruff test` runtime-summary/fallback-marker contracts referenced by test names.

- [ ] **ER-P1-005**: Root-to-docs consistency pass.
  - Scope: verify README, roadmap, release process, security posture, and migration docs are aligned with current runtime behavior.
  - Acceptance:
    - docs contract tests pass,
    - contradictions removed,
    - examples runnable on default VM path unless explicitly marked otherwise.

- [ ] **ER-P2-001**: Packaging/distribution ergonomics improvement plan.
  - Scope: installer, package-manager guidance, and reproducible build/documentation improvements.
  - Acceptance:
    - clear install matrix published,
    - known platform caveats documented.

- [ ] **ER-P2-002**: Developer onboarding quality pass.
  - Scope: add concise "build your first real tool" path and operational cookbook for production scripting.
  - Acceptance:
    - runnable examples validated in CI or contract tests,
    - docs links audited for dead/outdated references.

## Immediate Next Suggested Order

1. `ER-P0-001` full verification matrix evidence run.
2. `ER-P0-006` unsafe budget reduction pass to clear strict contract gate.
3. `ER-P0-005` root hygiene cleanup and policy.
4. `ER-P1-001` binary size optimization baseline and pass.
5. `ER-P1-002` performance hot-path benchmark cycle.
6. `ER-P1-005` docs consistency finalization.
