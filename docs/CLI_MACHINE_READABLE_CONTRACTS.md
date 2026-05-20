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

Exception for `ruff run --json-runtime-diagnostics`:

- runtime/VM execution failures emit a machine-readable JSON error payload on `stdout`
- `stderr` is suppressed for those JSON-mode runtime failures
- non-zero exit code remains authoritative (`4` for runtime failures, `6` for internal failures in JSON serialization/panic paths)

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

### `ruff check --json`

Top-level object fields:

- `command` (string, constant `"check"`)
- `file` (string)
- `status` (string, currently `"ok"`)
- `statement_count` (number)
- `bytecode_instruction_count` (number)

### `ruff docgen --json`

Top-level object fields:

- `command` (string, constant `"docgen"`)
- `file` (string)
- `output_dir` (string)
- `module_doc_path` (string)
- `builtin_doc_path` (string or null)
- `item_count` (number)
- `languages` (array of strings)
- `project_json_path` (string)
- `gaps_json_path` (string)
- `capabilities_json_path` (string)
- `ai_tasks_path` (string or null)
- `diagnostics_count` (number)
- `undocumented_count` (number)
- `broken_link_count` (number)
- `warning_count` (number)
- `gate_failures` (array of strings)
- `discovery_skip_counts` (object)
  - `max_file_size` (number)
  - `max_depth` (number)
  - `max_files` (number)
  - `invalid_encoding` (number)
- `discovery_limits` (object)
  - `max_file_size_bytes` (number)
  - `max_depth` (number)
  - `max_files` (number)
- `link_validation_skip_counts` (object)
  - `max_link_checks` (number)
  - `max_external_checks` (number)
  - `max_total_time` (number)
- `summary` (object)
  - `schema_version` (string, constant `"docgen-summary/v1"`)
  - includes mirrored totals plus `discovery_limits`, `discovery_skip_counts`, and `link_validation_skip_counts`

Implementation note:
- The `ruff docgen --json` payload is emitted from a typed single-source builder in `src/docgen/core.rs` (`build_cli_json_payload`) and guarded by both shape assertions and a fixture-backed snapshot test in `tests/cli_json_contracts.rs`.

### `ruff run --json-runtime-diagnostics`

Top-level object fields:

- `command` (string, constant `"run"`)
- `status` (string, constant `"error"`)
- `kind` (string, constant `"runtime_diagnostic"`)
- `contract_version` (string, currently `"1.0.0-draft"`)
- `exit_code` (number, usually `4` for runtime/VM execution failures)
- `diagnostic` (object)
  - shape matches the shared diagnostic JSON contract fields:
    - `code`, `severity`, `subsystem`, `message`, `help`, `file`, `line`, `column`
- `runtime_kind` (string, optional; present when runtime error kind metadata is available)
- `call_stack` (array of strings, optional; present for runtime errors surfaced with stack context)

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
