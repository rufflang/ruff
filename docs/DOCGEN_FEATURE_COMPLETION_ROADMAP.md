# DocGen Feature Completion Roadmap (AI Agent)

## Purpose

Drive Ruff DocGen from currently stable generation to production-grade, CI-friendly, feature-complete behavior for Ruff-first projects and mixed-language repos.

Primary baseline and findings are documented in:

- docs/DOCGEN_EXTERNAL_REPOS_EVALUATION_2026-05-18.md

## Definition Of Done (Feature-Complete)

DocGen is considered feature-complete for this initiative when all items below are true:

1. Public API gating produces low-noise results for Ruff repos.
2. Strict mode is practical in CI and does not fail mainly from visibility false-positives.
3. Extraction accuracy is robust for common Ruff syntax patterns (beyond regex happy path).
4. Missing docs and examples can be remediated with deterministic workflow output.
5. Discovery and truncation behavior is transparent in machine-readable output.
6. Tests cover contract stability and new behavior changes.

## Current Baseline (Validated)

1. Doc generation works for ruff-ai-sdk, ruff-mcp, and ruff-scout.
2. Docgen-focused tests pass:
   - tests/docgen_universal.rs
   - tests/cli_json_contracts.rs (docgen contract test)
   - tests/package_module_workflow_integration.rs (workflow contract)
3. Strict runs fail on undocumented public symbols only (no broken links, no warnings).

## Workstream A: Visibility And Gate Signal Quality (P0)

### Goal

Make undocumented-symbol gate failures represent true exported/public API docs gaps.

### Tasks

1. Define Ruff visibility policy for docgen symbols.
2. [x] Update Ruff adapter visibility assignment for top-level functions. (Completed 2026-05-18)
3. Ensure private/internal helpers are not classified as public by default.
4. Add tests for:
   - top-level non-public helpers
   - explicit public symbols
   - method visibility in structs

### Acceptance Criteria

1. Strict mode gate counts drop to meaningful public API gaps on target repos.
2. New visibility tests pass consistently.
3. Existing docgen contract tests remain green.

## Workstream B: Extraction Robustness (P1)

### Goal

Reduce regex-related misses and misclassification in Ruff symbol extraction.

### Tasks

1. Expand extraction coverage for real-world Ruff syntax edge cases.
2. Add fixture-driven tests for edge patterns currently prone to miss.
3. Evaluate parser-backed extraction path (or hybrid fallback) and document decision.

### Acceptance Criteria

1. New extraction fixtures pass.
2. Symbol inventory for target repos is stable across repeated runs.
3. No regression in deterministic output tests.

## Workstream C: Documentation Attachment Coverage (P1)

### Goal

Improve auto-attachment of inline docs and reduce placeholder-only output.

### Tasks

1. Add support for additional Ruff doc-comment styles if language supports them.
2. Strengthen comment-to-symbol attachment heuristics where safe.
3. Add tests for spacing/proximity edge cases.

### Acceptance Criteria

1. Placeholder docs rate drops on sample repos without manual source edits.
2. Attachment tests cover all supported doc-comment formats.

## Workstream D: Discovery Transparency And Diagnostics (P2)

### Goal

Expose skipped/discarded files in diagnostics and JSON summary.

### Tasks

1. Emit diagnostics for max size, max depth, and max file count skips.
2. Add summary counters for skipped files by reason.
3. Preserve deterministic ordering of diagnostics.

### Acceptance Criteria

1. docgen JSON contains skip counters and reason categories.
2. Tests verify counters and deterministic ordering.

## Workstream E: Link Validation Modes (P2)

### Goal

Provide stronger optional link validation while keeping default runs fast.

### Tasks

1. Keep local existence check as default.
2. Add optional anchor validation mode for local docs.
3. Add optional external link validation mode with timeouts and allowlist.
4. Add clear warnings/errors and contract tests for mode behavior.

### Acceptance Criteria

1. Each link mode has deterministic test coverage.
2. Default mode runtime impact remains minimal.

## Workstream F: Summary Ergonomics And CI Usability (P3)

### Goal

Make outputs easier to consume in automation and release gates.

### Tasks

1. Add separate counts for project symbols vs builtin symbols.
2. Add per-kind counts (function, method, struct, enum, etc.).
3. Add stable machine-readable summary block for dashboards.

### Acceptance Criteria

1. JSON contract updated with versioned additions only.
2. Contract tests validate new summary fields and backward compatibility.

## Recommended Implementation Order

1. Workstream A
2. Workstream B
3. Workstream C
4. Workstream D
5. Workstream E
6. Workstream F

## Execution Checklist For AI Agent

1. Open docs/DOCGEN_EXTERNAL_REPOS_EVALUATION_2026-05-18.md and translate each limitation into issue-sized tasks.
2. Implement one workstream at a time in small PR-sized commits.
3. After each workstream, run:
   - cargo test --test docgen_universal -- --nocapture
   - cargo test --test cli_json_contracts docgen_json_contract_is_stable -- --nocapture
4. Re-run docsgen on:
   - /Users/robertdevore/2026/ruff-ai-sdk
   - /Users/robertdevore/2026/ruff-mcp
   - /Users/robertdevore/2026/ruff-scout
5. Compare strict gate deltas for undocumented_count, broken_link_count, and warning_count.
6. Update evaluation docs with before/after metrics.

## Suggested CI Profiles

### Strict Public API Gate

```bash
ruff docgen . \
  --languages ruff \
  --public-only \
  --no-builtins \
  --fail-on-undocumented \
  --fail-on-broken-links \
  --fail-on-warnings \
  --out-dir docs/generated
```

### Developer Audit Mode

```bash
ruff docgen . \
  --languages ruff \
  --include-private \
  --emit-ai-tasks \
  --search-index \
  --out-dir docs/generated-dev
```

## Reporting Template (Per Workstream)

1. What changed
2. Why it changed
3. Test evidence
4. Impact on strict gate counts
5. Remaining risks
