# Ruff Language - Development Roadmap

This roadmap outlines planned features and improvements for future versions of the Ruff programming language. For completed features in v0.1.0, v0.2.0, and v0.3.0 (in progress), see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.2.0 (January 2026)  
> **Next Planned Release**: v0.3.0

---

## ✅ Recently Completed (v0.3.0 In Progress)

### Lexical Scoping and Block Scope

**Status**: ✅ **COMPLETED** (January 2026)  
**Actual Effort**: 1 day  
**Priority**: Critical - Fixed major limitation

**What Was Fixed**:  
Implemented proper lexical scoping with environment stack. Variables now correctly update across scope boundaries.

**Before (Broken)**:
```ruff
sum := 0
for n in [1, 2, 3] {
    sum := sum + n  # Created NEW local variable (wrong!)
}
print(sum)  # Still 0 - broken!
```

**After (Working)**:
```ruff
sum := 0
for n in [1, 2, 3] {
    sum := sum + n  # Updates outer sum (correct!)
}
print(sum)  # 6 - works!
```

**Implementation Details**:
- Environment now uses Vec<HashMap> scope stack instead of single HashMap
- Variable lookup walks up scope chain from innermost to outermost
- `:=` operator updates existing variables or creates new ones
- `let x :=` always creates new variable (shadowing)
- For-loops, functions, and try/except create new scopes
- Comprehensive test suite with 12 scoping tests
- Example file: `examples/scoping.ruff`

---

## 🔥 High Priority (v0.3.0)

### 1. User Input Function

**Status**: Planned for v0.3.0  
**Estimated Effort**: Small (1-2 days)  
**Priority**: High - Enables interactive programs

**Description**:  
Add `input()` function to read user input from stdin.

**Syntax**:
```ruff
name := input("Enter your name: ")
print("Hello, " + name)

age := input("Enter your age: ")
age_num := parse_int(age)  # Will need parse_int helper
```

**Implementation Steps**:
1. Add native `input(prompt)` function
2. Reads line from stdin
3. Returns string value
4. Add `parse_int()` and `parse_float()` helpers
5. Add examples demonstrating interactive programs

---

### 3. File I/O Functions

**Status**: Planned for v0.3.0  
**Estimated Effort**: Medium (3-4 days)  
**Priority**: High - Essential for practical programs

**Description**:  
Built-in functions for reading and writing files.

**Planned Functions**:
```ruff
# Reading files
content := read_file("data.txt")  # Returns entire file as string
lines := read_lines("data.txt")   # Returns array of lines
exists := file_exists("data.txt") # Returns true/false

# Writing files
write_file("output.txt", "Hello World")     # Write string to file
append_file("log.txt", "New entry\n")      # Append to file

# Directory operations
files := list_dir("./src")        # List files in directory
create_dir("./output")            # Create directory
```

**Implementation Steps**:
1. Add native functions using Rust std::fs
2. Handle errors gracefully (file not found, permission denied)
3. Support relative and absolute paths
4. Add comprehensive examples
5. Document security considerations

---

### 4. Boolean as First-Class Type

**Status**: Planned for v0.3.0  
**Estimated Effort**: Medium (3-4 days)  
**Priority**: Medium - Improves type system

**Description**:  
Make booleans a proper type instead of string identifiers.

**Current Problem**:
- `true` and `false` are identifiers that evaluate to `Value::Str("true")`
- Special handling needed in if conditions
- Type system doesn't recognize bool as distinct type

**Planned Changes**:
- Add `Value::Bool(bool)` variant
- Parser recognizes `true`/`false` as bool literals
- Type system has `TypeAnnotation::Bool`
- Remove special string handling for "true"/"false"

**Benefits**:
- Cleaner implementation
- Proper type checking for booleans
- Better performance (no string comparisons)
- More intuitive semantics

---

## 🎯 Medium Priority (v0.3.x - v0.4.0)

### 5. Loop Control (break, continue, while)

**Status**: Planned  
**Estimated Effort**: Small (2-3 days)

**Description**:  
Add break, continue statements and while loops for better loop control.

**Syntax**:
```ruff
# While loops
x := 0
while x < 10 {
    print(x)
    x := x + 1
}

# Break statement
for i in 100 {
    if i > 10 {
        break
    }
    print(i)
}

# Continue statement
for i in 10 {
    if i % 2 == 0 {
        continue
    }
    print(i)  # Only odd numbers
}
```

---

### 6. String Interpolation

