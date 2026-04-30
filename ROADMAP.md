# Ruff Development Roadmap

This roadmap tracks work that is still current or upcoming. Completed implementation history belongs in [CHANGELOG.md](CHANGELOG.md) and release evidence notes under `notes/`.

> Current crate version: `0.14.0` in [Cargo.toml](Cargo.toml)
> Next planned release: `v1.0.0`
> Last audited: April 30, 2026

---

## Release Focus

`v0.14.0` is complete and released.
Roadmap planning now focuses on `v1.0.0` scope execution and compatibility hardening.

---

## v0.14.0 Release Summary

`v0.14.0` was completed and released with all stabilization checklist items closed.

Completed release tracks:

- release process hardening
- LSP protocol stability guarantees
- packaging and distribution follow-through
- tree-sitter and editor adapter maturity
- runtime and tooling reliability track
- v1.0.0 scope definition gate

For completed implementation details and evidence, see:

- [CHANGELOG.md](CHANGELOG.md)
- `notes/2026-04-30_23-40_release-process-hardening-and-ci-guard.md`
- `notes/2026-04-30_23-55_lsp-protocol-stability-golden-fixtures.md`
- `notes/2026-05-01_00-20_packaging-distribution-follow-through.md`
- `notes/2026-05-01_00-45_tree-sitter-editor-adapter-maturity.md`
- `notes/2026-05-01_01-10_runtime-tooling-reliability-track.md`
- `notes/2026-05-01_01-25_v1-scope-definition-gate.md`

## v1.0.0: Compatibility And Ecosystem Stabilization (Planning)

Primary planning inputs:

- `docs/V1_SCOPE.md`
- `docs/LANGUAGE_SPEC.md`
- `docs/PROTOCOL_CONTRACTS.md`

Top-level execution goals:

- finalize compatibility guarantees for language/runtime behavior
- keep machine-readable CLI/LSP contracts stable and versioned
- maintain reproducible release validation and artifact verification process
- keep editor integration and adapter ownership boundaries explicit

---

## v1.0.0 Readiness

`v0.14.0` stabilization work is complete. `v1.0.0` planning and execution can proceed.

Required before `v1.0.0`:

- stable and documented language/runtime compatibility guarantees
- stable and versioned machine-readable CLI/LSP contracts
- release process that is reproducible and CI-validated
- current and accurate installation/editor integration documentation
- clear long-term maintenance boundaries for core tooling vs adapter layers

---

## Version Strategy

- `v0.11.0`: released (SSG throughput and async scheduler reliability)
- `v0.12.0`: released (developer tooling surfaces)
- `v0.13.0`: released (cross-IDE tooling foundation)
- `v0.14.0`: released (stabilization and `v1.0.0` runway)
- `v1.0.0`: compatibility and ecosystem stabilization milestone

See also:

- [CHANGELOG.md](CHANGELOG.md): completed changes
- [docs/PROTOCOL_CONTRACTS.md](docs/PROTOCOL_CONTRACTS.md): protocol-level contract definitions
- [docs/INSTALLATION_LSP_EDITORS.md](docs/INSTALLATION_LSP_EDITORS.md): install/upgrade guidance
- [docs/RELEASE_ARTIFACT_CHECKLIST_V0_14_0.md](docs/RELEASE_ARTIFACT_CHECKLIST_V0_14_0.md): current release evidence model
- [docs/RELEASE_ARTIFACT_CHECKLIST_V0_13_0.md](docs/RELEASE_ARTIFACT_CHECKLIST_V0_13_0.md): prior release evidence model
