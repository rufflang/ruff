# Ruff Language - Development Roadmap

This roadmap outlines planned features and improvements for the Ruff programming language. Features are organized by priority and include detailed implementation notes.

---

## 🔥 High Priority Features

### 1. Better Error Messages with Line/Column Numbers

**Status**: ✅ Completed (v0.1.0)  
**Estimated Effort**: Small (2-3 days)  
**Blocking**: None

**Description**:  
Error reporting infrastructure with colored output and source location tracking. The interpreter now tracks source files and line numbers, providing context when errors occur.

**Implemented Features**:
- ✅ Source location tracking in Token struct (line/column)
- ✅ SourceLocation type with file, line, and column info
- ✅ RuffError type with structured error kinds
- ✅ Colored error output using the `colored` crate
- ✅ Interpreter tracks source file and content for error reporting
- ✅ Foundation for future parser and runtime error improvements

**Example Usage**:
```ruff
func check(val) {
    if val == 0 {
        throw("Cannot process zero")
    }
    return val * 2
}

try {
    result := check(0)
} except err {
    print("Error:", err)
}
```

**Future Enhancements**:
- Attach line numbers to all AST nodes
- Improve parser errors to use RuffError with locations
- Add "help" hints and suggestions to error messages
- Add error recovery in parser
- Show multiple errors at once instead of stopping at first error

---

### 2. Type Annotations and Type Checking

**Status**: ✅ Completed (v0.1.0)  
**Estimated Effort**: Large (1-2 weeks)  
**Blocking**: Better error messages (recommended)

**Description**:  
Optional static type annotations and type checking to catch errors before runtime. Types are optional (gradual typing) to maintain simplicity and backward compatibility.

**Implemented Features**:
- ✅ Primitive types: `int`, `float`, `string`, `bool`
- ✅ Type annotations for variables (`let x: int := 5`)
- ✅ Type annotations for constants (`const PI: float := 3.14`)
- ✅ Function parameter types (`func add(a: int, b: int)`)
- ✅ Function return types (`func add(...) -> int`)
- ✅ Type inference from literals and expressions
- ✅ Type checking for assignments and function calls
- ✅ Type checking for binary operations
- ✅ Gradual typing - mix typed and untyped code
- ✅ Symbol table for tracking variable and function types
- ✅ Helpful type mismatch error messages

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

**Future Enhancements**:
- Generic types: `Array<T>`, `Option<T>`
- Union types: `int | string`
- Type aliases: `type UserId = int`
- Enum variant types for pattern matching
- Null/Option type safety
- Structural typing for interfaces
- Type narrowing in control flow
- Better type inference across function boundaries

---

### 3. Module System and Imports

**Status**: ✅ Completed (v0.1.0)  
**Estimated Effort**: Medium (1 week)  
**Blocking**: None

**Description**:  
Enable code organization across multiple files with imports and exports.

**Implemented Features**:
- ✅ Import entire modules: `import module_name`
- ✅ Selective imports: `from module_name import symbol1, symbol2`
- ✅ Export declarations: `export fn function_name() { }`
- ✅ Module loading with caching
- ✅ Circular import detection
- ✅ Module search paths (current directory, ./modules)

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
import math_module

result := add(5, 3)
print("5 + 3 =", result)
print("PI:", PI)

