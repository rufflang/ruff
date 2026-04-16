# Ruff — Known Gotchas & Sharp Edges

This document contains the most important non-obvious pitfalls in the Ruff codebase.

If you are new to the project, read this first.

---

## Parser & Syntax

### Expression precedence is NOT inferred
- **Problem:** Parser/evaluator behavior can look "wrong" when new syntax is added without explicit precedence placement.
- **Rule:** New expression forms and operators must be wired into the existing precedence chain intentionally; Ruff does not infer precedence from token shape.
- **Why:** The parser is hand-rolled with explicit parse stages and lookahead.

(Discovered during: 2026-01-25_14-30_destructuring-spread-implementation.md)

### Spread (`...`) is container-context only
- **Problem:** Treating spread as a standalone expression leads to invalid mental models and dead-end refactors.
- **Rule:** Spread is valid only inside `ArrayElement::Spread` and `DictElement::Spread`, not as a general `Expr`.
- **Why:** Semantics depend on container context (array flatten/concat vs dict merge).

(Discovered during: 2026-01-25_14-30_destructuring-spread-implementation.md)

### `Ok`/`Err`/`Some`/`None` are contextual identifiers, not lexer keywords
- **Problem:** Turning these into keywords breaks `match case` parsing and can hang parse flows.
- **Rule:** Keep them tokenized as identifiers and interpret contextually in parser/evaluator logic.
- **Why:** Pattern matching expects identifier tokens in case arms.

(Discovered during: 2026-01-25_17-30_result-option-types-COMPLETED.md)

### Multi-character operators require explicit lexer lookahead
- **Problem:** Operators like `...` fragment into punctuation tokens when lookahead logic is incomplete.
- **Rule:** Any multi-character operator must have dedicated lookahead path in lexer tokenization.
- **Why:** Lexer is character-driven and does not auto-compose operator tokens.

(Discovered during: 2026-01-25_14-30_destructuring-spread-implementation.md)

---

## Runtime / Evaluator

### Variable scope in scripts is global-by-default
- **Problem:** Reusing variable names across top-level blocks causes state bleed in tests and scripts.
- **Rule:** Assume script-level assignments mutate shared global scope unless inside function boundaries.
- **Implication:** Use unique names or function wrappers for isolation in integration tests.

(Discovered during: 2026-01-27_07-46_phase2-vm-optimizations.md)

### Native dispatch and declaration can drift independently
- **Problem:** Builtins may appear declared but still fail at runtime if dispatcher wiring is missing.
- **Rule:** Treat declaration, registration, and dispatch handling as separate required surfaces.
- **Implication:** Any builtin change needs end-to-end contract tests (module + dispatcher + runtime behavior).

(Discovered during: 2026-02-15_09-18_release-hardening-alias-api-contract.md)

### Strict-arity hardening must preserve legacy fallback contracts unless intentionally changed
- **Problem:** Rejecting extra args can accidentally alter established missing/invalid-arg behavior.
- **Rule:** Add explicit extra-arg rejection while preserving existing fallback semantics unless contract change is deliberate.
- **Implication:** Tests should assert both strict-arity and compatibility fallback behavior.

(Discovered during: 2026-02-16_20-23_release-hardening-strict-arity-follow-through-slices.md)

### Promise receivers are single-consumer
- **Problem:** Re-awaiting or re-aggregating promise handles can fail if cached result checks are skipped.
- **Rule:** Aggregation helpers must short-circuit on cached completion before touching receiver paths.
- **Implication:** `Promise.all`/`await_all` optimizations must preserve single-consumer safety.

(Discovered during: 2026-02-14_10-10_promise-cache-reuse-and-parallel-map-overhead.md)

### Shared state is process-global
- **Problem:** Tests interfere when shared keys are reused.
- **Rule:** Use unique shared keys per test and clean up deterministically.
- **Implication:** Parallel tests require namespacing discipline for `shared_*` APIs.

