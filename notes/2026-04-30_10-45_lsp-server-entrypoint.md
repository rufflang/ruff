# v0.13.0 Official Ruff LSP Server Entrypoint Evidence

Date: 2026-04-30
Track: v0.13.0 Cross-IDE Foundation
Checklist item: Official Ruff LSP Server

## Implemented

- Added `ruff lsp` long-running JSON-RPC server entrypoint over stdio.
- Implemented lifecycle handling for:
  - `initialize`
  - `initialized`
  - `shutdown`
  - `exit`
- Added deterministic stderr log mode:
  - `ruff lsp --deterministic-logs`
- Wired server handlers to shared analysis modules (no editor-specific analysis logic):
  - completion
  - hover
  - definition
  - references
  - rename
  - code actions
  - diagnostics publication

## Verification

Command:

`cargo test lsp_server`

Result:

- PASS
- `lsp_server` module tests passed in both lib and main test targets.

## Remaining Acceptance Gap

- External-client smoke validation across at least two clients remains to be recorded as release evidence.
