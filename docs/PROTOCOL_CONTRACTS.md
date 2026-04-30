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

Golden fixture set (shape-locked contracts):

- `tests/lsp_fixtures/all_required_methods_success_error.json`
- `tests/lsp_fixtures/completion_ordering.json`
- `tests/lsp_fixtures/edit_range_stability.json`
- `tests/lsp_fixtures/error_payload_consistency.json`
- `tests/lsp_fixtures/multi_file_workspace_symbol_rename_references.json`

## LSP Method Compatibility Table

The table below is the canonical support matrix for Ruff's LSP server at this contract baseline.

| Method | Status | Contract Notes |
| --- | --- | --- |
| `initialize` | Supported | Returns capability set and server version. |
| `shutdown` | Supported | Returns `null` result and marks clean exit path. |
| `textDocument/didOpen` | Supported | Stores in-memory document and emits diagnostics. |
| `textDocument/didChange` | Supported | Applies latest content change and emits diagnostics. |
| `textDocument/didClose` | Supported | Removes document and emits empty diagnostics. |
| `textDocument/completion` | Supported | Returns deterministic item ordering for identical source/position. |
| `textDocument/hover` | Supported | Returns markdown content and symbol range or `null`. |
| `textDocument/definition` | Supported | Returns location in target document or `null`. |
| `textDocument/references` | Supported | Returns declaration-aware references for the requested document. |
| `textDocument/rename` | Supported | Returns text edits under `result.changes[uri]`. |
| `textDocument/codeAction` | Supported | Returns syntax quick-fix actions derived from diagnostics. |
| `textDocument/formatting` | Supported | Returns full-document edit list or empty list if unchanged. |
| `textDocument/rangeFormatting` | Supported | Currently returns full-document-style edit behavior. |
| `textDocument/documentSymbol` | Supported | Returns symbol list for requested open/resolved document. |
| `workspace/symbol` | Supported | Returns workspace symbols across open documents; output is URI-sorted for deterministic fixtures. |
| `$/cancelRequest` | Supported | Cancellation IDs map to JSON-RPC cancelled error envelope. |
| `initialized` | Supported | Notification accepted as no-op. |
| `exit` | Supported | Triggers process exit code based on shutdown sequence. |

Unsupported method behavior:

- Any unknown request method returns a JSON-RPC error envelope with:
	- `code: -32601`
	- `message: Method '<method>' is not supported`
- Requests with missing required parameters return:
	- `code: -32602`
	- stable method-specific error message (for example: `Missing textDocument.uri`)
