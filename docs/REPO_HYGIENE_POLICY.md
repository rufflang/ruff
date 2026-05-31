# Repository Hygiene Policy

Status: Active
Last updated: 2026-05-26

## Purpose

Keep the repository root production-facing and predictable by ensuring only intentional project assets are tracked at top level.

## Root Surface Contract

Tracked files at repository root must be limited to canonical project metadata and operator entry docs.

Current allowed tracked root files:

- `.editorconfig`
- `.gitignore`
- `BUG_HUNT_REPORT.md`
- `CHANGELOG.md`
- `CONTRIBUTING.md`
- `Cargo.lock`
- `Cargo.toml`
- `DOGFOOD_NOTES.md`
- `INSTALLATION.md`
- `LICENSE`
- `README.md`
- `ROADMAP.md`
- `rustfmt.toml`

## Non-root Placement Rules

Generated, local, and transient artifacts must not be tracked at root and should live under purpose-specific directories:

- Build/test outputs: `target/`, `tmp/`, `test_dir/`, `var/`
- Generated docs/artifacts: `docs/generated/`
- Local databases/backups/scratch files: untracked and ignored via `.gitignore`
- Runtime/source implementation: `src/`, `tests/`, `examples/`, `scripts/`, `docs/`, `notes/`

## Retention And Cleanup

- Keep local-only scratch artifacts untracked.
- If a temporary file is required for debugging, place it under `tmp/` or `var/` and do not commit it.
- Remove stale local backup/database artifacts before release-candidate verification runs.

## Enforcement

- Fast guard script: `bash scripts/repo_hygiene_audit.sh`
- Contract test: `cargo test --test repo_hygiene_contract`
- Checklist reference: `ER-P0-005` in `docs/V1_0_ENTERPRISE_READINESS_ENHANCEMENT_CHECKLIST.md`
