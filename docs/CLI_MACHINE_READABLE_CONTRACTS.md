# CLI Machine-Readable Contracts

Status: v1.0.0 baseline draft (active)
Contract version: `1.0.0-draft`
Last updated: 2026-05-16

This document defines automation-facing contracts for Ruff CLI JSON outputs and exit behavior.

## Exit-Code Policy

Ruff user-facing commands follow this policy:

- `0`: command completed successfully
- `1`: command completed with a generic command failure or unmet gate (for example `format --check`, `lint` errors, failed `test-run` assertions, benchmark throughput gate failures)
- `2`: command-line usage/argument parse error (Clap-level usage failure)
- `3`: lexical/parser diagnostic failure
- `4`: runtime execution/semantic failure
- `5`: IO failure (for example missing input file, read/write/create failure)
- `6`: internal/tooling failure (for example unexpected runtime panic surfaces or JSON serialization failure)

Notes:

- Commands that intentionally gate behavior (for example format check mode) use `1` when the requested gate is not met.
- For automation, treat any non-zero exit as failure unless a command-specific policy explicitly documents otherwise.
- `tests/cli_contracts.rs` and `tests/cli_json_contracts.rs` lock these exit-code contracts.

## Error Shape Policy

Current v1.0.0 baseline draft for CLI `--json` mode:

- `stdout` is reserved for machine-readable JSON payloads on successful command execution.
- `stderr` carries human-readable error text for failures.
- A non-zero exit code indicates failure.

Automation recommendation:

- Read JSON only from `stdout` on zero exit.
- Treat non-zero exit as authoritative failure signal.
- Capture `stderr` for troubleshooting and classification.

## Stable JSON Output Surfaces

The following command families are covered by schema-gating integration tests in `tests/cli_json_contracts.rs`.

### `ruff format --json`

Top-level object fields:

- `command` (string, constant `"format"`)
- `file` (string)
- `status` (string: `"preview" | "already_formatted" | "needs_formatting" | "written"`)
- `changed` (boolean)
- `options` (object)
  - `indent` (number)
  - `line_length` (number)
  - `sort_imports` (boolean)
  - `check` (boolean)
  - `write` (boolean)
- `formatted_source` (string or null)

### `ruff lint --json`

Top-level array of issue objects. Per item fields:

- `rule_id` (string)
- `line` (number)
- `column` (number)
- `severity` (string)
- `message` (string)
- `fix` (object or null)

### `ruff docgen --json`

Top-level object fields:

- `command` (string, constant `"docgen"`)
- `file` (string)
- `output_dir` (string)
- `module_doc_path` (string)
- `builtin_doc_path` (string or null)
- `item_count` (number)

### LSP CLI helper surfaces (`--json`)

Covered commands:

- `lsp-complete`
- `lsp-definition`
- `lsp-references`
- `lsp-hover`
- `lsp-diagnostics`
- `lsp-rename`
- `lsp-code-actions`

Each command has a stable top-level payload kind (array/object/null as applicable) with required field assertions in `tests/cli_json_contracts.rs`.

`ruff lsp-diagnostics --json` item fields:

- `code` (string, stable diagnostic code such as `RUFLEX001` or `RUFPARSE001`)
- `severity` (string, currently `"error"`)
- `subsystem` (string, one of `lexer`, `parser`, `lsp`)
- `message` (string)
- `line` (number)
- `column` (number)
- `file` (string or null)
- `help` (string or null)

## Contract Change Rules

Any payload-affecting change to the documented JSON shapes requires all of the following in the same change set:

- update this document
- update `tests/cli_json_contracts.rs` contract assertions
- add/update a `CHANGELOG.md` contract-impact note

## Negative-Path Fixture Guarantees

`tests/cli_json_contracts.rs::cli_json_negative_paths_have_stable_failure_signals` locks these failure-path automation guarantees:

- missing input files for JSON-mode `format`/`lint` exit with code `5`, emit no JSON payload on `stdout`, and report a deterministic read-failure message on `stderr`
- malformed CLI parameters (for example non-numeric `--line`) exit with code `2`, emit no JSON payload on `stdout`, and return Clap usage diagnostics on `stderr`
- unknown-symbol rename requests exit with code `4`, emit no JSON payload on `stdout`, and emit deterministic symbol-resolution failure text on `stderr`
