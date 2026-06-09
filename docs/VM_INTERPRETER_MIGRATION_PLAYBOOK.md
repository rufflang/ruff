# VM-First Migration Playbook (From `--interpreter`-Pinned Workflows)

Status: active migration guidance for teams moving from interpreter-default habits to VM-first execution.

## Goal

Use VM-first paths for normal modular projects while keeping deterministic fallback/debug options available.

## Quick Decision Table

| Scenario | Recommended command | Why |
| --- | --- | --- |
| Day-to-day script execution | `ruff run <file>` | VM is the default runtime and supports ordinary dotted/flat module imports. |
| Package bootstrap and lockfile verification | `ruff package-install` / `ruff package-install --frozen` | Keeps package manifests reproducible and verifies `ruff.lock` without rewriting it. |
| Legacy snapshot compatibility sweeps | `ruff test --runtime dual` | Runs VM first and uses bounded interpreter fallback only when snapshot drift requires it. |
| Strict VM migration gate in CI | `ruff test --runtime vm` | Fails on VM drift directly so teams can burn down fallback dependencies. |
| Targeted compatibility/debug isolation | `ruff run --interpreter <file>` | Explicit fallback path for runtime-difference diagnosis; not required for normal modular layout. |

## Migration Steps

1. Replace `ruff run --interpreter <file>` with `ruff run <file>` in developer docs/scripts.
2. Replace blanket `ruff test`/`--interpreter` test guidance with explicit runtime modes:
   - compatibility lane: `ruff test --runtime dual`
   - strict lane: `ruff test --runtime vm`
3. Keep one debug recipe that still uses interpreter explicitly for diagnosis.
4. Track VM drift burn-down by monitoring dual runtime summary counters (`vm_primary`, `interpreter_fallback`).
5. Verify package projects with `ruff package-install --frozen` before release or CI promotion.

## Recommended Verification Commands

```bash
# 1) Validate strict VM behavior for the fixture corpus
cargo run -- test --runtime vm

# 2) Validate bounded fallback behavior remains deterministic
cargo run -- test --runtime dual

# 3) Re-run parity suite for runtime-path drift coverage
cargo test --test vm_interpreter_parity_surfaces

# 4) Re-run nested module workflow integration coverage
cargo test --test package_module_workflow_integration

# 5) Verify package manifests and lockfiles without rewriting them
cargo run -- package-install --frozen
```

## Known Caveat

If strict VM mode still fails for legacy fixtures, keep user-facing docs on `--runtime dual` while parity burn-down continues. Do not revert general module-import usage docs back to interpreter-required language.
