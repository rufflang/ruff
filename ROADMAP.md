# Ruff Development Roadmap

This roadmap tracks only work that is still current or upcoming. Completed implementation history belongs in [CHANGELOG.md](CHANGELOG.md) and release evidence notes under `notes/`.

> Current crate version: `0.14.0` in [Cargo.toml](Cargo.toml)
> Next planned release: `v1.0.0`
> Last audited: May 1, 2026

---

## Release Focus

Ruff's pre-v1 implementation, compatibility hardening, documentation baseline, release automation, and local artifact validation are complete.

The remaining work before public `v1.0.0` is release execution: version bump, final validation, tag publication, artifact verification, and public release evidence.

---

## v1.0.0 Remaining Release Checklist

### P0: Required Before Public v1.0.0

- [ ] Resolve or explicitly waive the locked dependency warning for `core2 v0.4.0`.
  - Current source: `image -> ravif -> rav1e -> bitstream-io -> core2`.
  - Current status: local locked/offline artifact validation passes, but Cargo warns that `core2 v0.4.0` is yanked.
  - Release expectation: either refresh the dependency graph so the warning disappears, or record a signed-off release exception explaining why the locked transitive dependency is acceptable for `v1.0.0`.

- [ ] Run final pre-release validation on the release candidate commit.
  - `cargo test --quiet`
  - `bash .github/scripts/check-release-state.sh`
  - `bash .github/scripts/validate-release-artifact.sh`
  - YAML parse/sanity check for release workflows:
    - `.github/workflows/release-binaries.yml`
    - `.github/workflows/release-published-artifact-smoke.yml`

- [ ] Bump version metadata for `v1.0.0`.
  - Update `[package].version` in `Cargo.toml` from `0.14.0` to `1.0.0`.
  - Update `README.md` current-version text.
  - Update this roadmap's current-version text.
  - Re-run `bash .github/scripts/check-release-state.sh`.

- [ ] Finalize `CHANGELOG.md` for `v1.0.0`.
  - Move finalized `[Unreleased]` entries under a `[1.0.0] - YYYY-MM-DD` section.
  - Update bottom comparison links so `[Unreleased]` compares from `v1.0.0` to `HEAD`.
  - Add or update the `[1.0.0]` tag comparison link.

- [ ] Create and push the release commit.
  - Include version metadata, changelog, roadmap, and any final release-process/doc edits.
  - Push the release commit to `main`.

- [ ] Create and push the annotated `v1.0.0` tag.
  - Example: `git tag -a v1.0.0 -m "Ruff v1.0.0"`
  - Push the tag to `origin`.

- [ ] Publish the GitHub release for `v1.0.0`.
  - Use the `v1.0.0` tag.
  - Use finalized changelog notes as release notes.
  - Confirm `.github/workflows/release-binaries.yml` attaches Linux/macOS archives and checksum files.

- [ ] Verify published release artifacts.
  - Confirm Linux and macOS artifact assets are attached.
  - Confirm per-asset `.sha256` files and consolidated `checksums.txt` are attached.
  - Confirm `.github/workflows/release-published-artifact-smoke.yml` passes for the published release.
  - Run the documented artifact install flow from [INSTALLATION.md](INSTALLATION.md) using the published `v1.0.0` assets.

- [ ] Record final public release evidence.
  - Create a dated note under `notes/`.
  - Include artifact URLs, checksum values, workflow URLs/statuses, exact validation commands, and pass/fail logs.
  - Update [docs/RELEASE_ARTIFACT_CHECKLIST_V1_0_0.md](docs/RELEASE_ARTIFACT_CHECKLIST_V1_0_0.md) only after the published-artifact checks pass.

### P1: Strongly Recommended Before Announcement

- [ ] Smoke-test the most visible examples you plan to keep in the public README/docs.
  - This is not a language/runtime release blocker, but it is important for first-user experience.
  - At minimum, confirm any examples linked from `README.md`, `INSTALLATION.md`, or release notes run with the published binary.

- [ ] Review public-facing install wording after release assets exist.
  - Confirm all `v1.0.0` artifact names in docs match the actual published asset names.
  - Confirm the install guide does not imply package-manager support beyond the shipped archive flow.

- [ ] Decide whether image conversion support is in or out of public v1 messaging.
  - If it remains out of scope, keep examples/docs from implying reliable image format conversion.
  - If it becomes in scope, complete the work tracked in [docs/IMAGE_CONVERSION_AGENT_HANDOFF.md](docs/IMAGE_CONVERSION_AGENT_HANDOFF.md) before announcing it.

---

## Current Release Position

There are no remaining known P0 language/runtime parity blockers in the tracked v1 surface.

The official public `v1.0.0` release is blocked only by the open release-execution items above.

---

## References

- [CHANGELOG.md](CHANGELOG.md): completed changes and release notes source.
- [docs/V1_SCOPE.md](docs/V1_SCOPE.md): v1 scope definition and compatibility commitments.
- [docs/LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md): language/tooling compatibility baseline.
- [docs/PROTOCOL_CONTRACTS.md](docs/PROTOCOL_CONTRACTS.md): protocol-level contract definitions.
- [docs/RELEASE_PROCESS.md](docs/RELEASE_PROCESS.md): release execution process.
- [docs/RELEASE_ARTIFACT_CHECKLIST_V1_0_0.md](docs/RELEASE_ARTIFACT_CHECKLIST_V1_0_0.md): artifact publication and tag-time sign-off checklist.
