# Ruff

Ruff is a small programming language and runtime implemented in Rust. It is built for local scripting, automation, runtime experiments, and benchmarking work where a compact language with a Rust-hosted standard library is useful.

The project is currently at `0.11.0` in `Cargo.toml`, following a release focused on async static-site-generation benchmark throughput and VM scheduler reliability. Next planned roadmap work targets `0.12.0` developer-experience surfaces. Ruff is usable from source today, but the language and runtime APIs are still evolving. Treat the repository tests, examples, and native-function dispatch tests as the source of truth for current behavior.

## Current Status

- Ruff builds from source with Cargo. Prebuilt binary/package-manager installation is not the current path.
- `ruff run` uses the bytecode VM by default.
- `ruff run --interpreter` runs the tree-walking interpreter fallback.
- The runtime includes a broad native standard library for strings, collections, files, data formats, HTTP, databases, crypto, process/system helpers, concurrency, and network primitives.
- Optional type annotations are parsed. In the CLI, type-checking warnings are emitted on the interpreter path; VM execution does not currently enforce a static type gate before running.
- Some advanced language surfaces are experimental or have runtime-mode gaps. See [Known Boundaries](#known-boundaries).

## Install

Ruff currently installs by building this repository.

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
| `ruff run <file>` | Run a `.ruff` script with the default VM (`--scheduler-timeout-ms` can override cooperative scheduler timeout). |
| `ruff run --interpreter <file>` | Run a `.ruff` script with the tree-walking interpreter. |
| `ruff repl` | Start the interactive REPL. |
| `ruff format <file>` | Format Ruff source files with opinionated defaults (`--indent`, `--line-length`, `--no-sort-imports`, `--check`, `--write`). |
| `ruff test` | Run `.ruff` files under `tests/`; `--update` regenerates expected-output snapshots. |
| `ruff test-run <file>` | Run tests declared with Ruff's `test "name" { ... }` syntax. |
| `ruff bench [path]` | Run benchmark scripts. |
| `ruff bench-cross` | Compare Ruff `parallel_map` against a Python `ProcessPoolExecutor` benchmark. |
| `ruff bench-ssg` | Run the async SSG benchmark, with optional Python comparison and measurement controls. |
| `ruff profile <file>` | Profile a Ruff script for CPU, memory, and JIT stats. |
| `ruff lsp-complete <file> --line <N> --column <N>` | Return completion candidates (builtins, functions, variables) for editor/LSP integration; add `--json` for structured output. |
| `ruff lsp-definition <file> --line <N> --column <N>` | Return the go-to-definition location for the identifier under the cursor; add `--json` for structured output. |
| `ruff lsp-references <file> --line <N> --column <N>` | Return all references for the identifier under the cursor; add `--include-definition false` to exclude declarations and `--json` for structured output. |
| `ruff lsp-hover <file> --line <N> --column <N>` | Return hover details for the identifier under the cursor (kind, detail text, definition location when applicable); add `--json` for structured output. |
| `ruff lsp-diagnostics <file>` | Return source diagnostics for editor refresh loops (delimiter mismatches and parser panic-derived syntax errors); add `--json` for structured output. |
| `ruff lsp-rename <file> --line <N> --column <N> --new-name <NAME>` | Return rename edits for the symbol under the cursor and the updated source text; add `--json` for structured output. |
| `ruff lsp-code-actions <file>` | Return syntax quick-fix actions derived from diagnostics (for example unmatched/unclosed delimiters); add `--json` for structured output. |

Useful environment variables:

| Variable | Effect |
| --- | --- |
| `DISABLE_JIT=1` | Disables JIT support in the VM path. |
| `DEBUG_AST=1` | Prints the parsed AST before VM execution. |
| `RUFF_SCHEDULER_TIMEOUT_MS=<ms>` | Overrides the VM cooperative scheduler completion timeout when `ruff run --scheduler-timeout-ms` is not provided. The default is `120000`. |

Scheduler timeout precedence for `ruff run` is:

1. `--scheduler-timeout-ms <ms>`
2. `RUFF_SCHEDULER_TIMEOUT_MS=<ms>`
3. default `120000 ms`

## v0.12.0 LSP Progress

The highest-priority v0.12.0 roadmap track is Language Server Protocol support. Ruff now includes initial editor-integration primitives:

- `ruff lsp-complete` for builtin/function/variable completion candidates at a cursor position.
- `ruff lsp-definition` for go-to-definition lookup of function/variable/parameter symbols.
- `ruff lsp-references` for symbol reference lookup with optional declaration inclusion.
- `ruff lsp-hover` for symbol hover details across builtins and user-defined symbols.
- `ruff lsp-diagnostics` for syntax-oriented diagnostics suitable for editor refresh cycles.
- `ruff lsp-rename` for scope-aware symbol renaming with deterministic edit output.
- `ruff lsp-code-actions` for diagnostics-driven syntax quick-fix actions.

These are targeted LSP groundwork slices, not a full language server implementation yet.

## v0.12.0 Formatter Progress

Ruff now includes an initial formatter surface via `ruff format` with:

- opinionated spacing/indentation normalization
- configurable indentation width
- line-length-aware wrapping for comma-separated expressions
- leading import-block sorting (optionally disabled)

## Language Overview

Ruff source files use the `.ruff` extension. The implemented syntax includes:

- Numeric, string, boolean, `null`, array, and dictionary values.
- `let`, `mut`, and `const` bindings with `:=` assignment syntax.
- Optional annotations on variables and functions, such as `let x: int := 42` and `func add(a: int, b: int) -> int`.
- Functions declared with `func`, including anonymous function expressions and `async func`.
- `if`/`else`, `while`, `loop`, `for ... in`, `break`, and `continue`.
- Arrays and dictionaries with indexing and standard library helpers.
- String interpolation with `${...}` inside double-quoted strings.
- `Result` and `Option` values through `Ok(...)`, `Err(...)`, `Some(...)`, and `None`.
- `match`/`case` statements, including cases that bind `Ok(value)`, `Err(error)`, and `Some(value)` in interpreter mode.
- `try`/`except` blocks and `throw(...)` for runtime error flow.
- `await` expressions and promise-returning async/native operations.
- `spawn { ... }` syntax and native concurrency helpers.
- `struct` declarations, field access, and struct instance literals.
- `test`, `test_setup`, `test_teardown`, and `test_group` declarations for the test runner.
- `#`, `//`, `/* ... */`, and `///` comments.

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

The standard library is broad, but not all functions have polished reference documentation yet. When adding user-facing docs for a specific API, verify the handler contract in `src/interpreter/native_functions/` and the corresponding regression tests.

## Testing

Ruff has two testing layers.

Run the Rust test suite:

```bash
cargo test
```

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

- Ruff is source-built today. Do not assume official release binaries or package-manager taps are available.
- VM execution is the default, but the tree-walking interpreter still matters for some language surfaces and diagnostics.
- Static typing is optional and not a VM-enforced compile-time contract in the current CLI path.
- `import`/`export` syntax and a module loader exist, but `src/module.rs` currently resolves/parses modules without executing them and collecting exports. Treat the module system as incomplete.
- Struct fields and instance literals exist. Struct method behavior is still a moving target across runtime paths, so avoid documenting it as a stable production feature without a fresh test.
- Spread literals and destructuring syntax exist, but behavior is uneven across runtime modes. Verify them against the exact execution path before documenting them as stable.
- `spawn { ... }` is parsed and executed as fire-and-forget work; do not rely on spawned output or shared state without using the explicit concurrency/native async APIs.
- `Result`/`Option` values are implemented, but richer pattern-binding behavior should be verified against the runtime mode you intend to use.
- Benchmark numbers are environment-sensitive. Prefer `bench-ssg --runs <N> --warmup-runs <N>` and read the measurement warnings before drawing conclusions.

## Documentation

- [INSTALLATION.md](INSTALLATION.md): build and installation notes.
- [CHANGELOG.md](CHANGELOG.md): release and unreleased changes.
- [ROADMAP.md](ROADMAP.md): planned work.
- [CONTRIBUTING.md](CONTRIBUTING.md): contribution workflow.
- [docs/CONCURRENCY.md](docs/CONCURRENCY.md): async/concurrency model notes.
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md): benchmark and profiling notes.
- [docs/VM_INSTRUCTIONS.md](docs/VM_INSTRUCTIONS.md): bytecode VM instruction reference.
- [docs/EXTENDING.md](docs/EXTENDING.md): adding native functionality.

Some older docs may lag the current VM-default execution path. Prefer the code and tests when behavior differs.

## License

Ruff is licensed under the MIT License. See [LICENSE](LICENSE).
