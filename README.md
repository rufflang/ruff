# 🐾 Ruff Programming Language

**Ruff** is a lean, expressive programming language built from scratch in Rust. It borrows inspiration from Go, Python, and functional design — but stands on its own.

> **Status**: v0.4.0 (in development) - Standard library enhancements with random, date/time, system operations, and path utilities! Production-ready with comprehensive features including structs, methods, collections, type checking, and modules.

---

## 🎯 Project Status

### ✅ Implemented Features (v0.4.0)

* **Variables & Constants**
  - `let` and `mut` for mutable variables
  - `const` for constants
  - Shorthand assignment with `:=` (e.g., `x := 5`)
  - Optional type annotations: `x: int := 5`
  - **NEW**: `:=` now properly updates existing variables across scopes

* **Lexical Scoping** (v0.3.0) ✨
  - Proper scope chain with environment stack
  - Variables update correctly across scope boundaries
  - Accumulator pattern works: `sum := sum + n` in loops
  - Function local variables properly isolated
  - Nested functions can read and modify outer variables
  - For-loop variables don't leak to outer scope
  - Variable shadowing with `let` keyword

* **Functions**
  - Function definitions with `func` keyword
  - Parameter passing with optional type annotations
  - Return values with optional return type annotations
  - Lexical scoping with access to outer variables
  - Functions as first-class values
  - Nested function definitions

* **Control Flow**
  - `if`/`else` statements
  - Pattern matching with `match`/`case`
  - `loop` and `for` loops
  - **NEW**: `while` loops (v0.3.0)
  - **NEW**: `break` and `continue` statements (v0.3.0)
  - For-in iteration over arrays, dicts, strings, and ranges
  - `try`/`except`/`throw` error handling
  - **NEW**: Enhanced error handling (v0.4.0) with error properties, custom error types, and stack traces

* **Data Types**
  - Numbers (f64)
  - Strings with escape sequences
  - **NEW**: String interpolation with `${}` (v0.3.0): `"Hello, ${name}!"`
  - Booleans: `true`, `false` (v0.3.0)
  - Enums with tagged variants
  - Arrays: `[1, 2, 3]`
  - Dictionaries: `{"key": value}`
  - Structs with fields and methods
  - Functions as first-class values

* **Collections** (v0.2.0)
  - Array literals and nested arrays
  - Dictionary (hash map) literals
  - Index access: `arr[0]`, `dict["key"]`
  - Element assignment: `arr[0] := 10`, `dict["key"] := value`
  - For-in iteration: `for item in array { }`, `for key in dict { }`
  - Built-in methods: `push()`, `pop()`, `slice()`, `concat()`, `keys()`, `values()`, `has_key()`, `remove()`
  - `len()` function for strings, arrays, and dicts

* **Array Higher-Order Functions** (v0.3.0) ✨
  - Functional programming operations for data transformation
  - `map(array, func)`: Transform each element
    ```ruff
    squared := map([1, 2, 3], func(x) { return x * x })  # [1, 4, 9]
    ```
  - `filter(array, func)`: Select elements matching condition
    ```ruff
    evens := filter([1, 2, 3, 4], func(x) { return x % 2 == 0 })  # [2, 4]
    ```
  - `reduce(array, initial, func)`: Accumulate into single value
    ```ruff
    sum := reduce([1, 2, 3, 4], 0, func(acc, x) { return acc + x })  # 10
    ```
  - `find(array, func)`: Get first matching element
    ```ruff
    found := find([10, 20, 30], func(x) { return x > 15 })  # 20
    ```
  - Chainable for complex data processing
  - Anonymous function expressions: `func(x) { return x * 2 }`

* **Structs & Methods** (v0.2.0)
  - Struct definitions with typed fields
  - Struct instantiation: `Point { x: 3.0, y: 4.0 }`
  - Field access: `point.x`
  - Method calls: `rect.area()`, `point.distance()`
  - **Self parameter** (v0.5.0): Explicit `self` for method composition
    ```ruff
    struct Calculator {
        base: float,
        
        func add(self, x) {
            return self.base + x;
        }
        
        func chain(self, x) {
            return self.add(x) * 2.0;  # Call other methods
        }
    }
    
    calc := Calculator { base: 10.0 };
    result := calc.chain(5.0);  # 30.0
    ```
  - Builder patterns and fluent interfaces
  - Backward compatible - methods without `self` still work

* **Type System** (v0.1.0)
  - Optional type annotations
  - Type inference
  - Type checking for assignments and function calls
  - Gradual typing - mix typed and untyped code
  - Helpful type mismatch error messages

