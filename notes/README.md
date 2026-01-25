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

- **GOTCHAS.md** - Start here! Curated list of the most important non-obvious pitfalls
- Session notes below - Raw, chronological records of each work session

---

## Session Notes (Chronological)

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
