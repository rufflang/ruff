# Ruff — Known Gotchas & Sharp Edges

This document contains the most important non-obvious pitfalls in the Ruff codebase.

If you are new to the project, read this first.

---

## ✅ RESOLVED: JIT Now Executes Correctly (as of 2026-01-28)

### ~~JIT Compiles But Doesn't Execute~~ — FIXED!

- **STATUS:** ✅ RESOLVED
- **Original Problem:** JIT compiled functions weren't being executed
- **Fix Applied:** Wired up JIT execution path in `src/vm.rs`
- **Current Performance:** Ruff is now **52-68x FASTER** than Python on compute-heavy benchmarks
- **Benchmark Results (2026-01-28):**
  - fib(25): 0.54ms (Python: 35.45ms) — **66x faster**
  - fib(30): 6.14ms (Python: 323ms) — **53x faster**
  - array_sum(100k): 0.2ms (Python: 10.36ms) — **52x faster**
  - nested_loops(500): 0.36ms (Python: 24.13ms) — **68x faster**

(Resolved during: 2026-01-28_04-08_jit-execution-success.md, 2026-01-28_18-50_compiler-stack-pop-fix.md)

---

## Parser & Syntax

### Reserved keywords cannot be used as function names
- **Problem:** Function named `default()` causes parse errors and returns Tagged values instead of normal results
- **Rule:** Cannot use reserved keywords as function names: `if`, `else`, `loop`, `while`, `for`, `in`, `break`, `continue`, `return`, `func`, `struct`, `match`, `case`, `default`, etc.
- **Why:** The lexer tokenizes these as `TokenKind::Keyword()` before the parser can interpret them as identifiers
- **Symptom:** Functions with keyword names may appear to work but produce strange behavior or parse errors
- **Solution:** Use alternative names (e.g., `get_default()` instead of `default()`)
- **Check:** Search lexer.rs for the keyword list before naming new built-in functions

(Discovered during: 2026-01-25_18-00_enhanced-collections-implementation.md)

### Ok/Err/Some/None must be identifiers, NOT keywords
- **Problem:** Match statements hang indefinitely when trying to parse `case Ok:` or `case Err:` patterns
- **Rule:** `Ok`, `Err`, `Some`, `None` are **identifiers with special meaning in expression context**, not reserved keywords
- **Why:** Pattern matching needs to recognize these names as identifiers to match against Result/Option variants. If they're keywords, the parser can't use them as pattern identifiers.
- **Symptom:** Parser returns `None` from `parse_match()` when it encounters `TokenKind::Keyword("Ok")` instead of `TokenKind::Identifier("Ok")` after `case`
- **Fix:** In lexer.rs, do NOT add Ok/Err/Some/None to the keyword list. In parser.rs, check for `TokenKind::Identifier` when parsing Ok/Err/Some/None expressions.
- **Implication:** These are **contextual identifiers** - they have special meaning only when used as function calls (e.g., `Ok(42)`), but are regular identifiers in patterns. This design allows maximum flexibility.

(Discovered during: 2026-01-25_17-30_result-option-types-COMPLETED.md)

### Spread operator is context-dependent, NOT a standalone expression
- **Problem:** `Expr::Spread` exists in the AST but generates "unused variant" warning
- **Rule:** Spread (`...`) is ONLY valid inside `ArrayElement::Spread` and `DictElement::Spread`. It cannot appear as a standalone expression.
- **Why:** Spread semantics depend on container context (array concatenation vs dict merge). Making it a general expression would require complex validation to reject `let x := ...arr` and similar invalid uses.
- **Implication:** Don't try to "fix" the warning by using `Expr::Spread`. The warning is intentional documentation that spread is special-cased.

(Discovered during: 2026-01-25_14-30_destructuring-spread-implementation.md)

### Method Call vs Field Access Requires Lookahead

- **Problem:** `obj.method()` and `obj.field` both start with same tokens (`.` + identifier)
- **Rule:** Parser checks for `(` after identifier to distinguish method call from field access
- **Location:** `src/parser.rs` in `parse_call()` function (~line 1094)
- **Implication:** Method syntax requires one-token lookahead; no backtracking available

(Discovered during: 2026-01-26_02-44_iterators-generators-implementation.md)

### Generator Syntax Uses `*` Operator Token

- **Problem:** `func*` generator syntax reuses multiplication operator
- **Rule:** Parser checks for `*` immediately after `func` keyword, before parsing function name
- **Why:** Avoids adding new token type; mirrors JavaScript generator syntax
- **Location:** `src/parser.rs` in `parse_func()` and `parse_func_expr()`

(Discovered during: 2026-01-26_02-44_iterators-generators-implementation.md)

### Patterns can start with punctuation, not just identifiers
- **Problem:** Parser fails to recognize `[a, b] := ...` or `{x, y} := ...` syntax
- **Rule:** `parse_stmt()` must check for `TokenKind::Punctuation('[')` and `'{'` in addition to identifiers when detecting pattern-based assignments
- **Why:** Destructuring patterns are syntactically distinct from identifier-based assignments. Array patterns start with `[`, dict patterns start with `{`.
- **Implication:** When adding new statement forms, consider ALL possible starting tokens, not just identifiers.

(Discovered during: 2026-01-25_14-30_destructuring-spread-implementation.md)

### Multi-character operators require explicit lookahead
- **Problem:** Lexer treats `...` as three separate `.` tokens instead of one spread operator
- **Rule:** Lexer must explicitly peek ahead and check for multi-character sequences when tokenizing operator characters
- **Why:** Ruff's lexer processes one character at a time. Without lookahead, `...` becomes `Punctuation('.')` three times instead of `Operator("...")`
- **Example:** In lexer's `'.'` case, must check if next two chars are also `.` before emitting token
- **Implication:** Any multi-char operator (`==`, `!=`, `<=`, `>=`, `...`, etc.) needs explicit lookahead logic.

(Discovered during: 2026-01-25_14-30_destructuring-spread-implementation.md)

---

## AST & Type System

### Type checker doesn't know about runtime-registered native functions
- **Problem:** Type checker gives "function not found" errors for valid native functions like env_or()
- **Rule:** Type checker (src/type_checker.rs) and runtime function registry (src/interpreter.rs) are separate. Type checker only knows about functions explicitly added to its symbol table.
- **Why:** Type checking happens before interpretation. Native functions registered at runtime aren't visible during type checking phase.
- **Symptom:** "Function 'env_or' not found in scope" errors even though function exists and works
- **Solution:** For now, type checking is optional (run with `--skip-type-check`). Future work: sync type checker with native function signatures or make it aware of runtime registry.
- **Workaround:** Use `--skip-type-check` flag or ignore type checker errors for known-good native functions

(Discovered during: 2026-01-25_21-30_env-helpers-implementation.md)

### Pattern enum replaced simple name strings in Stmt::Let
- **Problem:** Tests fail with "no field 'name'" after AST changes
- **Rule:** `Stmt::Let` uses `pattern: Pattern`, not `name: String`. Patterns can be complex (arrays, dicts, nested).
- **Why:** Destructuring requires representing complex binding patterns, not just simple names
- **Implication:** When constructing `Stmt::Let` in tests, use `pattern: Pattern::Identifier("x".to_string())` instead of `name: "x".to_string()`
- **How to find:** Search for `Stmt::Let` construction: `rg "Stmt::Let" --type rust`

(Discovered during: 2026-01-25_14-30_destructuring-spread-implementation.md)

### Pattern binding is recursive by design
- **Problem:** Nested destructuring might seem complex to implement
- **Rule:** `bind_pattern()` naturally mirrors the `Pattern` enum structure. Each variant handles its own case, then recurses for sub-patterns.
- **Why:** Patterns are recursive: `[a, [b, c], d]` contains sub-patterns. The binding logic should match this structure.
- **Implication:** Don't try to flatten pattern binding. Keep it recursive and it will handle arbitrarily nested patterns automatically.

(Discovered during: 2026-01-25_14-30_destructuring-spread-implementation.md)

### is_generator Field Required in Two Places

- **Problem:** Generator functions need tracking in both definitions and expressions
- **Rule:** Both `Stmt::FuncDef` and `Expr::Function` have `is_generator: bool` field
- **Why:** Functions can be declared (`func* foo()`) or used as expressions (`let f := func*() {}`)
- **Prevention:** When adding function-related features, check both AST nodes

(Discovered during: 2026-01-26_02-44_iterators-generators-implementation.md)

---

## Compiler & Bytecode Optimizer

### New modules must be declared in BOTH main.rs AND lib.rs

- **Problem:** `error[E0432]: unresolved import` when trying to use new module
- **Rule:** Ruff has dual crate structure (binary + library). Add `mod <name>;` to both `src/main.rs` AND `src/lib.rs`
- **Why:** Binary crate (main.rs) is the executable, library crate (lib.rs) enables testing and external use. Both need module declarations for full visibility.
- **Symptom:** Module compiles when building lib but fails when building bin, or vice versa
- **Solution:** Always declare new modules in both files
- **Prevention:** When creating `src/foo.rs`, immediately add `mod foo;` to both main.rs and lib.rs

(Discovered during: 2026-01-27_07-46_phase2-vm-optimizations.md)

### Division by zero must NOT be constant-folded

- **Problem:** Folding `10 / 0` causes compile-time panic instead of runtime error
- **Rule:** Optimizer must skip folding any operation that can fail at runtime
- **Why:** Division by zero should produce a runtime error, not crash the compiler
- **Affected operations:** Division (`/`), modulo (`%`) when divisor is zero or 0.0
- **Code location:** `src/optimizer.rs::try_fold_binary_op()` includes explicit guards
- **Prevention:** Any operation with runtime failure modes (array access, dict lookup, type coercion) must NOT be folded if inputs can cause failure

(Discovered during: 2026-01-27_07-46_phase2-vm-optimizations.md)

### Dead code elimination must update ALL index-based metadata

- **Problem:** Exception handlers point to wrong instructions after DCE removes code
- **Rule:** When removing instructions, build an `old_index → new_index` map and update: jump targets, exception handlers, source maps, debug info
- **Why:** Removing instruction at index N shifts all subsequent indices down
- **Critical structures:** `BytecodeChunk.exception_handlers`, all Jump* opcodes, BeginTry opcodes, source_map HashMap
- **Algorithm:** 1) Mark reachable instructions, 2) Build index_map during emission, 3) Update all index references
- **Prevention:** Check BytecodeChunk for ANY field containing instruction indices before implementing optimizations

(Discovered during: 2026-01-27_07-46_phase2-vm-optimizations.md)

### 🚨 StoreVar uses PEEK semantics - NOT POP! (CRITICAL for JIT)

- **Problem:** Loop JIT fails with "Stack empty" errors at loop headers
- **Root cause:** `StoreVar`/`StoreGlobal` use **PEEK** semantics (store TOS but LEAVE it on stack), not POP
- **Why this breaks JIT:** Without explicit `Pop` after `let`/`assign` statements, values accumulate on stack between statements. JIT loop headers expect identical stack state on every entry, but stack pollution causes mismatch.
- **Symptom:** Functions with loops fall back to interpreter; benchmarks show 1000x+ slower than expected
- **Rule:** Every `Stmt::Let` and `Stmt::Assign` MUST emit `Pop` after `compile_pattern_binding()` / `compile_assignment()`
- **Code location:** `src/compiler.rs` lines ~102-128
- **Prevention:** Remember: **StoreVar = PEEK, not POP**. Statement hygiene requires explicit Pop.
- **Performance impact:** Without fix: loops at interpreter speed (~1800ms). With fix: JIT speed (~0.2ms) - **9000x improvement**

