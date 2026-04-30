# v0.14.0 Tree-sitter And Editor Adapter Maturity Evidence

Date: 2026-05-01
Context: local development machine (macOS)
Scope: ROADMAP section "4. Tree-sitter And Editor Adapter Maturity"

## Implemented

- Expanded tree-sitter corpus coverage:
  - `tree-sitter-ruff/test/corpus/regressions.txt`
- Expanded highlight keyword coverage:
  - `tree-sitter-ruff/queries/highlights.scm`
- Hardened tree-sitter asset regression checks:
  - `tests/tree_sitter_ruff_assets.rs`
- Published adapter maintenance policy boundaries:
  - `docs/EDITOR_ADAPTER_BASELINES.md`
- Documented explicit `.vsix` install flow for VS Code/Cursor-compatible editors:
  - `docs/INSTALLATION_LSP_EDITORS.md`
- Hardened first-party extension smoke checks:
  - `tools/vscode-ruff-extension/scripts/check.js`
- Wired extension smoke check into CI artifact validation matrix:
  - `.github/workflows/release-artifact-validation-matrix.yml`

## Verification Commands

1. `cargo test --test tree_sitter_ruff_assets`
- Result: PASS (`1 passed; 0 failed`)

2. `cd tools/vscode-ruff-extension && npm run check`
- Result: PASS
- Output: `Extension static checks passed.`

## Acceptance Mapping

- Grammar corpus regression fixture coverage now includes representative parser/highlight edge-case families.
- Adapter policy docs are thin and anchored to canonical Ruff contract docs.
- Extension baseline still provides `.ruff` registration and syntax grammar with no manual mode-switch requirement in supported hosts.
- Extension smoke check now runs both locally and in release artifact validation CI sequence.

## Remaining v0.14.0 Checklist Work

- runtime and tooling reliability track
- v1.0.0 scope definition gate
