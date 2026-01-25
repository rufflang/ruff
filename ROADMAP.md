# Ruff Language - Development Roadmap

This roadmap outlines planned features and improvements for future versions of the Ruff programming language. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.6.0 (Production Database & Streaming - COMPLETE!)  
> **Next Planned Release**: v0.7.0 (Core Language Completion)  
> **Path to v1.0**: See [PATH_TO_PRODUCTION.md](PATH_TO_PRODUCTION.md) for comprehensive roadmap

---

## Priority Levels

- **P1 (High)**: Core language features needed for real-world applications
- **P2 (Medium)**: Quality-of-life improvements and developer experience
- **P3 (Low)**: Nice-to-have features for advanced use cases

---

## v0.7.0 - Core Language Completion

**Focus**: Complete foundational language features before building developer tooling  
**Timeline**: Q1 2026 (3-4 weeks)  
**Priority**: **P0 - CRITICAL** - These features block everything else

**See**: [CORE_FEATURES_NEEDED.md](CORE_FEATURES_NEEDED.md) for detailed implementation guide

### 10. Timing Functions (P0)

**Status**: ✅ Complete (see CHANGELOG)  
**Completed**: January 25, 2026

~~**Critical Bug Fix**: Fix `current_timestamp()` bug in `examples/projects/ai_model_comparison.ruff`~~ ✅ Fixed

Implemented both timing functions with high-precision support. See CHANGELOG for details.

---

### 🔥 NEXT TO IMPLEMENT (Top Priority)

These are the immediate next items after Array Utilities completion:

#### 16. Assert & Debug (P2)

**Status**: Planned  
**Estimated Effort**: Small (2-3 hours)
**Priority**: MEDIUM - Helpful for testing and debugging

**Features**:
```ruff
# Runtime assertions
assert(x > 0, "x must be positive")
assert_equal(actual, expected)

# Debug output
debug(complex_object)    # Pretty-printed output
```

**Implementation**: Add to `builtins.rs`, throw error on assertion failure

---

**Alternative Next Priority**: Range Function (P2) - Generate number sequences

---

### ✅ COMPLETED FEATURES

#### 14. Array Utilities (P1)

**Status**: ✅ Complete (see CHANGELOG)  
**Completed**: January 25, 2026

Implemented six essential array utility functions: `sort()`, `reverse()`, `unique()`, `sum()`, `any()`, `all()`. Provides comprehensive array manipulation and analysis capabilities for data processing. Works with numbers and strings, supports mixed types, and includes predicate-based filtering. Added 18 comprehensive tests covering all functions, edge cases, and chaining operations. See CHANGELOG for complete feature details and examples.

---

#### 15. File Operations (P1)

**Status**: ✅ Complete (see CHANGELOG)  
**Completed**: January 25, 2026

Implemented four essential file operation functions: `file_size()`, `delete_file()`, `rename_file()`, `copy_file()`. Provides common file manipulation capabilities for practical scripting. Added 9 comprehensive tests including integration tests. See CHANGELOG for complete feature details and examples.

---

#### 14. Type Conversion Functions (P0)

**Status**: ✅ Complete (see CHANGELOG)  
**Completed**: January 25, 2026

Implemented four type conversion functions: `to_int()`, `to_float()`, `to_string()`, `to_bool()`. Handles all major type conversions with intuitive semantics. Added 17 comprehensive tests. See CHANGELOG for complete feature details and examples.

---

#### 13. Type Checker Updates for Int/Float (P0)

**Status**: ✅ Complete (see CHANGELOG)  
**Completed**: January 25, 2026

Implemented complete type promotion system for Int and Float types in the static type checker. Eliminates false warnings when using integers with math functions. Added 12 comprehensive tests. See CHANGELOG for complete feature details and examples.

---

#### 10. Timing Functions (P0)

**Status**: ✅ Complete (see CHANGELOG)  
**Completed**: January 25, 2026

---

#### 11. Integer Type (P0)

**Status**: ✅ Complete (see CHANGELOG)  
**Completed**: January 25, 2026

~~**Current Issue**: All numbers were `f64`, causing precision loss in integer operations~~ ✅ Fixed

Implemented full integer type system with separate `Int(i64)` and `Float(f64)` types. See CHANGELOG for complete feature details and examples.