(Discovered during: 2026-01-28_18-50_compiler-stack-pop-fix.md)

### Peephole optimizer StoreVar+LoadVar pattern

- **Problem:** Replacing `StoreVar(x) + LoadVar(x)` optimization needs Dup
- **Rule:** Since StoreVar PEEKs (leaves value on stack), the pattern `StoreVar(x); LoadVar(x)` can become `Dup; StoreVar(x)` - the Dup ensures a copy remains after store.
- **Why:** This is an optimization. StoreVar leaves value, LoadVar would push same value again. Dup+StoreVar achieves same effect with one less opcode.
- **Correct pattern:** `StoreVar(x) + LoadVar(x)` → `Dup + StoreVar(x)` 
- **Prevention:** Understand stack effect of each opcode before pattern-matching optimizations

(Discovered during: 2026-01-27_07-46_phase2-vm-optimizations.md)

### Use `BytecodeChunk::new()` in tests, not full struct literals

- **Problem:** VM/unit tests fail to compile after `BytecodeChunk` field additions.
- **Symptom:** Rust error `E0063` with "missing fields ... in initializer of `bytecode::BytecodeChunk`".
- **Rule:** Prefer `BytecodeChunk::new()` and set only fields needed for the test (commonly `name`, `instructions`).
- **Why:** `BytecodeChunk` evolves; struct literals become brittle and require touching many tests when internal metadata fields are added.
- **Implication:** Only use full literals when a test must explicitly validate non-default metadata fields.

(Discovered during: 2026-02-13_19-45_vm-cooperative-scheduler-rounds.md)

### Ruff variables are globally scoped within script files

- **Problem:** Test file reusing variable names like `sum` and `i` showed unexpected values
- **Rule:** Variable assignment in Ruff UPDATES existing bindings at global scope, doesn't create new local bindings (unless inside function)
- **Why:** Ruff's default scoping is global within a script. Loops/blocks don't create new scopes (only functions do)
- **Symptom:** Assigning to `x := 10` then later `x := 20` updates the same global `x`, not shadowing
- **Solution:** Use unique variable names in test files, or wrap tests in functions for isolated scopes
- **Implication:** This is a Ruff language characteristic, not a bug. Test files should account for this.

(Discovered during: 2026-01-27_07-46_phase2-vm-optimizations.md)

---

## Runtime / Evaluator

### `env_bool(...)` is permissive: non-truthy values become `false`, not parse errors
- **Problem:** Tests may incorrectly expect `env_bool("KEY")` to return an error for values like `"definitely-not-bool"`.
- **Rule:** `env_bool` returns `Ok(true)` only for truthy strings (`true`, `1`, `yes`, `on`, case-insensitive); all other present values resolve to `false`.
- **Why:** `builtins::env_bool` uses `matches!(...)` and wraps that in `Ok(...)`; only missing env vars produce an error.
- **Implication:** Contract tests should assert `Value::Bool(false)` for unrecognized present values, and `Value::ErrorObject` only for missing-variable path.

(Discovered during: 2026-02-16_00-28_release-hardening-env-os-assert-follow-through.md)

### `format(...)` uses printf-style placeholders, not `{}` interpolation
- **Problem:** Tests and examples written as `format("Hello {}, {}", ...)` fail to substitute values.
- **Rule:** Ruff `format(...)` currently supports `%s`, `%d`, `%f` placeholders (and `%%` escape) via `builtins::format_string`.
- **Why:** Placeholder parsing in `src/builtins.rs` is explicitly `%`-token based.
- **Implication:** Use `%` placeholders in contracts/docs unless implementation is intentionally changed.

(Discovered during: 2026-02-15_23-40_release-hardening-contract-slices-continuation.md)

### `Value` is not `PartialEq`; tests must assert by shape, not direct equality
- **Problem:** Rust tests fail to compile when using `assert_eq!(value_a, value_b)` on `Value`.
- **Rule:** Compare `Value` with `matches!`/`match` and variant-specific assertions.
- **Why:** `interpreter::value::Value` does not derive/implement `PartialEq`.
- **Implication:** Contract tests should be explicit about expected variant/contents; avoid blanket equality checks.

(Discovered during: 2026-02-15_23-40_release-hardening-contract-slices-continuation.md)

### Promise receivers are single-consumer; aggregation must check cached results first
- **Problem:** Reusing a previously-awaited promise in aggregators (e.g., `promise_all([p, p], ...)`) can fail with channel-closed errors.
- **Rule:** Any aggregation path that consumes `Value::Promise.receiver` must first check `is_polled` + `cached_result` and use cached result when available.
- **Why:** Promise receiver is a oneshot channel moved out via `std::mem::replace(...)`; it is not reusable after first consumption.
- **Implication:** `Promise.all(...)` / `parallel_map(...)` must treat cache-aware reuse as correctness behavior, not just optimization.

(Discovered during: 2026-02-14_10-10_promise-cache-reuse-and-parallel-map-overhead.md)

### Keep immediate mapper outputs immediate in `parallel_map(...)`
- **Problem:** Wrapping every non-Promise mapper output into synthetic Promise channels adds avoidable overhead.
- **Rule:** In `parallel_map(...)`, store immediate values directly and await only true Promise receivers.
- **Why:** Synthetic oneshot Promise normalization creates allocation/churn without semantic benefit for already-immediate values.
- **Implication:** Split aggregation into immediate lane + pending async lane for lower overhead and clearer behavior.

(Discovered during: 2026-02-14_10-10_promise-cache-reuse-and-parallel-map-overhead.md)

### `spawn` uses transferable parent-binding snapshots; parent write-back is still isolated
- **Problem:** It is easy to assume either full lexical sharing or full isolation and write the wrong coordination logic.
- **Rule:** `spawn` workers receive a transferable snapshot of parent bindings at spawn time, but spawned assignments do NOT write through to parent scope.
- **Why:** `Stmt::Spawn` still creates a fresh interpreter; parent data is copied for supported value variants instead of sharing the parent `Environment` by reference.
- **Implication:** Use parent capture for worker inputs (e.g., shared-store keys), and use `shared_set/get/has/delete/add_int` for cross-thread observable updates.

(Discovered during: 2026-02-14_13-09_shared-thread-safe-value-ops.md and 2026-02-14_17-08_spawn-parent-binding-snapshot-concurrency.md)

### `shared_*` state is process-global; test keys must be unique
- **Problem:** Shared-state tests can become flaky or order-dependent when reusing fixed key names.
- **Rule:** Treat `shared_set/get/has/delete/add_int` keys as global resources and generate unique keys per test/session.
- **Why:** Shared values are backed by a static `OnceLock<Mutex<HashMap<...>>>` in `src/interpreter/native_functions/concurrency.rs`, so state outlives individual `Interpreter` instances.
- **Implication:** Use timestamp/nonce-suffixed keys in tests and clean up with `shared_delete` to avoid cross-test contamination.

(Discovered during: 2026-02-16_09-34_release-hardening-async-batch-shared-task-pool-contracts.md)

### ErrorObject has specific field structure, not a generic HashMap
- **Problem:** Creating ErrorObject with made-up fields like "details" causes runtime failures
- **Rule:** ErrorObject has exactly 4 fields: `message` (String), `stack` (Array), `line` (Number), `cause` (Value/null)
- **Why:** ErrorObject is a specific struct in interpreter.rs (~line 340), not a flexible dict. Wrong fields are silently ignored or cause errors.
- **Symptom:** Errors without proper fields may not display correctly or fail pattern matching in try/except
- **Solution:** Search for "ErrorObject {" in interpreter.rs to see the exact structure before creating error objects
- **Pattern:** `Value::Struct("ErrorObject".to_string(), HashMap::from([("message", Value::String(...)), ("stack", Value::Array(vec![])), ("line", Value::Number(0.0)), ("cause", Value::Null)]))`

(Discovered during: 2026-01-25_21-30_env-helpers-implementation.md)

### Native functions must set return_value for try/except to work
- **Problem:** Native functions returning ErrorObject didn't trigger try/except blocks
- **Rule:** When a native function returns ErrorObject or Error, the Call expression handler must set `self.return_value = Some(result.clone())` 
- **Why:** TryExcept statement checks `self.return_value.is_some()` to detect errors (~line 5590). User functions set this automatically, but native functions returned early without setting it.
- **Symptom:** Try/except blocks silently ignore errors from native functions, continuing as if nothing happened
- **Fix:** In interpreter.rs Call expression handler, after calling native function, check if result is ErrorObject/Error and set return_value before returning
- **Code location:** src/interpreter.rs Call expression evaluation (~line 3800-3900)

(Discovered during: 2026-01-25_21-30_env-helpers-implementation.md)

### Dictionary field access with dot notation returns 0, not the value
- **Problem:** `dict.field` syntax doesn't work for dictionary access, returns 0 or wrong value
- **Rule:** Dictionaries require bracket notation `dict["key"]`. Dot notation `object.field` is ONLY for Structs.
- **Why:** FieldAccess expression checks if value is a Struct and looks up the field. For non-Structs, it returns Number(0) as fallback.
- **Symptom:** Silent failure - no error, just wrong value (0)
- **Solution:** Use bracket notation for dictionaries: `config["host"]`, `db["port"]`, etc.
- **Context:** Created example using dict.field pattern, spent time debugging why values were 0

(Discovered during: 2026-01-25_21-30_env-helpers-implementation.md)

### args() returns ALL arguments including ruff command - must filter
- **Problem:** args() was returning `["ruff", "run", "script.ruff", "actual", "args"]` 
- **Rule:** args() should only return user-provided arguments, not the ruff command/subcommand/script
- **Why:** Users expect argv-style behavior where args() contains only their arguments, not the interpreter invocation
- **Solution:** Smart filtering logic that detects:
  - First arg contains "ruff" → skip it
  - Next arg is "run" or "test" → skip it  
  - Next arg ends with ".ruff" → skip it
  - Everything after that → user arguments
- **Code location:** src/builtins.rs get_args() function

(Discovered during: 2026-01-25_21-30_env-helpers-implementation.md)

### Rest elements must consume ALL remaining values
- **Problem:** Rest patterns like `[a, ...rest, b]` or `[...rest1, ...rest2]` are ambiguous
- **Rule:** Rest element (`...name`) must be the LAST element in array patterns. Only one rest element allowed per pattern level.
- **Why:** Multiple rest elements create ambiguity about which values go where. Trailing rest is unambiguous.
- **Implication:** Parser should reject patterns with mid-pattern or duplicate rest elements during parsing, not evaluation.

(Discovered during: 2026-01-25_14-30_destructuring-spread-implementation.md)

### Iterator State Must Be Mutable During Collection

- **Problem:** Iterator operations hang in infinite loop
- **Symptom:** `collect()` never completes, program times out
- **Root cause:** Iterator's `index` field must be mutated in place, not on a clone
- **Rule:** `collect_iterator()` takes `mut iterator: Value` and mutates index through pattern matching
- **Why:** Cloning iterator creates new copy with same index → same element read forever
- **Prevention:** Iterator operations that consume the iterator (like collect) MUST mutate the iterator state. Iterator methods that create new iterators (filter, map, take) create new Iterator values with copied state.

