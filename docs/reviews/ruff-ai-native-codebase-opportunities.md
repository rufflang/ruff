# Ruff AI-Native Codebase Opportunities Review

## Executive summary

Ruff already has strong machine-readable JSON contracts and a clear diagnostics model, but human-readable output and script-level reporting patterns are still highly repetitive and mostly ad hoc.

The biggest near-term opportunities are:

1. Add a small shared output/report rendering primitive set (Rust-side first, Ruff-script-facing second).
2. Reduce duplicated JSON/human output branching in `src/main.rs` command handlers.
3. Add snapshot/golden coverage for human-readable renderers (not just JSON contracts).
4. Update docs/examples to teach intent-first output patterns (instead of manual `print` + alignment mechanics).
5. Burn down expected-fail docs/example debt so agent context is cleaner and more trustworthy.

Recommended implementation order:

1. Output/render helper foundations + tests.
2. Refactor high-churn CLI/render call sites.
3. Documentation and example refresh.
4. Regression and compatibility validation.

## Highest-priority opportunities

### 1) Consolidate Rust CLI output emission paths

- Problem
  - Many command handlers hand-roll near-identical `if json { serialize } else { println! }` logic with repeated serialization error handling and repeated line formatting.
- Evidence from the codebase
  - `src/main.rs:2349-2620` (`LspComplete`, `LspDefinition`, `LspReferences`, `LspHover`, `LspDiagnostics`, `LspRename`, `LspCodeActions`) duplicates JSON serialization + plain-text rendering patterns.
  - `src/main.rs:2365-2369`, `2394-2398`, `2436-2440`, `2474-2478`, `2508-2512`, `2561-2565`, `2602-2606` repeat the same serialization-failure flow.
  - `src/main.rs:2031-2270` SSG benchmark output includes a long sequence of manually formatted sections/warnings/hints.
- Why it matters
  - High drift risk between commands.
  - Harder for agents to safely edit one command without missing cross-command conventions.
  - Higher token and diff noise for any output-contract-related change.
- Recommended direction
  - Introduce a minimal Rust-side `CliOut` helper (or `output::emit`) with:
    - `emit_json<T: Serialize>(value, context)`
    - `line`, `blank`, `section`, `kv`, `list_item`
    - optional `warn`/`error` wrappers for consistent stderr style.
- Risks
  - Could accidentally change exact human output shape.
  - Could unintentionally alter stderr/stdout routing.
- Suggested implementation checklist
  - Add helper module with unit tests for JSON emission failure behavior and plain line composition.
  - Migrate one small command family first (LSP helpers), preserving existing output text.
  - Add contract tests for migrated commands before broad rollout.

### 2) Introduce a standard report/output DSL for Ruff scripts and examples

- Problem
  - Script-facing patterns are verbose and mechanical (`print` storms, manual padding, `push` reassignment loops).
- Evidence from the codebase
  - `examples/benchmarks/run_benchmarks.ruff:72-178` manually builds headers, paddings, aligned columns, and repeated status lines.
  - `examples/expense_tracker.ruff:16-24`, `53-55`, `74-111` repeatedly prints banners, sections, and list items manually.
  - `docs/WORKFLOW_PACKS.md:103-131` sample command manually pushes checks and computes summary counters.
  - `docs/FIRST_TOOL_COOKBOOK.md:64` explicitly teaches reassignment-based `push` update semantics.
- Why it matters
  - Teaches low-level mechanics instead of intent.
  - Generates long repetitive code blocks in agent context.
  - Increases chance of formatting/label inconsistency.
- Recommended direction
  - Provide a tiny, explicit output/report helper surface for Ruff scripts (module or stdlib-level):
    - `out.section(title)`
    - `out.kv(label, value)`
    - `out.list(items)`
    - `out.table(headers, rows)`
    - `report.new/check/summary/render` for doctor/checklist-like tools.
- Risks
  - Prematurely over-scoping a DSL.
  - Confusion if helpers overlap existing JSON-only workflows.
- Suggested implementation checklist
  - Start with 4-5 minimal helpers and keep them composable.
  - Port one canonical example (`run_benchmarks.ruff` or workflow-pack sample) to validate ergonomics.
  - Add example-output contract tests.