(Discovered during: 2026-02-14_13-09_shared-thread-safe-value-ops.md)

### SSG helper contracts are compatibility surfaces
- **Problem:** Throughput refactors can silently break checksum, stage-metric key, or output-path invariants.
- **Rule:** Preserve benchmark-facing contracts (`checksum`, file counts, stage keys) while optimizing internals.
- **Implication:** SSG performance work must include contract regressions, not only speed checks.

(Discovered during: 2026-03-12_10-43_ssg-read-render-write-fusion-follow-through.md)

---

## Compiler / VM / JIT

### `StoreVar`/`StoreGlobal` are PEEK-semantics, not POP
- **Problem:** Missing explicit `Pop` after assignment/pattern-binding pollutes stack and breaks loop/JIT invariants.
- **Rule:** Statement-level assignment paths must enforce stack hygiene explicitly.
- **Why:** Store opcodes keep top-of-stack value by design.

(Discovered during: 2026-01-28_18-50_compiler-stack-pop-fix.md)

### Dead-code elimination must remap all index-based metadata
- **Problem:** Jump targets/handlers/source-map entries become invalid after instruction removal.
- **Rule:** Build old→new index mapping and rewrite every metadata surface carrying instruction indices.
- **Implication:** Optimizer edits require holistic metadata audits, not opcode-only updates.

(Discovered during: 2026-01-27_07-46_phase2-vm-optimizations.md)

### New modules must be declared in both crate roots
- **Problem:** `unresolved import` appears when module is wired in `main.rs` but not `lib.rs` (or vice versa).
- **Rule:** Ruff uses binary+library crate roots; add module declarations to both when introducing new modules.
- **Implication:** Module-split refactors should include dual-root compile checks.

(Discovered during: 2026-01-27_07-46_phase2-vm-optimizations.md)

### Cranelift block sealing and terminators are correctness-critical
- **Problem:** Misordered sealing or missing terminators causes verifier failures and invalid SSA graphs.
- **Rule:** Ensure each basic block terminates and seal in control-flow-safe order.
- **Implication:** JIT edits require explicit CFG reasoning, not local opcode translation only.

(Discovered during: 2026-01-28_04-08_jit-execution-success.md)

---

## CLI & Tooling

### `cargo test` only accepts one positional test filter
- **Problem:** Supplying multiple positional filters leads to command misuse and misleading failures.
- **Rule:** Use one positional filter; pass additional selection logic via module path or `-- --nocapture`/other flags.
- **Implication:** Keep repro commands minimal and exact in notes/tests.

(Discovered during: 2026-02-17_18-48_io-strict-arity-hardening.md)

### Benchmark harness paths are sensitive to working directory
- **Problem:** Running bench commands outside repo-root can resolve fixtures/scripts incorrectly.
- **Rule:** Normalize CWD assumptions and prefer explicit tmp/artifact roots for harnesses.
- **Implication:** Bench tooling changes should be validated with non-root invocation paths.

(Discovered during: 2026-02-13_18-52_bench-cross-cwd-gotcha.md)

### Full-suite timing assertions are brittle under load
- **Problem:** Throughput/timing tests that assert monotonic runtime behavior flake in CI and loaded local environments.
- **Rule:** Assert deterministic correctness signals (checksums/counts/non-negative timing) instead of strict timing trends.
- **Implication:** Performance tests should detect regressions by contract, not raw wall-clock ordering.

(Discovered during: 2026-03-19_07-47_ssg-rayon-pool-cache-and-timing-test-stability.md)

---

## Mental Model Summary

- Ruff favors explicit contracts and defensive compatibility over implicit inference.
- Parser behavior is token- and precedence-driven; contextual identifiers are intentional.
- Runtime builtin reliability depends on synchronized declaration/registration/dispatch/testing.
- VM/JIT work is highly stack- and CFG-sensitive; local edits often have global invariants.
- Do **not** assume performance refactors are safe unless benchmark/output contracts are explicitly preserved.