(Discovered during: 2026-01-26_02-44_iterators-generators-implementation.md)

### Iterator Chaining Has Composition Limits

- **Problem:** Chaining multiple filters or multiple maps doesn't work as expected
- **Symptom:** `array.filter(a).map(b).filter(c).collect()` - second filter sees original array, not mapped values
- **Root cause:** Iterator wraps original source, transformers don't compose into pipeline
- **Workaround:** Use `.collect()` between operations: `array.filter(a).collect().map(b).filter(c).collect()`
- **Future improvement:** Make transformers/filters compose, or make source be previous iterator

(Discovered during: 2026-01-26_02-44_iterators-generators-implementation.md)

### Generator Execution Requires Statement-Level Control

- **Problem:** Generator syntax parses but calling generators doesn't work
- **Root cause:** Yield is not like return - requires:
  1. Program counter (PC) to track which statement to resume from
  2. Statement-level execution (not just expression evaluation)
  3. Environment preservation between yield/resume
  4. Converting yield's Return value to iterator protocol
- **Rule:** Don't implement generators with just expression evaluation - need dedicated state machine
- **Estimated effort:** 1-2 weeks for proper implementation

(Discovered during: 2026-01-26_02-44_iterators-generators-implementation.md)

### `read_file(...)` bad-shape errors currently report `read_file_sync` in message text
- **Problem:** Contract tests expecting `read_file` in bad-argument errors fail unexpectedly.
- **Rule:** Current runtime contract is: `read_file(...)` and `read_file_sync(...)` share one handler arm and emit the same error text (`read_file_sync requires a string path argument`).
- **Why:** In `src/interpreter/native_functions/filesystem.rs`, the `"read_file_sync" | "read_file"` match arm returns a fixed `read_file_sync`-named error string.
- **Implication:** For release-hardening contract tests, assert the actual emitted text (or change implementation first) instead of assuming alias-specific naming.

(Discovered during: 2026-02-16_08-05_release-hardening-filesystem-core-contract-follow-through.md)

### Positional argument matching does not enforce strict arity
- **Problem:** Public builtins can silently accept trailing arguments even when required argument types are validated.
- **Rule:** For hardening-sensitive public APIs, enforce explicit arity (`len() == N` or bounded range) before positional type matching.
- **Why:** Patterns like `arg_values.first()` + `arg_values.get(1)` validate required positions but do not reject extras.
- **Implication:** Release-hardening tests must include both missing-argument and extra-argument contracts; otherwise API drift can hide behind lenient matching.

(Discovered during: 2026-02-16_10-15_release-hardening-crypto-strict-arity-contracts.md, 2026-02-16_10-37_release-hardening-filesystem-strict-arity-contracts.md, 2026-02-16_11-54_release-hardening-network-strict-arity-contracts.md)

---

## CLI & Arguments

### `cargo test` takes one positional test-name filter
- **Problem:** Passing multiple test names positionally (e.g. `cargo test test_a test_b -- --nocapture`) fails with `unexpected argument ... found`.
- **Rule:** Use one positional test filter per command; run multiple targeted tests sequentially.
- **Why:** Cargo CLI accepts at most one optional positional `TESTNAME` filter.
- **Workflow:** `cargo test test_a -- --nocapture && cargo test test_b -- --nocapture`
- **Implication:** Multi-test targeted validation should be command-chained, not combined in one positional argument list.

(Discovered during: 2026-02-16_00-46_release-hardening-set-queue-stack-contracts.md)

### Native contract tests cannot nest `call_native_function(&mut interpreter, ...)`
- **Problem:** Inline nested native calls in argument arrays fail to compile with `E0499` mutable-borrow errors.
- **Rule:** Evaluate inner native call into a local variable first, then pass that value to the outer call.
- **Why:** `call_native_function` requires `&mut Interpreter`; nesting attempts overlapping mutable borrows while outer-call args are being evaluated.
- **Workflow:**
  1. `let inner = call_native_function(&mut interpreter, "foo", &args);`
  2. `let outer = call_native_function(&mut interpreter, "bar", &[inner, other]);`
- **Implication:** When writing release-hardening tests, stage intermediate values explicitly; do not inline chained `call_native_function` expressions.

(Discovered during: 2026-02-16_09-42_release-hardening-async-runtime-task-channel-contracts.md)

### Full-suite async runtime failures should be isolated before treating as regressions
- **Problem:** `cargo test` may occasionally report `interpreter::async_runtime::tests::test_concurrent_tasks` as failed, then pass immediately in isolation.
- **Rule:** When an unrelated async runtime test fails once, re-run the exact test in isolation first, then require one full-suite green run before concluding status.
- **Why:** Full-suite scheduling/timing can produce transient contention-sensitive outcomes.
- **Implication:** Avoid misattributing transient async test failures to unrelated feature slices.

(Discovered during: 2026-02-15_23-40_release-hardening-contract-slices-continuation.md)

### `bench-cross` defaults are CWD-relative
- **Problem:** Running `cargo run --release -- bench-cross` from `benchmarks/cross-language` fails with script-not-found errors.
- **Rule:** `bench-cross` default paths (`benchmarks/cross-language/bench_parallel_map.ruff`, `benchmarks/cross-language/bench_process_pool.py`) are resolved relative to the current working directory.
- **Why:** Clap default values are plain relative strings; they are not normalized to repo root automatically.
- **Solution:** Run from repo root or pass explicit `--ruff-script` / `--python-script` values appropriate for current directory.
- **Implication:** CLI behavior varies by CWD unless the command normalizes defaults in code.

(Discovered during: 2026-02-13_18-52_bench-cross-cwd-gotcha.md)

### `bench-ssg` subprocess execution must normalize CWD and use workspace `tmp/`
- **Problem:** `bench-ssg` can fail with process spawn/path errors or trigger local permission prompts when temp paths are outside workspace.
- **Rule:** Run Ruff/Python benchmark subprocesses from detected workspace root and write generated files under repo-local `tmp/`.
- **Why:** Relative benchmark script paths and OS temp directories are environment-sensitive; workspace-local paths are deterministic and permission-friendly.
- **Solution:** `src/benchmarks/ssg.rs` determines workspace root before process spawn; `benchmarks/cross-language/bench_ssg.py` writes under `<repo>/tmp`; `bench_ssg.ruff` ensures `tmp` exists.
- **Implication:** `bench-ssg` remains reproducible across invocation locations and avoids unnecessary system temp permission friction.

(Discovered during: 2026-02-13_23-03_bench-ssg-harness-and-cwd-tmp-gotchas.md)

### `bench-ssg` timing comparisons are noisy; use Ruff-only stage profiling for optimization signals
- **Problem:** Single-run `bench-ssg --compare-python` numbers can swing significantly between runs, obscuring whether a Ruff-side optimization helped.
- **Rule:** For Ruff runtime optimization decisions, compare `ruff bench-ssg --profile-async` before/after in the same environment and use multiple runs when possible.
- **Why:** Cross-language runs include additional contention/variability (filesystem cache state and two sequential workloads), while Ruff-only stage profiling isolates Ruff-side changes.
- **Implication:** Treat one-off compare-python numbers as trend indicators, not hard pass/fail gates for micro-optimizations.

(Discovered during: 2026-02-15_08-36_native-ssg-render-builtin-optimization.md)

### Large format-only diffs must be committed by subsystem
- **Problem:** A single formatting sweep across many areas (`jit`, interpreter runtime, benchmarks, tests, examples) creates noisy history and hard code review.
- **Rule:** For broad formatting/reflow changes, split commits by subsystem ownership and intent.
- **Why:** Reviewability and rollback safety collapse when unrelated formatting churn is mixed.
- **Workflow:** 1) `git status --short`, 2) bucket files by subsystem, 3) stage explicit file lists, 4) commit each bucket with scoped message.
- **Implication:** If `src/jit.rs` changed, prefer a dedicated JIT formatting commit; it is usually too large to co-mingle.

(Discovered during: 2026-02-12_16-17_commit-grouping-and-field-notes-ops.md)

### `cargo fmt` can touch unrelated modules in this workspace
- **Problem:** Running formatter during a focused feature slice can modify unrelated native-function files.
- **Rule:** Always re-scope the working tree after `cargo fmt` before staging commits.
- **Why:** Workspace formatting settings and file-level normalization can produce spillover edits outside the intended slice.
- **Workflow:** 1) run `cargo fmt`, 2) inspect `git status --short`, 3) `git restore` unrelated files, 4) stage only feature files.
- **Implication:** Treat formatter spillover as expected maintenance overhead; do not commit unrelated formatting churn with behavior changes.

(Discovered during: 2026-02-15_22-01_release-hardening-load-image-network-dispatch.md)

### Dotted native names are fragile in Ruff call sites
- **Problem:** Calling `Promise.all(...)` directly in Ruff code can behave differently than identifier aliases
- **Rule:** Prefer identifier-safe aliases (`promise_all(...)`, `await_all(...)`) for user code and tests
- **Why:** Dotted syntax intersects with field/method access parsing/evaluation rules and can surprise call resolution
- **Implication:** For any dotted built-in API, provide and test a plain identifier alias, then use the alias in docs/tests

(Discovered during: 2026-02-12_15-33-await-all-batching-and-jwt-provider-fix.md)

### Clap intercepts script arguments without trailing_var_arg
- **Problem:** `ruff run script.ruff --flag value` fails because clap tries to parse `--flag` as a ruff option
- **Rule:** Must use `trailing_var_arg = true` and `allow_hyphen_values = true` on `script_args` field in clap command definition
- **Why:** Clap's default behavior is to parse all flags/options, even after positional arguments
- **Solution:** Add `script_args: Vec<String>` with `#[arg(trailing_var_arg = true, allow_hyphen_values = true)]`
- **Implication:** Enables natural CLI argument passing: `ruff run script.ruff --input file.txt --verbose`

(Discovered during: 2026-01-25_23-15_arg-parser-implementation.md)

### Optional arguments without defaults need explicit Null
- **Problem:** When optional arguments aren't provided and have no default, accessing them in dict returns 0
- **Rule:** Always add `Value::Null` for optional non-bool arguments without defaults in parse results
- **Why:** Dictionary access for missing keys returns 0 (default behavior), which is confusing
- **Solution:** In `parse_arguments()`, explicitly insert `Value::Null` for missing optional arguments
- **Code location:** src/builtins.rs, parse_arguments() function

(Discovered during: 2026-01-25_23-15_arg-parser-implementation.md)

---

## Testing

### `run_code(...)` integration tests should isolate negative scenarios
- **Problem:** A single test script that intentionally triggers multiple runtime errors can produce missing follow-up variables and flaky assertions.
- **Rule:** Use one `run_code(...)` invocation per expected runtime error scenario in integration tests.
- **Why:** Once an error value path is produced, subsequent statements in that same script may not execute as expected in the test harness flow.
- **Implication:** For validation-error coverage, split checks into separate scripts rather than chaining failures in one program string.

(Discovered during: 2026-02-12_16-35-configurable-task-pool-sizing.md)