---

#### 12. Type Introspection (P0)

**Status**: ✅ Complete (see CHANGELOG)  
**Completed**: January 25, 2026

Implemented complete runtime type introspection system with `type()` function and eight type predicate functions (`is_int()`, `is_float()`, `is_string()`, `is_array()`, `is_dict()`, `is_bool()`, `is_null()`, `is_function()`). Enables defensive coding, generic functions, and runtime type validation. See CHANGELOG for complete feature details and examples.

---

### 🔜 PLANNED FEATURES

**Note**: All P0 (Critical) features for v0.7.0 are now complete! 🎉

The features below are planned for future versions.

---

## v1.0.0 - Production Ready

**Focus**: Polish, documentation, community  
**Timeline**: Q4 2026 (3 months)  
**Goal**: Production-ready language competitive with Go/Python

**Milestones**:
- ✅ Complete core features (v0.7.0)
- ✅ 10-20x performance improvement (v0.8.0)
- ✅ World-class tooling (v0.9.0)
- ✅ Comprehensive documentation
- ✅ Production example projects
- ✅ Security audit
- ✅ Package registry with 50+ packages
- ✅ Community of 1000+ users

**See**: [PATH_TO_PRODUCTION.md](PATH_TO_PRODUCTION.md) for complete v1.0 roadmap

---

## Future Versions (v1.1.0+)

### 27. Generic Types (P2)

**Status**: Research Phase  
**Estimated Effort**: Large (4-6 weeks)

**Planned Features**:
```ruff
# Generic functions
func first<T>(arr: Array<T>) -> Option<T> {
    if len(arr) > 0 {
        return Some(arr[0])
    }
    return None
}

# Ge30ric structs
struct Container<T> {
    value: T
    
    func get() -> T {
        return self.value
    }
}

# Type constraints
func process<T: Serializable>(item: T) {
    data := item.serialize()
}
```

---

### 28. Union Types (P2)

**Status**: Research Phase  
**Estimated Effort**: Medium (2-3 weeks)

**Fe31. Advancedres**:
```ruff
# Union type annotations
func process(value: int | string | null) {
    match type_of(value) {
        case "int": print("Number: ${value}")
        case "string": print("Text: ${value}")
        case "null": print("Empty")
    }
}

# Type aliases
type UserID = int
type Handler = func(Request) -> Response
```

---

### 29ures**:
```ruff
# sprintf-style formatting
format("Hello, %s!", ["World"])           # "Hello, World!"
format("Number: %d", [42])                # "Number: 42"
format("Float: %.2f", [3.14159])          # "Float: 3.14"
format("User %s scored %d points", ["Alice", 100])
```

**Implementation**: Add `format()` to `builtins.rs` with pattern matching

---

### 16. Assert & Debug (P2)

**Status**: Planned  
**Estimated Effort**: Small (2-3 hours)

**Features**:
```ruff
# Runtime assertions
assert(x > 0, "x must be positive")
assert_equal(actual, expected)

# Debug output
debug(complex_object)    # Pretty-printed output
```

**Implementation**: Add to `builtins.rs`, throw error on assertion failure

---

### 17. Range Function (P2)

**Status**: Planned  
**Estimated Effort**: Small (2-3 hours)

**Current Issue**: Examples use `range()` but it doesn't exist

**Features**:
```ruff
# Generate array of numbers
range(5)              # [0, 1, 2, 3, 4]
range(2, 8)           # [2, 3, 4, 5, 6, 7]
range(0, 10, 2)       # [0, 2, 4, 6, 8]

# Use in loops
for i in range(10) {
    print(i)
}
```

**Implementation**: Add `range()` to `builtins.rs`, return `Value::Array`

---

## v0.8.0 - Performance & Error Handling

**Focus**: Speed improvements and modern error handling  
**Timeline**: Q2 2026 (2-3 months)  
**See**: [PATH_TO_PRODUCTION.md](PATH_TO_PRODUCTION.md) for details

### 18. Bytecode Compiler & VM (P1)

**Status**: Planned  
**Estimated Effort**: Large (6-8 weeks)

**Goal**: **10-20x performance improvement** over tree-walking interpreter

