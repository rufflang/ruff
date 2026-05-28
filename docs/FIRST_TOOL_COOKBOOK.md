# Ruff First Tool Cookbook

Status: v1.0.0 baseline draft (active)
Last updated: 2026-05-25

This guide is a concise, production-oriented path to build your first practical Ruff tool.

## Objective

Build a small CLI quality-gate tool that:

- accepts a JSON policy file path from CLI args,
- validates required keys,
- prints pass/fail diagnostics,
- exits with deterministic behavior suitable for automation.

## 10-Minute Build Path

### 1) Create the tool script

Create `quality_gate.ruff`:

```ruff
argv := args()
if len(argv) < 1 {
  print("usage: ruff run quality_gate.ruff <policy.json>")
  exit(2)
}

policy_path := argv[0]
raw := read_file(policy_path)
parsed := parse_json(raw)

if type(parsed) == "Error" {
  print("policy parse failure")
  exit(4)
}

if has_key(parsed, "name") != 1 || has_key(parsed, "rules") != 1 {
  print("invalid policy: required keys are missing")
  exit(1)
}

if len(parsed["rules"]) == 0 {
  print("invalid policy: rules must not be empty")
  exit(1)
}

print("quality gate ok: " + parsed["name"])
```

Note: `args()` contains only user arguments after the script path.

Semantics notes:

- Use `mut` for values you reassign (for example counters or accumulators).
- Predicate helpers like `contains`, `starts_with`, and `has_key` return `1`/`0`; compare explicitly when needed.
- Collection helpers like `push` return a new array value; reassign (`items = push(items, x)`) to keep the update.

Module export note:

- Imported functions must be declared with `export func` in the source module.

### 2) Create a policy input

Create `policy.json`:

```json
{
  "name": "enterprise-default",
  "rules": ["no-secrets", "docs-present", "tests-green"]
}
```

### 3) Run on VM default path

```bash
ruff run quality_gate.ruff policy.json
```

Expected output:

```text
quality gate ok: enterprise-default
```

## Operational Patterns

- Use non-zero exits for machine decisions:
  - `2` usage errors
  - `1` policy/gate failures
  - `4` runtime/semantic failures
- Keep inputs explicit and validated at script start.
- Prefer `parse_json` + shape checks for deterministic diagnostics.
- Keep tool output short and automation-friendly.

## Extend To A Real Project

- Add file traversal checks with `list_dir` / `path_*` helpers.
- Add network checks with `http_get` under explicit capability policy:
  - trusted mode for local dev
  - `--untrusted --allow-net-client` in controlled automation
- Wrap in CI as a single command gate.

## Scaffold A New Ruff Tool Project

If you want a project skeleton instead of creating every file manually, use Kennel:

```bash
ruff run /path/to/ruff-kennel/kennel.ruff --interpreter -- new my-tool
```

Then adapt the generated `kennel.toml`, entrypoint, and scripts to your tool.

## Verification

Use these suites when evolving onboarding examples:

```bash
cargo test --test docs_examples
cargo test --test readme_contracts
```

These tests ensure docs snippets/examples remain parseable/runnable according to project policy.