### `matches!` on `Value::Error(message)` can partially move test values
- **Problem:** Contract tests that both pattern-match error messages and print the full `Value` can fail to compile with move errors.
- **Rule:** Use borrowed bindings in `matches!` for error variants when the full value is reused: `Value::Error(ref message)`.
- **Why:** Binding `message` without `ref` moves the inner `String`, partially moving the parent `Value`.
- **Implication:** For assertion blocks that include debug output (`{:?}`), borrowed pattern bindings avoid `E0382` (`borrow of partially moved value`).

(Discovered during: 2026-02-16_17-28_release-hardening-array-higher-order-collection-contracts.md)

### `FuturesUnordered` requires one concrete future type
- **Problem:** Replacing batched await logic with `FuturesUnordered` fails to compile with type mismatch errors (`E0308`) when pushing multiple inline `async` blocks.
- **Rule:** Build all in-flight futures through one closure/function so every pushed future has the same concrete type.
- **Why:** Distinct `async` block literals always have different anonymous types, even when bodies are equivalent.
- **Implication:** For bounded async polling in native runtime code, centralize future creation (`let make_future = |...| async move { ... }`) before pushing.

(Discovered during: 2026-02-12_18-19_promise-all-large-array-optimization.md)

### Disk pressure can break fmt/build workflows
- **Problem:** `cargo fmt`/build can fail with `No space left on device (os error 28)` during heavy local iteration.
- **Rule:** Check and prune `target/` when formatter/build failures look like I/O errors.
- **Why:** Ruff build artifacts can grow to multi-GB size in active sessions.
- **Implication:** Before deep debugging odd formatter/build failures, run `du -sh target` and clean artifacts if needed.

(Discovered during: 2026-02-12_18-19_promise-all-large-array-optimization.md)

### jsonwebtoken 10.x requires explicit CryptoProvider feature selection
- **Problem:** Full test suite panics in JWT tests with `Could not automatically determine the process-level CryptoProvider...`
- **Rule:** Configure `jsonwebtoken` with exactly one provider feature in `Cargo.toml` (e.g. `features = ["rust_crypto"]`)
- **Why:** jsonwebtoken v10 requires a deterministic process-level crypto backend
- **Implication:** Targeted feature tests may pass while full suite fails; always run `cargo test` after JWT/dependency changes

(Discovered during: 2026-02-12_15-33-await-all-batching-and-jwt-provider-fix.md)

### Each feature should have dedicated test file with 10-15 cases
- **Problem:** Test coverage can be sparse if cases are scattered
- **Rule:** Create dedicated `tests/<feature>.ruff` file with comprehensive test cases for each new feature
- **Why:** Easier to verify complete coverage and find regressions. Each case tests one specific aspect.
- **Example:** `tests/destructuring.ruff` has 15 cases covering arrays, dicts, nested, rest, ignore, edge cases
- **Implication:** Budget time for comprehensive test creation as part of feature work, not an afterthought.

(Discovered during: 2026-01-25_14-30_destructuring-spread-implementation.md)

---

## Mental Model Summary

- Ruff favors **explicit AST structure** over overloaded general-purpose nodes (e.g., dedicated `Pattern` enum vs reusing expressions)
- The parser assumes **context determines syntax validity** (spread operator only valid in specific contexts)
- The runtime guarantees **lexical scoping with Environment chains** (variable lookup walks parent scopes)
- Do NOT assume **single-character lookahead is sufficient** for lexing (multi-char operators like `...` need explicit peek-ahead)
- Do NOT assume **all syntax starts with identifiers** (patterns can start with punctuation like `[` and `{`)

---

## Value Enum & Type System

### Adding Value Enum Variants Triggers Exhaustive Match Errors

- **Problem:** Compilation fails in multiple files when adding new Value type
- **Locations:** 
  - `src/interpreter.rs` - `type()` introspection (~line 3119), Debug impl (~line 460)
  - `src/builtins.rs` - `format_debug_value()` (~line 1561)
  - All pattern matches on Value enum
- **Rule:** Search codebase for `match.*Value` to find all locations needing updates
- **Why:** Rust exhaustive pattern matching - compiler forces handling of all variants
- **Prevention:** Use compiler errors as checklist; each error shows one location to update

(Discovered during: 2026-01-26_02-44_iterators-generators-implementation.md)

### call_user_function Only Handles Specific Value Types

- **Problem:** Calling non-function values silently returns Int(0)
- **Location:** `src/interpreter.rs` ~line 1495
- **Rule:** Pattern match handles Function, GeneratorDef, falls through to `_ => Value::Int(0)` for others
- **Implication:** NativeFunction, StructDef methods, and other callables handled elsewhere

### BytecodeFunction Mappers Must Use VM Execution APIs

- **Problem:** Passing `Value::BytecodeFunction` into interpreter-side call helpers can silently produce wrong fallback behavior.
- **Rule:** Do NOT route bytecode closures through `call_user_function`; use VM execution (`call_function_from_jit`) and optional eager compile helpers.
- **Why:** Interpreter call helper does not execute bytecode closures; VM call path owns bytecode/JIT execution semantics.
- **Location:** `src/interpreter/mod.rs` (`call_user_function`) + `src/vm.rs` (`call_function_from_jit`)
- **Implication:** For native helpers like `parallel_map`, add explicit bytecode-function branch rather than relying on generic callable fallback.

(Discovered during: 2026-02-13_18-31_parallel-map-jit-closures-and-rayon.md)

(Discovered during: 2026-01-26_02-44_iterators-generators-implementation.md)

### User-creatable types need constructor functions
- **Problem:** Internal Value types (like Bytes) can't be created from Ruff code without a constructor
- **Rule:** If users need to create instances of a Value type, provide a native constructor function
- **Example:** `bytes(array)` converts `[72, 101, 108, 108, 111]` to binary data
- **Why:** Not all Value types should be directly constructible, but common ones (especially for I/O) need constructors
- **Implication:** When adding new Value variants for resources, consider if user code needs a way to create them

(Discovered during: 2026-01-26_02-13_net-module-tcp-udp.md)

---

## Native Function Implementation

### Native functions require two-part registration
- **Problem:** Function exists but isn't callable from Ruff code
- **Rule:** Native functions must be:
  1. Registered in `Interpreter::new()` via `self.env.define("name", Value::NativeFunction("name"))`
  2. Implemented in `call_native_function_impl()` match statement
- **Why:** Registration binds the name in the environment; implementation provides the logic
- **Implication:** Function name strings must match exactly in both places. Search for similar functions to find the right location.

(Discovered during: 2026-01-26_02-13_net-module-tcp-udp.md)

### Use ErrorObject, not Error for new code
- **Problem:** Legacy `Value::Error(String)` still exists for backward compatibility
- **Rule:** New code should use `Value::ErrorObject { message, stack, line, cause }`
- **Why:** ErrorObject provides stack traces and better debugging; Error is deprecated pattern
- **Example:** 
  ```rust
  Value::ErrorObject {
      message: format!("Failed to bind on '{}': {}", addr, e),
      stack: Vec::new(),
      line: None,
      cause: None,
  }
  ```
- **Implication:** When writing error-returning functions, always use ErrorObject unless maintaining backward compatibility

(Discovered during: 2026-01-26_02-13_net-module-tcp-udp.md)

### Native function dispatcher pattern enables modular organization
- **Problem:** Adding native functions to monolithic `call_native_function_impl()` doesn't scale
- **Rule:** As of v0.9.0, native functions are organized in category modules under `src/interpreter/native_functions/`
- **Pattern:** Each module has `handle(interp, name, args) -> Option<Value>` that returns:
  - `Some(Value)` if the function was handled by this module
  - `None` if the function name is not recognized (try next module)
- **Dispatcher:** Main `call_native_function()` tries each module in order until one returns `Some`
- **Modules:** math, strings, collections, io, filesystem, http, system, type_ops, concurrency, json, crypto, database, network
- **Why:** First match wins - allows logical grouping by category while keeping dispatcher simple
- **Adding functions:** Just add a match case to appropriate module's `handle()` function - no registration needed!
- **Implication:** Function names must be unique across all modules (no two modules can handle the same name)
- **Location:** `src/interpreter/native_functions/mod.rs` (dispatcher), individual modules for implementations
- **Documentation:** See `docs/EXTENDING.md` for complete guide

(Discovered during: 2026-01-27_architecture-documentation-complete.md)

---

## I/O & Network Programming

### Network functions should handle both string and binary data
- **Problem:** Binary protocols fail if only string handling implemented
- **Rule:** Network send/receive functions must check for both `Value::Str` and `Value::Bytes`
- **Pattern:**
  ```rust
  match data {
      Value::Str(s) => stream.write_all(s.as_bytes()),
      Value::Bytes(b) => stream.write_all(b),
      _ => return Error(...)
  }
  ```
- **Why:** Network protocols often require binary data (headers, serialization, raw protocols)
- **Implication:** Any I/O function that transmits data should support both text and binary

(Discovered during: 2026-01-26_02-13_net-module-tcp-udp.md)

### Auto-detect string vs bytes in receive operations
- **Problem:** Unknown if received data is UTF-8 text or binary
- **Rule:** Try UTF-8 decode first, fall back to Bytes if invalid
- **Pattern:**
  ```rust
  match String::from_utf8(buffer) {
      Ok(s) => Value::Str(s),
      Err(_) => Value::Bytes(buffer),
  }
  ```
- **Why:** Provides best user experience - text is string, binary stays binary
- **Implication:** Receiving functions should be smart about data types rather than forcing users to handle conversion

(Discovered during: 2026-01-26_02-13_net-module-tcp-udp.md)

### Connectionless protocols must return sender information
- **Problem:** UDP receiver doesn't know who sent the datagram
- **Rule:** Functions like `udp_receive_from()` should return a dictionary with data + metadata
- **Pattern:** Return `{ "data": Value, "from": String, "size": Int }`
- **Why:** Enables bidirectional communication without separate connection establishment
- **Implication:** Don't just return the data - include context needed for protocols to function

(Discovered during: 2026-01-26_02-13_net-module-tcp-udp.md)

---

## Resource Management

### Stateful resources use Arc<Mutex<>> pattern
- **Problem:** Value enum must be Clone, but resources like sockets/files aren't
- **Rule:** Wrap stateful resources in `Arc<Mutex<T>>` within Value variants
- **Example:** `TcpStream { stream: Arc<Mutex<std::net::TcpStream>>, peer_addr: String }`
- **Why:** Allows Value to be cloned while sharing single underlying resource
- **Pattern also used for:** Database, HttpServer, Image, ZipArchive, Channel
- **Implication:** All resource-based Value types should follow this pattern for consistency

(Discovered during: 2026-01-26_02-13_net-module-tcp-udp.md)

### Close functions rely on RAII, not explicit cleanup
- **Problem:** "Close" functions don't actually close the resource
- **Rule:** Dropping the Value closes the resource via Rust's RAII; close functions just return success
- **Why:** Rust automatically closes resources when Arc refcount reaches zero
- **Implication:** Users can call close() for clarity, but it's not strictly necessary. The function exists for API completeness.
- **Pattern:** `tcp_close()`, `udp_close()`, `db_close()` all just return `Value::Bool(true)`

(Discovered during: 2026-01-26_02-13_net-module-tcp-udp.md)

