# Ruff Field Notes Index

This directory contains session-based field notes and curated gotchas for the Ruff programming language.

**Always read `GOTCHAS.md` first** - it contains the highest-signal, most important pitfalls.

---

## Session Notes (Chronological)

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