* **Module System** (v0.1.0)
  - Import entire modules: `import module_name`
  - Selective imports: `from module_name import func1, func2`
  - Export declarations: `export func function_name() { }`
  - Module caching and circular import detection

* **Built-in Functions**
  - **Math**: `abs()`, `sqrt()`, `pow()`, `floor()`, `ceil()`, `round()`, `min()`, `max()`, `sin()`, `cos()`, `tan()`, constants `PI` and `E`
  - **Random** (v0.4.0): `random()`, `random_int(min, max)`, `random_choice(array)` - Random number generation
  - **Strings**: `len()`, `to_upper()`, `to_lower()`, `trim()`, `substring()`, `contains()`, `replace_str()`, `starts_with()`, `ends_with()`, `index_of()`, `repeat()`, `split()`, `join()`
  - **Regex** (v0.4.0): `regex_match()`, `regex_find_all()`, `regex_replace()`, `regex_split()` - Pattern matching and text processing
  - **Arrays**: `push()`, `pop()`, `slice()`, `concat()`, `len()`
  - **Array Higher-Order**: `map()`, `filter()`, `reduce()`, `find()` (v0.3.0)
  - **Dicts**: `keys()`, `values()`, `has_key()`, `remove()`, `len()`
  - **JSON**: `parse_json()`, `to_json()` - Parse and serialize JSON data (v0.3.0)
  - **Date/Time** (v0.4.0): `now()`, `format_date()`, `parse_date()` - Timestamp and date operations
  - **System** (v0.4.0): `env()`, `args()`, `exit()`, `sleep()`, `execute()` - System operations
  - **Paths** (v0.4.0): `join_path()`, `dirname()`, `basename()`, `path_exists()` - Path manipulation
  - **I/O**: `print()`, `input()`
  - **Type Conversion**: `parse_int()`, `parse_float()`
  - **File I/O**: `read_file()`, `write_file()`, `append_file()`, `file_exists()`, `read_lines()`, `list_dir()`, `create_dir()`
  - **Error handling**: `throw()`

* **Operators**
  - Arithmetic: `+`, `-`, `*`, `/`, `%` (modulo - v0.3.0)
  - Comparison: `==`, `!=` (v0.3.0), `>`, `<`, `>=`, `<=` (return `true`/`false` - v0.3.0)
  - String concatenation with `+`
  - **Operator Overloading** (v0.4.0+): Structs can define custom operator behavior
    - Binary operators: `op_add`, `op_sub`, `op_mul`, `op_div`, `op_mod`, `op_eq`, `op_ne`, `op_gt`, `op_lt`, `op_ge`, `op_le`
    - Unary operators: `op_neg` (unary minus `-`), `op_not` (logical not `!`)

* **Error Messages**
  - Colored error output
  - Source location tracking
  - Line and column information

* **Testing Framework**
  - Built-in test runner
  - Snapshot testing with `.out` files
  - Test result reporting

---

## 🧩 Installation

See [Install Guide](INSTALLATION.md) for platform setup instructions.

---

## 🚀 Getting Started

Install Rust and run:

```bash
# Clean output (recommended)
cargo run --quiet -- run examples/your_script.ruff

# Or with build messages
cargo run -- run examples/your_script.ruff
```

---

## 📄 Writing `.ruff` Scripts

Example:

```ruff
enum Result {
    Ok,
    Err
}

func check(x) {
    if x > 0 {
        return Result::Ok("great")
    }
    return Result::Err("bad")
}

res := check(42)

match res {
    case Result::Ok(msg): {
        print("✓", msg)
    }
    case Result::Err(err): {
        print("✗", err)
    }
}
```

### Enhanced Error Handling (v0.4.0) ✨

Comprehensive error handling with error properties, custom error types, and stack traces:

```ruff
# Access error properties
try {
    throw("Something went wrong")
} except err {
    print("Message:", err.message)
    print("Line:", err.line)
    print("Stack depth:", len(err.stack))
}

# Custom error types with structs
struct ValidationError {
    field: string,
    message: string
}

func validate_email(email: string) {
    if !contains(email, "@") {
        error := ValidationError {
            field: "email",
            message: "Email must contain @ symbol"
        }
        throw(error)
    }
}

try {
    validate_email("invalid")
} except err {
    print("Validation failed:", err.message)
}

# Error chaining with cause
struct DatabaseError {
    message: string,
    cause: string
}

try {
    connect_to_db()
} except conn_err {
    error := DatabaseError {
        message: "Failed to initialize app",
        cause: conn_err.message
    }
    throw(error)
}

# Stack traces from nested function calls
func inner() {
    throw("Error from inner")
}

func outer() {
    inner()
}

try {
    outer()
} except err {
    print("Error:", err.message)
    print("Call stack:", err.stack)
}
```

