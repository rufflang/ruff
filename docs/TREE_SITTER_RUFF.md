# Tree-sitter Ruff Grammar

Status: v0.13.0 baseline scaffold

Ruff now includes a dedicated Tree-sitter grammar package at:

- `tree-sitter-ruff/`

## Included Assets

- Grammar definition: `tree-sitter-ruff/grammar.js`
- Corpus fixtures: `tree-sitter-ruff/test/corpus/core.txt`
- Regression corpus fixtures: `tree-sitter-ruff/test/corpus/regressions.txt`
- Highlight queries: `tree-sitter-ruff/queries/highlights.scm`
- Injection queries: `tree-sitter-ruff/queries/injections.scm`

## CI Contract Coverage

Asset/corpus guard test:

```bash
cargo test --test tree_sitter_ruff_assets
```

This test enforces that required grammar/corpus/query assets exist and include expected baseline rules/tokens.

Regression fixture highlights include:

- async/await function patterns
- struct literal/field-access patterns
- match/option pattern structures

Highlight query coverage is validated against representative keyword/function/variable captures used by these fixtures.

## Editor Integration Baseline

A baseline integration path for Neovim highlighting is documented in `docs/EDITOR_ADAPTER_BASELINES.md` and can consume `tree-sitter-ruff/queries/highlights.scm` once the grammar package is generated and installed through the editor's Tree-sitter toolchain.

Canonical goal:

- `.ruff` files highlighted through Tree-sitter grammar + Ruff query files
