# Ruff Development Roadmap

This roadmap tracks work that is still current or upcoming. Completed features and implementation history belong in [CHANGELOG.md](CHANGELOG.md), not here.

> Current crate version: `0.12.0` in [Cargo.toml](Cargo.toml)
> Next planned release: `v0.13.0`
> Last audited: April 30, 2026

---

## Release Focus

`v0.11.0` has been released and `v0.12.0` roadmap work is complete.
Roadmap planning is now focused on `v0.13.0` cross-IDE language tooling and `v1.0.0` readiness prerequisites.

For historical `v0.11.0` release evidence and completion details, see [CHANGELOG.md](CHANGELOG.md) and the dated notes under `notes/`.

---

## v0.13.0: Cross-IDE Foundation (Release Checklist)

`v0.13.0` is complete only when all required items below are done.

### 1. Language Specification And Compatibility Policy

Required features:

- [x] publish `docs/LANGUAGE_SPEC.md` with versioned grammar and runtime semantics
- [x] define compatibility guarantees for syntax, runtime behavior, and CLI/LSP machine-readable output
- [x] publish breaking-change policy and versioning rules for language/tooling contracts

Acceptance criteria:

- [x] spec review sign-off captured in release notes
- [x] compatibility policy linked from README and release docs

### 2. Official Ruff LSP Server

Required features:

- [x] add `ruff lsp` long-running JSON-RPC server entrypoint
- [x] implement LSP initialize/shutdown/exit lifecycle and capability negotiation
- [x] support both stdio transport and deterministic logging mode for debugging
- [x] wire server handlers to shared analysis logic (not editor-specific code)

Acceptance criteria:

- [x] protocol startup tests pass for initialize, initialized, shutdown, and exit
- [ ] server can be launched by at least two external LSP clients without code changes

### 3. LSP Feature Parity (Required For v0.13.0)

Required features:

- [x] textDocument/publishDiagnostics
- [x] textDocument/completion
- [x] textDocument/hover
- [x] textDocument/definition
- [x] textDocument/references
- [x] textDocument/rename
- [x] textDocument/codeAction
- [x] textDocument/formatting and textDocument/rangeFormatting
- [x] textDocument/documentSymbol
- [x] workspace/symbol

Acceptance criteria:

- [ ] each required method has passing protocol-level fixtures for success and error cases
- [ ] response payloads are stable and versioned where applicable

### 4. CLI And Machine-Readable Contract Hardening

Required features:

- [x] enforce stable `--json` output schemas for `format`, `lint`, `docgen`, and LSP CLI surfaces
- [x] standardize exit-code policy for all user-facing commands
- [x] add explicit error-shape documentation for automation use cases

Acceptance criteria:

- [x] snapshot/schema tests gate output contract changes in CI
- [ ] CHANGELOG policy requires contract-change notes for payload-affecting changes

### 5. Tree-sitter Grammar For Universal Highlighting

Required features:

- [ ] create `tree-sitter-ruff` grammar crate/repo with syntax coverage for current Ruff language constructs
- [ ] add corpus tests for core grammar and edge-case constructs
- [ ] add highlight/query files sufficient for editor consumption

Acceptance criteria:

- [ ] corpus tests run in CI and block regressions
- [ ] at least one editor integration confirms `.ruff` highlighting via Tree-sitter grammar

### 6. Conformance Test Harness

Required features:

- [x] build fixture-driven protocol harness for LSP request/response validation
- [x] add deterministic fixtures for completion ordering and edit-range stability
- [x] add regression fixtures for diagnostics and error payload consistency

Acceptance criteria:

- [ ] harness runs in CI across Linux and macOS
- [ ] incompatible protocol changes fail tests by default

### 7. Performance, Reliability, And Crash Safety

Required features:

- [ ] baseline and track latency for diagnostics/completion/hover on representative code samples
- [ ] add cancellation and timeout handling for long-running analysis requests
- [ ] ensure server handles malformed requests and parse failures without panicking

Acceptance criteria:

- [ ] no known panic paths in LSP request handling under fuzz/invalid-input tests
- [ ] performance guardrails documented and validated in CI/perf job

### 8. Packaging And Distribution

Required features:

- [ ] provide release artifacts that include `ruff lsp` functionality
- [ ] document install/upgrade path for users integrating Ruff with editors
- [ ] verify binary compatibility for supported target platforms

Acceptance criteria:

- [ ] release checklist confirms LSP entrypoint availability in shipped artifacts
- [ ] install docs are validated in a clean-environment smoke test

### 9. Thin Editor Adapter Baselines

Required features:

- [x] publish minimal VS Code/Cursor adapter guidance that launches official Ruff LSP
- [x] publish setup guidance for Neovim and JetBrains (via LSP plugin path)
- [x] keep adapter docs free of duplicated parser/analyzer implementation details

Acceptance criteria:

- [x] one canonical setup path per editor family is documented and smoke-tested
- [x] adapter guidance points to shared Ruff contracts and server behavior docs

### 10. Release Evidence And Completion Gate

Required features:

- [ ] add `v0.13.0` completion checklist artifact under `notes/` with command/test evidence
- [ ] add changelog release summary entries for all shipped `v0.13.0` tracks
- [ ] define post-release follow-up list for deferred `v1.0.0` items

Acceptance criteria:

- [ ] roadmap checkboxes for required `v0.13.0` items are complete
- [ ] release tag is created only after evidence checklist is signed off

---

## v1.0.0 Readiness

`v1.0.0` should not be planned in detail until `v0.11.0` and `v0.12.0` are complete.

Required before `v1.0.0`:

- `v0.11.0` performance release complete (done).
- `v0.12.0` developer tooling substantially complete.
- Stable language/runtime API policy.
- Current, accurate user documentation.
- Clear compatibility policy for native builtins and CLI output contracts.

Possible post-`v0.12.0` design tracks:

- generic types
- union types
- enum methods
- macros/metaprogramming
- FFI
- WebAssembly target
- ML/AI libraries

---

## v0.13.0: Cross-IDE Foundation (Execution Plan)

`v0.13.0` is focused on making Ruff first-class across editor ecosystems by prioritizing universally reusable language tooling over editor-specific logic.

### P0 (Must ship)

1. **Canonical language/tooling contract**

   Planned features:

   - [x] versioned language-spec baseline document (`docs/LANGUAGE_SPEC.md`) covering syntax, runtime semantics, and compatibility guarantees
   - [ ] machine-consumable protocol contracts for diagnostics, symbol metadata, and edits used by CLI/LSP outputs
   - [x] compatibility policy document for breaking vs non-breaking language-tooling changes

   Acceptance criteria:

   - a new test fixture can validate compatibility of structured output fields across versions
   - at least one CI job fails when output contracts regress without an explicit contract update

2. **Official Ruff LSP server binary**

   Planned features:

   - [x] single `ruff lsp` server entrypoint supporting standard JSON-RPC transport
   - [x] diagnostics, completion, hover, definition, references, rename, and code actions implemented through server handlers
   - [x] formatter/linter/doc features exposed through LSP methods where applicable

   Acceptance criteria:

   - LSP protocol-level integration tests cover startup, initialization, and each core request type
   - cross-client smoke validation passes in at least two LSP-capable editors

3. **Deterministic machine-readable CLI outputs**

   Planned features:

   - [x] consistent `--json` output shape coverage for format/lint/docgen/LSP CLI commands
   - [x] explicit schema tests for error and success payloads
   - [x] stable exit-code policy documentation for scripting and IDE task runners

   Acceptance criteria:

   - [x] JSON output snapshots exist for key commands and are enforced in CI
   - command output contract changes require changelog + schema update in one PR

### P1 (Should ship)

4. **Tree-sitter grammar for universal highlighting**

   Planned features:

   - [ ] `tree-sitter-ruff` grammar with coverage for current language syntax
   - [ ] grammar corpus tests for language constructs and edge cases
   - [ ] published query files for highlighting/injections where applicable

   Acceptance criteria:

   - corpus tests pass in CI
   - at least one editor client can consume the grammar for `.ruff` highlighting

5. **Language-server conformance test suite**

   Planned features:

   - [x] fixture-driven protocol test harness for request/response/diagnostic behavior
   - [x] deterministic snapshots for completion ordering, symbol locations, and edit ranges
   - [x] regression tests for failure/error payload consistency

   Acceptance criteria:

   - [x] all core LSP feature handlers have protocol-level test coverage
   - [x] tests detect incompatible response-shape changes before release

### P2 (Can follow)

6. **Thin editor adapters**

   Planned features:

   - [x] lightweight VS Code/Cursor adapter that launches Ruff LSP without reimplementing language logic
   - [x] adapter guidance for JetBrains (via generic LSP plugin), Neovim, and other LSP clients
   - [x] editor setup docs focused on shared Ruff LSP configuration

   Acceptance criteria:

   - [x] adapter repos/extensions avoid duplicate parsing/analysis logic
   - [x] onboarding docs show one canonical Ruff LSP configuration path per editor family

---

## Version Strategy

- `v0.11.0`: released (SSG throughput, async scheduler reliability, benchmark release evidence).
- `v0.12.0`: developer experience and project tooling (complete).
- `v0.13.0`: cross-IDE language tooling foundation (`ruff lsp`, output contracts, grammar, conformance tests).
- `v1.0.0`: stabilization, documentation, compatibility policy, ecosystem polish.

See also:

- [CHANGELOG.md](CHANGELOG.md): completed changes.
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md): performance guide.
- [docs/CONCURRENCY.md](docs/CONCURRENCY.md): concurrency notes.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): architecture notes. Some sections may be stale and should be reviewed before release.
