# v0.13.0 Tree-sitter Ruff Baseline Evidence

Date: 2026-04-30
Track: v0.13.0 Cross-IDE Foundation
Checklist item: Tree-sitter Grammar For Universal Highlighting

## Implemented

Added grammar package scaffold:

- `tree-sitter-ruff/package.json`
- `tree-sitter-ruff/grammar.js`

Added corpus fixtures:

- `tree-sitter-ruff/test/corpus/core.txt`

Added query files:

- `tree-sitter-ruff/queries/highlights.scm`
- `tree-sitter-ruff/queries/injections.scm`

Added CI guard test:

- `tests/tree_sitter_ruff_assets.rs`

Added documentation:

- `docs/TREE_SITTER_RUFF.md`

## Verification

Command:

- `cargo test --test tree_sitter_ruff_assets`

Result:

- PASS

## Notes

- Baseline adapter guidance in `docs/EDITOR_ADAPTER_BASELINES.md` references Tree-sitter highlight-query consumption path for editor integrations.
