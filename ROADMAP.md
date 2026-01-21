# Ruff Language - Development Roadmap

This roadmap outlines planned features and improvements for the Ruff programming language. Features are organized by priority and include detailed implementation notes.

---

## 🔥 High Priority Features

### 1. Better Error Messages with Line/Column Numbers

**Status**: Not Started  
**Estimated Effort**: Small (2-3 days)  
**Blocking**: None

**Description**:  
Currently, parsing and runtime errors provide minimal context. Users need to see exactly where errors occur with line and column numbers, plus helpful messages.

**Implementation Steps**:
1. Update `Token` struct to track source position (already has line/column fields)
2. Add `SourceLocation` to AST nodes during parsing
3. Create an `Error` type with:
   - Error kind (ParseError, RuntimeError, TypeError)
   - Message
   - File path
   - Line and column numbers
   - Source snippet context (show the problematic line)
4. Modify parser to return `Result<Stmt, ParseError>` instead of `Option<Stmt>`
5. Modify interpreter to track current statement location
6. Add error formatting with colored output (using `colored` crate)

**Example Output**:
```
Error: Undefined variable 'x'
  --> examples/test.ruff:5:10
   |
 5 |     print(x + 1)
   |           ^ variable not defined in current scope
```

**Files to Modify**:
- `src/lexer.rs` - Token already has position info
- `src/parser.rs` - Return Results, propagate errors with context
- `src/interpreter.rs` - Track execution position, format errors
- `src/ast.rs` - Add location info to AST nodes
- `Cargo.toml` - Add dependencies: `colored`, `ariadne` (for pretty error reporting)

