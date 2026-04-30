# Ruff Development Roadmap

This roadmap tracks work that is still current or upcoming. Completed features and implementation history belong in [CHANGELOG.md](CHANGELOG.md), not here.

> Current crate version: `0.11.0` in [Cargo.toml](Cargo.toml)
> Next planned release: `v0.12.0`
> Last audited: April 29, 2026

---

## Release Focus

`v0.11.0` has been released.
Roadmap planning is now focused on `v0.12.0` developer-experience work and `v1.0.0` readiness prerequisites.

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

   - tab completion
   - syntax highlighting
   - stronger multi-line editing
   - `.help <function>` documentation

6. **Documentation generator**

   Planned features:

   - HTML docs from `///` comments
   - examples extracted from doc comments
   - builtin/native API reference generation

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

## Version Strategy

- `v0.11.0`: released (SSG throughput, async scheduler reliability, benchmark release evidence).
- `v0.12.0`: developer experience and project tooling.
- `v1.0.0`: stabilization, documentation, compatibility policy, ecosystem polish.

See also:

- [CHANGELOG.md](CHANGELOG.md): completed changes.
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md): performance guide.
- [docs/CONCURRENCY.md](docs/CONCURRENCY.md): concurrency notes.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): architecture notes. Some sections may be stale and should be reviewed before release.