### 3) Add human-render output golden tests beyond diagnostics

- Problem
  - JSON surfaces are well-covered, but many human-readable renderers are not contract-protected.
- Evidence from the codebase
  - Strong coverage exists for diagnostics and JSON contracts:
    - `tests/diagnostics_golden.rs:107-169`
    - `tests/cli_json_contracts.rs:73+`
    - `docs/CLI_MACHINE_READABLE_CONTRACTS.md:47-207`
  - But renderer-heavy modules have little/no snapshot coverage:
    - `src/workflow_pack/renderer.rs:10-88`, `135-220` (no output snapshot tests)
    - `src/benchmarks/reporter.rs:10-169`
    - `src/benchmarks/profiler.rs:355-422`
    - `src/repl.rs:124-327`
- Why it matters
  - Human-facing output is a stability surface for operators, onboarding docs, and AI scraping workflows.
  - Refactors become risky without byte-shape guards.
- Recommended direction
  - Add golden tests for representative human-rendered outputs in these modules.
- Risks
  - Brittle tests if color control and environment normalization are inconsistent.
- Suggested implementation checklist
  - Add `NO_COLOR`-normalized snapshot helpers.
  - Snapshot workflow pack human + markdown outputs.
  - Snapshot benchmark and REPL help/banner output.

### 4) Align docs/examples with AI-native style guidance

- Problem
  - Current docs intentionally teach valid behavior, but not yet a preferred concise output-building style.
- Evidence from the codebase
  - `docs/FIRST_TOOL_COOKBOOK.md:24-50` teaches direct `print` chains.
  - `docs/STANDARD_LIBRARY_REFERENCE.md:81-85` emphasizes reassigning collection helper results.
  - `docs/WORKFLOW_PACKS.md:103-131` sample emphasizes manual summary/aggregation mechanics.
  - `tests/docs_examples.rs:91-161` tracks 29 expected-fail examples; this reduces trust in examples as canonical agent context.
- Why it matters
  - Docs are high-priority context for both humans and agents.
  - Verbose/legacy patterns get copied forward.
- Recommended direction
  - Add a style section for "human + agent readable CLI/report output" and migrate canonical docs examples to that style.
- Risks
  - Mixed style during migration can confuse contributors.
- Suggested implementation checklist
  - Define one "preferred" and one "acceptable low-level" style.
  - Update cookbook + workflow docs after helpers exist.
  - Add policy checks or doc lints for newly added examples.

### 5) Reduce repetitive text-construction mechanics in Rust renderers

- Problem
  - Several modules manually append/format lines where semantic helpers would be clearer.
- Evidence from the codebase
  - `src/errors.rs:244-267` builds human output via `Vec<String>` + `push` + `join("\n")`.
  - `src/benchmarks/profiler.rs:341-351` manually builds flamegraph line arrays.
  - `src/interpreter/test_runner.rs:178-219`, `src/workflow_pack/renderer.rs:13-87`, `src/benchmarks/reporter.rs:12-166` are print-heavy imperative rendering blocks.
- Why it matters
  - High local repetition and low semantic signaling.
  - Agents editing one line risk missing adjacent required output conventions.
- Recommended direction
  - Introduce tiny local helper functions before broader abstractions (e.g., `print_header`, `print_kv`, `status_badge`, `render_section`).
- Risks
  - Over-abstraction can hide order-sensitive behavior.
- Suggested implementation checklist
  - Apply small helper extraction with no behavior changes.
  - Add before/after snapshot assertions.

## Repeated output/rendering patterns

1. Rust CLI JSON/human branching duplication
- `src/main.rs:2349-2620` repeats per-command serialization and fallback formatting.

2. Repeated human status rendering
- `src/workflow_pack/renderer.rs:61-87` manually maps statuses + suggested fix output.
- `src/interpreter/test_runner.rs:182-219` repeats pass/fail summary layout.

3. Manual section headers and separators
- `src/benchmarks/reporter.rs:10-20`, `123-167`.
- `src/benchmarks/profiler.rs:358-422`.
- `src/repl.rs:124-139`, `257-289`.

