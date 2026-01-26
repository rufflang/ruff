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

---

## CLI & Arguments

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

1. Add variant to `Value` enum in `src/interpreter.rs`
2. Update `Debug` impl for Value (same file, ~line 328)
3. Update `format_debug_value()` in `src/builtins.rs` (~line 1550)
4. Update `type()` function match in `src/interpreter.rs` (~line 2800)
5. If user-creatable, add constructor function
6. If stateful resource, use `Arc<Mutex<T>>` pattern
7. Test with `cargo build` to catch any missed matches

