# CHANGELOG

All notable changes to the Ruff programming language will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Boolean Type as First-Class Value**: Booleans are now proper runtime values
  - Added `Value::Bool(bool)` variant to replace string-based "true"/"false"
  - Added `Expr::Bool(bool)` to AST for boolean literals
  - Lexer tokenizes `true` and `false` as `TokenKind::Bool` instead of identifiers
  - Parser creates `Expr::Bool` for boolean tokens
  - All comparison operators (`==`, `!=`, `<`, `>`, `<=`, `>=`) now return `Value::Bool`
  - Type checker recognizes `TypeAnnotation::Bool` and infers boolean types from comparisons
  - Boolean values work directly in if conditions: `if my_bool { }`
  - Print function correctly displays boolean values as "true" or "false"
  - File I/O functions (`write_file`, `append_file`, `create_dir`, `file_exists`) return proper booleans
  - Backwards compatible: string-based "true"/"false" still work in if conditions
  - 10 comprehensive integration tests covering: literals, comparisons, if conditions, equality, variables, structs, arrays
  - Enhanced `examples/test_bool.ruff` with comprehensive demonstrations
  - Fixed parser bug where `if x {` was incorrectly parsed as struct instantiation
- **File I/O Functions**: Complete filesystem operations support
  - `read_file(path)`: Reads entire file as string
  - `write_file(path, content)`: Writes/overwrites file content
  - `append_file(path, content)`: Appends content to existing file
  - `file_exists(path)`: Checks if file or directory exists
  - `read_lines(path)`: Reads file and returns array of lines
  - `list_dir(path)`: Lists all files in directory
  - `create_dir(path)`: Creates directory with parents (like mkdir -p)
  - All functions return `Value::Error` on failure, caught by try/except
  - 6 comprehensive unit tests for all file operations
  - Fixed `Expr::Tag` evaluation to check for native/user functions before treating as enum constructors
  - Example programs: `file_logger.ruff`, `config_manager.ruff`, `directory_tools.ruff`, `backup_tool.ruff`, `note_taking_app.ruff`
- **User Input Functions**: Added interactive I/O capabilities
  - `input(prompt)`: Reads a line from stdin, displays prompt without newline
  - `parse_int(str)`: Converts string to integer (returns Error on failure)
  - `parse_float(str)`: Converts string to float (returns Error on failure)
  - All functions integrate with try/except error handling
  - Example programs: `interactive_greeting.ruff`, `guessing_game.ruff`, `interactive_calculator.ruff`, `quiz_game.ruff`
- **Lexical Scoping**: Implemented proper lexical scoping with environment stack
  - Variables now correctly update across scope boundaries
  - Accumulator pattern works: `sum := sum + n` in loops
  - Function local variables properly isolated
  - Nested functions can read and modify outer variables
  - For-loop variables don't leak to outer scope
  - `let` keyword creates shadowed variables in current scope
- **Scope Management**: Environment now uses Vec<HashMap> scope stack
  - `push_scope()`/`pop_scope()` for nested contexts
  - Variable lookup walks up scope chain (innermost to outermost)
  - Assignment updates in correct scope or creates in current
- **Comprehensive Tests**: 12 new integration tests for scoping
  - Nested function scopes
  - For-loop variable isolation
  - Variable shadowing with `let`
  - Function modifying outer variables
  - Scope chain lookup
  - Try/except scoping
  - Accumulator patterns
  - Multiple assignments in loops
- **Example File**: `examples/scoping.ruff` demonstrates all scoping features
  - Accumulator pattern (sum in loop)
  - Function counters
  - Variable shadowing
  - Nested functions
  - Loop variable isolation
  - Factorial-like patterns

### Fixed
- **Assignment Operator**: Fixed `:=` to update existing variables instead of always creating new
  - Changed parser to emit `Stmt::Assign` instead of `Stmt::Let` for `:=`
  - `Stmt::Assign` uses `Environment::set()` which updates existing or creates new
  - `let x :=` still creates new variable (shadowing)
  - Fixes critical bug where `sum := sum + n` created new local variable
- **Function Call Cleanup**: Fixed `return_value` not being cleared after function calls
  - Functions now properly clear return state after execution
  - Prevents early termination of parent statement evaluation
  - Allows multiple statements after function calls to execute

### Changed
- **Environment Architecture**: Replaced single HashMap with Vec<HashMap> scope stack
  - Stack index 0 is global scope
  - Higher indices are nested scopes (functions, loops, try/except)
  - All statement handlers updated to use push_scope/pop_scope

## [0.2.0] - 2026-01-22

### Added
- **Field Assignment**: Full support for mutating struct fields with `:=` operator
  - Direct field mutation: `person.age := 26`
  - Nested field mutation: `todos[0].done := true`
  - Works with array indexing and dictionary keys
- **Truthy/Falsy Evaluation**: If conditions now properly handle boolean values and collections
  - Boolean identifiers (`true`/`false`) work in conditionals
  - Strings: "true" → truthy, "false" → falsy, empty → falsy
  - Arrays: empty → falsy, non-empty → truthy
  - Dictionaries: empty → falsy, non-empty → truthy
- **Test Suite**: Added 10 comprehensive integration tests covering:
  - Field assignment for structs and arrays
  - Boolean conditions and truthy values
  - Array and dict operations
  - String concatenation
  - For-in loops
  - Variable assignment behavior
  - Struct field access
- **Example Projects**: Two demonstration projects showcasing language features
  - Todo Manager: struct mutation, arrays, control flow
  - Contact Manager: dictionaries, string functions, error handling
- **Clean Build**: Zero compiler warnings - all infrastructure code properly annotated

### Fixed
- **Variable Assignment**: `:=` operator now consistently creates or updates variables
  - Previously would fail if variable didn't exist in certain contexts
  - Now always inserts/updates in environment
- **Boolean Handling**: Fixed if statements not recognizing boolean struct fields
  - Was only checking numeric values for truthiness
  - Now properly evaluates boolean identifiers and other types
- **Pattern Matching**: Corrected struct pattern matching syntax in field assignment
  - Changed from incorrect `Value::Struct(ref mut fields)` 
  - To correct `Value::Struct { name: _, fields }`

### Changed
- **Documentation**: Clarified that example projects are demonstrations, not interactive applications
- **Build Output**: Added `--quiet` flag recommendation for clean execution output
- **README**: Updated with clearer feature descriptions and usage examples

### Known Limitations (Documented)
- No lexical scoping - uses single global environment
- Variable shadowing in blocks doesn't update outer scope (design limitation)
- Booleans stored as string identifiers internally (architectural choice)
- No user input function yet (`input()` planned for future release)

### Technical Details
- Total tests: 14 (up from 4)
- Compiler warnings: 0 (down from 14)
- Lines of test code added: ~200
- Files modified: interpreter.rs, ast.rs, errors.rs, builtins.rs, module.rs

---

## [0.1.0] - 2026-01-21

### Added
- Initial release of Ruff programming language
- Core language features:
  - Variables and constants
  - Functions with optional type annotations
  - Control flow (if/else, loops, pattern matching)
  - Data types (numbers, strings, enums, arrays, dicts, structs)
  - Struct definitions with methods
  - Type system with inference and checking
  - Module system with imports/exports
  - Error handling (try/except/throw)
- Built-in functions:
  - Math: abs, sqrt, pow, floor, ceil, round, min, max, trig functions
  - Strings: len, to_upper, to_lower, trim, substring, contains, replace
  - Arrays: push, pop, slice, concat
  - Dicts: keys, values, has_key, remove
- Command-line interface with run and test commands
- Comprehensive documentation and examples