**Architecture**:
- Compile AST to bytecode instructions
- Stack-based virtual machine
- Register-based optimization passes

**Expected Performance**: Move from ~50-100x slower than Python to competitive speeds

---

### 19. Result & Option Types (P1)

**Status**: Planned  
**Estimated Effort**: Medium (2-3 weeks)

**Features**:
```ruff
# Result type for operations that can fail
func divide(a: float, b: float) -> Result<float, string> {
    if b == 0.0 {
        return Err("Division by zero")
    }
    return Ok(a / b)
}

# Pattern matching on Result
match divide(10, 2) {
    case Ok(value): print("Result: ${value}")
    case Err(error): print("Error: ${error}")
}

# Option type for nullable values
func find_user(id: int) -> Option<User> {
    if exists {
        return Some(user)
    }
    return None
}

# Error propagation with ? operator
func complex_operation() -> Result<Data, Error> {
    data1 := fetch_data()?      # Returns early if Err
    data2 := process(data1)?     # Chains operations
    return Ok(finalize(data2))
}
```

---

### 20. Standard Library Expansion (P1)

**Status**: Planned  
**Estimated Effort**: Large (3 months)

**Core Modules** (First Priority):
- `os` - Operating system interface (getcwd, chdir, mkdir, environ)
- `path` - Path manipulation (join, absolute, exists, is_dir)
- `io` - Buffered I/O and binary operations
- `net` - TCP/UDP sockets beyond HTTP
- `crypto` - Hashing (SHA256, MD5) and encryption (AES)

**See**: [PATH_TO_PRODUCTION.md](PATH_TO_PRODUCTION.md) Section 3.1 for complete module list

---

## v0.9.0 - Developer Experience

**Focus**: World-class tooling for productivity  
**Timeline**: Q3 2026 (3 months)  
**See**: [PATH_TO_PRODUCTION.md](PATH_TO_PRODUCTION.md) Pillar 4

### 21. Language Server Protocol (LSP) (P1)

**Status**: Planned  
**Estimated Effort**: Large (4-6 weeks)

**Features**:
- Autocomplete (built-ins, variables, functions)
- Go to definition
- Find references
- Hover documentation
- Real-time error diagnostics
- Rename refactoring
- Code actions (quick fixes)
- VS Code, IntelliJ, Vim, Emacs support

**Implementation**: Use `tower-lsp` Rust framework

---

### 22. Code Formatter (ruff-fmt) (P1)

**Status**: Planned  
**Estimated Effort**: Medium (2-3 weeks)

**Features**:
```bash
$ ruff fmt myproject/
Formatted 47 files in 1.2s
```

- Opinionated formatting (like gofmt, black, prettier)
- Configurable indentation
- Line length limits
- Import sorting

---

### 23. Linter (ruff-lint) (P1)

**Status**: Planned  
**Estimated Effort**: Medium (3-4 weeks)

**Rules**:
- Unused variables
- Unreachable code
- Type mismatches
- Suspicious comparisons
- Missing error handling
- Auto-fix for simple issues

---

### 24. Package Manager (P1)

**Status**: Planned  
**Estimated Effort**: Large (8-12 weeks)

**Features**:
- `ruff.toml` project configuration
- Dependency management with semver
- Package registry (like npm, crates.io)
- CLI commands: `ruff init`, `ruff add`, `ruff install`, `ruff publish`

---

### 25. Debugger (P2)

**Status**: Planned  
**Estimated Effort**: Medium (3-4 weeks)

**Features**:
```bash
$ ruff debug script.ruff
> break 25        # Set breakpoint
> run            # Start execution
> step           # Step into
> print x        # Inspect variable
> continue       # Continue to next breakpoint
```

---

### 26. Profiler (P2)

**Status**: Planned  
**Estimated Effort**: Medium (2-3 weeks)

**Features**:
```bash
$ ruff profile --cpu myapp.ruff
Top 10 functions by CPU time:
  43.2%  process_data    (1240ms)
  21.5%  http_get        (630ms)
  
$ ruff profile --memory myapp.ruff
Memory allocations:
  12.5 MB  Array allocations
   8.3 MB  Dict allocations
```

---

## v0.9.0+ - Advanced Features

### 13. Advanced Type System (P2)

