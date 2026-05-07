# Ruff Language Specification

Status: v1.0.0 baseline draft (active, not a release-ready claim)
Spec version: 1.0.0-draft
Last updated: 2026-05-07

## 1. Scope

This document defines the Ruff language and tooling compatibility contract for the active v1.0.0 baseline draft.

This draft status does not imply that Ruff is release-ready; see `ROADMAP.md` for the active 1.0 readiness gate and remaining blockers.

It is normative for:

- source-language syntax and parsing behavior
- runtime/evaluation semantics for core language constructs
- machine-readable contract rules for CLI and LSP output payloads
- breaking-change and versioning policy for language/tooling surfaces

Repository tests and fixtures remain the executable source of truth for implementation behavior, but any behavior that conflicts with this document requires either:

- an implementation fix, or
- a documented spec update with compatibility-policy classification

## 2. File Model

- Ruff source files use the `.ruff` extension.
- Source text is UTF-8.
- Line endings `\n` and `\r\n` are accepted by parser/lexer pathways.
- CLI parse entrypoints reject source files larger than `1,048,576` bytes with a parser diagnostic (`RUFPARSE001`).

## 3. Lexical Model

The lexer tokenizes source into:

- identifiers
- keywords (`func`, `let`, `mut`, `const`, `if`, `else`, `for`, `while`, `loop`, `return`, `break`, `continue`, `async`, `await`, `match`, `case`, `try`, `except`, `throw`, `struct`, `test`, `test_group`, `test_setup`, `test_teardown`)
- literals (numeric, string, boolean, `null`)
- punctuation and operators
- comments (`#`, `//`, `/* ... */`, `///`)

Contextual constructors `Ok`, `Err`, `Some`, and `None` are identifiers in tokenization and parser flow (not lexer keywords).

Lexing failures are reported as structured diagnostics with source location metadata.
Malformed source must not be silently accepted as valid tokens.
Current lexer diagnostics include invalid character, null byte, unterminated string, unterminated block comment, invalid escape, malformed numeric literal, numeric overflow, and identifier/string/numeric token-length limit violations.

## 4. Core Grammar Baseline (v0.14.0)

This section is an EBNF-style baseline for currently supported syntax.

```ebnf
program           = { declaration_or_statement } ;

declaration_or_statement
                  = function_decl
                  | struct_decl
                  | binding_stmt
                  | assignment_stmt
                  | control_stmt
                  | test_decl
                  | expression_stmt ;

function_decl     = [ "async" ] "func" identifier
                    "(" [ parameter_list ] ")"
                    [ "->" type_expr ]
                    block ;

parameter_list    = parameter { "," parameter } ;
parameter         = identifier [ ":" type_expr ] ;

struct_decl       = "struct" identifier "{" { struct_field } "}" ;
struct_field      = identifier [ ":" type_expr ] [ "=" expression ] ;

binding_stmt      = ( "let" | "mut" | "const" ) identifier
                    [ ":" type_expr ] ":=" expression ;

control_stmt      = if_stmt | while_stmt | loop_stmt | for_stmt
                    | return_stmt | break_stmt | continue_stmt
                    | match_stmt | try_except_stmt ;

if_stmt           = "if" expression block [ "else" ( if_stmt | block ) ] ;
while_stmt        = "while" expression block ;
loop_stmt         = "loop" block ;
for_stmt          = "for" identifier "in" expression block ;

match_stmt        = "match" expression "{" { case_clause } "}" ;
case_clause       = "case" pattern [ "if" expression ] block ;

try_except_stmt   = "try" block "except" [ identifier ] block ;

test_decl         = "test" string_literal block
                    | "test_group" string_literal block
                    | "test_setup" block
                    | "test_teardown" block ;

assignment_stmt   = assign_target assignment_op expression ;
assign_target     = identifier | field | index ;
expression_stmt   = expression ;

expression        = pipe_expr ;
assignment_op     = ":=" | "=" | "+=" | "-=" | "*=" | "/=" | "%=" ;

pipe_expr         = null_coalescing { "|>" null_coalescing } ;
null_coalescing   = logical_or { "??" logical_or } ;
logical_or        = logical_and { "||" logical_and } ;
logical_and       = equality { "&&" equality } ;
equality          = comparison { ( "==" | "!=" ) comparison } ;
comparison        = term { ( "<" | "<=" | ">" | ">=" ) term } ;
term              = factor { ( "+" | "-" ) factor } ;
factor            = unary { ( "*" | "/" | "%" ) unary } ;
unary             = ( "!" | "-" | "await" ) unary | postfix ;

postfix           = primary { call | index | field | method_call } ;
call              = "(" [ argument_list ] ")" ;
method_call       = "." identifier "(" [ argument_list ] ")" ;
index             = "[" expression "]" ;
field             = "." identifier ;

argument_list     = expression { "," expression } ;

primary           = literal
                  | identifier
                  | array_literal
                  | dict_literal
                  | function_expr
                  | spawn_expr
                  | "(" expression ")" ;

array_literal     = "[" [ array_elements ] "]" ;
array_elements    = array_element { "," array_element } ;
array_element     = expression | "..." expression ;

dict_literal      = "{" [ dict_elements ] "}" ;
dict_elements     = dict_element { "," dict_element } ;
dict_element      = ( string_literal | identifier ) ":" expression
                  | "..." expression ;

function_expr     = [ "async" ] "func" "(" [ parameter_list ] ")"
                    [ "->" type_expr ] block ;

spawn_expr        = "spawn" block ;

type_expr         = identifier { type_suffix } ;
type_suffix       = "?" | "[]" | "<" type_expr { "," type_expr } ">" ;
```

