# Ruff Development Roadmap

This roadmap tracks work that is still current or upcoming. Completed features and implementation history belong in [CHANGELOG.md](CHANGELOG.md), not here.

> Current crate version: `0.11.0` in [Cargo.toml](Cargo.toml)
> Next planned release: `v0.13.0`
> Last audited: April 30, 2026

---

## Release Focus

`v0.11.0` has been released and `v0.12.0` roadmap work is complete.
Roadmap planning is now focused on `v0.13.0` cross-IDE language tooling and `v1.0.0` readiness prerequisites.

For historical `v0.11.0` release evidence and completion details, see [CHANGELOG.md](CHANGELOG.md) and the dated notes under `notes/`.

---

## v0.12.0: Developer Experience

`v0.12.0` is the active roadmap cycle after the `v0.11.0` performance release.

Priority work:

1. **Language Server Protocol**

   Status: in progress.

   Planned features:

   - [x] autocomplete for builtins, variables, and functions (initial completion engine via `ruff lsp-complete`)
   - [x] go to definition (initial symbol-definition lookup via `ruff lsp-definition`)
   - [x] find references (initial symbol-reference lookup via `ruff lsp-references`)
   - [x] hover documentation (initial hover symbol details via `ruff lsp-hover`)
   - [x] real-time diagnostics (initial syntax diagnostics via `ruff lsp-diagnostics`)
   - [x] rename refactoring (initial symbol rename edits via `ruff lsp-rename`)
   - [x] code actions (initial syntax quick-fixes via `ruff lsp-code-actions`)

2. **Formatter**

   Planned features:

   - [x] opinionated formatting (initial spacing/indentation normalization via `ruff format`)
   - [x] configurable indentation (`ruff format --indent <N>`)
   - [x] line-length policy (`ruff format --line-length <N>` wrapping for comma-separated expressions)
   - [x] import ordering once module semantics are stable (initial leading import-block sorting; disable with `--no-sort-imports`)

3. **Linter**

   Planned rules:

   - [x] unused variables (initial token-based declaration/use checks)
   - [x] unreachable code (initial post-terminator statement checks)
   - [x] obvious type mismatches (initial annotation-literal mismatch checks)
   - [x] missing error-handling patterns (initial fallible-call pattern checks)
   - [x] auto-fix for safe rules (initial unused-variable underscore-prefix fix)

4. **Package/project workflow**

   Planned features:

   - [x] `ruff.toml` (initial manifest generation and parsing)
   - [x] dependency metadata (initial dependency table support)
   - [x] `ruff init` (project scaffold generation with `src/main.ruff`)
   - [x] package install/add/publish workflow (initial `package-add`, `package-install`, and `package-publish` command surfaces)

5. **REPL improvements**

   Planned features:

   - [x] tab completion (initial builtin + command completion in `ruff repl`)
   - [x] syntax highlighting (initial command-line highlighting in `ruff repl`)
   - [x] stronger multi-line editing (validator-backed incomplete input detection)
   - [x] `.help <function>` documentation (initial builtin help surface)

6. **Documentation generator**

   Planned features:

   - [x] HTML docs from `///` comments (initial `ruff docgen` module-function page generation)
   - [x] examples extracted from doc comments (initial fenced-code example extraction)
   - [x] builtin/native API reference generation (initial builtin registry HTML reference)

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

   - [ ] versioned language-spec baseline document (`docs/LANGUAGE_SPEC.md`) covering syntax, runtime semantics, and compatibility guarantees
   - [ ] machine-consumable protocol contracts for diagnostics, symbol metadata, and edits used by CLI/LSP outputs
   - [ ] compatibility policy document for breaking vs non-breaking language-tooling changes

   Acceptance criteria:

   - a new test fixture can validate compatibility of structured output fields across versions
   - at least one CI job fails when output contracts regress without an explicit contract update

2. **Official Ruff LSP server binary**

   Planned features:

   - [ ] single `ruff lsp` server entrypoint supporting standard JSON-RPC transport
   - [ ] diagnostics, completion, hover, definition, references, rename, and code actions implemented through server handlers
   - [ ] formatter/linter/doc features exposed through LSP methods where applicable

   Acceptance criteria:

   - LSP protocol-level integration tests cover startup, initialization, and each core request type
   - cross-client smoke validation passes in at least two LSP-capable editors

3. **Deterministic machine-readable CLI outputs**

   Planned features:

   - [ ] consistent `--json` output shape coverage for format/lint/docgen/LSP CLI commands
   - [ ] explicit schema tests for error and success payloads
   - [ ] stable exit-code policy documentation for scripting and IDE task runners

   Acceptance criteria:

   - JSON output snapshots exist for key commands and are enforced in CI
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

   - [ ] fixture-driven protocol test harness for request/response/diagnostic behavior
   - [ ] deterministic snapshots for completion ordering, symbol locations, and edit ranges
   - [ ] regression tests for failure/error payload consistency

   Acceptance criteria:

   - all core LSP feature handlers have protocol-level test coverage
   - tests detect incompatible response-shape changes before release

### P2 (Can follow)

6. **Thin editor adapters**

   Planned features:

   - [ ] lightweight VS Code/Cursor adapter that launches Ruff LSP without reimplementing language logic
   - [ ] adapter guidance for JetBrains (via generic LSP plugin), Neovim, and other LSP clients
   - [ ] editor setup docs focused on shared Ruff LSP configuration

   Acceptance criteria:

   - adapter repos/extensions avoid duplicate parsing/analysis logic
   - onboarding docs show one canonical Ruff LSP configuration path per editor family

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