**Status**: Research Phase  
**Estimated Effort**: Large (2-3 weeks)

**Planned Features**:
- Generic types: `Array<T>`, `Option<T>`, `Result<T, E>`
- Union types: `int | string | null`
- Type aliases: `type UserId = int`
- Null safety with `Option<T>`

---
32
### 14. Macros & Metaprogramming (P3)

**Status**: Research Phase  
**Estimated Effort**: Large (3-4 weeks)

**Description**:  
Compile-time code generation and transformation.

**Planned Features**:
```ruff
# Macro definitions
macro debug_print(expr) {
    print("${expr} = ${eval(expr)}")
}

# Usage
x := 42
debug_print!(x + 10)  # Output: "x + 10 = 52"
```

---

### 15. Foreign Function Interface (FFI) (P3)

**Status**: Research Phase  
**Estimated Effort**: Large (3-4 weeks)

**Description**:  
Call external C libraries and system functions from Ruff.

**Planned Features**:
```ruff
# Load C library
lib := load_library("libmath.so")

# Declare external function
exte33. Additional Compilation Targets (P3)

**Status**: Research Phase  
**Estimated Effort**: Very Large (1-2 months per target)

**Options** (after bytecode VM in v0.8.0):
1. **WebAssembly** - Compile to WASM for browser/embedded use
2. **Native Code** - AOT compilation to native executables via LLVM
3. **JIT Compilation** - Just-in-time compilation for hot paths (100x+ speedup)
**Status**: Research Phase  
**Estimated Effort**: Very Large (2-3 months)

**Description**:  
Automatic memory management with garbage collection or reference counting.

**Planned Features**:
- Automatic garbage collection
- Reference counting for immediate cleanup
- Cycle detection
- Memory profiling tools
- Leak detection and warnings

---

### 17. Graphics & GUI (P3)

**Status**: Research Phase  
**Estimated Effort**: Very Large (2-3 months)

**Description**:  
Graphics and GUI capabilities for visual applications.

**Terminal UI**:
```ruff
import tui

app := tui.App()
window := app.create_window(80, 24)

button := tui.Button { 7.0):
- Timing functions (`current_timestamp`, `performance_now`)
- Type introspection (`type_of`, `is_string`, etc.)
- String formatting (`format` function)
- Array utilities (`sort`, `reverse`, `unique`)

**Medium Complexity** (v0.8.0):
- Result/Option types
- Standard library modules (os, path, io)
- Bytecode instruction design

**Advanced Projects** (v0.9.0+):
- Language Server Protocol (LSP)
- Package manager & registry
- Code formatter and linter
- Debugger implementation
## Version Strategy

**Current Approach**:
- **v0.6.0**: Production database support, HTTP streaming, collections ✅
- **v0.7.0**: Core language completion (foundation features)
- **v0.8.0**: Performance (bytecode, 10x speedup) + error handling
- **v0.9.0**: Developer experience (LSP, package manager, tooling)
- **v1.0.0**: Production-ready, Go/Python competitive 🎉

**Philosophy**: Build the foundation first (language features), then performance, then tooling. This ensures LSP autocomplete and package manager are built on a complete, stable language.

**See Also**:
- [CORE_FEATURES_NEEDED.md](CORE_FEATURES_NEEDED.md) - v0.7.0 implementation guide
- [PATH_TO_PRODUCTION.md](PATH_TO_PRODUCTION.md) - Complete roadmap to world-class language
- [CHANGELOG.md](CHANGELOG.md) - Completed features and release history

---

*Last Updated: January 24(800, 600)
canvas.set_color(255, 0, 0)  # Red
canvas.draw_rect(100, 100, 200, 150)
canvas.draw_circle(400, 300, 50)
canvas.save("output.png")
```

---

### 18. Compilation Targets (P3)

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

## 🤝 Contributing

Want to help implement these features? Check out [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Good First Issues** (v0.6.0):
- Serialization formats (TOML, YAML, CSV)
- Image processing (resize, crop, filters)

**Medium Complexity** (v0.7.0):
- PostgreSQL/MySQL support
- Testing enhancements
- Package manager foundations

**Advanced Projects** (v0.8.0+):
- Concurrency & async
- LSP support
- Advanced type system

---

*Last Updated: January 23, 2026*
