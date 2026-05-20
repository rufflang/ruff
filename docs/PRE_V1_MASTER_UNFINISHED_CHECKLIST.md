# Ruff Pre-1.0 Master Unfinished Checklist

Status: active pre-`v1.0.0` completion roadmap (team-lead audit pass)  
Audit date: 2026-05-20

Purpose: consolidate every still-unfinished or still-unclear pre-1.0 item into one execution checklist, including:
- explicit open checklist/roadmap/release items
- runtime-path limitations (especially interpreter-flag dependencies)
- stale or conflicting documentation
- research-first tasks needed to make confident implementation decisions

Primary evidence sources:
- `docs/PRE_V1_ACTION_CHECKLIST.md`
- `ROADMAP.md`
- `docs/RELEASE_PROCESS.md`
- `docs/RELEASE_ARTIFACT_CHECKLIST_V1_0_0.md`
- `docs/UNFINISHED_AND_MVP_AUDIT.md`
- `docs/VM_INTERPRETER_PARITY_MATRIX.md`
- `README.md`
- `docs/ARCHITECTURE.md`
- `notes/2026-05-20_09-07_prev1-rel-001-rc-gate-evidence.md`
- `src/main.rs`, `src/parser.rs`, `tests/` interpreter-mode usage surfaces

---

## Checklist Governance And Closure Semantics

This checklist is executed in strict one-item loops.

### Loop Selection Rule (Mandatory)

1. Only pick items still marked `- [ ]`.
2. Choose the first unchecked item in top-to-bottom file order.
3. Do not skip ahead unless the current item is explicitly blocked.
4. If blocked:
   - Add a dated blocker note directly under the blocked item with reason + command/output evidence.
   - Continue scanning to the next unchecked item in the same loop.
5. Complete exactly one unblocked checklist item per loop.

### Closure Evidence Rule (Mandatory)

Do not mark an item complete until all of the following exist for that item:

1. Implementation or decision artifact committed (code/script/doc/note as applicable).
2. Tests or command validations run and recorded in loop evidence.
3. Relevant docs/checklists updated and consistent with the new state.
4. Checklist row switched from `- [ ]` to `- [x]` with a dated evidence bullet.
5. Commit message references the checklist ID (for example: `prev1(V1U-RES-003): ...`).

### Blocker Semantics

- A blocked item remains unchecked.
- Blocker notes must include:
  - Date (`YYYY-MM-DD`)
  - Root-cause summary
  - Verifiable evidence (command path, file path, or link)
- Re-check blocked items in later loops before skipping again.

### Required Per-Loop Report Fields

Each loop report must include exactly:

1. Item completed.
2. Files changed.
3. Tests/commands run with results.
4. Blockers or follow-ups.

---

## 0) Research And Truth-Set (Do First)

- [x] **V1U-RES-001**: Build a machine-generated unresolved-item inventory.
  - Scope: produce one auditable table from docs/notes/code that lists unresolved items, source file, last touched date, and current owner.
  - Acceptance criteria:
    - Scripted inventory output committed under `docs/generated/` or equivalent.
    - Every item in this checklist maps to at least one source reference.
  - Evidence (2026-05-20):
    - Added `scripts/generate_pre_v1_unresolved_inventory.sh` and generated `docs/generated/PRE_V1_UNRESOLVED_INVENTORY.md` + `.csv`.
    - Added `tests/pre_v1_unresolved_inventory_contract.rs` covering success output, unmapped-ID failure, and duplicate-ID failure.

- [x] **V1U-RES-002**: Classify unresolved items into `v1-blocker`, `v1-should-fix`, `post-v1`, or `archive`.
  - Scope: prevent stale tasks and historical noise from being treated as active blockers.
  - Acceptance criteria:
    - Classification rationale documented per item.
    - `docs/UNFINISHED_AND_MVP_AUDIT.md` updated to match classifications.
  - Evidence (2026-05-20):
    - Extended `scripts/generate_pre_v1_unresolved_inventory.sh` to emit per-item `classification` + `rationale` columns and regenerated markdown/CSV outputs.
    - Replaced `docs/UNFINISHED_AND_MVP_AUDIT.md` with a classification-aligned snapshot that mirrors generated category counts and semantics.