4. Line-vector construction patterns
- `src/errors.rs:244-267`.
- `src/benchmarks/profiler.rs:341-351`.

5. Ruff-script repetitive print/table mechanics
- `examples/benchmarks/run_benchmarks.ruff:126-171`.
- `examples/expense_tracker.ruff:16-24`, `53-55`, `97-107`.

6. Docs teaching manual push/reassign loops
- `docs/FIRST_TOOL_COOKBOOK.md:64`.
- `docs/WORKFLOW_PACKS.md:109-121`.

## Token-efficiency opportunities

1. Duplicate serializer/error branches in `src/main.rs` LSP commands
- Why it costs tokens: same control flow repeated across 7 commands.
- Affects: humans + agents.
- Best fix: Rust helper + local refactor.

2. Manual alignment/padding in Ruff examples
- Evidence: `examples/benchmarks/run_benchmarks.ruff:142-163`.
- Why it costs tokens: alignment mechanics dominate business intent.
- Affects: mostly agents and onboarding humans.
- Best fix: helper convention (`out.table`) + example rewrite.

3. Repeated status/report boilerplate in workflow docs example
- Evidence: `docs/WORKFLOW_PACKS.md:103-131`.
- Why it costs tokens: check accumulation and summary recomputation boilerplate.
- Affects: both.
- Best fix: report helper API + docs update.

4. `push` reassignment verbosity in iterative construction
- Evidence: `docs/STANDARD_LIBRARY_REFERENCE.md:83-85`, `examples/ssg/test_push_fix.ruff:8-31`.
- Why it costs tokens: extra identifier repetition (`items = push(items, v)`) across loops.
- Affects: mostly agents.
- Best fix: convention/helper first; potential language ergonomic feature later.

5. Example debt in expected-fail corpus
- Evidence: `tests/docs_examples.rs:91-161`.
- Why it costs tokens: agents consume examples that are intentionally stale/invalid and require exception logic.
- Affects: both.
- Best fix: docs/example cleanup program + expected-fail reduction policy.

## Semantic compression opportunities

1. Status line emission
- Current: multiple `println!` format blocks with manual status color mapping.
- Better shape: `out.status(check.status, check.label, check.observed)`.

2. Section and separator rendering
- Current: repeated `println!("{}", "=".repeat(...))` and blank-line calls.
- Better shape: `out.section("Summary")`, `out.separator()`.

3. Key-value summaries
- Current: repeated `println!("  Label: {}", value)`.
- Better shape: `out.kv("Label", value)` with stable alignment.

4. Table output
- Current: manual pad calculation and concatenation in Ruff scripts.
- Better shape: `out.table(headers, rows)`.

5. Report composition in workflow scripts
- Current: manual `checks` push + `summary` mutation loops.
- Better shape: `report.add(check)` + `report.render_json()`.

## Potential standard primitives or helpers

### A) `out.line` / `out.blank` / `out.section`

- Name candidates
  - `out.line`, `out.blank`, `out.section`
  - `print_line`, `print_blank`, `print_section`
- Purpose
  - Remove repetitive section/header boilerplate in CLI/report output.
- Example API
  - `out.section("Summary")`
  - `out.line("done")`
  - `out.blank()`
- Example before/after code
  - Before: repeated `println!` blocks in `src/repl.rs:257-289`.
  - After: concise section calls with explicit intent.
- Where it would be used
  - `src/repl.rs`, `src/benchmarks/reporter.rs`, `src/interpreter/test_runner.rs`.
- Test requirements
  - Snapshot tests for spacing/newline behavior.
  - Color/no-color parity tests.

### B) `out.kv(label, value)`

- Name candidates
  - `out.kv`, `out.field`, `out.pair`
- Purpose
  - Standardize aligned key-value output formatting.
- Example API
  - `out.kv("Warmup runs", warmup_runs)`
- Example before/after code
  - Before: `src/main.rs:2033-2036`, `2078-2079`, `2136-2139`.
  - After: repeated metrics become single-shape statements.
- Where it would be used
  - SSG benchmark and profiling outputs.
- Test requirements
  - Width/alignment tests.
  - Numeric formatting stability tests.