Notes:

- Spread (`...`) is valid in array/dictionary literal element positions.
- `Ok/Err/Some/None` pattern matching remains contextual and parser-driven.
- Parser safety limits: expression nesting depth is capped at `256` and statement-block nesting depth is capped at `128`. Inputs beyond either limit fail with parser diagnostics instead of recursing indefinitely.
- Assignment operators (`:=`, `=`, `+=`, `-=`, `*=`, `/=`, `%=`) are statement-level only. Chained assignments (for example `a := b := 1`) are rejected with parser diagnostics.

### 4.1 Operator Precedence And Associativity

From highest precedence to lowest:

| Level | Operators | Associativity |
| --- | --- | --- |
| Postfix | `()`, `[]`, `.`, method call `.` + `()` | Left |
| Unary | `!`, unary `-` | Right |
| Multiplicative | `*`, `/`, `%` | Left |
| Additive | `+`, `-` | Left |
| Comparison | `<`, `<=`, `>`, `>=` | Left |
| Equality | `==`, `!=` | Left |
| Logical AND | `&&` | Left |
| Logical OR | `||` | Left |
| Null coalescing | `??` | Left |
| Pipe | `|>` | Left |
| Assignment statements | `:=`, `=`, `+=`, `-=`, `*=`, `/=`, `%=` | Non-associative (chaining rejected) |

## 5. Runtime Semantics Baseline

### 5.1 Bindings and mutability

- `let` introduces immutable bindings.
- `mut` introduces mutable bindings.
- `const` introduces constant bindings that cannot be reassigned.

### 5.2 Scope model

- Top-level script bindings resolve in the global scope.
- Function bodies introduce lexical scope boundaries.
- Shadowing is allowed and resolved by nearest lexical definition.
- Referencing an identifier with no visible binding is a runtime error of the form `Undefined variable: <name>`. Ruff does not convert unknown identifiers into strings; quote string literals explicitly.

### 5.3 Function execution

- Functions support positional parameters.
- Return without explicit value yields `null`.
- `async func` values produce awaitable handles in runtime modes that support async scheduling.

### 5.4 Control flow

- `if`/`else` branches evaluate condition truthiness using runtime truthiness rules.
- `for ... in` iterates over iterable runtime values.
- `break` and `continue` are valid only within loop contexts.

Truthiness rules are centralized across interpreter and VM:

- Falsey values: `false`, `null`, integer `0`, float `0.0`, empty string `""`, empty array `[]`, empty dictionary `{}`.
- Truthy values: every other runtime value, including non-empty strings/arrays/dictionaries, functions, structs/objects, and native handles.
- String values are not parsed as boolean keywords in truthiness checks (`"false"` is a non-empty string and therefore truthy).