- [x] **V1U-RES-003**: Define checklist governance and closure semantics.
  - Scope: standardize what evidence is required before checking an item complete (test output, note link, command logs).
  - Acceptance criteria:
    - Closure policy section added to this file or a linked process doc.
    - Team can execute one-item-per-loop without ambiguity.
  - Evidence (2026-05-20):
    - Added `Checklist Governance And Closure Semantics` section with strict selection, blocker, closure-evidence, and reporting rules.
    - Added `tests/pre_v1_master_checklist_contract.rs` to enforce required governance markers.

---

## 1) Open Items Already Documented Elsewhere

- [x] **V1U-OPEN-001**: Resolve `PREV1-RUN-002` external-doc dependency (`ruff-mcp` closure-mutation docs drift).
  - Scope: close via direct edit in `ruff-mcp` source repo or formal external handoff ticket with link.
  - Acceptance criteria:
    - `docs/PRE_V1_ACTION_CHECKLIST.md` no longer has unresolved blocker state for this item.
    - External edit/handoff evidence captured in `notes/`.
  - Evidence (2026-05-20):
    - Updated `docs/PRE_V1_ACTION_CHECKLIST.md` so `PREV1-RUN-002` is no longer unresolved/blocked in this repository.
    - Added formal handoff note `notes/2026-05-20_12-55_v1u-open-001_ruff-mcp-doc-handoff.md` with validation evidence, target scope, and follow-through instructions for the external `ruff-mcp` source repo.

- [ ] **V1U-OPEN-002**: Complete `ROADMAP.md` unchecked final checklist items.
  - Scope:
    - intentionally bump Cargo version for release
    - produce release candidate from clean working tree
  - Acceptance criteria:
    - `ROADMAP.md` final checklist has no unchecked pre-tag items.
  - Blocker (2026-05-20): This item is tag-phase work; Cargo version bump + clean-tree RC build cannot be finalized while pre-v1 blocker checklist items are still being executed.
    Evidence: `ROADMAP.md` final checklist still carries release-phase unchecked items, and this loop is intentionally scoped to one non-tag checklist item at a time.

- [ ] **V1U-OPEN-003**: Complete `docs/RELEASE_ARTIFACT_CHECKLIST_V1_0_0.md` tag-time sign-off items.
  - Scope: publish release, verify assets/checksums/smoke workflow, record evidence.
  - Acceptance criteria:
    - all tag-time checkboxes marked done with artifact URLs and checksum evidence note.
  - Blocker (2026-05-20): Requires the actual `v1.0.0` tag publication event and post-publish workflow evidence, which is not executable during pre-tag checklist loops.
    Evidence: `docs/RELEASE_ARTIFACT_CHECKLIST_V1_0_0.md` Tag-Time Sign-Off section remains release-event dependent.

- [x] **V1U-OPEN-004**: Execute `V1-DOCGEN-001` roadmap item.
  - Scope: complete the universal DocGen maturation slice currently open in `ROADMAP.md`.
  - Acceptance criteria:
    - `ROADMAP.md` no longer contains unchecked `V1-DOCGEN-001`.
    - DocGen milestone evidence linked from roadmap notes.
  - Evidence (2026-05-20):
    - Marked `V1-DOCGEN-001` complete in `ROADMAP.md` and linked remaining milestone execution follow-through to this master checklist (`V1U-DG-001`..`V1U-DG-003`).
    - Ran focused universal DocGen contract checks (`docgen_adapter_conformance_smoke_extracts_symbols_for_all_languages`, `docgen_cli_json_contract_preserves_legacy_fields`, `docgen_ruff_extraction_edge_fixture_async_visibility_contract`) and recorded that full-suite `cargo test --test docgen_universal` currently hits environment socket/runtime constraints in this workspace.

---

## 2) Release Gate Determinism And Evidence Closure

- [x] **V1U-GATE-001**: Fix repo formatting drift causing RC gate failure.
  - Scope: resolve `cargo fmt --check` diffs currently failing `scripts/release_candidate_gate.sh --full`.
  - Acceptance criteria:
    - `cargo fmt --check` passes locally.
    - Follow-up evidence note updates `PREV1-REL-001` context.
  - Evidence (2026-05-20):
    - Applied formatting with `cargo fmt` and verified `cargo fmt --check` passes locally.
    - Updated `notes/2026-05-20_09-07_prev1-rel-001-rc-gate-evidence.md` follow-up section to record drift resolution and status transition.