### LeakyFunctionBody prevents stack overflow during drop
- **Problem:** Program crashes with stack overflow during shutdown when functions have deeply nested AST
- **Root cause:** Function bodies are `Vec<Stmt>`, and Stmt contains nested `Vec<Stmt>` (in If, While, For, etc.). Rust's automatic Drop implementation recurses deeply through these structures, causing stack overflow.
- **Rule:** `LeakyFunctionBody` uses `ManuallyDrop<Arc<Vec<Stmt>>>` to prevent automatic drop - the memory is intentionally leaked
- **Why:** OS reclaims all memory at program shutdown anyway, so leaking is acceptable. This avoids stack overflow for deeply nested code.
- **Symptom:** Stack overflow errors during program exit (not during execution)
- **Workaround:** Current implementation intentionally leaks function bodies. They're never freed during runtime.
- **Future fix:** Roadmap Task #29 will implement iterative drop or arena allocation to properly free memory without recursion
- **Implication:** Long-running programs that define many functions dynamically will accumulate leaked memory. Acceptable for most use cases but may matter for REPL or servers.
- **Location:** `src/interpreter/value.rs:20-42`

(Discovered during: 2026-01-27_architecture-documentation-complete.md)

---

## Async/Await & Concurrency

### Cooperative VM Await currently uses an internal suspend sentinel

- **Problem:** Cooperative suspension must propagate through VM internals even though `execute()` returns `Result<Value, String>`.
- **Rule:** Suspension is encoded internally with a sentinel-prefixed error string and converted back to `VmExecutionResult` only at cooperative API boundaries.
- **Why:** Existing VM execution contract is string-error based; changing the core loop signature in one step would cause broad churn.
- **Location:** `src/vm.rs` (`VM_SUSPEND_ERROR_PREFIX`, `execute_until_suspend`, `resume_execution_context`, `parse_suspend_error`)
- **Implication:** External callers should never parse suspend sentinel strings directly; they should only use cooperative APIs.

(Discovered during: 2026-02-13_19-31_vm-cooperative-await-yield-resume.md)

### Suspend points in opcode handlers must rewind for replay correctness

- **Problem:** A suspended `Await` can lose state or skip execution on resume.
- **Rule:** If an opcode pre-mutates IP/stack before deciding to suspend, restore replay state before snapshotting.
- **Why:** The VM dispatch loop increments `ip` before opcode execution and `Await` pops the promise before completion.
- **Fix pattern:** For pending cooperative `Await`, push promise back and set `self.ip = self.ip.saturating_sub(1)` before saving context.
- **Implication:** Any future suspendable opcode must preserve deterministic replay boundaries.

(Discovered during: 2026-02-13_19-31_vm-cooperative-await-yield-resume.md)

### VM unit tests need explicit native symbol bindings for direct bytecode execution

- **Problem:** VM tests fail with `Undefined global` for native functions (for example `async_sleep`) even when the runtime supports them.
- **Rule:** In VM-only tests, register each native function name used by the compiled Ruff snippet into globals explicitly.
- **Why:** Direct bytecode VM test paths do not implicitly register every interpreter builtin symbol.
- **Implication:** Before debugging opcode behavior, verify required native names are defined in test globals.

(Discovered during: 2026-02-13_19-31_vm-cooperative-await-yield-resume.md)

### Tokio oneshot receivers are single-use and must be extracted from Arc<Mutex<>>

- **Problem:** Cannot await tokio::oneshot::Receiver while holding mutex guard - causes type errors or deadlocks
- **Root cause:** Oneshot receivers are consumed on await (moved, not borrowed), but they're stored in `Arc<Mutex<Receiver<T>>>` for thread safety
- **Rule:** Extract receiver using `std::mem::replace()` with dummy closed channel BEFORE awaiting
- **Pattern:**
  ```rust
  let actual_rx = {
      let mut recv_guard = receiver.lock().unwrap();
      let (dummy_tx, dummy_rx) = tokio::sync::oneshot::channel();
      drop(dummy_tx); // Close dummy immediately
      std::mem::replace(&mut *recv_guard, dummy_rx)
  };
  // Now can await actual_rx without holding lock
  let result = AsyncRuntime::block_on(actual_rx);
  ```
- **Why:** Oneshot channels are designed for single-use - send once, receive once. This matches Promise semantics perfectly (resolve once, cache result).
- **Location:** `src/interpreter/mod.rs:3881-3950` (Await expression), `src/vm.rs:1000-1077` (VM Await opcode)
- **Implication:** This pattern appears in both tree-walking interpreter and bytecode VM. Any code awaiting Promises must use this pattern.

(Discovered during: 2026-01-27_20-54_phase5-tokio-async-runtime.md)

### VM and Interpreter require SEPARATE builtin registration

- **Problem:** Function works with `--interpreter` flag but fails with "Undefined global" in VM (default mode)
- **Root cause:** VM uses compile-time constant `NATIVE_FUNCTIONS` array for name resolution, interpreter uses runtime `register_builtins()` method
- **Rule:** ALWAYS update BOTH locations when adding a new native function:
  1. `src/interpreter/mod.rs:~388` - Add to `NATIVE_FUNCTIONS` const array
  2. `src/interpreter/mod.rs:~903` - Add to `register_builtins()` method
- **Why:** VM compiler checks function names at compile time for optimization. Interpreter resolves at runtime.
- **Testing:** Always test both modes: `cargo run -- run file.ruff` (VM default) and `cargo run -- run --interpreter file.ruff`
- **Symptom:** Type checker may warn about undefined function, but more critically, VM will reject it at compile time
- **Prevention:** Make it a checklist item when adding native functions - register in both places

(Discovered during: 2026-01-27_20-54_phase5-tokio-async-runtime.md)

### Builtin declaration in `mod.rs` does NOT guarantee runtime implementation after modularization

- **Problem:** A builtin can appear fully registered (`get_builtin_names`, `register_builtins`) but still behave incorrectly at runtime.
- **Root cause:** Actual execution goes through `src/interpreter/native_functions/mod.rs` dispatcher; missing handler branches in category modules (`filesystem.rs`, `collections.rs`, etc.) are easy to miss.
- **Rule:** For every builtin or alias, update all three surfaces:
  1. `Interpreter::get_builtin_names()`
  2. `Interpreter::register_builtins()`
  3. Native handler match arm in the correct `src/interpreter/native_functions/*.rs` module
- **Why:** Post-modularization API drift can pass superficial registration checks while failing behaviorally.
- **Implication:** Prefer behavior-level contract tests (execute builtin and validate output), not name-list checks alone.

(Discovered during: 2026-02-15_09-18_release-hardening-alias-api-contract.md)

### Unknown native dispatch is fail-fast; unexpected `0` means builtin semantics, not dispatcher fallback

- **Problem:** It is easy to keep debugging parser/call-site logic when a builtin appears to return `0`.
- **Rule:** Unknown native names now return explicit errors (`Unknown native function: <name>`). If you still see `0`, investigate the builtin's own implementation path rather than dispatcher unknown-name fallback.
- **Why:** `src/interpreter/native_functions/mod.rs` was hardened from silent `Value::Int(0)` fallback to `Value::Error(...)` for unknown names.
- **Implication:** Keep dispatcher contract tests for high-risk builtins so registration/handler drift fails loudly.

(Discovered during: 2026-02-15_09-18_release-hardening-alias-api-contract.md; updated during: 2026-02-15_09-51_release-hardening-native-dispatch-contract.md)

### Exhaustive builtin dispatch probes should use safe-probe arguments for side-effecting APIs

- **Problem:** Full dispatch probe tests can block, mutate environment/process state, or terminate the test run if side-effecting APIs are called on success paths.
- **Rule:** Do NOT execute side-effecting success paths in exhaustive drift tests; probe those builtins with deterministic invalid-shape arguments instead of skipping them.
- **Current safe probes:**
  - `input` → call with `Int` (returns immediate argument-shape error)
  - `exit` → call with `String` (returns immediate argument-shape error)
- **Why:** This keeps exhaustive declared-builtin drift coverage complete while avoiding interactive/blocking/terminal side effects.
- **Implication:** Prefer contract-error-path probes over skip lists when a builtin has a deterministic non-side-effect validation branch.

(Discovered during: 2026-02-15_16-17_release-hardening-dispatch-gap-slices.md; updated during: 2026-02-16_16-12_release-hardening-safe-probe-input-exit.md)

### Treat dispatch known-gap list as an explicit migration ledger

- **Problem:** Asserting "all declared builtins dispatch" fails immediately while modular extraction is intentionally incomplete.
- **Rule:** During migration, assert drift test output against an explicit expected known-gap list and shrink it in the same commit as each migrated API slice.
- **Why:** This catches accidental regressions/new drift without blocking incremental hardening progress.
- **Implication:** Keep gap-list updates tightly coupled with implementation changes (`system`, data-format, regex, etc.) so test signal stays trustworthy.

(Discovered during: 2026-02-15_16-17_release-hardening-dispatch-gap-slices.md)

### `Set(...)` constructor migration needs behavior-contract coverage, not just dispatch coverage

- **Problem:** A migrated constructor builtin can pass unknown-native fallback checks while still regressing constructor semantics.
- **Rule:** When migrating `Set(...)`, validate all constructor contracts together:
  - `Set()` must return an empty set,
  - `Set(array)` must deduplicate using `Interpreter::values_equal(...)`,
  - non-array input and invalid arity must return explicit `Value::Error`.
- **Why:** Dispatcher-level probes only prove routing; they do not prove constructor shape/equality behavior.
- **Implication:** Pair constructor migration with dedicated behavior tests and update known-gap ledger in the same change.

(Discovered during: 2026-02-15_20-53_set-constructor-dispatch-hardening.md)

### Database hardening tests should anchor on SQLite for deterministic CI

- **Problem:** Modular `db_*` handlers support sqlite/postgres/mysql, but test environments typically do not guarantee live Postgres/MySQL services.
- **Rule:** Validate dispatch/contract behavior with SQLite-backed tests first; keep Postgres/MySQL paths implemented but avoid hard test dependencies on external DB infrastructure.
- **Why:** SQLite runs in-process and makes argument-shape, transaction, pool, and query contract tests stable and reproducible.
- **Implication:** Use SQLite for baseline regression confidence, then treat Postgres/MySQL integration verification as environment-specific follow-up work.

(Discovered during: 2026-02-15_17-13_release-hardening-modular-dispatch-gap-closures.md)

### Value enum cannot derive PartialEq due to interior mutability

- **Problem:** Cannot use `assert_eq!(value1, value2)` in tests - compiler error about missing PartialEq
- **Root cause:** Value::Promise contains `Arc<Mutex<tokio::sync::oneshot::Receiver<...>>>` which doesn't implement PartialEq
- **Rule:** Value enum intentionally CANNOT be compared for equality - use pattern matching instead
- **Why:** Mutex provides interior mutability, channels have no meaningful equality semantics
- **Test pattern:**
  ```rust
  match result {
      Value::Int(42) => {}, // Success
      _ => panic!("Expected Int(42)"),
  }
  ```
- **Implication:** This is fundamental to Ruff's design - Values contain stateful resources (channels, mutexes, file handles) that don't have equality semantics

(Discovered during: 2026-01-27_20-54_phase5-tokio-async-runtime.md)

---

## Quick Reference: Adding a New Built-in Function

1. Register in `Interpreter::new()`: `self.env.define("func_name", Value::NativeFunction("func_name"))`
2. Implement in `call_native_function_impl()` match statement
3. Use `ErrorObject` for errors, not `Error`
4. Handle both string and binary data if I/O related
5. Update type checker (separate task, not blocking)
6. Write tests in `tests/` directory
7. Add example in `examples/` if user-facing
8. Document in CHANGELOG.md