**Status**: Planned  
**Estimated Effort**: Small (2-3 days)

**Description**:  
Embed expressions directly in strings with `${}` syntax.

**Syntax**:
```ruff
name := "World"
x := 42

message := "Hello, ${name}!"  # "Hello, World!"
result := "The answer is ${x * 2}"  # "The answer is 84"
```

---

### 7. Enhanced String Functions

**Status**: Planned  
**Estimated Effort**: Small (2-3 days)

**Planned Functions**:
```ruff
starts_with("hello world", "hello")  # true
ends_with("test.ruff", ".ruff")      # true
index_of("hello", "ll")              # 2
repeat("ha", 3)                      # "hahaha"
split("a,b,c", ",")                  # ["a", "b", "c"]
join(["a", "b", "c"], ",")           # "a,b,c"
```

---

### 8. Array Higher-Order Functions

**Status**: Planned  
**Estimated Effort**: Medium (3-4 days)

**Description**:  
Functional programming operations on arrays.

**Planned Functions**:
```ruff
# Map - transform each element
squared := map([1, 2, 3], func(x) { return x * x })  # [1, 4, 9]

# Filter - select elements
evens := filter([1, 2, 3, 4], func(x) { return x % 2 == 0 })  # [2, 4]

# Reduce - accumulate
sum := reduce([1, 2, 3], 0, func(acc, x) { return acc + x })  # 6

# Find - first matching element
first_even := find([1, 2, 3, 4], func(x) { return x % 2 == 0 })  # 2
```

---

### 9. Multi-Line and Doc Comments

**Status**: Planned  
**Estimated Effort**: Small (1-2 days)

**Syntax**:
```ruff
# Single-line comment (already supported)

/*
 * Multi-line comment
 * Spans multiple lines
 */

/// Documentation comment for functions
/// @param x The input value
/// @return The squared value
func square(x) {
    return x * x
}
```

---

### 10. JSON Support

**Status**: Planned  
**Estimated Effort**: Medium (3-4 days)

**Planned Functions**:
```ruff
# Parse JSON string to Ruff value
data := parse_json('{"name": "Alice", "age": 30}')
print(data["name"])  # Alice

# Convert Ruff value to JSON string
person := {"name": "Bob", "score": 95}
json_str := to_json(person)  # '{"name":"Bob","score":95}'
```

---

## 🚀 Long Term (v0.5.0+)

### 11. REPL (Interactive Shell)

**Status**: Planned  
**Estimated Effort**: Medium (3-4 days)

**Features**:
- Interactive Read-Eval-Print Loop
- Multi-line input support
- Command history (up/down arrows)
- Tab completion
- Special commands (`:help`, `:clear`, `:quit`)

---

### 12. Advanced Type System Features

**Status**: Research Phase  
**Estimated Effort**: Large (2-3 weeks)

**Planned Features**:
- Generic types: `Array<T>`, `Option<T>`, `Result<T, E>`
- Union types: `int | string | null`
- Type aliases: `type UserId = int`
- Optional chaining: `user?.profile?.email`
- Null safety with `Option<T>`

---

### 13. LSP (Language Server Protocol)

**Status**: Planned  
**Estimated Effort**: Large (2-3 weeks)

**Features**:
- Syntax highlighting
- Autocomplete
- Go to definition
- Find references
- Hover information
- Real-time error checking
- VS Code extension

---

### 14. Package Manager

**Status**: Planned  
**Estimated Effort**: Large (2-3 weeks)

**Features**:
- `ruff.toml` project configuration
- Dependency management
- Package registry
- Semantic versioning
- CLI commands: `ruff init`, `ruff add`, `ruff install`

---

### 15. Compilation Targets

**Status**: Research Phase  
**Estimated Effort**: Very Large (1-2 months)

**Options**:
1. **Bytecode Interpreter** - Compile AST to bytecode for faster execution
2. **WebAssembly** - Compile to WASM for browser/embedded use
3. **Native Code** - AOT compilation to native executables
4. **JIT Compilation** - Just-in-time compilation for hot paths

---

## 📋 Implementation Guidelines

### For Each New Feature:

1. **Plan** - Write specification with examples
2. **Test** - Create test cases before implementation
3. **Implement** - Update lexer, parser, AST, interpreter as needed
4. **Validate** - Ensure all tests pass, zero warnings
5. **Document** - Add examples and update README
6. **Release** - Update CHANGELOG with feature

### Code Quality Standards:

- ✅ Zero compiler warnings
- ✅ All tests must pass
- ✅ Document public APIs
- ✅ Add examples for new features
- ✅ Follow existing code style
- ✅ Update CHANGELOG and README

---

## 📊 Progress Tracking

| Feature | Priority | Target Version | Status |
|---------|----------|----------------|--------|
| Lexical Scoping | Critical | v0.3.0 | Planned |
| User Input | High | v0.3.0 | Planned |
| File I/O | High | v0.3.0 | Planned |
| Boolean Type | Medium | v0.3.0 | Planned |
| Loop Control | Medium | v0.3.x | Planned |
| String Interpolation | Medium | v0.3.x | Planned |
| Enhanced Strings | Medium | v0.4.0 | Planned |
| Array Higher-Order Fns | Medium | v0.4.0 | Planned |
| Multi-line Comments | Low | v0.4.0 | Planned |
| JSON Support | Medium | v0.4.0 | Planned |
| REPL | Long Term | v0.5.0 | Planned |
| Advanced Types | Long Term | v0.6.0 | Research |
| LSP | Long Term | v0.6.0 | Planned |
| Package Manager | Long Term | v0.7.0 | Planned |
| Compilation | Long Term | v0.8.0+ | Research |

---

## 🎯 Version Milestones

**v0.3.0 - "Practical"** (Target: Q1 2026)
- Lexical scoping fix
- User input
- File I/O
- Boolean type improvements

**v0.4.0 - "Expressive"** (Target: Q2 2026)
- String interpolation
- Array higher-order functions
- JSON support
- Enhanced string functions

**v0.5.0 - "Interactive"** (Target: Q3 2026)
- REPL implementation
- Improved error messages
- Better debugging tools

**v0.6.0 - "Professional"** (Target: Q4 2026)
- LSP support
- Advanced type system
- Comprehensive standard library

**v0.7.0 - "Ecosystem"** (2027)
- Package manager
- Community packages
- Documentation generator

---

## 🤝 Contributing