Logical operators use these same rules:

- `a && b` short-circuits when `a` is falsey; `b` is evaluated only when `a` is truthy.
- `a || b` short-circuits when `a` is truthy; `b` is evaluated only when `a` is falsey.
- Both operators return boolean results (`true`/`false`), not the original operand values.

### 5.5 Error flow

- `throw(value)` signals runtime exceptions.
- `try`/`except` catches exceptions thrown in protected regions.
- parse/compile/runtime error pathways must produce deterministic message shapes for machine-readable mode.

### 5.6 Data structures

- Arrays preserve insertion order.
- Dictionaries preserve key/value associations; merge/spread behavior is right-biased for duplicate keys.
- Dictionary indexing with a missing key is a runtime error. Programs that need fallback behavior should use explicit dictionary helpers such as `has_key`, `get`, or `get_default`.
- Dictionary indexing accepts string keys and integer keys. Other key types are invalid index operations.
- Array/string indexing outside bounds is a runtime error (`Index out of bounds: <index>`), not a sentinel-value fallback.
- Invalid index assignment targets (for example assigning through index access on non-indexable values) are runtime errors.
- Unsupported unary/binary operations are runtime errors; Ruff does not silently coerce invalid operations to `Int(0)` or empty-string values.
- Struct fields are resolved by declared field names.
- Struct method behavior and runtime-path parity are tracked in `docs/VM_INTERPRETER_PARITY_MATRIX.md`.

### 5.7 Concurrency and await

- `await` blocks expression completion on pending async values.
- `spawn { ... }` schedules detached async work where supported by runtime mode.
- Current VM/interpreter parity and capability notes for `spawn`, spread/destructuring, and match-binding surfaces are tracked in `docs/VM_INTERPRETER_PARITY_MATRIX.md`.

## 6. Tooling Contract Compatibility Guarantees

The following compatibility classes apply to language and tooling behavior:

1. Syntax compatibility
- No previously valid v0.13.0 syntax may become invalid in a patch release.
- Grammar additions must be backward compatible within a minor line unless explicitly documented as breaking.

2. Runtime compatibility
- Existing successful programs should preserve observable behavior across patch releases.
- Error-message wording may be improved, but machine-readable error fields must remain compatible.

3. CLI/LSP machine-readable compatibility
- For documented `--json` or JSON-RPC payloads, field names and top-level shape are stable within a minor line.
- Adding optional fields is non-breaking.
- Removing fields, renaming fields, or changing field types is breaking.

4. Native builtin compatibility
- Existing builtin names and documented argument contracts are stable within a minor line unless a security fix requires immediate breakage.

## 7. Breaking-Change Policy

A change is breaking if it does any of the following:

- makes previously valid syntax invalid
- changes runtime behavior for existing valid programs in non-bug-fix scenarios
- removes or renames machine-readable fields in CLI/LSP payloads
- changes command exit code meaning for established command/result classes
- removes or renames stable builtin APIs

Breaking changes require:

- a minor (or major) version bump consistent with semantic versioning intent
- explicit changelog entry under a compatibility-impact note
- migration guidance in release documentation

Emergency exception rule:

- security and correctness hotfixes may break behavior in patch releases when required to prevent data loss, corruption, or exploitation
- such exceptions must include rationale and follow-up migration guidance in release notes

## 8. Versioning Rules For Language/Tooling Contracts

- Patch (`x.y.Z`): backward-compatible fixes, performance work, diagnostics improvements, additive optional output fields.
- Minor (`x.Y.z`): additive features and planned contract updates; may include documented breaking changes while pre-1.0 if explicitly noted.
- Major (`X.y.z`): post-1.0 contract-reset level change requiring migration treatment.

While Ruff remains pre-1.0, this project still follows strict contract discipline:

- breaking changes are discouraged
- any intentional break must be documented in changelog and roadmap/release evidence
- machine-readable outputs should preserve stability expectations used by automation and IDE integrations

## 9. Specification Governance

- Any change to this document must be reviewed alongside relevant tests/fixtures.
- Spec updates that alter compatibility guarantees require matching changelog notes.
- If implementation and spec disagree, either code or spec must be updated in the same development cycle before release sign-off.
