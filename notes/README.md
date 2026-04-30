# Ruff Field Notes Index

- `GOTCHAS.md` — Curated, deduplicated sharp edges. Read this first.
- `FIELD_NOTES_SYSTEM.md` — Mandatory workflow and session-note template.

High-signal session notes:

- `2026-01-25_14-30_destructuring-spread-implementation.md` — Pattern binding and spread semantics.
- `2026-01-25_17-30_result-option-types-COMPLETED.md` — Contextual identifiers for `Ok/Err/Some/None`.
- `2026-01-27_07-46_phase2-vm-optimizations.md` — VM optimizer invariants and stack behavior.
- `2026-01-28_18-50_compiler-stack-pop-fix.md` — Assignment stack hygiene and JIT loop correctness.
- `2026-02-15_09-18_release-hardening-alias-api-contract.md` — Builtin declaration/dispatch drift.
- `2026-02-16_20-23_release-hardening-strict-arity-follow-through-slices.md` — Strict-arity + fallback contract preservation.
- `2026-03-12_10-43_ssg-read-render-write-fusion-follow-through.md` — SSG helper contract preservation during perf refactors.
- `2026-03-19_07-47_ssg-rayon-pool-cache-and-timing-test-stability.md` — Benchmark timing-test stability rules.
- `2026-04-27_22-31_bench-ssg-range-spread-warnings.md` — Range-spread warning contracts and threshold-wiring gotchas.
- `2026-04-28_18-16_ssg-reused-output-path-buffer.md` — Reusable output-path worker buffers in Rayon SSG hot path and `map_init` tuple-typing gotcha.
- `2026-04-28_22-06_busy-machine-ssg-gate-smoke.md` — Busy-machine SSG gate PASS classified as smoke/local evidence, not final release-gate evidence.
- `2026-04-29_17-02_image-method-dispatch-parity.md` — Interpreter/VM image method-call dispatch unification, VM FieldGet marker parity, and real conversion test coverage.
- `2026-04-29_17-17_animated-gif-to-webp-conversion.md` — Added `gif_to_webp` animated conversion path, strict contracts, and external tool dependency guidance.
- `2026-04-29_21-00_scheduler-timeout-cli-override.md` — Added `ruff run --scheduler-timeout-ms` override with deterministic `CLI > env > default` precedence and timeout-resolution contract tests.
- `2026-04-29_21-22_release-mode-ssg-gate-local-evidence.md` — Release-mode SSG gate PASS recorded as local evidence because idle-machine status was not confirmed.
- `2026-04-29_21-25_release-mode-ssg-gate-local-smoke-followup.md` — Release-mode SSG gate PASS with CV warning output and high host load recorded as local smoke evidence only.
- `2026-04-29_21-52_cross-language-and-release-checks-local-evidence.md` — Captured cross-language benchmark context plus focused release-checklist test/dispatch coverage with explicit local-smoke release-exception documentation.
- `2026-04-29_22-28_lsp-go-to-definition-cli.md` — Added `ruff lsp-definition` with deterministic function/variable/parameter definition lookup and release-cycle verification evidence.
