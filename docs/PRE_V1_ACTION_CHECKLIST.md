# Ruff Pre-v1 Action Checklist

Status: active working checklist before `v1.0.0` release prep.

Purpose: track high-value work we can do now, before final RC/tag-time release tasks.

How to use with an AI agent:
- Pick one unchecked item.
- Complete implementation + tests + docs for that item only.
- Mark it done and link evidence (commands + results).

---

## 1) Runtime / Language / Parity Follow-through

- [x] **PREV1-RUN-001**: Add async named-nested closure capture parity coverage.
  - Scope: add explicit async parity test(s) for named nested closure capture/mutation behavior.
  - Acceptance criteria:
    - New test(s) added in `tests/vm_interpreter_parity_surfaces.rs`.
    - `cargo test --test vm_interpreter_parity_surfaces` passes.
  - Source context: `notes/2026-05-12_23-23_NO-ROADMAP_named-nested-closure-capture-parity.md`.

- [ ] **PREV1-RUN-002**: Close closure-mutation docs drift in Ruff MCP docs.
  - Scope: update the referenced `ruff-mcp` docs note so it no longer overstates old closure-mutation limitations.
  - Acceptance criteria:
    - Docs text aligned with current Ruff runtime behavior.
    - Linked note entry updated with what changed.
  - Source context: `notes/2026-05-12_23-23_NO-ROADMAP_named-nested-closure-capture-parity.md`.
  - Blocker (2026-05-20): `ruff-mcp` docs are not present in this repository workspace, so there is no local docs target to edit for this item.
    Evidence: `rg -n "ruff-mcp|mcp.ruff" README.md docs notes -g '*.md'` only returns references/notes, and `rg --files | rg "mcp|MCP|ruff-mcp|mcp.ruff"` returns no matching docs files.

---

## 2) Docs/Examples Quality Debt

- [x] **PREV1-DOC-001**: Burn down expected-fail example debt in smoke harness.
  - Scope: convert parse/runtime-drift examples to parse-clean (or run-clean where appropriate), then remove them from expected-fail classification.
  - Acceptance criteria:
    - Expected-fail examples list in `tests/docs_examples.rs` is reduced.
    - `cargo test --test docs_examples` passes.
  - Source context: `notes/2026-05-16_17-10_v1-test-005-docs-examples-smoke-suite.md`.

- [x] **PREV1-DOC-002**: Keep docs snippet smoke set parse-clean.
  - Scope: maintain zero stale expected-fail doc snippets as docs evolve.
  - Acceptance criteria:
    - `expected_fail_doc_blocks()` stays empty (or trends toward empty if new temporary debt is introduced intentionally with reasons).
    - `cargo test --test docs_examples` passes after doc edits.
  - Source context: `tests/docs_examples.rs`, `notes/2026-05-16_17-10_v1-test-005-docs-examples-smoke-suite.md`.

- [ ] **PREV1-DOC-003**: Add a roadmap-tracked item for universal docgen maturation.
  - Scope: add/update explicit roadmap tracking for universal docgen staging and next milestones.
  - Acceptance criteria:
    - `ROADMAP.md` includes a clear item/slice for universal docgen follow-through.
    - `docs/DOCGEN.md` and roadmap references are aligned.
  - Source context: `notes/2026-05-17_21-57_universal-docgen-architecture-and-gates.md`.

---

## 3) Diagnostics / Contracts / Test Infrastructure

- [ ] **PREV1-DIAG-001**: Expand diagnostics goldens for runtime JSON diagnostic surfaces as they evolve.
  - Scope: add fixture + golden coverage when new machine-readable runtime diagnostics are added.
  - Acceptance criteria:
    - `tests/diagnostics_golden.rs` includes new fixture category coverage.
    - `cargo test --test diagnostics_golden` passes.
  - Source context: `notes/2026-05-16_17-03_v1-test-004-diagnostics-golden-snapshots.md`.

- [ ] **PREV1-DIAG-002**: Keep runtime-security diagnostics represented in golden coverage where feasible.
  - Scope: add selected `tests/runtime_security.rs`-aligned diagnostics snapshot cases.
  - Acceptance criteria:
    - At least one runtime-security-oriented diagnostics fixture added (when output is stable enough).
    - Golden tests remain deterministic cross-platform (CRLF-safe).
  - Source context: `notes/2026-05-16_16-59_v1-test-003-runtime-native-security-regressions.md`.

---

## 4) Fuzzing Operational Hardening

- [ ] **PREV1-FUZZ-001**: Add local fuzz-smoke helper script with prerequisite checks.
  - Scope: script for nightly/cargo-fuzz/toolchain checks and clear error guidance.
  - Acceptance criteria:
    - Script added under `scripts/` and documented in README/docs.
    - Smoke run command is reproducible on supported local environments.
  - Source context: `notes/2026-05-16_16-35_v1-test-002-lexer-parser-fuzzing.md`.

- [ ] **PREV1-FUZZ-002**: Add parser/lexer fuzz crash reproduction automation path.
  - Scope: standardized way to replay crash artifacts from fuzz CI.
  - Acceptance criteria:
    - Documented repro workflow and helper command/script.
    - Validated on at least one synthetic or real crash input.
  - Source context: `notes/2026-05-16_16-35_v1-test-002-lexer-parser-fuzzing.md`.

---

## 5) Cross-Platform Security Coverage

- [ ] **PREV1-SEC-001**: Define non-Unix strategy for module-escape regression coverage.
  - Scope: add Windows-compatible equivalent for symlink/escape boundary testing, or document and gate an equivalent deterministic strategy.
  - Acceptance criteria:
    - Cross-platform policy documented.
    - Tests updated to reflect the chosen strategy without flaky behavior.
  - Source context: `notes/2026-05-16_16-59_v1-test-003-runtime-native-security-regressions.md`.

---

## 6) Release-Readiness Prep (Pre-tag, non-version-bump)

- [ ] **PREV1-REL-001**: Run and record full release-candidate gate evidence in a low-contention environment.
  - Scope: execute `scripts/release_candidate_gate.sh --full` in an environment suitable for stable socket/timing-sensitive tests; capture results in a dated note.
  - Acceptance criteria:
    - Command results logged with pass/fail details.
    - Any instability is explicitly categorized with mitigation or follow-up.
  - Source context: `docs/RELEASE_PROCESS.md`, `docs/UNFINISHED_AND_MVP_AUDIT.md`.

- [ ] **PREV1-REL-002**: Keep deferred/non-goal boundaries explicit and current.
  - Scope: make sure docs consistently reflect what is intentionally deferred vs in-scope during ongoing pre-v1 work.
  - Acceptance criteria:
    - `README.md`, `docs/V1_SCOPE.md`, `docs/OPTIONAL_TYPING_DESIGN.md` stay aligned after each major feature/doc change.
    - No stale claims about readiness or enforcement guarantees.
  - Source context: `README.md`, `docs/V1_SCOPE.md`, `docs/OPTIONAL_TYPING_DESIGN.md`.

---

## Suggested Execution Order

1. `PREV1-DOC-001`
2. `PREV1-FUZZ-001`
3. `PREV1-RUN-001`
4. `PREV1-SEC-001`
5. `PREV1-DIAG-001`
6. `PREV1-REL-001`
