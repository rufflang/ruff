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
- ship v1.0.0 with downloadable prebuilt binaries so users can run Ruff directly from terminal without Cargo

### v1.0.0 Distribution And Install Release Gate (Explicit)

v1.0.0 is not considered release-ready until Ruff can be installed and used as a standalone CLI binary (for example, `ruff run build.ruff`) without requiring users to build from source via Cargo.

Required distribution outcomes:

- [ ] publish prebuilt release binaries for supported OS targets (at minimum Linux and macOS)
- [ ] publish SHA-256 checksums for each shipped artifact and document verification steps
- [ ] publish copy/paste install instructions for standalone binary usage (no Cargo dependency)
- [ ] validate fresh-machine install + execution flow using published artifacts only
- [ ] verify direct command usage in terminal (`ruff --version`, `ruff run <file>`, `ruff lsp --help`) after install

Acceptance criteria:

- [ ] a new user can install Ruff from release artifacts and run Ruff commands without cloning repository source
- [ ] release evidence notes include artifact URLs, checksums, and pass/fail logs for install/exec validation
- [ ] CI/release checklist includes artifact publication and checksum verification as required sign-off steps

### Comprehensive Pre-v1 Enhancement Backlog

Use this checklist as the execution queue for follow-up sessions.

#### P0: Must-Do Before v1.0.0 Tag

- [x] Complete module system execution and export semantics.
	- [x] Implement module evaluation and export collection in `src/module.rs` (remove parser-only placeholder behavior).
	- [x] Replace silent import failures with deterministic runtime diagnostics in `src/interpreter/mod.rs`.
	- [x] Add circular-import and missing-symbol regression tests with stable error shapes.
- [x] Remove parser panic paths for user-provided syntax.
	- [x] Replace `panic!`-based type-annotation parse failures with structured parse errors in `src/parser.rs`.
	- [x] Add malformed `Result<T, E>` and `Option<T>` syntax tests to ensure non-panicking behavior.
- [ ] Close VM/interpreter behavior parity gaps for currently documented language surfaces.
	- [x] Build a parity matrix and test coverage baseline for struct method behavior, spread/destructuring, match bindings, and spawn semantics.
	- [x] Close VM destructuring binding gap surfaced by parity tests (array + dict destructuring now bind correctly on VM path).
	- [x] Close VM spread literal capability gap surfaced by parity tests (array/dict spread semantics now align in parity-covered scenarios).
	- [ ] Close remaining capability gaps identified by the parity matrix (tag-style `match` binding capability gap).
	- [x] Ensure documented behavior in `README.md` and `docs/LANGUAGE_SPEC.md` matches current runtime-path status.
- [x] Freeze and verify CLI/LSP contract versioning for v1 baseline.
	- [x] Align contract status/version metadata in `docs/CLI_MACHINE_READABLE_CONTRACTS.md`, `docs/PROTOCOL_CONTRACTS.md`, and `docs/LANGUAGE_SPEC.md`.
	- [x] Add CI check(s) that fail on contract-doc/version drift.
- [ ] Expand negative-path contract fixtures for automation reliability.
	- Add fixtures/tests for malformed params, unknown symbols, IO failures, and consistent stderr/exit-code behavior.
	- Ensure failure payload guarantees are documented and fixture-locked.

#### P1: Strongly Recommended For v1 Completeness

- [ ] Publish explicit v1 optional-typing stance.
	- Convert exploratory wording in `docs/OPTIONAL_TYPING_DESIGN.md` into a concrete v1 policy (supported vs experimental vs deferred).
	- Add tests for whichever v1 typing contract is declared.
- [ ] Expand native standard library documentation coverage to canonical reference quality.
	- Generate/maintain per-function docs with stability tiers and examples for major builtin categories.
	- Ensure docs map to actual runtime dispatch behavior and tests.
- [ ] Add end-to-end package + module workflow integration tests.
	- Cover `ruff init`, `package-add`, `package-install`, import/export execution, and run/lint/format/docgen flows together.

#### P2: Nice-To-Have Hardening Before/Just After v1

- [ ] Publish a formal deprecation policy for CLI/LSP/runtime surfaces.
	- Define warning format, deprecation windows, and removal policy tied to semver.
- [ ] Add a security posture pass for high-risk native APIs.
	- Document trust model and operational caveats for process/network/filesystem/crypto/database builtins.
	- Add targeted regression tests for failure and misuse boundaries.

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