---

## Quick Reference: Adding a New Value Variant

1. Add variant to `Value` enum in `src/interpreter/value.rs` (updated path as of 2026-01-26)
2. Update `Debug` impl for Value (same file, ~line 328)
3. Update `format_debug_value()` in `src/builtins.rs` (~line 1550)
4. Update `type()` function match in `src/interpreter/mod.rs` in `call_native_function_impl()` (~line 2800)
5. If user-creatable, add constructor function
6. If stateful resource, use `Arc<Mutex<T>>` pattern
7. Test with `cargo build` to catch any missed matches

---

## Codebase Architecture & Module Design

### Methods with &mut self must stay in the same impl block

- **Problem:** Large functions like `call_native_function_impl` (5,700 lines) cannot be easily extracted to separate modules
- **Rule:** Rust requires all methods accessing `&mut self` to be in the same `impl` block. You cannot split an impl across files without trait-based refactoring.
- **Why:** The 5,700-line `call_native_function_impl` in `src/interpreter/mod.rs` is a dispatch table that accesses `self.env`, `self.output`, and calls dozens of other interpreter methods (`self.eval_expr()`, `self.call_user_function()`, etc.)
- **Symptom:** Attempting to move this function to a separate module results in "cannot find value `self` in this scope" errors
- **Solution:** Keep the function in mod.rs. It's well-organized with category comments (I/O, math, strings, collections, file I/O, HTTP, database, crypto, image processing, networking).
- **Implication:** Don't try to modularize this function without a complete architectural redesign using traits. A 5,700-line method is acceptable if it's well-organized internally.

(Discovered during: 2026-01-26_14-30_interpreter-modularization-gotchas.md)

### pub use re-exports are essential for refactoring

- **Problem:** Moving types to submodules breaks import paths for downstream code
- **Rule:** When extracting types to submodules, always add `pub use submodule::Type;` in the parent module
- **Why:** This preserves the original import path (`interpreter::Value` still works even though Value is now in `interpreter::value::Value`)
- **Example:**
  ```rust
  // In src/interpreter/mod.rs:
  pub mod value;
  pub mod environment;
  
  pub use value::Value;
  pub use environment::Environment;
  ```
- **Implication:** Zero breaking changes possible even when reorganizing internal module structure. All external crates continue to work without modification.

(Discovered during: 2026-01-26_14-30_interpreter-modularization-gotchas.md)

### Circular type dependencies work between sibling modules

- **Problem:** `Value` enum needs `Environment` (for Function variants), and `Environment` needs `Value` (for variable storage)
- **Rule:** Use `use super::other_module::Type` in both files. Rust's compiler resolves circular type references between sibling modules.
- **Example:**
  ```rust
  // In src/interpreter/value.rs:
  use super::environment::Environment;
  
  // In src/interpreter/environment.rs:
  use super::value::Value;
  ```
- **Why:** Both are just type definitions, not circular initialization. The compiler builds the dependency graph from separate files.
- **Implication:** Don't fear circular type dependencies in the AST or runtime — they're safe and idiomatic in this pattern.

(Discovered during: 2026-01-26_14-30_interpreter-modularization-gotchas.md)

### Line count is not a success metric for refactoring

- **Problem:** Thinking "big file = bad design" leads to artificial splitting of tightly coupled code
- **Rule:** Modularize when there are independently meaningful units with minimal coupling. Don't split tightly coupled code just to reduce line counts.
- **Why:** A 5,700-line dispatch function is fine if it's well-organized internally (clear comments, logical categories). Shared mutable state (`&mut self`) is a signal to keep code together.
- **Example:** After extracting Value (500 lines) and Environment (110 lines), remaining mod.rs functions all need `&mut self` and make heavy cross-calls. Further extraction would require trait-based refactoring with questionable value.
- **Implication:** The interpreter's `call_native_function_impl` should stay in one place — it's a single logical dispatch point. Evaluate by structure and maintainability, not arbitrary line limits.

(Discovered during: 2026-01-26_14-30_interpreter-modularization-gotchas.md)

---

## VM & Bytecode

### Chunk restoration is critical during exception unwinding

- **Problem:** Exception caught but execution continues in wrong bytecode chunk
- **Rule:** When unwinding call frames during exception handling, must restore `self.chunk` from `frame.prev_chunk`
- **Why:** Function calls switch to function's bytecode chunk. When exception unwinds through function calls, must restore caller's chunk.
- **Solution:** Only restore chunk when reaching target frame depth (not on each frame pop)
- **Code location:** `src/vm.rs` OpCode::Throw implementation
- **Symptom:** Execution continues after catch block but in wrong function's bytecode, causing wrong instructions to execute

(Discovered during: 2026-01-26_vm-exception-handling.md)

### set_jump_target must handle all opcodes with jump addresses

- **Problem:** Compiler calls `set_jump_target(begin_try_index, catch_start)` but method only handles Jump/JumpIfFalse/JumpIfTrue
- **Rule:** Any opcode containing a jump target (usize address) must be handled in `set_jump_target()` and `patch_jump()` methods
- **Why:** Compiler generates opcodes with placeholder addresses (0) and patches them later when target address is known
- **Solution:** Add new opcode variants to match arms in bytecode.rs methods
- **Example:** `OpCode::BeginTry(ref mut addr)` needs to be in set_jump_target match
- **Symptom:** Panic: "Attempted to set target on non-jump instruction"

(Discovered during: 2026-01-26_vm-exception-handling.md)

### Exception handlers must pop BEFORE unwinding

- **Problem:** Should exception handler be popped before or after stack/frame unwinding?
- **Rule:** Pop exception handler BEFORE unwinding, so nested handlers work correctly
- **Why:** LIFO order - innermost try block's handler should catch first
- **Example:**
  ```ruff
  try {              # Handler A
      try {          # Handler B
          throw()    # Should unwind to B, not A
      } except {}
  } except {}
  ```
- **Implication:** Stack-based exception handling naturally supports nesting when handlers popped in LIFO order

(Discovered during: 2026-01-26_vm-exception-handling.md)

---

## JIT Compilation (Cranelift)

### Symbol registration must happen BEFORE JITModule::new()

- **Problem:** "undefined symbol" errors at JIT execution time even with `#[no_mangle]` functions
- **Rule:** Call `builder.symbol("name", ptr as *const u8)` BEFORE `JITModule::new(builder)`
- **Why:** `JITBuilder` is consumed by `JITModule::new()`; symbol registration state is frozen
- **Location:** `src/jit.rs` in `JitCompiler::new()` (~line 615-632)
- **Implication:** All external symbols must be registered during JIT compiler initialization, not per-compilation
- **Fix:**
  ```rust
  let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
  builder.symbol("jit_load_variable", jit_load_variable as *const u8);
  builder.symbol("jit_store_variable", jit_store_variable as *const u8);
  let module = JITModule::new(builder);  // AFTER registration
  ```

(Discovered during: 2026-01-27_14-51_jit-phase-3-completion.md)

### External function signatures must EXACTLY match runtime

- **Problem:** Segfaults, wrong values, or mysterious crashes when calling external functions
- **Rule:** Cranelift `sig.params` must exactly match runtime function parameter types and order
- **Why:** No type checking across FFI boundary; Cranelift generates raw function calls
- **Example:**
  ```rust
  // Runtime: jit_load_variable(ctx: *mut VMContext, hash: u64) -> i64
  // Cranelift declaration:
  sig.params.push(AbiParam::new(pointer_type));  // *mut VMContext
  sig.params.push(AbiParam::new(types::I64));    // hash u64
  sig.returns.push(AbiParam::new(types::I64));   // return i64
  ```
- **Implication:** Document function signatures clearly; validate with end-to-end tests

(Discovered during: 2026-01-27_14-51_jit-phase-3-completion.md)

### FuncRef is scoped to function being built

- **Problem:** Cannot store `FuncRef` in struct before entering builder context
- **Rule:** Store `FuncId` globally, obtain `FuncRef` per function with `declare_func_in_func()`
- **Why:** `FuncRef` is tied to builder context and specific function definition
- **Location:** `src/jit.rs` in `compile()` method (~line 700-702)
- **Pattern:**
  ```rust
  // In compile(): Declare once
  let func_id = module.declare_function("name", Linkage::Import, &sig)?;
  
  // In translate loop: Get per function
  let func_ref = module.declare_func_in_func(func_id, builder.func);
  self.store_var_func = Some(func_ref);  // Store in translator
  ```
- **Implication:** External function FuncRef must be obtained inside function builder scope

(Discovered during: 2026-01-27_14-51_jit-phase-3-completion.md)

### Every basic block MUST have a terminator

- **Problem:** Panic with "block has no terminator" during IR finalization
- **Rule:** Every code path must end with `return`, `jump`, or `branch` instruction
- **Why:** Cranelift IR requires explicit control flow; no implicit fall-through
- **Location:** `src/jit.rs` in `translate_instruction()` (~line 253-620)
- **Fix:** Ensure all branches end with terminator; add explicit `builder.ins().return_()` at function end
- **Implication:** Check all code paths in control flow; empty blocks still need terminators

(Discovered during: 2026-01-27_14-51_jit-phase-3-completion.md)

### Block sealing order matters for SSA

- **Problem:** Panics with "unsealed block" or "block not filled" errors
- **Rule:** Use two-pass translation - create all blocks first, then translate instructions, and seal blocks only after predecessor edges are finalized and terminators are emitted.
- **Rule (critical):** Each block must be sealed exactly once. Early sealing or duplicate sealing on alternate control-flow paths can trigger Cranelift assertions (including already-sealed failures).
- **Why:** Cranelift requires all predecessors of a block known before sealing (SSA form)
- **Pattern:**
  ```rust
  // Pass 1: Create all blocks
  for _ in 0..num_blocks { builder.create_block(); }
  
  // Pass 2: Translate instructions
  for instr in bytecode { translate_instruction(instr); }
  
  // Pass 3: Seal after terminators added
  builder.seal_block(block);
  ```
- **Implication:** Cannot seal blocks until all jumps to them are generated

(Discovered during: 2026-01-27_14-51_jit-phase-3-completion.md, reinforced in: 2026-02-12_14-52_hashmap-fusion-jit-sealing-release.md)

### Hash-based variable resolution has collision risk

- **Problem:** Two different variable names could hash to same value
- **Rule:** Using `DefaultHasher` acceptable for small variable counts; consider SHA256 for production
- **Probability:** Negligible for realistic function sizes (< 1000 variables)
- **Location:** `src/jit.rs` in `translate_instruction()` LoadVar/StoreVar cases (~line 489-558)
- **Alternatives:**
  - Cryptographic hash (zero collision risk)
  - Variable name indices (requires stable mapping)
  - Collision detection in var_names HashMap
- **Implication:** Current solution sufficient; don't over-engineer unless profiling shows problem

(Discovered during: 2026-01-27_14-51_jit-phase-3-completion.md)

### Loop headers require consistent stack state (SSA block parameters)

