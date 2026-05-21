# Ruff

Ruff is a small programming language and runtime implemented in Rust. It is built for local scripting, automation, runtime experiments, and benchmarking work where a compact language with a Rust-hosted standard library is useful.

The project is currently at `1.0.0` in `Cargo.toml`, after completing the stabilization and v1-runway release cycle. Ruff is usable from source today, but the language and runtime APIs are still evolving. Treat the repository tests, examples, and native-function dispatch tests as the source of truth for current behavior.

## Current Status

- Ruff can be installed from tagged release artifacts or built from source with Cargo.
- `ruff run` uses the bytecode VM by default.
- `ruff run --interpreter` runs the tree-walking interpreter fallback.
- The runtime includes a broad native standard library for strings, collections, files, data formats, HTTP, databases, crypto, process/system helpers, concurrency, and network primitives.
- Optional type annotations are parsed. In the CLI, type-checking warnings are emitted on the interpreter path; VM execution does not currently enforce a static type gate before running.
- Some advanced language surfaces are experimental or have runtime-mode gaps. See [Known Boundaries](#known-boundaries).

## 1.0 Readiness Status

- Ruff is not yet ready for a `1.0.0` release.
- [ROADMAP.md](ROADMAP.md) is the single source of truth for release readiness and blocker tracking.
- Ruff `1.0.0` must not be released until all P0/P1 roadmap items and the final release checklist are complete.
- Canonical readiness boundary: Ruff remains pre-1.0 until `ROADMAP.md` and `docs/PRE_V1_MASTER_UNFINISHED_CHECKLIST.md` release gates are closed.
- Deferred/non-goal boundaries are tracked in [docs/V1_SCOPE.md](docs/V1_SCOPE.md) and [docs/OPTIONAL_TYPING_DESIGN.md](docs/OPTIONAL_TYPING_DESIGN.md).

## Safety Model Snapshot

- Ruff is not a sandbox.
- `ruff run` and `ruff test-run` default to trusted mode (host-effect APIs enabled).
- For untrusted code, start with `--untrusted` and add only required `--allow-*` flags.
- Review [docs/NATIVE_API_SECURITY_POSTURE.md](docs/NATIVE_API_SECURITY_POSTURE.md) before running untrusted scripts or shared-environment automation.

## Core Reference Links

- [ROADMAP.md](ROADMAP.md) (release readiness and blockers)
- [docs/LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md) (language/runtime contract baseline)
- [docs/STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md) (native API inventory + capability mapping)
- [docs/RELEASE_PROCESS.md](docs/RELEASE_PROCESS.md) (versioning, compatibility, release policy)
- [docs/DOCGEN.md](docs/DOCGEN.md) (universal documentation generator architecture and CLI)

## Cross-IDE Strategy

Ruff language tooling is being aligned around a universal-first architecture:

- shared language/tooling contracts
- one canonical Ruff LSP server for editor intelligence
- deterministic machine-readable CLI/LSP outputs
- shared grammar path for syntax highlighting
- thin editor adapters that launch/configure Ruff tooling instead of reimplementing it

Execution details and phased acceptance criteria are tracked in `ROADMAP.md` under the active `v1.0.0` planning track.

The versioned language and compatibility contract baseline for this cycle is published in [docs/LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md).

## Install

Ruff can be installed from release artifacts or by building this repository.

```bash
git clone https://github.com/rufflang/ruff.git
cd ruff
cargo build --release
./target/release/ruff --version
```

For development, use the debug build through Cargo:

```bash
cargo run -- --help
cargo run -- run examples/hello.ruff
```

To install the local checkout into Cargo's binary directory:

```bash
cargo install --path .
ruff --version
```

See [INSTALLATION.md](INSTALLATION.md) for platform notes and troubleshooting.

## Quick Start

Create `hello.ruff`:

```ruff
func total(values) {
    let sum := 0
    for value in values {
        sum := sum + value
    }
    return sum
}

let scores := [8, 13, 21]
let report := {"name": "build", "total": total(scores)}

if report["total"] > 40 {
    print("ok: " + report["name"] + " = " + to_string(report["total"]))
} else {
    print("too low")
}
```

Run it:

```bash
ruff run hello.ruff
```

Expected output:

```text
ok: build = 42
```

## CLI

The current CLI exposes these subcommands:

| Command | Purpose |
| --- | --- |
| `ruff run <file>` | Run a `.ruff` script with the default VM (`--scheduler-timeout-ms` can override cooperative scheduler timeout); parse diagnostics (including source-size/depth limits) exit non-zero before execution. Experimental JIT is opt-in via `--jit`; if the program contains unsupported JIT surfaces Ruff reports a deterministic warning and falls back to non-JIT VM execution. Runtime failures can be emitted as machine-readable JSON with `--json-runtime-diagnostics`. Native host-effect capability policy can be scoped with `--untrusted` plus `--allow-*` flags (`--allow-fs-read`, `--allow-fs-write`, `--allow-fs-delete`, `--allow-process-exec`, `--allow-shell-exec`, `--allow-env-read`, `--allow-env-write`, `--allow-net-client`, `--allow-net-server`, `--allow-net`, `--allow-database`, `--allow-clock`, `--allow-random`, or `--allow-all`). |
| `ruff run --interpreter <file>` | Run a `.ruff` script with the tree-walking interpreter. |
| `ruff check <file>` | Validate Ruff source without executing it (lex/parse/compile only). `--quiet` suppresses success output, `--verbose` prints statement/bytecode counts, and `--json` emits a machine-readable success payload. |
| `ruff serve [dir]` | Serve a directory over HTTP/HTTPS for local preview/testing (`--host`, `--port`, `--index`, `--hardened`, `--cache-max-age`, `--access-log`, `--tls-cert`, `--tls-key`, `--max-request-line-bytes`, `--max-header-bytes`, `--max-header-count`, `--max-request-body-bytes`, `--read-timeout-ms`, `--write-timeout-ms`, `--max-connections`). |
| `ruff repl` | Start the interactive REPL (tab completion, command highlighting, multiline continuation validation, and `.help <function>` support). |
| `ruff format <file>` | Format Ruff source files with opinionated defaults (`--indent`, `--line-length`, `--no-sort-imports`, `--check`, `--write`, `--json`). |
| `ruff lint <file>` | Lint Ruff source files for common issues (`--fix` for safe autofixes, `--json` for structured output). |
| `ruff init` | Initialize a Ruff project with `ruff.toml` and `src/main.ruff`. |
| `ruff package-add <name>` | Add a dependency to `ruff.toml` (`--version`, `--manifest`). |
| `ruff package-install` | Generate deterministic `ruff.lock` output from `ruff.toml` and enumerate dependencies (`--manifest`, `--lockfile`, `--frozen` verify mode). |
| `ruff package-publish` | Preview or execute package publish metadata flow from `ruff.toml`. |
| `ruff docgen <path>` | Generate universal docs from Ruff/PHP/Python/TypeScript/JavaScript/Ruby/Go/Haskell/Zig codebases (`--language`, `--languages`, `--ruff-parser-assisted`, `--format`, `--emit-ai-tasks`, `--public-only`, `--fail-on-undocumented`, `--fail-on-broken-links`, `--fail-on-warnings`, `--validate-local-anchors`, `--validate-external-links`, `--external-link-timeout-ms`, `--external-link-allowlist`, `--allow-private-network-links`, `--max-link-checks`, `--max-external-link-checks`, `--max-total-validation-time-ms`, `--max-discovery-file-size-bytes`, `--max-discovery-files`, `--max-discovery-depth`, `--cache-dir`, `--search-index`, `--source-links`, `--source-link-template`, `--out-dir`, `--no-builtins`, `--json`). |
| `ruff lsp` | Run the official Ruff LSP server over stdio JSON-RPC (`--deterministic-logs` for reproducible stderr tracing), including advanced editor metadata surfaces (`textDocument/semanticTokens/full`, `textDocument/inlayHint`, `textDocument/codeLens`). |
| `ruff test` | Discover `.ruff` fixtures under `tests/` and compare output against sibling `.out` snapshots. Runtime path is configurable with `--runtime dual|vm|interpreter` (default `dual`): dual runs VM first and falls back to interpreter only when VM output drifts from the snapshot; `--update` regenerates snapshots using the selected strategy. |
| `ruff test-run <file>` | Run tests declared with Ruff's `test "name" { ... }` syntax; parse diagnostics exit non-zero before test collection. Supports the same `--untrusted` / `--allow-*` native capability policy flags as `ruff run`. |
| `ruff bench [path]` | Run benchmark scripts. |
| `ruff bench-cross` | Compare Ruff `parallel_map` against a Python `ProcessPoolExecutor` benchmark. |
| `ruff bench-ssg` | Run the async SSG benchmark, with optional Python comparison and measurement controls. |
| `ruff profile <file>` | Profile a Ruff script for CPU, memory, and JIT stats. |
| `ruff lsp-complete <file> --line <N> --column <N>` | Return completion candidates (builtins, functions, variables) for editor/LSP integration; add `--json` for structured output. |
| `ruff lsp-definition <file> --line <N> --column <N>` | Return the go-to-definition location for the identifier under the cursor; add `--json` for structured output. |
| `ruff lsp-references <file> --line <N> --column <N>` | Return all references for the identifier under the cursor; add `--include-definition false` to exclude declarations and `--json` for structured output. |
| `ruff lsp-hover <file> --line <N> --column <N>` | Return hover details for the identifier under the cursor (kind, detail text, definition location when applicable); add `--json` for structured output. |
| `ruff lsp-diagnostics <file>` | Return source diagnostics for editor refresh loops (lexer failures, delimiter mismatches, and parser diagnostics); `--json` emits stable `code` and `subsystem` metadata per diagnostic. |
| `ruff lsp-rename <file> --line <N> --column <N> --new-name <NAME>` | Return rename edits for the symbol under the cursor and the updated source text; add `--json` for structured output. |
| `ruff lsp-code-actions <file>` | Return syntax quick-fix actions derived from diagnostics (for example unmatched/unclosed delimiters); add `--json` for structured output. |

Ruff currently requires explicit subcommands; `ruff <file>` is not treated as an implicit alias for `ruff run <file>`.

For Ruff adapter visibility in `ruff docgen`, top-level functions are treated as public only when declared with `pub func`; non-`pub` top-level helpers remain private, and symbols nested under private containers (for example `pub` methods inside private structs or variants inside private enums) remain private for `--public-only` gating.
The full Ruff visibility policy used by DocGen is documented in [docs/DOCGEN.md](docs/DOCGEN.md).
Ruff DocGen extraction also supports `async func` declarations (including `pub async func`) for top-level functions and struct methods.
Ruff DocGen extraction remains hybrid for now (regex-first with fixture-backed edge coverage); parser-assisted extraction is explicitly documented as a future bounded fallback path in [docs/DOCGEN.md](docs/DOCGEN.md).
DocGen adapter extraction now caches regex compilation through static/lazy initialization across Ruff/PHP/Python/TypeScript/JavaScript/Ruby/Go/Haskell/Zig adapters, removing per-file regex recompilation overhead while preserving extraction output contracts.
Ruff DocGen inline docs support `///`, `//!`, and `/** ... */` comment styles; plain `/* ... */` comments are ignored for API docs attachment.
Ruff DocGen doc attachment is decorator-aware and skips intermediate `@...` / `#[...]` lines when mapping a doc block to its symbol target.
Ruff doc attachment keeps stable proximity rules: blank-line spacing is allowed, regular non-doc comment lines break attachment, and the nearest eligible doc block wins.
DocGen now emits explicit discovery-limit warnings in JSON output when files are skipped for size/depth/file-count limits (`DOCGEN_DISCOVERY_MAX_FILE_SIZE`, `DOCGEN_DISCOVERY_MAX_DEPTH`, `DOCGEN_DISCOVERY_MAX_FILES`).
DocGen CLI JSON output (`ruff docgen ... --json`) also includes deterministic per-reason discovery skip counters in `discovery_skip_counts` (`max_file_size`, `max_depth`, `max_files`, `invalid_encoding`).
DocGen diagnostics are emitted in deterministic sorted order to keep repeated JSON outputs stable for CI diffing.
DocGen JSON summary now separates discovered symbol volume into `project_symbol_count` and `builtin_symbol_count` (while preserving `item_count` as total count).
DocGen JSON summary also includes deterministic `symbol_kind_counts` (for example `function`, `method`, `struct`, `enum`, `builtin`) to support CI dashboards and trend tracking.
DocGen JSON now includes a stable versioned dashboard block at `summary` (`schema_version = docgen-summary/v1`) that mirrors key totals and gate-state fields for automation consumers.
DocGen JSON includes deterministic link-validation budget truncation counts in `link_validation_skip_counts` (`max_link_checks`, `max_external_checks`, `max_total_time`) for CI-visible bounded runs.
DocGen link validation now reuses one HTTP client per run in external-link mode and caches parsed local anchors per file path in local-anchor mode, avoiding repeated client construction and repeated file reads for repeated link targets.
DocGen gap generation now builds known-call-site hints from a one-pass source index instead of per-symbol full-source rescans, preserving deterministic ordering and per-symbol call-site limits while reducing large-repo scan overhead.
DocGen TypeScript/JavaScript adapters now share C-style extraction helpers for class-scope tracking, brace-depth updates, and JSDoc block parsing, reducing duplicated parsing logic while preserving adapter symbol/doc contracts.
DocGen visibility classification now uses shared adapter helper rules (explicit modifier mapping, naming-convention mapping, and container-aware effective visibility) with regression-locked Ruff and TypeScript visibility semantics.
DocGen CLI JSON contract output is now assembled via a typed single-source payload builder in `src/docgen/core.rs`, with fixture-backed snapshot coverage to guard key-shape drift.
DocGen discovery limits are configurable from CLI flags (`--max-discovery-file-size-bytes`, `--max-discovery-files`, `--max-discovery-depth`) or environment (`RUFF_DOCGEN_MAX_FILE_SIZE_BYTES`, `RUFF_DOCGEN_MAX_FILES`, `RUFF_DOCGEN_MAX_DEPTH`), and effective values are emitted under `discovery_limits` in both top-level and `summary` JSON outputs.
DocGen now emits per-language adapter health counters under `adapter_health` (`files_scanned`, `symbols_extracted`, `doc_blocks_attached`, `placeholders_emitted`) and warns with `DOCGEN_ADAPTER_LOW_YIELD` when extraction yield is suspiciously low for scanned language inputs.
DocGen cache mode (`--cache-dir`) reuses per-file extraction artifacts keyed by source content hash and adapter cache version; JSON outputs include deterministic `cache_stats` counters (`hits`, `misses`) at both top-level and `summary`.
DocGen source-link provider templates are configurable with `--source-link-template` (supports `{path}` and `{line}` placeholders) when `--source-links` is enabled; unsafe source paths (absolute or parent-traversal) are rejected from template expansion.
DocGen renderers now share symbol source-location formatting helpers and have no-op duplicate HTML branches removed, with output-shape regression coverage to guard deterministic rendering behavior.
DocGen default link validation remains local-file existence checking (fragments/query suffixes are ignored for local paths, while `http(s)` and `mailto` links are not validated by default).
DocGen optional local-anchor validation can be enabled with `--validate-local-anchors` when strict anchor checks are needed for local docs.
DocGen optional external-link validation can be enabled with `--validate-external-links`, scoped to allowlisted hosts via `--external-link-allowlist`, and bounded by `--external-link-timeout-ms`.
DocGen link-validation budgets can be configured with `--max-link-checks`, `--max-external-link-checks`, and `--max-total-validation-time-ms`; when budgets truncate checks, deterministic warnings are emitted and surfaced in JSON summaries.
For SSRF safety, external-link validation blocks targets that are private/loopback/link-local/multicast by default (including DNS-resolved hosts); use `--allow-private-network-links` to opt in when internal-network validation is intentionally required.
When external-link validation follows redirects, DocGen now re-validates the allowlist on every redirect hop and reports deterministic `external-redirect-allowlist` broken-link diagnostics if a hop leaves the allowlisted host set.
DocGen link-mode diagnostics and gate failures are now mode-specific for CI triage: local-file, local-anchor, and external link failures are reported separately, and external-mode misconfiguration (`--validate-external-links` without allowlist, or allowlist without external validation) emits explicit warnings.

### CLI Exit Codes

Ruff uses stable exit-code categories for automation:

- `0`: success
- `1`: generic command failure or unmet gate (for example `format --check` needs changes, lint/test failures)
- `2`: command-line usage/argument parse error
- `3`: lexer/parser diagnostic failure
- `4`: runtime execution/semantic failure
- `5`: IO failure (missing/unreadable/unwritable paths)
- `6`: internal/tooling failure

All user-facing diagnostics are emitted on `stderr`; successful program or JSON output is emitted on `stdout`.

### Static Server (`ruff serve`)

`ruff serve` is intended to be a universal local static preview surface for Ruff users.

Examples:

```bash
# Basic HTTP preview
ruff serve output --host 127.0.0.1 --port 8080

# Hardened mode with access logs and explicit cache controls
ruff serve output --hardened --access-log --cache-max-age 300

# HTTPS preview (both certificate and key are required)
ruff serve output --tls-cert ./certs/dev-cert.pem --tls-key ./certs/dev-key.pem
```

Behavior highlights:

- Supports `GET` and `HEAD` requests; standard non-read methods return `405 Method Not Allowed` with `Allow: GET, HEAD`, and non-standard methods return `501 Not Implemented`.
- Enforces canonical root-boundary checks to block path traversal.
- Validates request targets before filesystem resolution: path/query are parsed separately, fragments are rejected, request paths are percent-decoded exactly once, and malformed percent-encoding or decoded null bytes return `400 Bad Request`.
- Rejects unsafe decoded traversal paths with `403 Forbidden`, and rejects oversized request targets larger than `4096` bytes with `414 URI Too Long`.
- Blocks hidden/private path targets by default with `403 Forbidden` (dotfiles/directories such as `.env`, `.git`, `.svn`, `.hg`, `.DS_Store`, plus backup/swap suffixes such as `.bak`, `.backup`, `.tmp`, `.old`, `.orig`, `.swp`, `.swo`, and trailing `~`).
- Enforces static-server request limits by default: max request line `8192` bytes (`414 URI Too Long`), max combined header bytes `16384` (`413 Payload Too Large`), max header count `100` (`413`), and max request body `1048576` bytes (`413`).
- Applies configurable timeout/concurrency controls for server hardening (`--read-timeout-ms`, `--write-timeout-ms`, `--max-connections`); requests above the concurrent-handler limit return `503 Service Unavailable`.
- Uses no-follow file reads on Unix to reduce symlink race/swap risks.
- Streams static file bodies from disk instead of buffering whole files in memory, while preserving deterministic `Content-Length` for `GET` and `HEAD`.
- Returns deterministic status mapping for common file errors (`404`, `403`, `500`).
- Adds ETag-based conditional responses (`304`) and single-range byte serving (`206`/`416`).
- Detects and serves precompressed sibling assets (`.br`, `.gz`) when accepted.
- Uses one centralized, case-insensitive MIME extension registry for real-world static assets (web/text, images including `tif`/`tiff`, audio/video including `mov`, fonts including `eot`, and archives including `zip`/`tar`/`gz`/`tgz`/`7z`).
- Serves unknown-extension and extensionless files as `application/octet-stream` (including unknown active-content payloads).
- Adds baseline response-safety headers (`X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`) plus conservative cache defaults when explicit max-age is not configured.
- Adds stricter hardened-mode headers (`X-Frame-Options`, COOP/CORP, CSP, Permissions-Policy).
- Adds `Strict-Transport-Security` only for secure (TLS) requests.

Machine-readable output and automation contracts are documented in [docs/CLI_MACHINE_READABLE_CONTRACTS.md](docs/CLI_MACHINE_READABLE_CONTRACTS.md).

Cross-editor thin-adapter setup baselines are documented in [docs/EDITOR_ADAPTER_BASELINES.md](docs/EDITOR_ADAPTER_BASELINES.md).

Tree-sitter grammar assets and integration baseline are documented in [docs/TREE_SITTER_RUFF.md](docs/TREE_SITTER_RUFF.md).

Protocol contracts, install/upgrade guidance, and release artifact checklist docs:

- [docs/PROTOCOL_CONTRACTS.md](docs/PROTOCOL_CONTRACTS.md)
- [docs/INSTALLATION_LSP_EDITORS.md](docs/INSTALLATION_LSP_EDITORS.md)
- [docs/RELEASE_ARTIFACT_CHECKLIST_V0_14_0.md](docs/RELEASE_ARTIFACT_CHECKLIST_V0_14_0.md)
- [docs/RELEASE_ARTIFACT_CHECKLIST_V0_13_0.md](docs/RELEASE_ARTIFACT_CHECKLIST_V0_13_0.md)
- [docs/RELEASE_PROCESS.md](docs/RELEASE_PROCESS.md)
- [docs/RELEASE_ARTIFACT_VALIDATION.md](docs/RELEASE_ARTIFACT_VALIDATION.md)
- [docs/LSP_RELIABILITY.md](docs/LSP_RELIABILITY.md)
- [docs/V1_SCOPE.md](docs/V1_SCOPE.md)

`docs/RELEASE_PROCESS.md` now carries the canonical semantic-versioning, compatibility, release-candidate, changelog-format, and publication workflow policy used for release sign-off.

Useful environment variables:

| Variable | Effect |
| --- | --- |
| `DISABLE_JIT=1` | Forces JIT off even when `ruff run --jit` is requested. |
| `DEBUG_AST=1` | Prints the parsed AST before VM execution. |
| `RUFF_SCHEDULER_TIMEOUT_MS=<ms>` | Overrides the VM cooperative scheduler completion timeout when `ruff run --scheduler-timeout-ms` is not provided. The default is `120000`. |

Scheduler timeout precedence for `ruff run` is:

1. `--scheduler-timeout-ms <ms>`
2. `RUFF_SCHEDULER_TIMEOUT_MS=<ms>`
3. default `120000 ms`

## v0.14.0 Release Highlights

The v0.14.0 release delivered stabilization and release-hardening work required before v1.0.0:

- release process hardening with deterministic playbook + CI release-state guard
- fixture-locked LSP protocol stability guarantees and compatibility table
- packaging/distribution follow-through with Linux/macOS artifact validation and reproducible checksum flow
- tree-sitter and editor adapter maturity follow-through, including first-party extension smoke coverage in CI
- runtime/tooling reliability track with malformed-sequence, lifecycle churn, bounded-state, and latency guardrail coverage
- explicit v1.0.0 scope definition and deferred post-1.0 backlog commitments

Detailed release evidence and checklist completion notes are documented under `notes/`.

## v0.13.0 Release Highlights

The v0.13.0 release delivered the cross-IDE tooling baseline and made Ruff's LSP/server contracts a first-class surface:

- `ruff lsp` official JSON-RPC server entrypoint with lifecycle handling, deterministic logging, and shared analyzer wiring.
- Full required LSP method parity for `v0.13.0` including diagnostics, completion, hover, definition, references, rename, code actions, formatting/range-formatting, document symbols, and workspace symbols.
- Deterministic machine-readable CLI contracts with `--json` coverage for format/lint/docgen and LSP helper commands, plus `ruff run --json-runtime-diagnostics` for runtime failure envelopes.
- Fixture-driven LSP conformance harness and external client smoke coverage.
- Baseline tree-sitter grammar assets and editor-adapter setup guidance for VS Code/Cursor, Neovim, and JetBrains.

Detailed release evidence and artifact validation are documented in `docs/RELEASE_ARTIFACT_CHECKLIST_V0_13_0.md` and the completion notes under `notes/`.

## Earlier Milestone Highlights

Ruff still includes the `v0.12.0` tooling additions that remain part of the current release:

- opinionated spacing/indentation normalization
- configurable indentation width
- line-length-aware wrapping for comma-separated expressions
- leading import-block sorting (optionally disabled)

## Language Overview

Ruff source files use the `.ruff` extension. The implemented syntax includes:

- Numeric, string, boolean, `null`, array, and dictionary values.
- `let`, `mut`, and `const` bindings with `:=` assignment syntax.
- `let` and `const` bindings are immutable (reassignment and in-place mutation through those bindings are runtime errors); `mut` bindings allow reassignment and in-place mutation.
- Duplicate declarations in the same lexical scope are runtime/compile errors (`Duplicate declaration in the same scope: <name>`).
- Assignment/update statements support `:=`, `=`, `+=`, `-=`, `*=`, `/=`, and `%=` (chained assignment like `a := b := 1` is rejected).
- Operator precedence is explicit and documented in `docs/LANGUAGE_SPEC.md` (postfix/unary through multiplicative/additive/comparison/equality/boolean/null-coalescing/pipe).
- Optional annotations on variables and functions, such as `let x: int := 42` and `func add(a: int, b: int) -> int`.
- Functions declared with `func`, including anonymous function expressions and `async func`.
- `if`/`else`, `while`, `loop`, `for ... in`, `break`, and `continue`.
- `break` and `continue` are only valid inside loop bodies; using either outside a loop is a deterministic runtime/compile error.
- Top-level `return` remains supported as the script-exit value contract, while function `return` keeps standard early-exit behavior.
- Function body fallthrough and `return` without an expression both yield `null`; explicit `return 0` remains integer zero.
- Function bodies and control-flow bodies (`if`/`else`, `while`, `loop`, and `for`) use lexical scopes; inner shadowing is allowed, loop variables do not leak after loop completion, and closures capture the nearest lexical binding.
- Shared truthiness semantics across interpreter/VM/native predicates: `false`, `null`, `0`, `0.0`, empty string, empty array, and empty dictionary are falsey; all other values are truthy.
- Logical `&&` and `||` use truthiness and short-circuit (RHS is evaluated only when required), returning boolean results.
- Integer arithmetic (`+`, `-`, `*`, `/`, `%`) is checked at runtime (`i64`): overflow and divide/modulo-by-zero return deterministic runtime errors.
- Float division/modulo by zero returns deterministic runtime errors; NaN and infinity comparisons follow documented language-spec behavior.
- Equality/comparison semantics are explicit across interpreter and VM: `1 == 1.0` is true, arrays/dictionaries compare deeply by value, callable equality is identity-based, and unsupported ordering pairs (for example `bool < bool`) return deterministic runtime errors.
- Arrays and dictionaries with indexing and standard library helpers.
- Module imports support flat and dotted `from` paths (`import module`, `from module import symbol`, `from src.util import value`, `from src.core.math import add`).
- Resolution order is deterministic and compatibility-preserving: Ruff checks `<module>.ruff` first (legacy behavior), then for dotted module names checks `<seg1>/<seg2>/.../<segN>.ruff`; both forms remain constrained to the importing package root before configured search paths.
- Dotted module import workflows are supported on the default VM path; `--interpreter` is an optional compatibility/debug fallback, not a requirement for ordinary nested module layouts.
- Module resolution rejects unsafe traversal-style module names and symlink escapes that resolve outside the active module search root.
- Circular imports fail with an explicit import chain (`a -> b -> a`) to make cycle diagnosis deterministic.
- Module cache entries are context-aware (package root + canonical module path) and are invalidated when module source metadata changes during the same run.
- Undefined identifiers are runtime errors. Use quoted string literals when a string value is intended.
- Missing dictionary keys are runtime errors. Use helpers such as `has_key`, `get`, or `get_default` when a fallback value is intended.
- Out-of-bounds array/string indexing, indexing non-indexable values, and invalid index-assignment targets are runtime errors.
- Unsupported unary/binary operations are runtime errors (no implicit `Int(0)` or empty-string fallback).
- Function/method/native call arity is enforced at runtime. Too few or too many arguments return deterministic errors that include callable name plus expected and received counts/ranges.
- String interpolation with `${...}` inside double-quoted strings.
- `Result` and `Option` values through `Ok(...)`, `Err(...)`, `Some(...)`, and `None`.
- `match`/`case` statements, including parity-covered `Result::`/`Option::` tag-style bindings (`Ok(value)`, `Err(error)`, `Some(value)`) on both interpreter and VM paths.
- `try`/`except` blocks and `throw(...)` for runtime error flow.
- `await` expressions and promise-returning async/native operations.
- `spawn { ... }` syntax and native concurrency helpers.
- `struct` declarations, field access, and struct instance literals.
- `test`, `test_setup`, `test_teardown`, and `test_group` declarations for the test runner.
- `#`, `//`, `/* ... */`, and `///` comments.
- Lexing malformed source now fails with structured diagnostics (line/column anchored) instead of silently skipping invalid characters or coercing malformed numeric literals.
- Parsing malformed source now returns structured parser diagnostics with consistent source spans (start/end line+column and byte-offset bounds), including delimiter/EOF errors and invalid assignment targets; CLI parse failures exit non-zero before runtime execution.
- Parser safety limits are enforced for CLI parse entrypoints: source files over `1,048,576` bytes, string literals over `8,192` characters, collection literals over `4,096` items, expression nesting deeper than `256`, or statement-block nesting deeper than `128` fail with structured diagnostics.
- Runtime call-depth limits are enforced to prevent unbounded recursion: interpreter calls fail after depth `32` and VM bytecode calls fail after depth `256` with explicit runtime errors.

A small async example:

```ruff
async func fetch_label(id) {
    return "item-" + to_string(id)
}

let first := fetch_label(1)
let second := fetch_label(2)

print(await first)
print(await second)
```

## Native Standard Library

Native functions are registered in `src/interpreter/mod.rs` and dispatched through `src/interpreter/native_functions/`. The current categories include:

- I/O and debugging: `print`, `input`, `debug`, assertions.
- Math: `abs`, `sqrt`, `pow`, `floor`, `ceil`, `round`, `min`, `max`, trigonometry, `PI`, `E`.
- Strings and regex: length, case conversion, trimming, splitting/joining, padding, slug/case helpers, regex match/find/replace/split.
- Collections: array helpers, higher-order functions, dictionaries, sets, queues, stacks, ranges.
- Type conversion and introspection: `parse_int`, `parse_float`, `to_string`, `to_bool`, `type`, `is_*` helpers, `bytes`.
- Filesystem and binary I/O: text files, binary files, metadata, random access reads/writes, copy/truncate helpers, path helpers, OS helpers.
- Data formats: JSON, TOML, YAML, CSV, and Base64.
- System utilities: environment variables, CLI args, time/date helpers, sleep, process execution, process spawning, command piping.
- HTTP and auth: HTTP client calls, streaming/binary HTTP helpers, response helpers, JWT helpers, OAuth2 helpers, parallel HTTP, plus high-level AI helpers (`ai_chat`, `ai_stream_chat`, `ai_embedding`, `ai_tool_loop`) with deterministic `Result` failure contracts.
- Database helpers: SQLite, Postgres, MySQL-oriented connection/query APIs, pools, transactions, last-insert-id helpers.
- Async and concurrency: sleep/timeout promises, async file/HTTP operations, task handles, `Promise.all` aliases, `parallel_map`, shared state, channels, task-pool sizing.
- Media/archive/crypto/network: image loading, zip/unzip, SHA/MD5, bcrypt, AES, RSA, TCP, and UDP helpers.

The complete native API inventory (function signatures, arity labels, capability requirements, and examples) is documented in `docs/STANDARD_LIBRARY.md` and contract-checked against runtime registration in `tests/stdlib_reference_contract.rs`.
For data-format boundaries, `parse_json(...)` now enforces a `1,048,576`-byte input limit and `64`-level nesting limit with location-aware parse failures, and `to_json(...)` now rejects non-finite floats (`NaN`/`+/-inf`) while producing deterministic key ordering for dictionary-like values.
For helper-contract hardening, math/string/collection helpers now return deterministic argument/type/domain errors instead of silent fallback values, `parse_date(...)` only accepts `YYYY-MM-DD` with explicit parse errors, and `env_bool(...)` now accepts explicit true/false token sets and errors on invalid values.

High-risk native APIs (process/network/filesystem/crypto/database) are powerful. `ruff run` and `ruff test-run` default to trusted capability mode for local workflows, and can be switched to deny-by-default mode with `--untrusted` plus explicit `--allow-*` capability flags. Review `docs/NATIVE_API_SECURITY_POSTURE.md` before running untrusted scripts or deploying automation in shared environments.
The security posture guide now includes explicit threat-model boundaries, static-serve security defaults, and safe-vs-unsafe CLI configuration examples; treat it as required operator guidance, not optional reference reading.
For archive extraction, `unzip(...)` now rejects unsafe entry paths (absolute paths, `..` traversal components, drive-prefixed names, null-byte names, and symlink entries) and enforces extraction limits (1024 entries, 16 MiB per entry, 64 MiB total uncompressed size).
For file IO safety, whole-file reads (`read_file`, `read_lines`, `read_binary_file`) and file writes (`write_file`, `write_binary_file`, `append_file`) are bounded to 8 MiB payloads. `write_file` and `write_binary_file` no longer overwrite existing files by default; pass `overwrite=true` as a third argument to replace an existing file intentionally.
For process execution, `spawn_process(...)` and `pipe_commands(...)` execute direct argv arrays (no shell interpolation), while `execute(...)` / `execute_status(...)` execute shell command strings and should be treated as high risk. These process APIs now accept an optional options dictionary (`timeout_ms`, `max_output_bytes`, `inherit_env`, `env_allow`, `env_deny`, `env`) and `execute_status(...)` / `spawn_process(...)` return `ProcessResult` metadata (`exitcode`, `stdout`, `stderr`, `success`, `timed_out`, `stdout_truncated`, `stderr_truncated`) for deterministic boundary handling.
For network safety, HTTP/TCP/UDP native APIs now apply default timeout and size guardrails: TCP connect timeout (`10s`), TCP/UDP read-write timeouts (`30s`), HTTP request timeout (`30s`), and bounded receive/response bodies (`8 MiB` max). Boundary violations return deterministic runtime errors instead of unbounded blocking or unbounded reads.

## Testing

Ruff has two testing layers.

Run the Rust test suite:

```bash
cargo test
```

Run the release-gate CI suite locally:

```bash
bash scripts/release_gate.sh
```

Run the release-candidate readiness gate (roadmap precheck + full release gate):

```bash
bash scripts/release_candidate_gate.sh --full
```

Fast smoke mode (useful before pushing, and the mode CI uses for lightweight script-validation):

```bash
bash scripts/release_gate.sh --minimal
```

To include socket-bound static-serve integration tests in local gate runs:

```bash
RUFF_ENABLE_SOCKET_TESTS=1 bash scripts/release_gate.sh
```

Run lexer/parser fuzz smoke locally with prerequisite checks:

```bash
bash scripts/fuzz_smoke.sh --check-prereqs
bash scripts/fuzz_smoke.sh --max-total-time 20
```

If prerequisites are missing, `scripts/fuzz_smoke.sh --check-prereqs` prints exact install guidance (nightly toolchain, `cargo-fuzz`, and C++ headers/toolchain requirements for `libfuzzer-sys`).
Nightly CI runs the same bounded fuzz smoke targets in `.github/workflows/fuzz-smoke.yml`.

Replay a fuzz crash artifact deterministically (from local or CI-downloaded artifacts):

```bash
bash scripts/fuzz_repro.sh --target lexer --artifact fuzz/artifacts/lexer/crash-123456
bash scripts/fuzz_repro.sh --artifact fuzz/artifacts/parser/crash-abcdef
bash scripts/fuzz_repro.sh --target parser --artifact tests/fixtures/fuzz/synthetic_crash_input.ruff --dry-run
```

`scripts/fuzz_repro.sh` supports explicit target mode (`--target`) and artifact-path inference mode (`.../artifacts/<target>/...`). Use `--dry-run` to validate command wiring before running `cargo-fuzz`.

Run the runtime/native security regression suites directly:

```bash
cargo test --test runtime_security
cargo test --test native_api_security_boundaries
cargo test --test serve_command_integration
```

Run diagnostics golden snapshot contracts:

```bash
cargo test --test diagnostics_golden
```

Refresh diagnostics golden snapshots intentionally:

```bash
RUFF_UPDATE_GOLDENS=1 cargo test --test diagnostics_golden
```

Run docs/examples smoke contracts (parse/run/expected-fail metadata):

```bash
cargo test --test docs_examples
```

`docs_examples` now expects all fenced Ruff docs snippets to parse clean; remaining `expected-fail` entries are limited to legacy `.ruff` example files with explicit per-file debt reasons in `tests/docs_examples.rs`.

Run language-spec semantic contract tests (scope/mutability/arity/truthiness/indexing):

```bash
cargo test --test language_spec_contracts
```

Release-gate prerequisites and runtime profile:

- Requires Rust toolchain with `cargo fmt`, `cargo clippy`, and `cargo test` available.
- Optionally runs `cargo audit` and `cargo deny check` in full mode when those tools are installed.
- Optional benchmark smoke can be enabled in full mode with `RUFF_RELEASE_GATE_RUN_BENCH=1`.
- Typical runtime:
  - `--minimal`: usually under a few minutes.
  - full mode: usually several minutes and longer on busy machines.

Run the async runtime concurrency stability regression directly:

```bash
cargo test test_concurrent_tasks
```

Run repository `.ruff` snapshots:

```bash
cargo run -- test
cargo run -- test --update
```

Run Ruff tests declared in a specific file:

```bash
cargo run -- test-run tests/testing_framework.ruff
```

Example Ruff test file:

```ruff
test "array length" {
    let values := [1, 2, 3]
    assert_equal(len(values), 3)
}

test "string predicate" {
    assert_true(starts_with("ruff", "ru"))
}
```

## Benchmarks And Profiling

Ruff includes benchmark tooling because performance work is a first-class part of the project, especially for the VM, async scheduler, and native SSG benchmark path.

```bash
cargo bench --bench v1_perf_benchmarks
cargo run -- bench examples/benchmarks
cargo run -- bench-cross
cargo run -- bench-ssg --runs 5 --warmup-runs 1
cargo run -- bench-ssg --compare-python --profile-async
cargo run -- profile examples/benchmark.ruff
```

`cargo bench --bench v1_perf_benchmarks` runs the core Criterion baseline suite for lexer, parser, interpreter, VM, module-resolution, and static-server request workloads.

`bench-ssg` supports repeated runs, warmups, percentile reporting, measurement-quality warnings, optional Python comparison, optional stage profiling, and a throughput gate via `--throughput-gate-ms`.

## Known Boundaries

These are intentional caveats for production readers rather than fine print:

- Tagged releases publish standalone Linux/macOS artifacts with checksums; package-manager taps remain outside the current v1 release gate.
- VM execution is the default; the interpreter remains part of compatibility workflows (`ruff test --runtime dual|interpreter`) where legacy fixture snapshots and diagnostics still require explicit fallback handling.
- Static typing is optional and not a VM-enforced compile-time contract in the current CLI path.
- `import`/`export` syntax and module execution/export collection are implemented; module file resolution is constrained to configured search roots and rejects symlink-resolved escapes.
- Struct fields and instance literals exist, and explicit `self` struct methods are parity-covered in VM/interpreter tests.
- Top-level generator functions (`func*`) are currently intentionally divergent: interpreter iteration is covered, while VM currently returns a deterministic error (`Yield can only be used inside generator functions`) for top-level generator `yield` paths.
- Struct generator methods (`func*` inside `struct`) are intentionally unsupported and fail with a deterministic error (`Generator methods are not supported for structs: <Struct>.<method>`).
- Spread literals and destructuring syntax are parity-covered in VM/interpreter tests for the tracked matrix scenarios.
- `spawn { ... }` is parsed and executed as fire-and-forget work; do not rely on spawned output or shared state without using the explicit concurrency/native async APIs.
- `Result`/`Option` values are implemented, and tag-style match bindings are parity-covered on both VM and interpreter paths for the tracked scenarios.
- Benchmark numbers are environment-sensitive. Prefer `bench-ssg --runs <N> --warmup-runs <N>` and read the measurement warnings before drawing conclusions.
- Runtime parity status is tracked in [docs/VM_INTERPRETER_PARITY_MATRIX.md](docs/VM_INTERPRETER_PARITY_MATRIX.md).

## Documentation

- [INSTALLATION.md](INSTALLATION.md): build and installation notes.
- [CHANGELOG.md](CHANGELOG.md): release and unreleased changes.
- [ROADMAP.md](ROADMAP.md): planned work.
- [CONTRIBUTING.md](CONTRIBUTING.md): contribution workflow.
- [docs/CONCURRENCY.md](docs/CONCURRENCY.md): async/concurrency model notes.
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md): benchmark and profiling notes.
- [docs/VM_INSTRUCTIONS.md](docs/VM_INSTRUCTIONS.md): bytecode VM instruction reference.
- [docs/EXTENDING.md](docs/EXTENDING.md): adding native functionality.
- [docs/STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md): complete native API inventory and support table.
- [docs/DEPRECATION_POLICY.md](docs/DEPRECATION_POLICY.md): semver-tied deprecation windows and removal policy.
- [docs/NATIVE_API_SECURITY_POSTURE.md](docs/NATIVE_API_SECURITY_POSTURE.md): trust model and operational caveats for high-risk native APIs.

Some older docs may lag the current VM-default execution path. Prefer the code and tests when behavior differs.

## License

Ruff is licensed under the MIT License. See [LICENSE](LICENSE).
