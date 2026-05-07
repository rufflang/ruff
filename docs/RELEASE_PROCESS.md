# Ruff Release Process

This document defines the canonical release workflow for Ruff.

Current active target: v1.0.0

## Goals

- make release execution reproducible and reviewable
- prevent version/status drift between Cargo metadata and release docs
- define patch release policy for post-release fixes

## Release Types

- Major compatibility release: `v1.0.0`
- Patch release: `v1.0.x` where `x >= 1`

## Patch Release Policy (`v1.0.x`)

Patch releases are allowed only for:

- regressions introduced in `v1.0.0`
- correctness bugs in runtime, CLI, LSP, or packaging/distribution flow
- security fixes
- release-process and artifact-installation fixes that block adoption

Patch releases must not include:

- new language syntax
- major runtime behavior expansions
- roadmap features planned for a future minor/major cycle

Patch-release requirements:

1. Include regression tests for the fixed behavior.
2. Add a clear `CHANGELOG.md` patch entry with user impact.
3. Attach release evidence note under `notes/` with commands and outputs.
4. Keep docs consistent with shipped behavior and status text.

## Pre-Release Checklist (Before Version Bump)

Run from repository root.

1. Confirm clean intent and inspect local state.

```bash
git status --short
git branch --show-current
```

2. Run the canonical release gate command.

```bash
bash scripts/release_gate.sh
```

Release-gate notes:

- `scripts/release_gate.sh` runs full gate checks in this order: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, selected integration suites, Ruff self-test (`cargo run -- test`), and optional `cargo audit`/`cargo deny check` when installed.
- Enable socket-bound serve integration checks with `RUFF_ENABLE_SOCKET_TESTS=1`.
- Enable optional benchmark smoke with `RUFF_RELEASE_GATE_RUN_BENCH=1`.
- Use `bash scripts/release_gate.sh --minimal` for a lightweight smoke run (used by CI to validate script wiring quickly).
- Expected runtime: minimal mode is typically a few minutes; full mode is typically several minutes and may run longer on busy machines.

3. Validate extension/editor baseline when relevant to cycle scope.

```bash
cd tools/vscode-ruff-extension
npm ci
npm run check
cd ../..
```

4. Run the release-state consistency guard.

```bash
bash .github/scripts/check-release-state.sh
```

5. Ensure roadmap release-checklist items claimed complete have evidence in `notes/`.
6. Ensure `docs/RELEASE_ARTIFACT_CHECKLIST_V1_0_0.md` sign-off items are complete.

## Version Bump + Changelog Sectioning

1. Set `[package].version` in `Cargo.toml` to target version.
2. Add new release section to `CHANGELOG.md` (for example: `[1.0.0] - YYYY-MM-DD`) and move finalized unreleased entries under that section.
3. Update comparison links at the bottom of `CHANGELOG.md`:
	- `[Unreleased]` should compare from new tag to `HEAD`
	- add/update version link for the new tag
4. Update release-status strings in:
	- `README.md`
	- `ROADMAP.md`
5. Re-run:

```bash
bash .github/scripts/check-release-state.sh
```

## Tagging And Publish Order

Use this order exactly.

1. Create release commit.

```bash
git add Cargo.toml CHANGELOG.md README.md ROADMAP.md
git commit -m ":rocket: RELEASE: v1.0.0"
git push origin main
```

2. Create and push annotated tag.

```bash
git tag -a v1.0.0 -m "Ruff v1.0.0"
git push origin v1.0.0
```

3. Publish GitHub release using the same tag and changelog notes.

4. If release artifact/checksum docs are updated post-tag, commit doc-only follow-up immediately.

## Dry-Run Procedure (No Tag, No Publish)

Use this for rehearsal before final release.

1. Create a temporary branch.

```bash
git checkout -b release-dry-run-v1.0.0
```

2. Execute all pre-release checks.
3. Perform version/doc edits as if shipping.
4. Run full validation commands and release-state guard.
5. Create a dry-run commit only:

```bash
git add Cargo.toml CHANGELOG.md README.md ROADMAP.md docs/RELEASE_PROCESS.md
git commit -m ":ok_hand: IMPROVE: dry-run release workflow validation"
```

6. Record command log and outcomes in a dated `notes/` evidence file.
7. Open PR for review if needed, then discard or reset dry-run branch after review.

Dry-run success criteria:

- no undocumented manual steps were needed
- every command had deterministic expected output
- release-state guard passed after doc/version edits

## Required Evidence For Release Sign-Off

Create a dated note in `notes/` that includes:

- host/date/context (local vs CI)
- exact commands executed
- pass/fail status per command
- any warnings that remain release-relevant
- explicit exception rationale if local machine load affects benchmark confidence

Do not mark release-checklist items complete without this evidence.