- **Problem:** Loop JIT fails with "Stack empty" or "block parameter count mismatch" errors
- **Rule:** SSA block parameters for loop headers are determined by stack state at FIRST entry. Every subsequent entry (via JumpBack) must have IDENTICAL stack state.
- **Why:** Cranelift loop headers are SSA blocks with parameters representing live values. If loop entry and loop iteration have different stack depths, block parameters don't match.
- **Symptom:** Functions with loops silently fall back to interpreter; no compilation errors but huge performance loss
- **Root cause (usually):** Stack pollution from missing Pop after statements. See "StoreVar uses PEEK semantics" gotcha.
- **Debug technique:** Use `DEBUG_JIT=1` and trace stack depths at loop header vs JumpBack instruction
- **Prevention:** Ensure bytecode compiler maintains clean stack between statements

(Discovered during: 2026-01-28_18-50_compiler-stack-pop-fix.md)

---


## JIT Type Specialization (Phase 4)

### Type specialization is function-level, not operation-level

- **Problem:** You might think each Add/Sub/Mul/Div operation gets specialized individually based on operand types
- **Reality:** Specialization happens at function boundaries. A function is profiled, then recompiled with type assumptions for ALL its variables
- **Rule:** Type guards go at function entry. Assumptions hold for the entire function body. If guards fail, deoptimize the whole function.
- **Why:** Stack-based bytecode doesn't track value provenance. You don't know which variables produced stack values.
- **Implication:** 
  - Don't try to specialize per-operation
  - Think: "compile this function assuming x:Int, y:Int"
  - Not: "specialize this Add if operands are Int"
- **Reference:** `src/jit.rs` BytecodeTranslator specialization methods

(Discovered during: 2026-01-27_10-53_phase4b-specialized-codegen.md)

### Cranelift bitcast API is not obvious for i64⇄f64 conversion

- **Problem:** You assume `builder.ins().bitcast(types::F64, value)` will reinterpret i64 as f64
- **Symptom:** Compilation error: `the trait bound 'MemFlags: From<cranelift::prelude::Value>' is not satisfied`
- **Reality:** Cranelift's `bitcast()` requires 3 arguments: type, memory flags, and value. There's no `raw_bitcast()` method.
- **Rule:** Float specialization requires researching Cranelift's type conversion APIs. Options: fcvt instructions, load/store through memory, or bitcast with correct MemFlags.
- **Why:** Type conversion is architecture-dependent and needs explicit flag handling
- **Implication:** Int specialization is trivial (native i64 ops). Float specialization is a research task.
- **Location:** Phase 4B attempted float specialization, deferred to Phase 4C

(Discovered during: 2026-01-27_10-53_phase4b-specialized-codegen.md)

### JIT type profiles grow unbounded

- **Problem:** `JitCompiler::type_profiles` HashMap has no eviction policy
- **Symptom:** Memory grows if many functions are executed once or infrequently
- **Rule:** Current implementation trades memory for simplicity. Production code needs LRU or size limits.
- **Why:** Every function offset that hits JIT_THRESHOLD (100) creates a profile entry that never expires
- **Implication:** Fine for benchmarks and development. Needs attention before production use.
- **Location:** `src/jit.rs` line 439 - `type_profiles: HashMap<usize, SpecializationInfo>`

(Discovered during: 2026-01-27_10-53_phase4b-specialized-codegen.md)

### Test compilation success ≠ execution validation

- **Problem:** JIT tests that only check `compile().is_ok()` don't validate generated code correctness
- **Symptom:** Tests pass but compiled code might produce wrong results
- **Rule:** Compilation tests validate IR generation. Execution tests validate semantics. You need both.
- **Why:** Cranelift can successfully compile incorrect IR if instructions are well-formed
- **Implication:** 
  - Use `is_ok()` tests for IR generation coverage (Phase 4B approach)
  - Use execution tests with result validation for correctness (Phase 3 approach)
  - Need both types of tests for complete coverage
- **Location:** Phase 4B tests (lines 1630-1730 in `src/jit.rs`)

(Discovered during: 2026-01-27_10-53_phase4b-specialized-codegen.md)

### Specialized methods exist but aren't wired up yet (Phase 4B→4C boundary)

- **Problem:** Created `translate_add_specialized()` etc. but they're never called
- **Symptom:** Methods exist, tests compile, but specialization doesn't happen at runtime
- **Rule:** Phase 4B creates methods, Phase 4C integrates them. This is intentional separation.
- **Why:** Clear phase boundaries prevent scope creep and allow incremental testing
- **Implication:** 
  - Don't be surprised when specialized methods aren't used yet
  - Phase 4C task: Modify `translate_instruction()` to check `self.specialization` and route to specialized methods
  - Integration requires deciding: per-operation or per-function specialization strategy

(Discovered during: 2026-01-27_10-53_phase4b-specialized-codegen.md)

---

*Last updated: 2026-01-27 (Phase 4B - Type Specialization)*

### Cannot recompile same bytecode offset twice in one JitCompiler

- **Problem:** Test fails with "Duplicate definition of identifier: ruff_jit_XXX"
- **Symptom:** Second call to `compiler.compile(&chunk, offset)` with same offset fails
- **Reality:** Cranelift JITModule creates function names as `ruff_jit_{offset}`. Each name must be unique.
- **Rule:** Each bytecode offset can only be compiled once per JitCompiler instance.
- **Fix:** Use different offsets for each compilation:
  ```rust
  compiler.compile(&chunk1, 100);  // ruff_jit_100
  compiler.compile(&chunk2, 200);  // ruff_jit_200 ✓
  // NOT: compiler.compile(&chunk2, 100); // ERROR: duplicate
  ```
- **Why:** Cranelift's symbol table prevents duplicate function definitions
- **Implication:** In tests compiling multiple chunks, use unique offsets or create new JitCompiler instances

(Discovered during: 2026-01-27_11-02_phase4c-integration.md)

### Specialization decisions happen at compile-time, not runtime

- **Problem:** You might think specialization checks happen during code execution
- **Reality:** `self.specialization.is_some()` is checked during **Cranelift IR generation** (compilation), not during native code execution
- **Rule:** Specialization is a compilation strategy that produces different native code, not a runtime dispatch mechanism
- **Why:** The whole point is zero runtime overhead - no type checks per operation
- **Implication:**
  - Once compiled with specialization, function always uses that path
  - Type guards (Phase 4D) check assumptions at function **entry**, not per operation
  - Guard failures trigger deoptimization and recompilation
  - Can't dynamically switch between specialized/generic in same compiled function

(Discovered during: 2026-01-27_11-02_phase4c-integration.md)

---

## JIT Guard Generation (Phase 4D)

### Entry block must be sealed AFTER branching from it