### C) `out.list(items)` / `out.warn` / `out.error`

- Name candidates
  - `out.list`, `out.bullets`
  - `out.warn`, `out.error`, `out.success`
- Purpose
  - Normalize warning and action hint sections.
- Example API
  - `out.warn("High variability detected")`
  - `out.list(hints)`
- Example before/after code
  - Before: `src/main.rs:2232-2238`, `2258-2269`.
  - After: warnings/hints rendered with one helper family.
- Where it would be used
  - SSG warning sections and workflow summaries.
- Test requirements
  - Stable prefix/indent tests.

### D) `out.table(headers, rows)`

- Name candidates
  - `out.table`, `report.table`, `render_table`
- Purpose
  - Replace manual padding logic in Ruff scripts and Rust benchmark output.
- Example API
  - `out.table(["Benchmark", "VM", "JIT"], rows)`
- Example before/after code
  - Before: `examples/benchmarks/run_benchmarks.ruff:142-163`.
  - After: rows-as-data, renderer handles spacing.
- Where it would be used
  - Ruff benchmark examples and potential built-in workflow pack reports.
- Test requirements
  - Snapshot tests for column layout and long values.

### E) `Report` builder for workflow/check outputs

- Name candidates
  - `Report.new`, `report.check`, `report.summary`, `report.render`
  - `DoctorReportBuilder`
- Purpose
  - Reduce manual check aggregation + status computation in scripts.
- Example API
  - `report := Report.new("acme", "doctor")`
  - `report.add(check_warn(...))`
  - `print(report.to_json())`
- Example before/after code
  - Before: `docs/WORKFLOW_PACKS.md:103-131`.
  - After: no manual summary mutation loop.
- Where it would be used
  - Workflow pack docs samples and external pack authoring.
- Test requirements
  - Summary correctness tests (pass/warn/fail/skip/info).
  - JSON shape compatibility tests against existing schema.

### F) Rust `emit_json_or_text` helper

- Name candidates
  - `emit_json_or_text`, `CliEmitter`, `OutputModeEmitter`
- Purpose
  - Remove duplicated command-branch boilerplate and unify serialization failure handling.
- Example API
  - `emit_json_or_text(json, || render_text(...), mode)`
- Example before/after code
  - Before: `src/main.rs:2349-2620` repeated per command.
  - After: one standard emission path.
- Where it would be used
  - LSP command handlers and other dual-mode CLI surfaces.
- Test requirements
  - Output/stderr/exit-code contract tests for both modes.

## Agent-readability risks

1. Near-identical output branches in `src/main.rs` make partial edits error-prone.
- Likely failure mode: agent updates one command’s JSON shape but misses others.
- Mitigation: shared emission helpers + command-family tests.

2. Manual alignment strings and spacing math in examples hide intent.
- Likely failure mode: accidental column drift or broken layout during edits.
- Mitigation: table/kv helpers with snapshot tests.

3. Style-teaching docs currently emphasize mechanics (push reassignment, print chaining).
- Likely failure mode: generated scripts copy verbose anti-patterns.
- Mitigation: explicit "preferred style" section after helper rollout.

4. Human renderers are under-tested compared to JSON contracts.
- Likely failure mode: unnoticed regressions in operator-facing output.
- Mitigation: add human-render goldens for workflow/benchmark/repl surfaces.

5. Expected-fail example debt introduces ambiguous truth sources.
- Likely failure mode: agent consumes stale pattern as canonical usage.
- Mitigation: reduce expected-fail set and label legacy examples more explicitly.

## Documentation updates needed

1. `docs/FIRST_TOOL_COOKBOOK.md`
- Add preferred output/report style section with helper-based examples.

2. `docs/WORKFLOW_PACKS.md`
- Replace manual summary/push sample with report-builder style once available.
- Keep explicit JSON schema but reduce mechanics in tutorial snippet.

3. `docs/STANDARD_LIBRARY_REFERENCE.md`
- Add concise guidance for output/report helper usage and when low-level `push` patterns remain acceptable.

4. `README.md`
- Add short pointer to "AI-readable scripting style" section once created.

