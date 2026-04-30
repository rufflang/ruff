# Ruff Development Roadmap

This roadmap tracks work that is still current or upcoming. Completed implementation history belongs in [CHANGELOG.md](CHANGELOG.md) and release evidence notes under `notes/`.

> Current crate version: `0.13.0` in [Cargo.toml](Cargo.toml)
> Next planned release: `v0.14.0`
> Last audited: April 30, 2026

---

## Release Focus

`v0.13.0` is complete and release-ready from a feature perspective.
Roadmap planning now focuses on `v0.14.0` stabilization work needed to reduce risk before `v1.0.0`.

---

## v0.14.0: Stabilization And 1.0 Runway (Release Checklist)

`v0.14.0` is complete only when all required items below are done.

### 1. Release Process Hardening

Required features:

- [ ] add a release playbook in `docs/RELEASE_PROCESS.md` covering version bump, changelog sectioning, checklist verification, and tagging order
- [ ] add a CI guard that fails if `Cargo.toml` version and README/ROADMAP release status are inconsistent
- [ ] define and document patch-release policy (`v0.14.x`) for post-release fixes

Acceptance criteria:

- [ ] dry-run release execution can be completed with no manual guesswork
- [ ] CI catches version-state drift before merge

### 2. LSP Protocol Stability Guarantees

Required features:

- [ ] lock JSON-RPC error envelopes and success payload contracts behind golden fixtures
- [ ] publish compatibility table for supported LSP methods and known unsupported method behavior
- [ ] add regression fixtures for multi-file workspace symbol/rename/reference edge cases

Acceptance criteria:

- [ ] protocol fixtures fail on contract shape drift unless intentionally updated
- [ ] docs and fixture expectations match for all supported methods

### 3. Packaging And Distribution Follow-Through

Required features:

- [ ] validate release artifacts on Linux/macOS in clean environments using documented install flow
- [ ] add reproducible binary verification steps (checksum generation + verification docs)
- [ ] document minimum supported Rust toolchain and platform assumptions

Acceptance criteria:

- [ ] artifact validation logs are captured in a dated release evidence note
- [ ] install instructions can be executed end-to-end without repository context

### 4. Tree-sitter And Editor Adapter Maturity

Required features:

- [ ] expand `tree-sitter-ruff` corpus coverage for current parser edge cases
- [ ] verify highlight-query behavior against representative Ruff syntax samples
- [ ] publish adapter maintenance policy (what stays in Ruff docs vs editor-specific repos)
- [ ] ship first-party VS Code/Cursor/Codex extension baseline with `.ruff` language registration and syntax colorization enabled on file open
- [ ] ship Ruff LSP client wiring in first-party extension using canonical `ruff lsp` launch path
- [ ] document `.vsix` packaging/install flow so non-VS Code forks can use same extension artifact

Acceptance criteria:

- [ ] grammar corpus tests include regression fixtures for previously reported parse/highlight issues
- [ ] adapter docs remain thin and reference canonical Ruff contracts
- [ ] opening a `.ruff` file in VS Code/Cursor/Codex with extension installed shows Ruff language mode and syntax highlighting without manual mode switching
- [ ] extension smoke check (`npm run check`) passes in CI or release artifact validation sequence

### 5. Runtime And Tooling Reliability Track

Required features:

- [ ] add resilience tests for malformed LSP message sequences and document lifecycle churn (`didOpen`/`didChange`/`didClose` ordering)
- [ ] add bounded-memory checks for repeated diagnostics/completion request loops
- [ ] track startup/first-response latency baselines for the `ruff lsp` server

Acceptance criteria:

- [ ] no known panic paths for malformed protocol traffic
- [ ] reliability test suite is green in CI matrix and documented in release evidence

### 6. v1.0.0 Scope Definition Gate

Required features:

- [ ] publish `docs/V1_SCOPE.md` with explicit in-scope/out-of-scope boundaries for `v1.0.0`
- [ ] define compatibility commitments for language syntax/runtime behavior and machine-readable tooling contracts
- [ ] record deferred post-1.0 candidates (generics, FFI, WASM target, macro system) as non-blocking backlog

Acceptance criteria:

- [ ] `v1.0.0` scope can be reviewed without mining historical roadmap notes
- [ ] `v0.14.0` release notes clearly call out what remains before `v1.0.0`

---

## v1.0.0 Readiness

`v1.0.0` planning should proceed only after `v0.14.0` stabilization work is complete.

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
- `v0.14.0`: stabilization and `v1.0.0` runway
- `v1.0.0`: compatibility and ecosystem stabilization milestone

See also:

- [CHANGELOG.md](CHANGELOG.md): completed changes
- [docs/PROTOCOL_CONTRACTS.md](docs/PROTOCOL_CONTRACTS.md): protocol-level contract definitions
- [docs/INSTALLATION_LSP_EDITORS.md](docs/INSTALLATION_LSP_EDITORS.md): install/upgrade guidance
- [docs/RELEASE_ARTIFACT_CHECKLIST_V0_13_0.md](docs/RELEASE_ARTIFACT_CHECKLIST_V0_13_0.md): prior release evidence model
