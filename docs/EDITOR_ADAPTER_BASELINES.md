# Editor Adapter Baselines (v0.13.0)

This document defines canonical thin-adapter setup paths for Ruff editor integrations.

Adapter rule:

- editor adapters must launch the official `ruff lsp` server
- adapters must not duplicate parser/analyzer/runtime logic
- shared behavior contracts belong to Ruff server/CLI docs, not per-editor forks

## Adapter Maintenance Policy

Ruff repository responsibilities:

- canonical protocol and machine-readable contracts (`docs/PROTOCOL_CONTRACTS.md`, `docs/CLI_MACHINE_READABLE_CONTRACTS.md`)
- minimal adapter baseline docs and launch examples under `docs/editor-adapters/`
- first-party extension baseline assets under `tools/vscode-ruff-extension/`

Editor-specific repository responsibilities:

- editor UX polish and host-specific packaging details
- editor release cadence/versioning that does not alter Ruff protocol contracts
- optional integrations that remain outside canonical Ruff CLI/LSP guarantees

Policy constraint:

- adapter docs in Ruff must stay thin and must link back to canonical Ruff contracts instead of duplicating protocol semantics.

## VS Code / Cursor / Codex-Compatible Editors

Canonical path:

- extension baseline: `tools/vscode-ruff-extension/`
- command: `ruff lsp`
- sample workspace settings: `docs/editor-adapters/vscode-cursor-settings.json`

Implementation expectations:

- `.ruff` files are mapped to Ruff language id and syntax scope so code is colorized on open
- delegate all language intelligence to Ruff LSP
- keep extension-side logic to launch/config + UX glue only

Notes:

- The first-party extension baseline contributes Ruff language registration, TextMate grammar highlighting, and optional Ruff LSP client startup.
- VS Code forks (for example Codex-compatible builds) can consume the same `.vsix` package path.

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
