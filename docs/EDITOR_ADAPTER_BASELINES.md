# Editor Adapter Baselines (v0.13.0)

This document defines canonical thin-adapter setup paths for Ruff editor integrations.

Adapter rule:

- editor adapters must launch the official `ruff lsp` server
- adapters must not duplicate parser/analyzer/runtime logic
- shared behavior contracts belong to Ruff server/CLI docs, not per-editor forks

## VS Code / Cursor

Canonical path:

- command: `ruff lsp`
- sample adapter settings: `docs/editor-adapters/vscode-cursor-settings.json`

Implementation expectations:

- delegate all language intelligence to Ruff LSP
- keep extension-side logic to launch/config + UX glue only

## Neovim

Canonical path:

- command: `ruff lsp`
- sample lspconfig setup: `docs/editor-adapters/neovim-lspconfig.lua`

Implementation expectations:

- one LSP client instance per Ruff workspace root
- no duplicated Ruff syntax intelligence in Neovim Lua

## JetBrains (Generic LSP Plugin Path)

Canonical path:

- command: `ruff lsp`
- setup guide: `docs/editor-adapters/jetbrains-lsp.md`

Implementation expectations:

- map `.ruff` files to Ruff language id/server profile
- leave semantic behavior to server responses

## Smoke Contract

Baseline adapter descriptors are contract-tested in:

- `tests/editor_adapter_contracts.rs`

Smoke scope:

- descriptor files exist
- each descriptor explicitly points to `ruff lsp`
- canonical launch path is consistent across editor families