- **Problem:** Panic: "unsealed blocks after building function"
- **Reality:** Entry block is unique - we branch FROM it, but no blocks branch TO it
- **Rule:** Seal entry_block immediately after creating brif from it. Do NOT seal guard_success_block immediately.
- **Why:** Cranelift requires blocks sealed after all incoming edges established. Entry has zero incoming edges (it's the entry!). Guard_success_block gets more jumps from instruction translation.
- **Pattern:**
  ```rust
  // Create guards and branch
  builder.ins().brif(all_guards, success_block, &[], failure_block, &[]);
  builder.seal_block(entry_block);  // ✓ Seal now - no more edges to it
  
  // Do NOT: builder.seal_block(guard_success_block); // ✗ More jumps coming!
  ```
- **Implication:** Block sealing order depends on control flow structure, not declaration order

(Discovered during: 2026-01-27_11-15_phase4d-guards.md)

### brif takes boolean directly, not integer that needs conversion

- **Problem:** Type error: "brif expects b1, got i64"
- **Reality:** `icmp` returns `b1` (boolean type), which is exactly what `brif` wants
- **Rule:** Use comparison results directly in brif. Don't try to convert with brnz or bint.
- **Why:** Cranelift has distinct bool (b1) and integer (i64) types. Comparisons return bool.
- **Pattern:**
  ```rust
  let guard_passed = builder.ins().icmp_imm(IntCC::Equal, result, 1); // b1
  builder.ins().brif(guard_passed, success, &[], failure, &[]);  // ✓
  
  // NOT: builder.ins().brnz(result, ...);  // ✗ wrong instruction
  ```
- **Implication:** Learn Cranelift's type system - b1 vs i64 distinction matters

(Discovered during: 2026-01-27_11-15_phase4d-guards.md)

### Guards must be ANDed together for multiple specialized variables

- **Problem:** Function executes optimized code when only some guards pass
- **Reality:** ALL type assumptions must hold for specialization to be safe
- **Rule:** Initialize guards to true (1), AND each check result, branch on final value
- **Why:** Partial specialization is unsafe. If ANY variable has wrong type, deoptimize.
- **Pattern:**
  ```rust
  let mut all_guards = builder.ins().iconst(types::I64, 1);  // Start true
  for (var, expected_type) in specialized_vars {
      let check = call_jit_check_type(var, expected_type);
      let passed = builder.ins().icmp_imm(IntCC::Equal, check, 1);
      all_guards = builder.ins().band(all_guards, passed);  // AND accumulator
  }
  builder.ins().brif(all_guards, success, failure);
  ```
- **Implication:** Guard logic scales to any number of specialized variables

(Discovered during: 2026-01-27_11-15_phase4d-guards.md)

### Guard failures return error codes, not exceptions

- **Problem:** Uncertainty about how to signal deoptimization needs
- **Reality:** Native code across FFI boundary can't throw Rust exceptions
- **Rule:** Return -1 as error code when guards fail. Calling code checks return value.
- **Why:** JIT functions are called from Rust via FFI (no exception unwinding)
- **Pattern:**
  ```rust
  // In guard_failure_block:
  let error_code = builder.ins().iconst(types::I64, -1);
  builder.ins().return_(&[error_code]);
  
  // In caller:
  let result = jit_func.call();
  if result == -1 {
      // Fall back to interpreter, invalidate specialization
  }
  ```
- **Implication:** Deoptimization is cooperative between JIT and VM

(Discovered during: 2026-01-27_11-15_phase4d-guards.md)

### current_block variable scope must span guard generation and translation

- **Problem:** Borrow checker error: "cannot find value `current_block` in this scope"
- **Reality:** `current_block` declared inside `if let Some(spec)` block not visible outside
- **Rule:** Declare mutable `current_block` before conditional guard logic, update in branches
- **Why:** Guard generation changes which block instructions are added to (entry vs guard_success)
- **Pattern:**
  ```rust
  let mut current_block = entry_block;  // Before conditionals
  
  if let Some(spec) = self.specialization {
      // Generate guards...
      current_block = guard_success_block;  // Update for instruction translation
  }
  
  // Now current_block visible here for instruction loop
  ```
- **Implication:** Variable scope must match usage patterns, not implementation structure

(Discovered during: 2026-01-27_11-15_phase4d-guards.md)

### Guards are safety mechanisms, not performance optimizations

- **Mental Model:** Guards ENABLE optimization, they don't optimize themselves
- **Reality:** Guards check type assumptions at function entry. Cost: ~5-10 instructions.
- **Rule:** Guards let you generate aggressive specialized code safely. The specialization is the optimization.
- **Why:** Without guards, can't make type assumptions → must use slow generic code. With guards, can assume types → use fast specialized code. Guard cost << specialization gain.
- **Implication:** Don't worry about guard overhead. Focus on optimization quality.
- **Pattern:** Same as V8 (JavaScript), PyPy (Python), LuaJIT - all use guard-based specialization

(Discovered during: 2026-01-27_11-15_phase4d-guards.md)

---

## JIT Recursive Functions (Phase 7)

### SSA block parameters: PEEK don't POP for conditionals

- **Problem:** JumpIfFalse/JumpIfTrue cause stack underflow or wrong values
- **Root cause:** Consuming condition value via `pop()` before branching leaves SSA in broken state
- **Rule:** Use `peek()` for JumpIfFalse/JumpIfTrue. Only `pop()` when opcode actually consumes value.
- **Why:** SSA block parameters need the value to flow to the target block. Popping removes it.
- **Pattern:**
  ```rust
  // ✓ CORRECT - peek for conditionals
  Opcode::JumpIfFalse(_) => {
      let condition = self.operand_stack.last().copied()...  // peek
  }
  
  // ✗ WRONG - pop loses the value
  Opcode::JumpIfFalse(_) => {
      let condition = self.operand_stack.pop()...  // destroys value
  }
  ```
- **Symptom:** Stack underflow panics, wrong branch taken, values corrupted
- **Location:** `src/jit.rs` translate_instruction() JumpIfFalse/JumpIfTrue handling

(Discovered during: 2026-01-28_15-30_phase7-step6-recursive-jit.md)

### LessEqual/GreaterEqual: DON'T use bnot to invert comparison

- **Problem:** Comparison returns wrong result (e.g., `n <= 1` fails for n=0)
- **Root cause:** `bnot` is BITWISE NOT, not LOGICAL NOT. `bnot(0)` = `-1`, not `1`
- **Rule:** Use correct `IntCC` variant directly. Never try to invert with bnot.
- **Why:** `bnot(x)` inverts ALL bits. For 64-bit: `bnot(0) = 0xFFFFFFFFFFFFFFFF = -1`
- **Fix:**
  ```rust
  // ✓ CORRECT - use proper IntCC
  BinaryOp::LessEqual => builder.ins().icmp(IntCC::SignedLessThanOrEqual, left, right)
  BinaryOp::GreaterEqual => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, left, right)
  
  // ✗ WRONG - bnot doesn't work
  BinaryOp::LessEqual => {
      let gt = builder.ins().icmp(IntCC::SignedGreaterThan, left, right);
      builder.ins().bnot(gt)  // BROKEN: inverts bits, not boolean
  }
  ```
- **Location:** `src/jit.rs` translate_binary_op() comparison handling

(Discovered during: 2026-01-28_15-30_phase7-step6-recursive-jit.md)

### Recursive JIT calls DEADLOCK on mutex

- **Problem:** Program hangs when JIT function calls itself recursively
- **Root cause:** Mutex held during JIT execution blocks recursive `call_function_from_jit`
- **Rule:** ALWAYS drop mutex guard BEFORE executing JIT-compiled code
- **Why:** JIT code calls back to VM (jit_call_function → call_function_from_jit). If VM holds mutex, recursive call blocks forever.
- **Pattern:**
  ```rust
  // ✓ CORRECT - drop guard before execution
  let jit_func: fn(...) -> i64 = {
      let guard = self.jit_compiler.lock().unwrap();
      // get function pointer...
      jit_ptr  // guard drops at end of block
  };
  jit_func(...)  // Call AFTER guard dropped
  
  // ✗ WRONG - guard held during call
  let guard = self.jit_compiler.lock().unwrap();
  let jit_func = guard.get_function_ptr();
  jit_func(...)  // DEADLOCK on recursive call
  ```
- **Symptom:** Program freezes on recursive function call (no error, just hang)
- **Location:** `src/vm.rs` run() JIT execution path

(Discovered during: 2026-01-28_15-30_phase7-step6-recursive-jit.md)

### Hash type mismatch: i64 vs u64 across JIT boundary

- **Problem:** Variable lookup fails with wrong hash value
- **Root cause:** Cranelift uses i64, DefaultHasher returns u64. Sign extension corrupts high bit.
- **Rule:** Use consistent types. Convert u64 to i64 with `as i64` at definition, NOT at use.
- **Why:** Hash `9876543210` fits in u64 but may overflow i64. Different bit patterns → wrong lookup.
- **Fix:**
  ```rust
  // In JIT compilation (Cranelift)
  let hash = compute_hash(var_name) as i64;  // Convert HERE
  let hash_val = builder.ins().iconst(types::I64, hash);
  
  // In runtime lookup
  let hash = hash_param as u64;  // Convert back for HashMap
  var_names.get(&hash)
  ```
- **Symptom:** `None` returned from var_names lookup, "Unknown variable" errors
- **Location:** `src/jit.rs` LoadVar/StoreVar, `src/vm.rs` jit_load_variable

(Discovered during: 2026-01-28_15-30_phase7-step6-recursive-jit.md)

### jit_load_variable must handle function Values

- **Problem:** Function calls fail with "variable not found" or wrong value
- **Root cause:** `jit_load_variable` only handled `Value::Int`, not `Value::Function`
- **Rule:** Return sentinel value (-1) for functions, handle in JIT to trigger interpreter path
- **Why:** Function values can't be encoded as simple i64. Need marker to signal "use runtime call"
- **Pattern:**
  ```rust
  // In jit_load_variable
  match value {
      Value::Int(i) => *i,
      Value::Function(_) => -1,  // Sentinel: "call the function, don't load"
      _ => panic!("Unsupported type"),
  }
  
  // In JIT code generation for Call opcode
  // Check if loaded value is -1, if so, use call mechanism
  ```
- **Symptom:** Recursive calls don't work, function values corrupted
- **Location:** `src/vm.rs` jit_load_variable(), `src/jit.rs` Call opcode

(Discovered during: 2026-01-28_15-30_phase7-step6-recursive-jit.md)

### Backward jumps need special SSA block parameter handling

- **Problem:** Loop back-edges fail with "variable not defined" or wrong values
- **Root cause:** SSA requires all block parameters declared BEFORE any edges created
- **Rule:** For backward jumps: 1) Pre-analyze which vars are live, 2) Declare params at block creation, 3) Pass correct values on each edge
- **Why:** Forward-only translation doesn't know what vars exist when creating loop header block
- **Status:** Currently test `test_compile_simple_loop` is #[ignore] pending fix
- **Approach:**
  ```rust
  // Two-pass approach needed:
  // Pass 1: Scan bytecode, find backward jumps, identify live variables at targets
  // Pass 2: Create blocks with correct parameters, translate with proper phi values
  ```
- **Symptom:** Panic on backward jump, loop variables undefined
- **Location:** `src/jit.rs` Jump handling, needs architectural fix

(Discovered during: 2026-01-28_15-30_phase7-step6-recursive-jit.md)

---

## Async Runtime & Promises (Phase 5)

### Promise.all vs promise_all - Namespace Syntax Limitation

- **Problem:** `await Promise.all(promises)` fails with "Undefined global: Promise"
- **Root cause:** Ruff doesn't have true namespace/object method syntax. `Promise.all` is a single function name string, not a method on Promise class
- **Rule:** Functions with dot notation must have snake_case aliases: both `"Promise.all"` AND `"promise_all"`
- **Solution:** Use `await promise_all(promises)` instead
- **Prevention:** When registering functions with `.` in names, always provide alias
- **Location:** `src/interpreter/mod.rs` function registration

(Discovered during: 2026-01-28_02-50_phase5-async-runtime.md)

### MutexGuard Cannot Cross Await Points

- **Problem:** Compile error "future cannot be sent between threads safely" when holding MutexGuard across `.await`
- **Root cause:** Async futures must be `Send`. `std::sync::MutexGuard` is not `Send` by design (prevents deadlocks)
- **Rule:** Always extract values from mutex guards into local variables BEFORE any `.await`
- **Pattern:**
  ```rust
  let is_cancelled = {
      let guard = is_cancelled.lock().unwrap();
      *guard  // Copy value, guard drops here
  };
  // Now safe to await
  match h.await { ... }
  ```
- **Location:** Common in `src/interpreter/native_functions/async_ops.rs`
- **Prevention:** Never hold mutex locks across async boundaries. Extract → drop → await

(Discovered during: 2026-01-28_02-50_phase5-async-runtime.md)

### New Value Variants Require 4+ Updates

- **Problem:** Non-exhaustive pattern errors in seemingly unrelated files after adding Value enum variant
- **Rule:** When adding new `Value` variant (e.g., TaskHandle), update ALL these locations:
  1. `src/interpreter/value.rs` - enum definition
  2. `src/interpreter/value.rs` - Debug impl (`fmt()` method)
  3. `src/builtins.rs` - `format_debug_value()` match
  4. `src/interpreter/native_functions/type_ops.rs` - `type()` function match
- **Why:** Rust exhaustive matching enforces handling all variants
- **Prevention:** After adding variant, search for `match value {` or `match val {` - compiler will show locations
- **Tip:** Compiler errors are your checklist - each one shows a place to update

(Discovered during: 2026-01-28_02-50_phase5-async-runtime.md)

### Native Functions Need Two-Part Registration

- **Problem:** Function implemented but `Runtime Error: Undefined global: function_name`
- **Root cause:** Native functions must be registered in BOTH static list AND environment initialization
- **Rule:** Add to both places in `src/interpreter/mod.rs`:
  1. `const NATIVE_FUNCTIONS: &[&str]` array (~line 390)
  2. `initialize()` method: `self.env.define("name", Value::NativeFunction("name"))`
- **Why:** VM uses const array for compile-time validation, runtime needs environment bindings
- **Prevention:** Grep for existing similar function to see both registration points
- **Implication:** Missing either causes "undefined" errors even though implementation exists

(Discovered during: 2026-01-28_02-50_phase5-async-runtime.md)

### Tokio Features Are Opt-In

- **Problem:** `tokio::fs` or `tokio::time::timeout` not found despite having tokio dependency
- **Root cause:** Tokio features are opt-in to minimize compile time and binary size
- **Rule:** Enable required features in Cargo.toml:
  ```toml
  tokio = { version = "1", features = ["rt", "rt-multi-thread", "sync", "macros", "time", "io-util", "fs"] }
  ```
- **Why:** Not all projects need all tokio functionality. Selective features reduce bloat.
- **Prevention:** Check tokio documentation for module-to-feature mapping before using types
- **Common mappings:** `tokio::fs` → "fs", `tokio::time` → "time", `tokio::net` → "net"

(Discovered during: 2026-01-28_02-50_phase5-async-runtime.md)

### Array Building with push() in Loops Doesn't Work

- **Problem:** Test hangs indefinitely when building array in loop with `array := array.push(item)`
- **Root cause:** Unclear - possibly push() doesn't return updated array or scoping issue
- **Rule:** AVOID push() in loops. Use literal array construction instead.
- **Pattern:**
  ```ruff
  # DON'T do this:
  promises := []
  for item in items {
      promise := async_func(item)
      promises := promises.push(promise)  # Hangs or wrong result
  }
  
  # DO this instead:
  p1 := async_func(item1)
  p2 := async_func(item2)
  p3 := async_func(item3)
  promises := [p1, p2, p3]
  ```
- **TODO:** Investigate push() behavior - is it mutating or returning new array?
- **Prevention:** For now, build arrays manually with literals when items known at write-time

(Discovered during: 2026-01-28_02-50_phase5-async-runtime.md)

---

*Last updated: 2026-01-28 (Phase 7 Step 6 - Recursive JIT Functions)*
