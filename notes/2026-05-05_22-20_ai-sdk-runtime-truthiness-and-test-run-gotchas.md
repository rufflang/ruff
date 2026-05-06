# Ruff Field Notes — AI SDK runtime truthiness and test-run gotchas

**Date:** 2026-05-05
**Session:** 22:20 local
**Branch/Commit:** main / c1aad26
**Scope:** Stabilized Ruff runtime behavior that surfaced while hardening the Ruff AI SDK contract tests. Fixed control-flow truthiness edge cases, validated module/export and closure behavior, and captured test-run constraints that can mislead debugging.

---

## What I Changed
- Fixed integer truthiness in control-flow evaluation so `0` is treated as false in both interpreter paths:
  - `src/interpreter/mod.rs`
  - `src/interpreter/legacy_full.rs`
- Added a regression test for the integer-truthiness contract:
  - `tests/interpreter_tests.rs` (`test_integer_zero_is_falsey_in_if_conditions`)
- Validated and retained earlier runtime hardening from this same workstream:
  - module export function environment binding for imported exported functions (`src/module.rs`)
  - explicit `null` equality/inequality semantics in binary operations (`src/interpreter/mod.rs`, `src/interpreter/legacy_full.rs`)
  - closure argument evaluation order fix (evaluate call args in caller scope first) (`src/interpreter/mod.rs`, `src/interpreter/legacy_full.rs`)
- Updated SDK contract test assertion semantics in the SDK workspace to align with Ruff builtin behavior:
  - `../ruff-ai-sdk/tests/sdk_contract_tests.ruff`

## Gotchas (Read This Next Time)
- **Gotcha:** `has_key(...)` returns numeric truthy/falsy (`1`/`0`), not booleans.
  - **Symptom:** Test assertions using `assert_true(has_key(...))` fail with `assert_true requires a boolean argument`.
  - **Root cause:** Ruff test assertions are strict about boolean types; numeric truthy values are not auto-accepted.
  - **Fix:** Use `assert_equal(has_key(obj, key), 1)` when asserting key presence in tests.
  - **Prevention:** Treat builtin contracts as type-specific, not truthy/falsy-compatible by default.

- **Gotcha:** Before this fix, `if`/`while` conditions treated `Int(0)` as truthy.
  - **Symptom:** Missing-key branches unexpectedly executed as if `has_key(...)` succeeded, and fallback paths were skipped.
  - **Root cause:** Control-flow truthiness handled `Bool`, `Float`, `Str`, `Array`, and `Dict`, but not `Int`; unmatched variants fell through to truthy.
  - **Fix:** Added explicit `Value::Int(n) => n != 0` in `Stmt::If` and `Stmt::While` truthiness checks in both interpreter implementations.
  - **Prevention:** Any truthiness logic update must patch both `mod.rs` and `legacy_full.rs`, then add a focused regression test.

- **Gotcha:** `ruff test-run` does not execute top-level imports/statements before tests.
  - **Symptom:** Test symbols appear undefined unless imported inside setup; test files can pass in `run` but fail in `test-run`.
  - **Root cause:** Test runner executes `test_setup` and test blocks, not full file top-level evaluation semantics.
  - **Fix:** Move imports needed by tests into `test_setup`.
  - **Prevention:** Author test files with `test_setup` as the required import/bootstrap boundary.

- **Gotcha:** Type-check warnings about imported functions being undefined can still appear even when runtime execution is correct.
  - **Symptom:** Warnings like `Undefined function 'openai_provider'` on valid module-import usage in script runs.
  - **Root cause:** Static analysis path and runtime module resolution path are not fully parity-aligned for this usage pattern.
  - **Fix:** None for runtime correctness; validated behavior through execution and contract tests.
  - **Prevention:** Do not treat these warnings as automatic runtime blockers; confirm with `ruff run`/`ruff test-run` execution results.

## Things I Learned
- Control-flow truthiness is a runtime contract surface, not just a convenience detail; one missing variant can invalidate unrelated feature work.
- Ruff currently has dual interpreter surfaces (`mod.rs` and `legacy_full.rs`) that must be kept behaviorally in lockstep for bug fixes.
- For Ruff tests, assertion type strictness and builtin return types must be treated as first-class compatibility constraints.
- Rule: when `test-run` behavior differs from `run`, verify initialization/import timing before changing runtime code.

## Debug Notes (Only if applicable)
- **Failing test / error:**
  - `Error: assert_true requires a boolean argument`
  - `Error: assert_false requires a boolean argument`
- **Repro steps:**
  - `cd /Users/robertdevore/2026/ruff-ai-sdk`
  - `/Users/robertdevore/2026/ruff/target/release/ruff test-run tests/sdk_contract_tests.ruff --verbose`
- **Breakpoints / logs used:**
  - direct runtime probes with small Ruff scripts printing `type(...)` and values for `provider_supports(...)`
  - source inspection of truthiness branches in `eval_stmt` for `Stmt::If` and `Stmt::While`
- **Final diagnosis:**
  - one failure was assertion contract mismatch (`has_key` type is int)
  - one failure was actual runtime bug (`Int(0)` incorrectly truthy in condition evaluation)

## Follow-ups / TODO (For Future Agents)
- [ ] Add or update standard library reference docs to explicitly state `has_key(...)` return type contract (`1`/`0`) in test-facing examples.
- [ ] Evaluate whether static type-checker/module-resolution warnings for valid imports can be reduced without breaking existing analysis guarantees.

## Links / References
- Files touched:
  - `src/interpreter/mod.rs`
  - `src/interpreter/legacy_full.rs`
  - `tests/interpreter_tests.rs`
  - `../ruff-ai-sdk/tests/sdk_contract_tests.ruff`
- Related docs:
  - `README.md`
  - `ROADMAP.md`
  - `notes/GOTCHAS.md`
  - `notes/FIELD_NOTES_SYSTEM.md`