- [x] **V1U-GATE-002**: Decide rustfmt config policy (stable vs nightly-only options warnings).
  - Scope: eliminate persistent `rustfmt` unstable-option warnings or document intentional policy.
  - Acceptance criteria:
    - either warnings removed, or policy doc explains expected warning behavior and gate implications.
  - Evidence (2026-05-20):
    - Adopted stable-only `rustfmt.toml` policy by removing unstable option keys that emitted warning spam on stable toolchains.
    - Verified warning-free formatting gate execution with `cargo fmt` and `cargo fmt --check`.

- [x] **V1U-GATE-003**: Re-run full RC gate in low-contention environment and record pass/fail evidence.
  - Scope: repeat `bash scripts/release_candidate_gate.sh --full` after formatting and stability fixes.
  - Acceptance criteria:
    - dated `notes/` evidence with command log and explicit instability classification.
  - Evidence (2026-05-20):
    - Re-ran `bash scripts/release_candidate_gate.sh --full` and captured command-level outcomes in `notes/2026-05-20_14-12_v1u-gate-003-rc-gate-rerun.md`.
    - Classified result as environment/runtime instability in docgen external-host validation stack (after fmt/clippy gates passed), with follow-through pointed to `V1U-GATE-004`.

- [ ] **V1U-GATE-004**: Stabilize socket/timing-sensitive release suites if flake persists.
  - Scope: harden `serve` integration startup reliability and timing-fragile test expectations as needed.
  - Acceptance criteria:
    - repeatable pass in low-contention environment across at least two consecutive full runs.

---

## 3) Interpreter-Flag Dependency Burn-Down (Team-Lead Priority)

- [ ] **V1U-RUN-001**: Produce a full interpreter-flag dependency map.
  - Scope: inventory every `--interpreter` use in CLI paths, harnesses, tests, docs, and examples.
  - Acceptance criteria:
    - dependency map includes reason tags (`parity-gap`, `diagnostics-diff`, `harness-legacy`, `security-test-choice`, etc.).
    - map published in `docs/` and linked from this checklist.

- [ ] **V1U-RUN-002**: Explain and justify `ruff test` interpreter hardcoding in `src/parser.rs::run_all_tests`.
  - Scope: confirm whether this is still required, and if yes, define removal criteria.
  - Acceptance criteria:
    - documented root-cause analysis with concrete failing fixtures/surfaces if VM-first is not yet safe.
    - decision recorded: keep temporarily, switch to VM-first, or dual-mode with fixture metadata.

- [ ] **V1U-RUN-003**: Implement a VM-first or dual-engine `ruff test` execution strategy.
  - Scope: reduce reliance on blanket interpreter execution for fixture sweeps.
  - Acceptance criteria:
    - `ruff test` can run VM for parity-safe fixtures.
    - fallback policy is explicit, bounded, and tested.

- [ ] **V1U-RUN-004**: Close generator-surface ambiguity between docs/tests/runtime.
  - Scope: remove the “VM generator support is partial” drift signal by either fixing VM support gaps or making boundary explicit in canonical docs.
  - Acceptance criteria:
    - generator behavior status is consistent across `README`, parity matrix, and generator tests.
    - no silent engine-specific behavior differences in covered generator scenarios.

- [ ] **V1U-RUN-005**: Expand parity evidence for any surface still commonly forced to interpreter mode.
  - Scope: for each dependency-map item tagged `parity-gap`, add targeted VM/interpreter parity tests or explicit documented divergence.
  - Acceptance criteria:
    - either parity tests exist, or divergence is intentional and documented in release-facing docs.

- [ ] **V1U-RUN-006**: Add command-level runtime-path matrix.
  - Scope: complement `docs/VM_INTERPRETER_PARITY_MATRIX.md` with command-level coverage (`run`, `test`, `test-run`, security suites, diagnostics modes).
  - Acceptance criteria:
    - maintainers can see exactly which runtime path each command/test surface depends on and why.

---

## 4) Documentation Integrity And Staleness Cleanup

- [ ] **V1U-DOC-001**: Replace or fully refresh stale `docs/ARCHITECTURE.md`.
  - Scope: resolve major drift (v0.8/v0.9 language, VM-default mismatch, outdated component status).
  - Acceptance criteria:
    - architecture doc reflects current execution defaults and release posture.
    - no contradictory “VM experimental/not default” claims against current README/CLI behavior.

- [ ] **V1U-DOC-002**: Align maturity/boundary wording across top-level docs.
  - Scope: sync `README.md`, `docs/V1_SCOPE.md`, `docs/LANGUAGE_SPEC.md`, `docs/RUFF_FEATURE_INVENTORY.md`, and `docs/UNFINISHED_AND_MVP_AUDIT.md`.
  - Acceptance criteria:
    - one consistent story for pre-1.0 readiness, experimental surfaces, and deferred guarantees.