**Testing**:
- Add tests for various error types
- Ensure line numbers are accurate
- Test error recovery (don't crash on first error)

---

### 2. Type Annotations and Type Checking

**Status**: Not Started  
**Estimated Effort**: Large (1-2 weeks)  
**Blocking**: Better error messages (recommended)

**Description**:  
Add optional static type annotations and type checking to catch errors before runtime. Types should be optional (gradual typing) to maintain simplicity.

**Type System Features**:
- Primitive types: `int`, `float`, `string`, `bool`
- Function types: `func(int, int) -> int`
- Enum types: Already supported, add type checking
- Generic types: `Array<T>`, `Option<T>` (future)
- Type inference: Infer types from literals and expressions
- Union types: `int | string` for flexible typing

**Syntax Examples**:
```ruff
# Variable annotations
x: int := 5
name: string := "Alice"

# Function signatures
func add(a: int, b: int) -> int {
    return a + b
}

# Type inference
result := add(1, 2)  # Inferred as int

# Optional types work with existing code
func greet(name) {  # No type annotation required
    print("Hello", name)
}
```

**Implementation Steps**:
1. Extend AST to include type annotations
   - Add `TypeAnnotation` enum (Int, Float, String, Bool, Function, Enum, etc.)
   - Update `Let`, `Const`, `FuncDef` to include optional type annotations
2. Extend parser to parse type annotations
   - Parse `: type` after variable names
   - Parse `-> type` for function return types
3. Implement type checker
   - Create `TypeChecker` struct with symbol table
   - Implement type inference for expressions
   - Check assignments match declared types
   - Check function calls match signatures
   - Check return types match declarations
4. Add type checking phase between parsing and interpretation
5. Generate helpful type mismatch errors

**Files to Modify**:
- `src/ast.rs` - Add type annotation types
- `src/parser.rs` - Parse type syntax
- `src/type_checker.rs` - New file for type checking logic
- `src/main.rs` - Add type checking phase
- `src/lexer.rs` - Add type keywords (int, float, string, bool)

**Testing**:
- Test type inference
- Test type errors (mismatches, undefined types)
- Test gradual typing (mixed typed/untyped code)
- Ensure backward compatibility with existing code

---

### 3. Module System and Imports

**Status**: Not Started  
**Estimated Effort**: Medium (1 week)  
**Blocking**: None

**Description**:  
Enable code organization across multiple files with imports and exports.

**Syntax Examples**:
```ruff
# File: math.ruff
export func square(x) {
    return x * x
}

export const PI := 3.14159

func helper() {  # Not exported, private
    return 42
}

# File: main.ruff
import math

result := math.square(5)
print("Square:", result)
print("PI:", math.PI)

# Or import specific items
from math import square, PI
result := square(5)
```

**Implementation Steps**:
1. Add `import` and `export` keywords to lexer
2. Add `Import` and `Export` statement types to AST
3. Implement module resolver
   - Search for `.ruff` files relative to current file
   - Search in standard library path
   - Cache parsed modules to avoid re-parsing
4. Extend interpreter with module system
   - Create `Module` type holding exported symbols
   - Build module dependency graph
   - Detect circular imports
   - Execute modules and collect exports
5. Add namespace support for `module.symbol` access
6. Create standard library directory structure
   - `stdlib/io.ruff` - File operations
   - `stdlib/math.ruff` - Math functions
   - `stdlib/string.ruff` - String utilities

**Files to Modify**:
- `src/lexer.rs` - Add import/export keywords
- `src/ast.rs` - Add Import/Export statement types
- `src/parser.rs` - Parse import/export statements
- `src/module.rs` - New file for module resolution and loading
- `src/interpreter.rs` - Handle module namespaces
- `src/main.rs` - Initialize module system

**Testing**:
- Test basic imports
- Test circular import detection
- Test module namespaces
- Test error handling for missing modules

---

### 4. Standard Library (I/O, Math, Strings)

**Status**: Not Started  
**Estimated Effort**: Medium (1 week)  
**Blocking**: Module system

**Description**:  
Provide essential functionality through a standard library.

**Planned Modules**:

**`stdlib/io.ruff`** - File and console I/O
```ruff
export func read_file(path: string) -> Result<string, string>
export func write_file(path: string, content: string) -> Result<None, string>
export func read_line() -> string
export func file_exists(path: string) -> bool
```

**`stdlib/math.ruff`** - Mathematical operations
```ruff
export func abs(x: float) -> float
export func sqrt(x: float) -> float
export func pow(base: float, exp: float) -> float
export func floor(x: float) -> int
export func ceil(x: float) -> int
export func round(x: float) -> int
export func min(a: float, b: float) -> float
export func max(a: float, b: float) -> float
export const PI := 3.141592653589793
export const E := 2.718281828459045
```

**`stdlib/string.ruff`** - String manipulation
```ruff
export func len(s: string) -> int
export func substring(s: string, start: int, end: int) -> string
export func split(s: string, delimiter: string) -> Array<string>
export func join(arr: Array<string>, separator: string) -> string
export func to_upper(s: string) -> string
export func to_lower(s: string) -> string
export func trim(s: string) -> string
export func contains(s: string, substr: string) -> bool
export func replace(s: string, old: string, new: string) -> string
```

**`stdlib/array.ruff`** - Array operations
```ruff
export func len(arr: Array) -> int
export func push(arr: Array, item) -> None
export func pop(arr: Array) -> item
export func map(arr: Array, func) -> Array
export func filter(arr: Array, func) -> Array
export func reduce(arr: Array, func, initial) -> value
```

**Implementation Steps**:
1. Create `stdlib/` directory
2. Implement built-in functions as native Rust functions
3. Register built-in modules in interpreter initialization
4. Create wrapper functions in `.ruff` files that call native implementations
5. Add documentation for each standard library function

**Files to Create**:
- `stdlib/*.ruff` - Standard library modules
- `src/builtins.rs` - Native function implementations
- `src/interpreter.rs` - Register built-in modules

**Testing**:
- Test each standard library function
- Create example programs using stdlib
- Document all functions with examples

---

## 🎯 Medium Priority Features

### 5. Structs and Methods

**Status**: Not Started  
**Estimated Effort**: Medium (1 week)  
**Blocking**: Type system (recommended)

**Description**:  
Add user-defined structured data types with associated methods.

**Syntax Examples**:
```ruff
struct Point {
    x: float,
    y: float
}

impl Point {
    func new(x: float, y: float) -> Point {
        return Point { x: x, y: y }
    }
    
    func distance(self, other: Point) -> float {
        dx := self.x - other.x
        dy := self.y - other.y
        return sqrt(dx * dx + dy * dy)
    }
}

# Usage
p1 := Point::new(0, 0)
p2 := Point { x: 3, y: 4 }
dist := p1.distance(p2)
print("Distance:", dist)
```

**Implementation Steps**:
1. Add `struct` and `impl` keywords
2. Add `Struct` and `Impl` statement types to AST
3. Add struct construction expressions
4. Add field access expressions (`.` operator)
5. Add method call expressions
6. Implement struct types in type system
7. Store struct definitions and methods in environment
8. Implement `self` parameter in methods

**Files to Modify**:
- `src/lexer.rs` - Add struct/impl keywords
- `src/ast.rs` - Add struct-related AST nodes
- `src/parser.rs` - Parse struct definitions and method calls
- `src/interpreter.rs` - Evaluate struct construction and field access
- `src/type_checker.rs` - Type check structs

---

### 6. Arrays and Dictionaries

**Status**: Not Started  
**Estimated Effort**: Medium (4-5 days)  
**Blocking**: None

**Description**:  
Built-in collection types for storing multiple values.

**Syntax Examples**:
```ruff
# Arrays
numbers := [1, 2, 3, 4, 5]
print(numbers[0])  # Access by index
numbers[2] := 10   # Modify element
numbers.push(6)    # Add element
len := numbers.len()

# Array literals with different types
mixed := [1, "hello", 3.14]

# Dictionaries (hash maps)
person := {
    "name": "Alice",
    "age": 30,
    "city": "Portland"
}
print(person["name"])
person["age"] := 31
person["email"] := "alice@example.com"

# Iteration
for item in numbers {
    print(item)
}

for key, value in person {
    print(key, ":", value)
}
```

**Implementation Steps**:
1. Add `Array` and `Dict` value types
2. Add array/dict literal syntax to parser (`[...]`, `{...}`)
3. Add index access expressions (`arr[i]`, `dict[key]`)
4. Implement array/dict operations in interpreter
5. Add built-in methods (push, pop, len, keys, values)
6. Support for-in loops over collections

**Files to Modify**:
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

| Feature | Priority | Status | Estimated Effort |
|---------|----------|--------|------------------|
| Error Messages | High | Not Started | 2-3 days |
| Type System | High | Not Started | 1-2 weeks |
| Module System | High | Not Started | 1 week |
| Standard Library | High | Not Started | 1 week |
| Structs | Medium | Not Started | 1 week |
| Arrays/Dicts | Medium | Not Started | 4-5 days |
| Loop Control | Medium | Not Started | 2-3 days |
| String Interpolation | Medium | Not Started | 2-3 days |
| Enhanced Comments | Medium | Not Started | 1-2 days |
| REPL | Long Term | Not Started | 3-4 days |
| Package Manager | Long Term | Not Started | 2-3 weeks |
| WASM Target | Long Term | Not Started | 3-4 weeks |
| LSP | Long Term | Not Started | 2-3 weeks |
| JIT/Optimization | Long Term | Not Started | 1-2 months |

**Total Estimated Time**: ~3-4 months for all features

---

## 🎯 Recommended Implementation Order

1. **Better Error Messages** - Foundation for debugging all other features
2. **Arrays and Dictionaries** - Frequently needed, builds on existing code
3. **Loop Control (break/continue)** - Small wins, useful immediately
4. **Module System** - Enables code organization
5. **Standard Library** - Provides essential functionality
6. **Type System** - Major feature, enables many optimizations
7. **Structs and Methods** - Natural extension of type system
8. **String Interpolation** - Quality of life improvement
9. **REPL** - Great for testing and learning
10. **Enhanced Comments** - Documentation support
11. **LSP** - Editor integration
12. **Package Manager** - Ecosystem growth
13. **WASM/JIT** - Performance optimizations

---

*Last Updated: January 21, 2026*