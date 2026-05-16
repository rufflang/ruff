# Ruff

Ruff is a small programming language and runtime implemented in Rust. It is built for local scripting, automation, runtime experiments, and benchmarking work where a compact language with a Rust-hosted standard library is useful.

The project is currently at `0.14.0` in `Cargo.toml`, after completing the stabilization and v1-runway release cycle. Next planned roadmap work targets `1.0.0` scope execution and compatibility hardening. Ruff is usable from source today, but the language and runtime APIs are still evolving. Treat the repository tests, examples, and native-function dispatch tests as the source of truth for current behavior.

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
| `ruff run <file>` | Run a `.ruff` script with the default VM (`--scheduler-timeout-ms` can override cooperative scheduler timeout); parse diagnostics (including source-size/depth limits) exit non-zero before execution. Experimental JIT is opt-in via `--jit`; if the program contains unsupported JIT surfaces Ruff reports a deterministic warning and falls back to non-JIT VM execution. Native host-effect capability policy can be scoped with `--untrusted` plus `--allow-*` flags (`--allow-fs-read`, `--allow-fs-write`, `--allow-fs-delete`, `--allow-process-exec`, `--allow-shell-exec`, `--allow-env-read`, `--allow-env-write`, `--allow-net-client`, `--allow-net-server`, `--allow-net`, `--allow-database`, `--allow-clock`, `--allow-random`, or `--allow-all`). |
| `ruff run --interpreter <file>` | Run a `.ruff` script with the tree-walking interpreter. |
| `ruff serve [dir]` | Serve a directory over HTTP/HTTPS for local preview/testing (`--host`, `--port`, `--index`, `--hardened`, `--cache-max-age`, `--access-log`, `--tls-cert`, `--tls-key`, `--max-request-line-bytes`, `--max-header-bytes`, `--max-header-count`, `--max-request-body-bytes`, `--read-timeout-ms`, `--write-timeout-ms`, `--max-connections`). |
| `ruff repl` | Start the interactive REPL (tab completion, command highlighting, multiline continuation validation, and `.help <function>` support). |
| `ruff format <file>` | Format Ruff source files with opinionated defaults (`--indent`, `--line-length`, `--no-sort-imports`, `--check`, `--write`, `--json`). |
| `ruff lint <file>` | Lint Ruff source files for common issues (`--fix` for safe autofixes, `--json` for structured output). |
| `ruff init` | Initialize a Ruff project with `ruff.toml` and `src/main.ruff`. |
| `ruff package-add <name>` | Add a dependency to `ruff.toml` (`--version`, `--manifest`). |
| `ruff package-install` | Validate and enumerate dependencies declared in `ruff.toml`. |
| `ruff package-publish` | Preview or execute package publish metadata flow from `ruff.toml`. |
| `ruff docgen <file>` | Generate HTML docs from `///` comments (`--out-dir`, `--no-builtins`, `--json`). |
| `ruff lsp` | Run the official Ruff LSP server over stdio JSON-RPC (`--deterministic-logs` for reproducible stderr tracing). |
| `ruff test` | Run `.ruff` files under `tests/`; `--update` regenerates expected-output snapshots. |
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
- Deterministic machine-readable CLI contracts with `--json` coverage for format/lint/docgen and LSP helper commands.
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
- Module imports (`import module`, `from module import symbol`) resolve relative to the importing module's package root first, then configured search paths.
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
- Parser safety limits are enforced for CLI parse entrypoints: source files over `1,048,576` bytes, expression nesting deeper than `256`, or statement-block nesting deeper than `128` fail with structured parse diagnostics.

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
- HTTP and auth: HTTP client calls, streaming/binary HTTP helpers, response helpers, JWT helpers, OAuth2 helpers, parallel HTTP.
- Database helpers: SQLite, Postgres, MySQL-oriented connection/query APIs, pools, transactions, last-insert-id helpers.
- Async and concurrency: sleep/timeout promises, async file/HTTP operations, task handles, `Promise.all` aliases, `parallel_map`, shared state, channels, task-pool sizing.
- Media/archive/crypto/network: image loading, zip/unzip, SHA/MD5, bcrypt, AES, RSA, TCP, and UDP helpers.

The complete native API inventory (function signatures, arity labels, capability requirements, and examples) is documented in `docs/STANDARD_LIBRARY.md` and contract-checked against runtime registration in `tests/stdlib_reference_contract.rs`.
For data-format boundaries, `parse_json(...)` now enforces a `1,048,576`-byte input limit and `64`-level nesting limit with location-aware parse failures, and `to_json(...)` now rejects non-finite floats (`NaN`/`+/-inf`) while producing deterministic key ordering for dictionary-like values.

High-risk native APIs (process/network/filesystem/crypto/database) are powerful. `ruff run` and `ruff test-run` default to trusted capability mode for local workflows, and can be switched to deny-by-default mode with `--untrusted` plus explicit `--allow-*` capability flags. Review `docs/NATIVE_API_SECURITY_POSTURE.md` before running untrusted scripts or deploying automation in shared environments.
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

Fast smoke mode (useful before pushing, and the mode CI uses for lightweight script-validation):

```bash
bash scripts/release_gate.sh --minimal
```

To include socket-bound static-serve integration tests in local gate runs:

```bash
RUFF_ENABLE_SOCKET_TESTS=1 bash scripts/release_gate.sh
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
cargo run -- bench examples/benchmarks
cargo run -- bench-cross
cargo run -- bench-ssg --runs 5 --warmup-runs 1
cargo run -- bench-ssg --compare-python --profile-async
cargo run -- profile examples/benchmark.ruff
```

`bench-ssg` supports repeated runs, warmups, percentile reporting, measurement-quality warnings, optional Python comparison, optional stage profiling, and a throughput gate via `--throughput-gate-ms`.

## Known Boundaries

These are intentional caveats for production readers rather than fine print:

- Tagged releases publish standalone Linux/macOS artifacts with checksums; package-manager taps remain outside the current v1 release gate.
- VM execution is the default, but the tree-walking interpreter still matters for some language surfaces and diagnostics.
- Static typing is optional and not a VM-enforced compile-time contract in the current CLI path.
- `import`/`export` syntax and module execution/export collection are implemented; module file resolution is constrained to configured search roots and rejects symlink-resolved escapes.
- Struct fields and instance literals exist, and explicit `self` struct methods are parity-covered in VM/interpreter tests.
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
