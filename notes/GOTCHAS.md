# Ruff — Known Gotchas & Sharp Edges

This document contains the most important non-obvious pitfalls in the Ruff codebase.

If you are new to the project, read this first.

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

---

## Runtime / Evaluator

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

---

## Testing

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