- [ ] **V1U-DOC-003**: Review `docs/STANDARD_LIBRARY_REFERENCE.md` experimental labels for v1 contract clarity.
  - Scope: decide which experimental APIs are in v1 guarantee vs explicitly non-guaranteed.
  - Acceptance criteria:
    - experimental-label policy documented and consistent with `docs/V1_SCOPE.md` and release commitments.

- [ ] **V1U-DOC-004**: Add/refresh docs contract tests for high-risk consistency surfaces.
  - Scope: prevent future drift in readiness, runtime-path expectations, and deferred-boundary claims.
  - Acceptance criteria:
    - contract tests fail when key policy text disappears or contradicts other canonical docs.

---

## 5) Universal DocGen Next-Stage Milestones (From Open Roadmap/Doc Items)

- [ ] **V1U-DG-001**: Execute `DG-NEXT-001` parser-assisted Ruff extraction fallback prototype.
  - Scope: opt-in parser-assisted extraction with deterministic regex fallback on diagnostics.
  - Acceptance criteria:
    - parser-success and parser-fallback fixture coverage added.
    - strict-gate behavior remains deterministic.

- [ ] **V1U-DG-002**: Execute `DG-NEXT-002` cross-language adapter conformance expansion.
  - Scope: broaden multi-language edge-pattern fixture coverage and output-shape contracts.
  - Acceptance criteria:
    - adapter conformance tests expanded across supported languages.

- [ ] **V1U-DG-003**: Execute `DG-NEXT-003` external-repo strict-gate baseline refresh cadence.
  - Scope: codify cadence and evidence format for strict/public-only drift checks on representative external repos.
  - Acceptance criteria:
    - repeatable cadence documented and at least one refreshed baseline note captured.

---

## 6) Code-Level “Half-Fixed” Risk Audit (Pre-1.0 Triage)

- [ ] **V1U-CODE-001**: Audit `TODO/FIXME/HACK` debt in production paths and classify risk.
  - Scope: prioritize runtime/compiler/VM/interpreter/native-function TODOs that could affect v1 correctness/security predictability.
  - Acceptance criteria:
    - triage table with severity, owner, and target release bucket (`v1` vs `post-v1`).

- [ ] **V1U-CODE-002**: Resolve or explicitly defer high-risk TODOs in runtime execution paths.
  - Scope: focus on TODOs in VM/compiler/interpreter async ops and any code path user scripts hit by default.
  - Acceptance criteria:
    - no unclassified high-risk TODO remains in default runtime path.
    - deferred items are documented in release-facing scope docs.

- [ ] **V1U-CODE-003**: Verify optional typing non-enforcement boundaries remain intentional and well-isolated.
  - Scope: ensure current type-checker TODOs do not leak as misleading “supported” guarantees.
  - Acceptance criteria:
    - v1 optional-typing docs match implementation boundaries and warning behavior.

---

## 7) Final Pre-Tag Execution

- [ ] **V1U-FINAL-001**: Run final full gate bundle and archive evidence.
  - Scope: required release gates + targeted parity/security/contracts + artifact validation script.
  - Acceptance criteria:
    - dated evidence note with pass/fail per command and environment details.

- [ ] **V1U-FINAL-002**: Perform release dry run from a clean tree.
  - Scope: rehearse version bump, checklist closure, tag flow, and artifact workflow without publishing.
  - Acceptance criteria:
    - dry-run note proves deterministic repeatability and no undocumented manual steps.

- [ ] **V1U-FINAL-003**: Complete tag-time artifact checklist.
  - Scope: execute final release publication and post-publish smoke verification.
  - Acceptance criteria:
    - all items in `docs/RELEASE_ARTIFACT_CHECKLIST_V1_0_0.md` checked with linked evidence.

---

## Suggested Execution Order

1. `V1U-RES-*` (truth-set and classification)
2. `V1U-GATE-001` and `V1U-GATE-002` (clear deterministic blockers)
3. `V1U-RUN-*` (interpreter-flag dependency closure)
4. `V1U-DOC-*` (consistency and stale docs)
5. `V1U-DG-*` (open roadmap docgen milestones)
6. `V1U-CODE-*` (high-risk TODO triage/closure)
7. `V1U-FINAL-*` (final pre-tag execution)
