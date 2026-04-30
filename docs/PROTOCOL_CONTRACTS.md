# Ruff Protocol Contracts

Status: v0.13.0 baseline

This document defines machine-consumable protocol contracts used by Ruff CLI/LSP surfaces.

## Versioning

Contract version: `0.13.0`

Rules:

- additive optional fields are non-breaking
- field removal/rename/type-change is breaking
- payload-affecting changes must update this document, tests, and changelog

## Diagnostics Contract

Used by:

- `ruff lsp-diagnostics --json`
- LSP `textDocument/publishDiagnostics`

Required fields:

- `line` (number, 1-based for CLI; 0-based inside LSP ranges)
- `column` (number, 1-based for CLI; 0-based inside LSP ranges)
- `severity` (string for CLI, numeric in LSP notification)
- `message` (string)

## Symbol Metadata Contract

Used by:

- `ruff lsp-definition --json`
- `ruff lsp-references --json`
- `ruff lsp-hover --json`
- LSP `textDocument/documentSymbol`
- LSP `workspace/symbol`

Required symbol identity fields by surface:

- symbol `name`/`label`
- location coordinates (line/column or range)
- optional kind metadata (`function`, `variable`, etc)

## Edit Contract

Used by:

- `ruff lsp-rename --json`
- LSP `textDocument/rename`
- LSP `textDocument/formatting`
- LSP `textDocument/rangeFormatting`
- LSP `textDocument/codeAction`

Required edit metadata:

- target location/range
- replacement text
- deterministic edit ordering for identical input

## Error Contract

LSP protocol errors follow JSON-RPC standard envelope:

- `code`
- `message`

Ruff-specific codes currently used:

- `-32601` method not found
- `-32602` invalid params
- `-32800` request cancelled
- `-32001` request timeout

## Validation Sources

Primary validation is enforced by tests:

- `tests/cli_json_contracts.rs`
- `tests/lsp_conformance_harness.rs`