Want to help implement these features? Check out [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Priority features for community contributions:
- Enhanced string functions
- Array higher-order functions
- Multi-line comments
- JSON support

---

*Last Updated: January 22, 2026*
### 6. Arrays and Dictionaries (Hash Maps)

**Status**: ✅ Completed (v0.2.0) - Full collection support with element assignment, iteration, and built-in methods  
**Estimated Effort**: Medium (1 week)  
**Blocking**: None

**Description**:  
First-class support for arrays and hash maps (dictionaries) as fundamental collection types.

**Implemented Features**:
- ✅ Array literals: `[1, 2, 3]`
- ✅ Dict literals: `{"key": value}`
- ✅ Index access for arrays, dicts, and strings: `arr[0]`, `dict["key"]`, `str[i]`
- ✅ Nested arrays and dictionaries
- ✅ Mixed-type collections
- ✅ Value::Array and Value::Dict types
- ✅ String formatting/display for collections
- ✅ **Element assignment**: `arr[0] := 10`, `dict["key"] := value` (v0.2.0)
- ✅ **For-in loop iteration**: `for item in arr`, `for key in dict` (v0.2.0)
- ✅ **Built-in array methods**: `push()`, `pop()`, `slice()`, `concat()`, `len()` (v0.2.0)
- ✅ **Built-in dict methods**: `keys()`, `values()`, `has_key()`, `remove()`, `len()` (v0.2.0)

**Syntax Examples**:
```ruff
# Arrays
numbers := [1, 2, 3, 4, 5]
print(numbers[0])  # Access by index: 1
numbers[2] := 10   # ✅ Modify element (v0.2.0)
numbers := push(numbers, 6)  # ✅ Add element (v0.2.0)
len := len(numbers)  # ✅ Get length: 6 (v0.2.0)

# Array manipulation
sub := slice(numbers, 1, 4)  # Extract [2, 10, 4]
combined := concat([1, 2], [3, 4])  # [1, 2, 3, 4]
result := pop(numbers)  # Returns [new_array, popped_value]
numbers := result[0]
last := result[1]

# Array literals with different types
mixed := [1, "hello", 3.14]

# Nested arrays
matrix := [[1, 2], [3, 4], [5, 6]]
print(matrix[0])      # [1, 2]
print(matrix[0][0])   # 1
matrix[0][1] := 99    # ✅ Modify nested element (v0.2.0)

# Dictionaries (hash maps)
person := {
    "name": "Alice",
    "age": 30,
    "city": "Portland"
}
print(person["name"])  # Alice
print(person["age"])   # 30
person["age"] := 31  # ✅ Modify value (v0.2.0)
person["email"] := "alice@example.com"  # ✅ Add key (v0.2.0)

# Dict methods
dict_keys := keys(person)  # ["name", "age", "city", "email"]
dict_vals := values(person)  # ["Alice", 31, "Portland", "alice@example.com"]
has_email := has_key(person, "email")  # 1
result := remove(person, "city")  # [new_dict, "Portland"]
person := result[0]

# Nested dictionaries
user := {
    "username": "alice123",
    "profile": {
        "email": "alice@example.com",
        "verified": true
    }
}
print(user["profile"]["email"])  # alice@example.com

# Combined: array of dictionaries
users := [
    {"name": "Alice", "score": 95},
    {"name": "Bob", "score": 87}
]
print(users[0]["name"])  # Alice

# Dictionary with array values
scores := {
    "math": [95, 87, 92],
    "english": [88, 91, 85]
}
print(scores["math"][0])  # 95

# String indexing
str := "hello"
print(str[0])  # h

# ✅ Iteration (v0.2.0)
for item in numbers {
    print(item)  # Iterate over array elements
}

for key in person {
    print(key)  # Iterate over dict keys
}

for char in "Ruff" {
    print(char)  # Iterate over string characters
}

for i in 5 {
    print(i)  # Range iteration: 0, 1, 2, 3, 4
}
```

**Parser Fix Applied** (v0.2.0):
- Fixed critical bug where `for i in arr { }` was parsing `arr {` as struct instantiation
- Solution: Changed parse_for to use `parse_primary()` instead of `parse_expr()` to get just the identifier without postfix operations
- This prevents the parser from seeing Identifier + `{` and treating it as a struct instantiation

**Known Limitations**:
1. No nested index assignment yet: `arr[0][1] := x` not supported (only direct identifiers: `arr[i] := x`)
2. For-dict iteration only over keys, not key-value pairs simultaneously
3. Index out of bounds returns 0 instead of proper error (see Feature #9 for improvement)
4. No array slicing syntax: `arr[1:3]` not supported (use `slice()` function)
5. No spread operator or array concatenation syntax
6. Type system doesn't support generic types yet: no `Array<T>` or `Dict<K,V>` (see Feature #8)
7. Variable shadowing in loops prevents accumulator patterns (e.g., sum calculation resets to 0)

**Implementation Completed** (v0.2.0):
1. ✅ AST: Modified Stmt::Assign to accept Expr target (was String name)
2. ✅ Parser: Extended assignment parsing to handle IndexAccess as lvalue
3. ✅ Interpreter: Implemented mutable updates for array/dict elements
4. ✅ Type Checker: Updated pattern matching for new Assign structure
5. ✅ Parser: Fixed for-loop parsing ambiguity (struct instantiation vs iteration)
6. ✅ Interpreter: Extended for-loop to handle Array, Dict, String, Range iteration
7. ✅ Built-ins: Implemented and registered 9 collection functions
8. ✅ Testing: Created examples/builtins.ruff, examples/for_loops.ruff, verified all features

**Files Modified**:
- `src/ast.rs` - Changed Assign statement structure
- `src/parser.rs` - Extended assignment parsing, fixed for-loop parsing
- `src/interpreter.rs` - Indexed assignment, for-in iteration, 9 new built-in functions
- `src/type_checker.rs` - Updated Assign pattern matching
- `examples/builtins.ruff` - Comprehensive built-in function tests
- `examples/for_loops.ruff` - Iteration demonstrations
- `src/ast.rs` - Add array/dict literal expressions
- `src/lexer.rs` - Handle `[`, `]`, `{`, `}` for literals
- `src/parser.rs` - Parse array/dict syntax
- `src/interpreter.rs` - Add Array/Dict value types and operations

---

### 7. More Loop Constructs (break, continue, while)

**Status**: Not Started  
**Estimated Effort**: Small (2-3 days)  
**Blocking**: None

**Description**:  
Enhance loop control with break, continue, and while loops.

**Syntax Examples**:
```ruff
# While loops
x := 0
while x < 10 {
    print(x)
    x := x + 1
}

# Break statement
for i in range(100) {
    if i > 10 {
        break
    }
    print(i)
}

# Continue statement
for i in range(10) {
    if i % 2 == 0 {
        continue
    }
    print(i)  # Only odd numbers
}
```

**Implementation Steps**:
1. Add `break`, `continue`, `while` keywords
2. Add `Break` and `Continue` statement types
3. Update `Loop` statement to support while conditions
4. Add loop control flow handling in interpreter
5. Use special return values or flags to signal break/continue

**Files to Modify**:
- `src/lexer.rs` - Add break/continue/while keywords
- `src/ast.rs` - Add Break/Continue statements
- `src/parser.rs` - Parse loop control statements
- `src/interpreter.rs` - Handle break/continue flow

---

### 8. String Interpolation

**Status**: Not Started  
**Estimated Effort**: Small (2-3 days)  
**Blocking**: None

**Description**:  
Allow embedding expressions directly in strings.

**Syntax Examples**:
```ruff
name := "World"
x := 42

# String interpolation with ${}
message := "Hello, ${name}!"
print(message)  # "Hello, World!"

# Expressions in interpolation
result := "The answer is ${x * 2}"
print(result)  # "The answer is 84"

# Multiple interpolations
info := "${name} has ${x} items"
```

**Implementation Steps**:
1. Extend lexer to parse interpolated strings
2. Parse `${}` expressions inside string literals
3. Convert interpolated strings to string concatenation in AST
4. Ensure expressions inside `${}` are fully parsed

**Files to Modify**:
- `src/lexer.rs` - Parse interpolated strings
- `src/parser.rs` - Convert to concatenation expressions
- `src/ast.rs` - May need intermediate representation

---

### 9. Enhanced Comments

**Status**: Not Started  
**Estimated Effort**: Small (1-2 days)  
**Blocking**: None

**Description**:  
Support multi-line comments and doc comments.

**Syntax Examples**:
```ruff
# Single-line comment (already supported)

/*
 * Multi-line comment
 * Spans multiple lines
 */

/// Documentation comment for functions
/// @param x The input value
/// @return The squared value
func square(x) {
    return x * x
}
```

**Implementation Steps**:
1. Update lexer to handle `/*` and `*/` for multi-line comments
2. Add `///` doc comment support
3. Store doc comments in AST for later documentation generation

**Files to Modify**:
- `src/lexer.rs` - Parse multi-line and doc comments

---

## 🚀 Long Term Features

### 10. REPL Implementation

**Status**: Not Started  
**Estimated Effort**: Medium (3-4 days)  
**Blocking**: Better error messages

**Description**:  
Interactive Read-Eval-Print Loop for experimenting with Ruff code.

**Features**:
- Read input line by line
- Support multi-line input for functions/blocks
- Show results immediately
- Maintain state between commands
- Command history (up/down arrows)
- Tab completion for variables/functions
- Special commands (`:help`, `:clear`, `:quit`, `:load file.ruff`)

**Implementation Steps**:
1. Add `rustyline` crate for readline functionality
2. Create REPL loop that reads, parses, and evaluates
3. Maintain persistent interpreter state
4. Add special REPL commands
5. Handle incomplete input (multi-line)
6. Add syntax highlighting (optional)

**Files to Modify**:
- `src/main.rs` - Implement REPL command
- `src/repl.rs` - New file for REPL logic
- `Cargo.toml` - Add rustyline dependency

---

### 11. Package Manager

**Status**: Not Started  
**Estimated Effort**: Large (2-3 weeks)  
**Blocking**: Module system, standard library

**Description**:  
Tool for managing Ruff project dependencies and packages.

**Features**:
- `ruff.toml` configuration file
- Package registry (local or remote)
- Dependency resolution
- Semantic versioning
- Package installation and updates
- Package creation and publishing

**Commands**:
```bash
ruff init          # Create new project
ruff add <pkg>     # Add dependency
ruff remove <pkg>  # Remove dependency
ruff install       # Install dependencies
ruff build         # Build project
ruff publish       # Publish to registry
```

**Implementation Steps**:
1. Design package manifest format (`ruff.toml`)
2. Implement package resolver
3. Create local package cache
4. Implement dependency installation
5. Build package registry infrastructure
6. Add CLI commands for package management

---

### 12. WebAssembly Compilation Target

**Status**: Not Started  
**Estimated Effort**: Large (3-4 weeks)  
**Blocking**: Type system (strongly recommended)

**Description**:  
Compile Ruff code to WebAssembly for browser and embedded use.

**Features**:
- Compile `.ruff` files to `.wasm`
- JavaScript interop
- Browser compatibility
- Optimize for size and speed

**Implementation Steps**:
1. Research WASM code generation
2. Implement bytecode intermediate representation
3. Create WASM code generator
4. Handle memory management for WASM
5. Create JS bindings for interop
6. Optimize generated WASM

---

### 13. LSP (Language Server Protocol)

**Status**: Not Started  
**Estimated Effort**: Large (2-3 weeks)  
**Blocking**: Type system, better error messages

**Description**:  
Language server for IDE integration with syntax highlighting, autocomplete, and go-to-definition.

**Features**:
- Syntax highlighting
- Autocomplete (variables, functions, imports)
- Go to definition
- Find references
- Hover information (types, docs)
- Real-time error checking
- Code formatting
- Refactoring support

**Implementation Steps**:
1. Use `tower-lsp` crate for LSP implementation
2. Implement textDocument/didOpen, didChange
3. Implement completion provider
4. Implement hover provider
5. Implement goto definition
6. Add diagnostic reporting
7. Create VS Code extension
8. Test with multiple editors

---

### 14. Optimizing Interpreter / JIT Compiler

**Status**: Not Started  
**Estimated Effort**: Very Large (1-2 months)  
**Blocking**: Complete language feature set

**Description**:  
Improve runtime performance through optimization or just-in-time compilation.

**Options**:
1. **Bytecode Interpreter**: Compile AST to bytecode, interpret bytecode
2. **JIT Compilation**: Compile hot paths to native code
3. **AOT Compilation**: Compile entire program to native executable

**Implementation Steps** (Bytecode Interpreter):
1. Design bytecode instruction set
2. Implement bytecode compiler (AST → bytecode)
3. Implement bytecode VM
4. Optimize bytecode execution
5. Add bytecode caching

**Implementation Steps** (JIT):
1. Implement profiler to find hot paths
2. Use LLVM or Cranelift for code generation
3. Generate machine code for hot functions
4. Implement tiered compilation (interpreter → JIT)
5. Manage compiled code cache

---

## 📋 Implementation Guidelines

### Getting Started with Each Feature

1. **Read the specification** - Understand what you're building
2. **Write tests first** - Create test cases for the feature
3. **Update AST** - Add necessary AST nodes
4. **Update lexer** - Add new tokens if needed
5. **Update parser** - Parse the new syntax
6. **Update interpreter** - Implement the runtime behavior
7. **Test thoroughly** - Ensure all tests pass
8. **Document** - Add examples and documentation
9. **Update README** - Mark feature as complete

### Code Quality Standards

- Zero compiler warnings
- All tests must pass
- Document all public APIs
- Add inline comments for complex logic
- Follow existing code style
- Update test suite for new features

---

## 📊 Progress Tracking

| Feature | Priority | Status | Completed |
|---------|----------|--------|-----------|
| Error Messages | High | ✅ Complete | v0.1.0 |
| Type System | High | ✅ Complete | v0.1.0 |
| Module System | High | ✅ Complete | v0.1.0 |
| Standard Library | High | ✅ Complete | v0.2.0 |
| Structs & Methods | Medium | ✅ Complete | v0.2.0 |
| Arrays/Dicts | Medium | ✅ Complete | v0.2.0 |
| Loop Control | Medium | Not Started | - |
| String Interpolation | Medium | Not Started | - |
| Enhanced Comments | Medium | Not Started | - |
| REPL | Long Term | Not Started | - |
| Package Manager | Long Term | Not Started | - |
| WASM Target | Long Term | Not Started | - |
| LSP | Long Term | Not Started | - |
| JIT/Optimization | Long Term | Not Started | - |

**Completed Features**: 6/14  
**Remaining Estimated Time**: ~2-3 months for remaining features

---

## 🎯 Recommended Implementation Order

**✅ Completed (v0.1.0 - v0.2.0)**:
1. ✅ Better Error Messages - Foundation for debugging
2. ✅ Type System - Static type checking and inference
3. ✅ Module System - Code organization and imports
4. ✅ Standard Library - Built-in functions for math, strings, arrays, dicts
5. ✅ Arrays and Dictionaries - Full collection support with iteration
6. ✅ Structs and Methods - User-defined types with behavior

**Remaining Priority Order**:
7. **Loop Control (break/continue)** - Small wins, useful immediately
8. **String Interpolation** - Quality of life improvement
9. **Enhanced Comments** - Documentation support
10. **REPL** - Great for testing and learning
11. **LSP** - Editor integration
12. **Package Manager** - Ecosystem growth
13. **WASM/JIT** - Performance optimizations

---

*Last Updated: January 21, 2026*