5. `docs/CLI_MACHINE_READABLE_CONTRACTS.md`
- Add note on human-render consistency expectations for dual-mode command families where appropriate.

## Implementation checklist for the next agent

### Phase 1: Discovery and tests

- Inventory all dual-mode CLI commands (`--json` + human) and record current output shapes for representative fixtures.
- Add/extend snapshot harness for human renderer outputs in:
  - `src/workflow_pack/renderer.rs`
  - `src/benchmarks/reporter.rs`
  - `src/benchmarks/profiler.rs`
  - `src/repl.rs` (banner/help)
- Add focused contract tests for LSP command family output/stderr behavior before refactor.
- Document any behavior that is intentionally unstable vs stability-surface.

### Phase 2: Minimal helper implementation

- Add a Rust output helper module with:
  - `emit_json<T: Serialize>` (shared serialization + error handling)
  - `section`, `kv`, `blank`, `list_item` (small pure helpers)
- Keep helpers intentionally simple; no broad trait-heavy abstraction.
- Add unit tests for helper output formatting and serialization error paths.

### Phase 3: Refactor high-value call sites

- Refactor LSP command handlers in `src/main.rs` to use shared JSON emission helper without changing payload fields.
- Refactor one high-noise human renderer (suggest `src/workflow_pack/renderer.rs`) to helper calls while preserving output.
- Refactor one benchmark renderer (`src/benchmarks/reporter.rs` or `src/benchmarks/profiler.rs`) as a second validation target.
- Run and update snapshots only when output intent changes are explicitly approved.

### Phase 4: Documentation and examples

- Update `docs/FIRST_TOOL_COOKBOOK.md` with preferred helper-based output style.
- Update `docs/WORKFLOW_PACKS.md` sample to reduce manual summary and list mechanics.
- Add short style guidance to `docs/STANDARD_LIBRARY_REFERENCE.md` on output/report patterns.
- Convert at least one benchmark/example script to the new style as canonical reference.

### Phase 5: Regression review

- Run contract suites:
  - `cargo test --test cli_json_contracts`
  - `cargo test --test cli_contracts`
  - diagnostics goldens
  - docs/examples smoke tests
- Confirm no changes to documented JSON schemas and exit-code policy.
- Verify human-render snapshots are deterministic with `NO_COLOR=1`.
- Produce a migration note listing any call sites intentionally left on low-level patterns and why.

## Files reviewed

- `src/main.rs`
- `src/workflow_pack/mod.rs`
- `src/workflow_pack/renderer.rs`
- `src/workflow_pack/types.rs`
- `src/benchmarks/reporter.rs`
- `src/benchmarks/profiler.rs`
- `src/interpreter/test_runner.rs`
- `src/errors.rs`
- `src/repl.rs`
- `src/docgen/core.rs`
- `tests/cli_json_contracts.rs`
- `tests/cli_contracts.rs`
- `tests/diagnostics_contract.rs`
- `tests/diagnostics_golden.rs`
- `tests/docs_examples.rs`
- `docs/CLI_MACHINE_READABLE_CONTRACTS.md`
- `docs/FIRST_TOOL_COOKBOOK.md`
- `docs/STANDARD_LIBRARY_REFERENCE.md`
- `docs/WORKFLOW_PACKS.md`
- `docs/AI_CENTRIC_GAP_ANALYSIS.md`
- `examples/benchmarks/run_benchmarks.ruff`
- `examples/expense_tracker.ruff`
- `examples/projects/log_parser.ruff`

## Open questions

1. Do you want the first implementation pass to prioritize Rust CLI helper consolidation (`src/main.rs`) or Ruff script/report helper ergonomics first?
2. Should human-readable output for existing commands remain byte-for-byte stable, or are small readability-only changes acceptable if tests and docs are updated?
3. Should output helper primitives start as internal Rust modules only, or be exposed as Ruff standard-library features in the same milestone?
4. Is `push` reassignment ergonomics (for example `push_mut` or method-style mutation) in scope for pre-1.0, or should it remain a documentation/convention topic for now?
5. For workflow packs, do you want a strict render contract test suite similar to `cli_json_contracts` before adding new display features?
