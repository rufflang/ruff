# Ruff

Ruff is an AI-native programming language and runtime built in Rust.
It is designed for production automation, agentic workflows, and application scripting where deterministic behavior, strong native capabilities, and practical ergonomics matter.

Ruff is VM-first (`ruff run`), with a tree-walking interpreter available as an explicit fallback/debug path.

## Current Status

- Ruff is usable from source today.
- VM runtime parity for modular workflows has been significantly hardened.
- Dotted module import workflows are supported on the default VM path.
- Native capability controls are available for trusted and untrusted execution modes.
- Ruff is pre-1.0 and not yet universally production-ready for all enterprise workloads.

## Why Ruff

- VM-first execution for predictable runtime behavior in production scripts.
- Practical native APIs (filesystem, process, network, async, crypto, database).
- Security policy controls for trusted and untrusted execution.
- Module workflows that support both flat and dotted imports.
- Strong diagnostics, contract tests, and release-gate automation.

## 1.0 Readiness Status

- Ruff is not yet ready for a `1.0.0` release.
- [ROADMAP.md](ROADMAP.md) is the single source of truth for release readiness and blocker tracking.
- Ruff `1.0.0` must not be released until all P0/P1 roadmap items and the final release checklist are complete.
- Canonical readiness boundary: Ruff remains pre-1.0 until `ROADMAP.md` and `docs/PRE_V1_MASTER_UNFINISHED_CHECKLIST.md` release gates are closed.
- Deferred/non-goal boundaries are tracked in [docs/V1_SCOPE.md](docs/V1_SCOPE.md) and [docs/OPTIONAL_TYPING_DESIGN.md](docs/OPTIONAL_TYPING_DESIGN.md).

## Safety Model Snapshot

- Ruff is not a sandbox.
- `ruff run` and `ruff test-run` default to trusted mode.
- For untrusted code, start with `--untrusted` and add only required `--allow-*` flags.
- Review [docs/NATIVE_API_SECURITY_POSTURE.md](docs/NATIVE_API_SECURITY_POSTURE.md) before running untrusted scripts in shared or sensitive environments.

### Enterprise Hardening Quickstart

For untrusted scripts, use capability-minimal execution and explicit network intent:

```bash
ruff run --untrusted --allow-fs-read --allow-net-client script.ruff
```

When `--untrusted` and outbound network client access are enabled, Ruff now defaults the outbound destination policy to `deny_private` (unless `RUFF_NET_DESTINATION_POLICY` is already set). This helps reduce accidental private-network access in untrusted runs.

To allow private/local destinations in trusted environments:

```bash
export RUFF_NET_DESTINATION_POLICY=allow_all
# or keep strict mode and permit local/private overrides per execution
export RUFF_ALLOW_PRIVATE_NETWORK_DESTINATIONS=1
```

## Core Reference Links

- [ROADMAP.md](ROADMAP.md)
- [docs/LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md)
- [docs/STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md)
- [docs/INSTALL_MATRIX.md](docs/INSTALL_MATRIX.md)
- [docs/FIRST_TOOL_COOKBOOK.md](docs/FIRST_TOOL_COOKBOOK.md)
- [docs/RELEASE_PROCESS.md](docs/RELEASE_PROCESS.md)
- [docs/VM_INTERPRETER_PARITY_MATRIX.md](docs/VM_INTERPRETER_PARITY_MATRIX.md)

## Install

```bash
git clone https://github.com/rufflang/ruff.git
cd ruff
cargo build --release
./target/release/ruff --version
```

Development usage:

```bash
cargo run -- --help
cargo run -- run examples/hello.ruff
```

Install locally through Cargo:

```bash
cargo install --path .
ruff --version
```

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

## Runtime Mode Recommendations

- Use VM by default (`ruff run <file>`).
- Developers should not need `--interpreter` for ordinary modular project layouts.
- Use `--interpreter` only as an explicit compatibility/debug path when isolating runtime-path issues.
- Migration guidance and diagnostics workflow: [docs/VM_INTERPRETER_MIGRATION_PLAYBOOK.md](docs/VM_INTERPRETER_MIGRATION_PLAYBOOK.md)

## CLI Overview

Common commands:

- `ruff run <file>`: execute Ruff scripts on the VM path.
- `ruff run --interpreter <file>`: execute on the interpreter fallback path.
- `ruff check <file>`: validate source without execution.
- `ruff test`: run snapshot fixture corpus (`--runtime vm|dual|interpreter`, `--update`).
- `ruff test-run <file>`: run Ruff `test "..." {}` declarations in a file.
- `ruff serve [dir]`: static file server for local preview/testing.
- `ruff lsp`: run Ruff’s LSP server.

Machine-readable contracts and diagnostics behavior are documented in [docs/CLI_MACHINE_READABLE_CONTRACTS.md](docs/CLI_MACHINE_READABLE_CONTRACTS.md).

## Repository Layout

- `src/`: core runtime/compiler/parser/VM/interpreter implementation.
- `tests/`: contract, integration, and parity coverage.
- `docs/`: language spec, security posture, roadmap, release process, and readiness checklists.
- `examples/`: runnable scripts and integration fixtures.
- `scripts/`: release gates and generation/verification utilities.

## Repository Hygiene

- Canonical tracked root files are intentionally minimal (`README`, manifests, policy docs).
- Most generated artifacts and local backups are ignored and should not be committed.
- Use the hygiene audit script before publishing release branches:

```bash
bash scripts/repo_hygiene_audit.sh
```

## Language Snapshot

Implemented and actively used surfaces include:

- variables/bindings (`let`, `mut`, `const`), functions (`func`, `async func`), conditionals, loops, structs, enums, `match`, `try/except`, and `throw`.
- arrays/dictionaries, interpolation, string/collection helpers, and a broad native standard library.
- module imports with both flat and dotted paths (for example `from src.util import value`).

Detailed semantics and contracts are in [docs/LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md).

## Testing

Core validation commands:

```bash
cargo test
cargo run -- test --runtime vm
cargo run -- test --runtime dual
cargo test --test vm_interpreter_parity_surfaces
```

Security-focused suites:

```bash
cargo test --test runtime_security
cargo test --test native_api_security_boundaries
```

Release-gate scripts:

```bash
bash scripts/release_gate.sh
bash scripts/release_candidate_gate.sh --full
```
