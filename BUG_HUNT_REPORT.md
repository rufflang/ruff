# Bug Hunt Report

Date/time of review: 2026-05-31 08:54:43 EDT

## Project Structure Summary

- Main entry points:
  - CLI runtime entry: `src/main.rs`
  - VM/interpreter runtime cores: `src/vm.rs`, `src/interpreter.rs`
  - LSP surfaces: `src/lsp_*.rs`
- Modules:
  - Parsing/lexing/diagnostics: `src/parser.rs`, `src/lexer.rs`, `src/errors.rs`
  - Native capability/security paths: `src/interpreter/native_functions/*`, `src/network_policy.rs`, `src/path_security.rs`
  - Output formatting contracts: `src/cli_output.rs`, `tests/cli_contracts.rs`, `tests/cli_json_contracts.rs`
- Test layout:
  - Unit tests in `src/*`
  - Integration/contract suites in `tests/*`

## Commands Discovered

- Build/test:
  - `cargo test`
  - `cargo test --test cli_contracts`
  - `cargo test --test repo_hygiene_contract`
- Repo/navigation checks used during recon:
  - `git status -sb`
  - `rg --files`
  - `rg -n "..." src tests docs`

## Initial Risk Areas

- CLI/LSP contract boundaries where human/plain output and machine output must remain stable.
- Rename/reference logic that can produce edits resulting in invalid source.
- Security-sensitive execution and IO/network policy paths (reviewed for regressions while triaging).

## Verification Commands Available

- `cargo test`
- `cargo test --test cli_contracts`
- `cargo test --test repo_hygiene_contract`
- `cargo test lsp_rename::tests::...` (targeted unit coverage by filter)

## Limitations / Assumptions

- This pass prioritized actively exercised runtime surfaces and LSP CLI contracts.
- No external network/system dependencies were required for this fix loop.
- Existing pre-v1 deferred TODO items were treated as scope context, not auto-converted into bug fixes without confirmed defect evidence.

---

## Bug: LSP rename allows reserved keyword targets

Status: Fixed  
Severity: Medium  
Area: `src/lsp_rename.rs`, `tests/cli_contracts.rs`  
Type: Logic | Runtime | DX

### Evidence

- `validate_identifier` enforced lexical shape (`[A-Za-z_][A-Za-z0-9_]*`) but did not reject language keywords.
- Example bad path: renaming `value` to `if` succeeded through validation and can produce invalid source text.

### Impact

- `ruff lsp-rename` can emit edit sets that convert valid source into syntactically invalid Ruff code.
- Affects editor workflows and automated refactor tooling relying on LSP rename safety.

### Root Cause

- Validation logic did not confirm that a proposed identifier tokenizes as an identifier token (non-keyword), only that it matches a character-level pattern.

### Fix Plan

- Keep existing fast character checks.
- Add token-level validation using the lexer to require the new name to tokenize exactly as:
  - `Identifier`
  - `Eof`
- Reject all other token kinds with explicit error message: reserved keyword rename is not allowed.
- Add unit and CLI regression coverage.

### Verification

- Added unit test:
  - `src/lsp_rename.rs::tests::rejects_reserved_keyword_name`
- Added CLI integration contract test:
  - `tests/cli_contracts.rs::cli_lsp_rename_keyword_identifier_uses_runtime_error_stderr`
- Ran targeted and full suite:
  - `cargo test --test cli_contracts cli_lsp_rename_keyword_identifier_uses_runtime_error_stderr`
  - `cargo test`

### Result

Fixed.

---

## Bug: Reserved-alias contract test drifted from current CLI behavior

Status: Fixed  
Severity: Low  
Area: `tests/cli_contracts.rs`  
Type: Test | Documentation

### Evidence

- Full regression run surfaced:
  - `cli_reserved_alias_name_is_rejected_before_workflow_routing` expected `ruff doctor` to fail with usage exit code.
- `doctor` is now an explicit first-class CLI command and is correctly expected to succeed in other contract tests in the same file.

### Impact

- Produced false-negative CI failures and obscured real regressions by asserting outdated behavior.

### Root Cause

- Contract test was not updated when `doctor` became a direct subcommand.

### Fix Plan

- Keep reserved-alias rejection coverage but use an actual blocked alias that is not a real subcommand (`dev`).

### Verification

- Updated test assertion path:
  - `tests/cli_contracts.rs::cli_reserved_alias_name_is_rejected_before_workflow_routing`
- Ran targeted command:
  - `cargo test --test cli_contracts cli_reserved_alias_name_is_rejected_before_workflow_routing`

### Result

Fixed.

---

## Bug: Repo hygiene root-file contract drift

Status: Fixed  
Severity: Low  
Area: `docs/REPO_HYGIENE_POLICY.md`, `tests/repo_hygiene_contract.rs`  
Type: Test | Documentation | DX

### Evidence

- Full regression run surfaced:
  - `tracked_root_surface_matches_hygiene_allowlist` mismatch.
- Actual tracked root file set includes `DOGFOOD_NOTES.md`, but the policy/test allowlist omitted it.

### Impact

- Hygiene contract became self-inconsistent with repo reality, causing unnecessary failing gates.

### Root Cause

- Tracked root inventory changed without synchronized policy/test update.

### Fix Plan

- Add `DOGFOOD_NOTES.md` to both:
  - `docs/REPO_HYGIENE_POLICY.md` root contract list
  - `tests/repo_hygiene_contract.rs` expected tracked root list

### Verification

- Targeted:
  - `cargo test --test repo_hygiene_contract`
- Full:
  - `cargo test`

### Result

Fixed.

---

## Re-review Pass Notes

- Performed a post-fix re-review of LSP rename validation and CLI error-path reporting.
- Confirmed no API shape changes were introduced:
  - Existing exit code behavior remains `RuntimeError` for invalid rename requests.
  - Output channel behavior remains stderr for non-JSON rename failures.
- Re-ran end-to-end regression after each additional fix to ensure no hidden contract drift remained.
