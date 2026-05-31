# Ruff Doctor Architecture

## Why this is a first-party workflow extension

Ruff Doctor is intentionally implemented as a first-party workflow extension (not core language/runtime logic) so it can evolve independently while still using reserved CLI ownership (`doctor` command family).

## Reserved command-family intent

Long-term command-family model:

- `ruff doctor`
- `ruff doctor <profile>`
- `ruff doctor --profile <profile>`

Generic/default checks belong to Ruff Doctor itself, while profile-specific checks are contributed through profile extension points.

## Generic vs profile checks

- Generic checks: environment/repository/dependency readiness useful across projects.
- Profile checks: framework/vendor/team-specific checks layered on top of generic results.

## Discovery model

Current model:

- Built-in first-party Ruff Doctor (`ruff-doctor`) is bundled and registered through workflow builtins.
- Profile contributions are tracked in workflow-pack registry metadata.

Future model:

- Registry- and package-backed profile discovery with stronger trust identity.

## Permissions model

Doctor is read/probe-oriented:

- `writes_files: false`
- `requires_network: false`
- process probing enabled (`runs_processes: true`) via safe process-runner wrappers.