### String Interpolation (v0.3.0) ✨

```ruff
name := "Alice"
age := 30
score := 95

# Embed expressions directly in strings
greeting := "Hello, ${name}!"
bio := "${name} is ${age} years old"
result := "Score: ${score}/100 (${score >= 90}% = A)"

print(greeting)  # Hello, Alice!
print(bio)       # Alice is 30 years old
print(result)    # Score: 95/100 (true% = A)

# Complex expressions with parentheses
a := 2
b := 3
c := 4
calculation := "Result: (${a} + ${b}) * ${c} = ${(a + b) * c}"
print(calculation)  # Result: (2 + 3) * 4 = 20
```

### Comments (v0.3.0) ✨

Ruff supports three types of comments:

```ruff
# Single-line comment
# These continue to end of line

/*
 * Multi-line comments
 * Span multiple lines
 * Useful for longer explanations
 */

/// Doc comments for documentation
/// @param x The input value
/// @return The result
func square(x) {
    return x * x  /* inline comment */
}

# All comment types work together
/*
 * Block comment explaining the algorithm
 */
/// Documentation for the function
func calculate(n) {
    # Implementation details
    return n * 2
}
```

See [examples/comments.ruff](examples/comments.ruff) for comprehensive examples.

---

## 🧪 Running Tests

Place test files in the `tests/` directory. Each `.ruff` file can have a matching `.out` file for expected output:

```bash
cargo run -- test
```

To regenerate expected `.out` snapshots:

```bash
cargo run -- test --update
```

---

## 🧠 Language Features

* ✅ Mutable/const variables with optional type annotations
* ✅ Functions with return values and type annotations
* ✅ Pattern matching with `match`/`case`
* ✅ Enums with tagged variants
* ✅ Nested pattern matches
* ✅ `try`/`except`/`throw` error handling
* ✅ Structs with fields and methods (v0.2.0)
* ✅ Arrays with element assignment and iteration (v0.2.0)
* ✅ Dictionaries (hash maps) with built-in methods (v0.2.0)
* ✅ For-in loops over arrays, dicts, strings, and ranges (v0.2.0)
* ✅ Built-in collection methods: `push()`, `pop()`, `slice()`, `concat()`, `keys()`, `values()`, `has_key()`, `remove()`, `len()` (v0.2.0)
* ✅ Type system with type checking and inference (v0.1.0)
* ✅ Module system with import/export (v0.1.0)
* ✅ String interpolation with `${}` syntax (v0.3.0)
* ✅ Boolean type as first-class value (v0.3.0)
* ✅ Loop control with `while`, `break`, and `continue` (v0.3.0)
* ✅ Lexical scoping with proper environment stack (v0.3.0)
* ✅ Multi-line and doc comments (v0.3.0)
* ✅ Standard library with math, string, and I/O functions
* ✅ CLI testing framework with snapshot testing
* ✅ Colored error messages with source location tracking

---

## 📦 Roadmap

See [ROADMAP.md](ROADMAP.md) for detailed feature plans.

**Completed (9/15):**
* ✅ Error Messages & Diagnostics (v0.1.0)
* ✅ Type System & Type Checking (v0.1.0)
* ✅ Module System & Imports (v0.1.0)
* ✅ Standard Library Expansion (v0.2.0)
* ✅ Structs & Methods (v0.2.0)
* ✅ Arrays & Dictionaries (v0.2.0)
* ✅ Boolean Type (v0.3.0)
* ✅ Loop Control (`break`, `continue`) (v0.3.0)
* ✅ String Interpolation (v0.3.0)
* ✅ Multi-Line & Doc Comments (v0.3.0)

**High Priority (v0.3.0):**
* [ ] Array Higher-Order Functions (`map`, `filter`, `reduce`)
* [ ] JSON Support (parse/stringify)

**Future:**
* [ ] Interactive REPL
* [ ] Package manager
* [ ] WebAssembly compilation target
* [ ] Language Server Protocol (LSP)
* [ ] JIT compilation

---

## 👨‍💼 Contributing

View the [CONTRIBUTING](CONTRIBUTING.md)