# Or import specific items
from math_module import add, PI
result := add(10, 20)
```

**Future Enhancements**:
- Full module execution to populate exports
- Namespace support for qualified access: `math.square(5)`
- Package management system
- Module versioning and dependency resolution
- Better error messages for missing modules/symbols
- Re-export support: `export from other_module`
- Module aliases: `import math as m`
- Private exports (export only to specific modules)

---
### 4. Standard Library (Built-in Functions)

**Status**: ✅ Completed (v0.2.0) - Comprehensive built-ins for math, strings, arrays, and dicts  
**Estimated Effort**: Medium (1 week)  
**Blocking**: Module system

**Description**:  
Provide essential functionality through built-in native functions implemented in Rust for performance.

**Implemented Built-ins**:

**Math Functions** - Available globally
- `abs(x)` - Absolute value
- `sqrt(x)` - Square root
- `pow(base, exp)` - Power function
- `floor(x)`, `ceil(x)`, `round(x)` - Rounding functions
- `min(a, b)`, `max(a, b)` - Min/max values
- `sin(x)`, `cos(x)`, `tan(x)` - Trigonometric functions
- Constants: `PI`, `E`

**String Functions** - Available globally
- `len(s)` - String, array, or dict length
- `to_upper(s)`, `to_lower(s)` - Case conversion
- `trim(s)` - Remove whitespace
- `substring(s, start, end)` - Extract substring
- `contains(s, substr)` - Check if substring exists
- `replace_str(s, old, new)` - Replace substring

**Array Functions** (v0.2.0) - Available globally
- `push(arr, item)` - Add element to end of array, returns new array
- `pop(arr)` - Remove last element, returns `[new_array, popped_value]`
- `slice(arr, start, end)` - Extract subarray from start (inclusive) to end (exclusive)
- `concat(arr1, arr2)` - Combine two arrays into new array
- `len(arr)` - Get number of elements in array

**Dict Functions** (v0.2.0) - Available globally
- `keys(dict)` - Get array of all keys
- `values(dict)` - Get array of all values
- `has_key(dict, key)` - Check if key exists (returns 1/0)
- `remove(dict, key)` - Remove key, returns `[new_dict, removed_value]`
- `len(dict)` - Get number of key-value pairs

**Testing**:
- ✅ All functions tested in examples/builtins.ruff
- ✅ Functions work with arrays, dicts, and strings
- ✅ Proper return values and error handling

**Future Enhancements**:
- Fix type checking for numeric literals in function calls
- Add array manipulation: map, filter, reduce
- Add file I/O: read_file, write_file, file_exists, read_lines
- Add more string functions: starts_with, ends_with, index_of, repeat
- Add JSON parsing and serialization
- Add HTTP client functions
- Add date/time functions
- Create stdlib/ directory with .ruff wrappers for discoverability
- Add system functions: env vars, command execution

---

## 🎯 Medium Priority Features

### 5. Structs and Methods

**Status**: ✅ Completed (v0.2.0) - Full struct support with instantiation, field access, and method calls  
**Estimated Effort**: Medium (1 week)  
**Blocking**: Type system (recommended)

**Description**:  
User-defined data structures with named fields and methods.

**Implemented Features**:
- ✅ `struct`, `impl`, and `self` keywords added to lexer
- ✅ `.` operator for field access
- ✅ StructDef and StructInstance AST nodes
- ✅ Field and method parsing in struct definitions
- ✅ Struct instantiation syntax: `Point { x: 3.0, y: 4.0 }`
- ✅ Field access syntax: `point.x`
- ✅ **Method calls**: `rect.area()`, `point.distance()` (v0.2.0)
- ✅ Methods can access struct fields directly without explicit `self`
- ✅ Type checking for struct definitions and methods
- ✅ Runtime struct values and struct definitions
- ✅ Example files: struct_basic.ruff, struct_methods.ruff, structs_comprehensive.ruff

**Implementation Details** (v0.2.0):
- Special handling in `Expr::Call` for method calls via `FieldAccess`
- Struct fields automatically bound into method execution environment
- Methods work seamlessly with field access: `width * height` directly in method body

**Known Limitations**:
- Methods don't have explicit `self` parameter (fields accessed directly by name)
- No constructor functions or static methods yet
- Struct types not fully integrated into generic type system

**Future Enhancements**:
- Add explicit `self` parameter for more complex method patterns
- Add `self` parameter to methods with proper binding
- Constructor functions or static methods
- Struct inheritance or composition patterns

---

### 6. Arrays and Dictionaries

**Status**: Partially Complete  
**Estimated Effort**: Medium (4-5 days)  
**Blocking**: None

**Description**:  
Built-in collection types for storing multiple values.

**Implemented Features**:
- ✅ Array literal syntax: `[1, 2, 3]`
- ✅ Dictionary literal syntax: `{"key": value}`
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