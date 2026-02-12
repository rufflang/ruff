# Ruff Field Notes Index

This directory contains session-based field notes and curated gotchas for the Ruff programming language.

**Always read `GOTCHAS.md` first** - it contains the highest-signal, most important pitfalls.

---

# Ruff Development Session Notes

This directory contains **field notes** from AI agent work sessions on the Ruff programming language.

## Purpose

These notes capture:
- Surprising behavior and gotchas
- Incorrect assumptions that led to bugs
- Design decisions that aren't obvious from code alone
- Debugging techniques that worked
- Lessons learned for future work

**This is operational memory, not documentation.**

## Quick Navigation

- **FIELD_NOTES_SYSTEM.md** - Mandatory workflow and template for post-work field notes
- **GOTCHAS.md** - Start here! Curated list of the most important non-obvious pitfalls
- Session notes below - Raw, chronological records of each work session

---

## Session Notes (Chronological)

- **2026-02-12_15-33-await-all-batching-and-jwt-provider-fix.md** — ✅ COMPLETED: Implemented P0 async quick-win batching (`promise_all`/`await_all` with optional concurrency limit), fixed missing `await_all` builtin registration, added integration coverage, and resolved full-suite JWT failures by pinning `jsonwebtoken` provider feature (`rust_crypto`).
- **2026-02-12_14-52_hashmap-fusion-jit-sealing-release.md** — ✅ COMPLETED: Hashmap performance fusion + JIT sealing stability + v0.9.0 release prep. Added fused map opcodes in bytecode/compiler/VM, fixed Cranelift sealing regressions in `src/jit.rs`, validated with full build/tests/benchmarks, and finalized release metadata/tag push.
- **2026-01-27_phase4e-jit-benchmarking.md** — ✅ COMPLETED: Phase 4E JIT Performance Benchmarking & Validation (v0.9.0). Added 7 micro-benchmark tests + infrastructure validation test. Created 5 real-world benchmark programs. Validated all Phase 4 subsystems: type profiling (11.5M obs/sec), guard generation (46µs per guard), cache lookups (27M/sec), specialization (57M decisions/sec). Phase 4 now 100% complete! ~3 hours implementation time. 3 commits, 198 tests passing.
- **2026-01-26_vm-exception-handling.md** — ✅ COMPLETED: VM Exception Handling Implementation (v0.9.0 Phase 1). Implemented full exception handling in bytecode VM with BeginTry/EndTry/Throw/BeginCatch/EndCatch opcodes. Added exception handler stack for proper unwinding. Critical lesson: chunk restoration during call frame unwinding. Comprehensive test suite (9 scenarios). VM/interpreter parity achieved. ~3 hours implementation time.
- **2026-01-26_interpreter-modularization-phase2.md** — ✅ PHASE 2 COMPLETE: Interpreter Modularization. Extracted ControlFlow enum (22 lines) to control_flow.rs and test framework (230 lines) to test_runner.rs. Reduced mod.rs from 14,285 to 14,071 lines (additional -214 lines). Total reduction: -731 lines from original 14,802 (~5%). Documented key decisions: why call_native_function_impl and register_builtins must stay in impl block. Clarified that remaining size is appropriate. Zero warnings, all tests passing.
- **2026-01-26_14-30_interpreter-modularization-gotchas.md** — ✅ FIELD NOTES: Interpreter Modularization Phase 1. Captured key gotchas from extracting Value/Environment modules: why 5,700-line function must stay in impl block, circular type dependencies pattern, pub use re-exports, mental models about line counts vs maintainability.
- **2026-01-26_interpreter-modularization-phase1.md** — ✅ PHASE 1 COMPLETE: Interpreter Modularization. Extracted Value enum (500 lines) to value.rs and Environment struct (110 lines) to environment.rs. Reduced mod.rs from 14,802 to 14,285 lines. Zero warnings, all tests passing. Comprehensive technical documentation.
- **2026-01-26_interpreter-modularization-foundation.md** — ✅ FOUNDATION COMPLETE: Interpreter Modularization v0.9.0 Foundation (~15% of Task #27). Created src/interpreter/ module structure, moved interpreter.rs to interpreter/mod.rs. Created comprehensive docs/ARCHITECTURE.md (595 lines) documenting current structure and refactoring strategy. Updated ROADMAP and CHANGELOG. Next: Extract Value enum and Environment struct. Zero regressions, all code compiles successfully.
- **2026-01-25_testing-framework-foundation.md** — 🚧 IN PROGRESS: Built-in Testing Framework Foundation. Implemented syntax support (lexer, AST, parser) and 4 assertion functions (assert_equal/true/false/contains). Test runner and CLI integration remain to be completed (~40% complete, ~4 days remaining).
- **2026-01-25_io-module-implementation.md** — ✅ COMPLETED: Standard Library Expansion Milestone 3 (IO Module). Implemented 9 advanced binary I/O functions: io_read_bytes, io_write_bytes, io_append_bytes, io_read_at, io_write_at, io_seek_read, io_file_metadata, io_truncate, io_copy_range. Offset-based file access, comprehensive metadata, zero-copy range operations. Added 20 test cases (37 assertions, 100% pass rate) and 9 real-world examples (271 lines). Production-ready with full documentation. Completes third major stdlib milestone.
- **2026-01-25_vm-performance-optimization.md** — ✅ MAJOR MILESTONE: VM Native Function Integration Complete. Extended VM from 3 to 180+ built-in functions via interpreter delegation. Zero code duplication achieved through composition pattern. Created comprehensive test suite and 7 benchmark programs. Discovered VM loop execution bug blocking performance validation. 70% of VM Performance Optimization milestone complete. Production-ready for non-loop code.
- **2026-01-25_os-path-modules.md** — ✅ COMPLETED: Standard Library Expansion Milestone 2 (OS and Path modules). Implemented 9 new built-in functions: os_getcwd/chdir/rmdir/environ, path_join/absolute/is_dir/is_file/extension. Added 52 comprehensive tests and 15 detailed examples (615 lines). Discovered type() introspection issue with Error values. Updated CHANGELOG, ROADMAP, README. Production-ready with full documentation.
- **2026-01-25_stdlib-expansion.md** — ✅ COMPLETED: Standard Library Expansion Milestone 1 (compression, hashing, process management). Implemented 10 new built-in functions: zip_create/add_file/add_dir/close/unzip, sha256/md5/md5_file, hash_password/verify_password, spawn_process/pipe_commands. Added comprehensive tests (236 lines) and examples (617 lines). Updated CHANGELOG, ROADMAP, README. Production-ready with full documentation.
- **2026-01-25_23-15_arg-parser-implementation.md** — ✅ COMPLETED: Argument Parser (arg_parser) for CLI tools. Implemented fluent API for building professional command-line interfaces with flags, options, type validation, help generation. Supports bool/string/int/float types, required/optional, defaults, short/long forms. All tests passing.
- **2026-01-25_22-00_compiler-warnings-cleanup.md** — ✅ COMPLETED: Compiler Warnings Cleanup. Reduced clippy warnings from 271 to 30 (89% reduction). Fixed 179 .get(0), 27 needless_borrow, 21 doc comment spacing, 6 redundant closures, 6 unnecessary casts. All 208 tests passing.
- **2026-01-25_20-00_bytecode-compiler-vm-foundation.md** — 🚧 IN PROGRESS: Bytecode Compiler & VM Foundation for v0.8.0. Implemented complete OpCode instruction set (60+ instructions), AST-to-bytecode compiler, and stack-based VM. Foundation complete, function calls need refinement. Compiles successfully.
- **2026-01-25_18-00_enhanced-collections-implementation.md** — ✅ COMPLETED: Enhanced Collection Methods for v0.8.0. Implemented 20 new array/dict/string methods: chunk, flatten, zip, enumerate, take, skip, windows, invert, update, get_default, pad_left/right, lines, words, str_reverse, slugify, truncate, to_camel/snake/kebab_case. All tests passing.
- **2026-01-25_17-30_result-option-types-COMPLETED.md** — ✅ COMPLETED: Result<T, E> and Option<T> types with Ok/Err/Some/None/Try. Fixed critical lexer bug (keywords vs identifiers). All 208 tests pass.
- **2026-01-25_17-00_result-option-types-with-bug.md** — Implemented Result<T, E> and Option<T> types with Ok/Err/Some/None/Try, but pattern matching has critical hang bug that needs debugging.
- **2026-01-25_16-00_enhanced-error-messages.md** — Implemented enhanced error messages with Levenshtein "Did you mean?" suggestions, help/note context, and multiple error reporting for v0.8.0.
- **2026-01-25_14-30_destructuring-spread-implementation.md** — Implemented destructuring patterns and spread operators for v0.8.0. Added Pattern enum, ArrayElement/DictElement enums, updated lexer/parser/interpreter/type-checker. 30 tests created.

---

## How to Use These Notes

1. **Before starting work:** Read `GOTCHAS.md` to understand common pitfalls
2. **During work:** Reference session notes for similar tasks (use grep/search)
3. **After completing work:** Write new session notes following the template
4. **Periodically:** Update `GOTCHAS.md` with high-impact learnings from session notes

---

## Note-Taking Guidelines

- One session = one timestamped markdown file
- Follow template structure strictly (see any session note for reference)
- Be specific: include file paths, function names, exact error messages
- Document "justified behavior" (moments you had to explain why something is OK)
- Update `GOTCHAS.md` when discovering repeated or high-impact issues
