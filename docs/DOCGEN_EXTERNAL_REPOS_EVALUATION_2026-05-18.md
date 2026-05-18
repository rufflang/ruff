# DocGen External Repos Evaluation (2026-05-18)

## Scope

Validated `ruff docgen` against these repositories:

- `/Users/robertdevore/2026/ruff-ai-sdk`
- `/Users/robertdevore/2026/ruff-mcp`
- `/Users/robertdevore/2026/ruff-scout`

Goals:

- Confirm generation works end-to-end.
- Confirm docgen tests/contracts still pass.
- Identify practical limitations and best next fixes.

## Commands Run

Generation (non-strict):

```bash
target/debug/ruff docgen <repo> \
  --out-dir docs/generated/external/<name> \
  --format all \
  --emit-ai-tasks \
  --search-index \
  --source-links \
  --include-private \
  --json
```

Strict gate run:

```bash
target/debug/ruff docgen <repo> \
  --out-dir docs/generated/external/<name>-strict \
  --format all \
  --emit-ai-tasks \
  --search-index \
  --source-links \
  --include-private \
  --fail-on-undocumented \
  --fail-on-broken-links \
  --fail-on-warnings \
  --json
```

Docgen-related tests:

```bash
cargo test --test docgen_universal -- --nocapture
cargo test --test cli_json_contracts docgen_json_contract_is_stable -- --nocapture
cargo test --test package_module_workflow_integration package_module_workflow_end_to_end_contract -- --nocapture
```

## Results

### Generation status

- `ruff-ai-sdk`: success, `item_count=329`, `diagnostics=0`, `warnings=0`
- `ruff-mcp`: success, `item_count=328`, `diagnostics=0`, `warnings=0`
- `ruff-scout`: success, `item_count=311`, `diagnostics=0`, `warnings=0`

### Artifacts present for all 3 repos

- `index.html`
- `docgen.md`
- `docgen.json`
- `docgen-gaps.json`
- `docgen-capabilities.json`
- `docgen-ai-tasks.md`
- `search-index.json`
- `symbol-index.json`

### Gap scan highlights (non-strict outputs)

- `ruff-ai-sdk`: `MissingDocs=25`, `MissingExamples=25`
- `ruff-mcp`: `MissingDocs=14`, `MissingExamples=14`
- `ruff-scout`: `MissingDocs=22`, `MissingExamples=22`

### Strict gate results

All three strict runs failed on undocumented symbols only:

- `ruff-ai-sdk`: `19 undocumented public symbols detected`
- `ruff-mcp`: `16 undocumented public symbols detected`
- `ruff-scout`: `1 undocumented public symbols detected`

No strict run reported broken links or warnings.

### Workstream A Task 2 follow-up (2026-05-18)

After updating Ruff top-level function visibility to require explicit `pub`, strict-mode metrics changed as follows:

| Repo | undocumented_count (before) | undocumented_count (after) | delta | broken_link_count delta | warning_count delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ruff-ai-sdk` | 19 | 0 | -19 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-mcp` | 16 | 0 | -16 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-scout` | 1 | 0 | -1 | 0 -> 0 (0) | 0 -> 0 (0) |

Interpretation: strict failures previously attributed to undocumented public symbols were eliminated for these repos by removing top-level visibility false positives.

### Workstream A Task 3 follow-up (2026-05-18)

After updating Ruff container-member visibility to keep symbols under private containers private (for example `pub` methods on private structs and variants on private enums), strict-mode metrics for the same repos are:

| Repo | undocumented_count (before) | undocumented_count (after) | delta | broken_link_count delta | warning_count delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ruff-ai-sdk` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-mcp` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-scout` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |

Interpretation: the Workstream A task 3 visibility hardening did not change strict gate counts on these three repositories (their current symbol sets do not rely on private-container member edge cases), while preserving zero broken links and zero warnings.

### Workstream A Task 4 follow-up (2026-05-18)

After adding explicit `public_only` visibility-matrix regression coverage in `tests/docgen_universal.rs`, strict-mode metrics for the same repos are:

| Repo | undocumented_count (before) | undocumented_count (after) | delta | broken_link_count delta | warning_count delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ruff-ai-sdk` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-mcp` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-scout` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |

Interpretation: task 4 is test-coverage hardening; external strict gate counts remain stable while regression protection for visibility classification increased.

### Workstream A Task 1 follow-up (2026-05-18)

After documenting the Ruff DocGen visibility policy in `docs/DOCGEN.md` (top-level `pub` requirement plus container-aware member visibility rules), strict-mode metrics for the same repos are:

| Repo | undocumented_count (before) | undocumented_count (after) | delta | broken_link_count delta | warning_count delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ruff-ai-sdk` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-mcp` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-scout` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |

Interpretation: task 1 is policy-definition/documentation hardening and does not change external strict gate counts.

### Workstream B Task 1 follow-up (2026-05-18)

After expanding Ruff extraction coverage to include `async func` / `pub async func` declarations in the Ruff adapter, strict-mode metrics for the same repos are:

| Repo | undocumented_count (before) | undocumented_count (after) | delta | broken_link_count delta | warning_count delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ruff-ai-sdk` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-mcp` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-scout` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |

Interpretation: the new async-declaration extraction support did not change strict external counts for these repos, but it closes a real Ruff syntax extraction gap and is covered by new async visibility and strict-gate regressions.

### Workstream B Task 2 follow-up (2026-05-18)

After adding fixture-driven Ruff extraction edge-case regressions (async visibility + strict-gate fixtures under `tests/fixtures/docgen/`), strict-mode metrics for the same repos are:

| Repo | undocumented_count (before) | undocumented_count (after) | delta | broken_link_count delta | warning_count delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ruff-ai-sdk` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-mcp` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-scout` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |

Interpretation: task 2 is test-hardening; external strict gate metrics remain stable while extraction edge coverage is now fixture-locked.

### Workstream B Task 3 follow-up (2026-05-18)

After evaluating parser-backed extraction versus the current regex-first hybrid path and documenting the decision, strict-mode metrics for the same repos are:

| Repo | undocumented_count (before) | undocumented_count (after) | delta | broken_link_count delta | warning_count delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ruff-ai-sdk` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-mcp` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |
| `ruff-scout` | 0 | 0 | 0 | 0 -> 0 (0) | 0 -> 0 (0) |

Interpretation: task 3 is architecture-decision documentation only; strict external gate counts remain stable while the hybrid extraction strategy and parser-assisted fallback criteria are now explicit.

### Test results

- `docgen_universal`: passed
- `docgen_json_contract_is_stable`: passed
- `package_module_workflow_end_to_end_contract`: passed

Conclusion: docsgen implementation and contracts are functioning, but quality gates expose documentation coverage gaps.

## Observed Limitations

## 1) Public symbol visibility for Ruff can over-report undocumented API

In `src/docgen/adapters/ruff.rs`, top-level `func` symbols are treated as public unless they are methods without `pub`.
This can classify many internal helper functions as public and inflate `fail-on-undocumented` results.

Impact seen here:

- Strict runs fail in all 3 repos due to undocumented public symbols.

## 2) Ruff adapter extraction is regex-based, not AST-based

`src/docgen/adapters/ruff.rs` uses line regexes for `func`, `struct`, `enum`, constants, and variants.
Complex syntax edge cases can be missed or misclassified compared to parser-backed extraction.

## 3) Inline docs for Ruff currently rely on `///` proximity

Ruff inline docs are attached from `///` comment blocks near symbols.
Other doc styles are not extracted by this adapter path.

## 4) Discovery hard limits are fixed

`src/docgen/core.rs` sets:

- max file size: 2 MB
- max files: 20,000
- max depth: 64

These are good safety defaults, but can silently omit data in very large repos unless surfaced clearly in diagnostics.

## 5) Link checking is intentionally narrow

In `src/docgen/gaps.rs`, broken-link checks skip `http://`, `https://`, and `mailto:` links.
It checks local existence only and does not verify external availability or anchors.

## 6) Builtins can dominate counts

By default, builtins are included unless `--no-builtins` is set.
For repo-quality reporting, builtin symbol volume can dilute signal from project-defined symbols.

## Best Next Steps (Prioritized)

## P0: Improve signal quality for CI gating

1. Add a CI docsgen mode for repositories using:
   - `--public-only`
   - `--no-builtins`
   - `--fail-on-undocumented`
2. Keep developer preview mode (`--include-private`) for local insight, but do not gate on it.

Expected result: gate failures map to true exported API gaps, not internal helpers.

## P1: Strengthen Ruff visibility semantics

1. Update Ruff adapter visibility heuristics to avoid auto-public for all top-level `func`.
2. Align with Ruff language visibility rules (prefer explicit `pub` or module export signals).
3. Add regression tests in `tests/docgen_universal.rs` for private top-level helper patterns.

Expected result: fewer false-positive undocumented public symbols.

## P1: Expand Ruff doc comment extraction

1. Support additional Ruff doc comment forms if the language allows them.
2. Keep `///` support, but extend parsing rules and tests.

Expected result: better doc attachment coverage without changing source code style immediately.

## P2: Improve diagnostics for discovery truncation

1. Emit explicit diagnostics when files are skipped due to `max_file_size_bytes`, `max_files`, or `max_depth`.
2. Include skip counts in JSON summary so CI can monitor omission drift.

Expected result: easier trust and troubleshooting for large repos.

## P2: Enhance link validation modes

1. Keep current local-only link check as default (safe/fast).
2. Add optional strict modes:
   - anchor validation for local docs links
   - optional external URL reachability check with timeout and allowlist

Expected result: stronger doc quality checks when needed, without slowing normal runs.

## P3: Improve summary ergonomics

1. Include `project_symbol_count` separate from `builtin_symbol_count` in CLI JSON summary.
2. Include per-kind counts (function/method/struct/enum/etc.) in summary.

Expected result: easier quality dashboards and more actionable CI output.

## Suggested Immediate CI Profile

Use this command as a baseline quality gate:

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

And keep this developer audit command for improvement work:

```bash
ruff docgen . \
  --languages ruff \
  --include-private \
  --emit-ai-tasks \
  --search-index \
  --out-dir docs/generated-dev
```

## Final Assessment

`docsgen` works correctly for generation and contract stability across the three repos tested.
Current limitations are mainly around extraction depth and visibility heuristics, not runtime reliability.
Applying the P0/P1 steps above should significantly reduce false positives and make CI gating practical.
