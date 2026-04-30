# CLI Machine-Readable Contracts

Status: v0.13.0 baseline
Last updated: 2026-04-30

This document defines automation-facing contracts for Ruff CLI JSON outputs and exit behavior.

## Exit-Code Policy

Ruff user-facing commands follow this policy:

- `0`: command completed successfully
- `1`: command completed with runtime/validation/tooling failure (including failed `--check` style gates)
- `2`: command-line usage/argument parse error (Clap-level usage failure)

Notes:

- Commands that intentionally gate behavior (for example format check mode) use `1` when the requested gate is not met.
- For automation, treat any non-zero exit as failure unless a command-specific policy explicitly documents otherwise.

## Error Shape Policy

Current v0.13.0 baseline for CLI `--json` mode:

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

## Contract Change Rules

Any payload-affecting change to the documented JSON shapes requires all of the following in the same change set:

- update this document
- update `tests/cli_json_contracts.rs` contract assertions
- add/update a `CHANGELOG.md` contract-impact note
