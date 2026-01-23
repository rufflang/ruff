# 🐾 Ruff Programming Language

**Ruff** is a lean, expressive programming language built from scratch in Rust. It borrows inspiration from Go, Python, and functional design — but stands on its own.

> **Status**: v0.5.0 - Interactive REPL with multi-line support and command history! Full-featured interactive shell for experimentation and learning. Built with comprehensive features including structs, methods, collections, type checking, and modules.

**Quick Links**: [Installation](#-installation) • [Getting Started](#-getting-started) • [REPL](#-interactive-repl-v050-) • [Examples](#-writing-ruff-scripts) • [Features](#-project-status) • [Changelog](CHANGELOG.md) • [Roadmap](ROADMAP.md)

---

## Project Status

### Implemented Features (v0.5.0)

* **Interactive REPL** (v0.5.0)
  - Full-featured Read-Eval-Print Loop
  - Multi-line input with automatic detection
  - Command history with up/down arrow navigation
  - Line editing with cursor movement
  - Special commands: `:help`, `:quit`, `:clear`, `:vars`, `:reset`
  - Pretty-printed colored output
  - Persistent state across inputs
  - Error handling without crashes
  - Launch with: `ruff repl`

* **Variables & Constants**
  - `let` and `mut` for mutable variables
  - `const` for constants
  - Shorthand assignment with `:=` (e.g., `x := 5`)
  - Optional type annotations: `x: int := 5`
  - **NEW**: `:=` now properly updates existing variables across scopes

* **Lexical Scoping** (v0.3.0)
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
  - **NEW**: Closures with variable capturing (v0.6.0)
    - Functions capture their definition environment
    - Closure state persists across calls
    - Support for counter patterns and partial application
    ```ruff
    func make_counter() {
        let count := 0
        return func() {
            count := count + 1
            return count
        }
    }
    ```

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
  - **NEW**: Null values: `null` (v0.6.0) - for optional chaining and default values
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

* **Array Higher-Order Functions** (v0.3.0)
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
  - **HTTP** (v0.5.0): `http_get()`, `http_post()`, `http_put()`, `http_delete()`, `http_server()`, `http_response()`, `json_response()`, `redirect_response()`, `set_header()` (v0.5.1), `set_headers()` (v0.5.1) - HTTP client and server with full header control
  - **Binary Files** (v0.6.0): `http_get_binary()`, `read_binary_file()`, `write_binary_file()`, `encode_base64()`, `decode_base64()` - Download and work with binary data (images, PDFs, archives)
  - **Database** (v0.5.1): `db_connect()`, `db_execute()`, `db_query()`, `db_close()` - SQLite database operations
  - **I/O**: `print()`, `input()`
  - **Type Conversion**: `parse_int()`, `parse_float()`
  - **File I/O**: `read_file()`, `write_file()`, `append_file()`, `file_exists()`, `read_lines()`, `list_dir()`, `create_dir()`
  - **Error handling**: `throw()`

* **Operators**
  - Arithmetic: `+`, `-`, `*`, `/`, `%` (modulo - v0.3.0)
  - Comparison: `==`, `!=` (v0.3.0), `>`, `<`, `>=`, `<=` (return `true`/`false` - v0.3.0)
  - String concatenation with `+`
  - **Method Chaining** (v0.6.0):
    - **Null coalescing** `??`: Return left value if not null, otherwise right value
      ```ruff
      username := user?.name ?? "Anonymous"
      ```
    - **Optional chaining** `?.`: Safely access fields, returns null if left side is null
      ```ruff
      email := user?.profile?.email  # Returns null if any part is null
      ```
    - **Pipe operator** `|>`: Pass value as first argument to function
      ```ruff
      result := 5 |> double |> add_ten |> square  # Functional data pipelines
      ```
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

## Installation

See [Install Guide](INSTALLATION.md) for platform setup instructions.

---

## Getting Started

Install Rust and run:

```bash
# Clean output (recommended)
cargo run --quiet -- run examples/your_script.ruff

# Or with build messages
cargo run -- run examples/your_script.ruff
```

---

## � Interactive REPL (v0.5.0)

Launch the interactive shell for experimentation and learning:

```bash
cargo run --quiet -- repl
```

The REPL provides a powerful interactive environment:

**Features**:
- ✅ **Multi-line input** - Automatically detects incomplete statements
- ✅ **Command history** - Navigate with up/down arrows
- ✅ **Line editing** - Full cursor movement and editing support
- ✅ **Persistent state** - Variables and functions stay defined
- ✅ **Pretty output** - Colored, formatted value display
- ✅ **Special commands** - `:help`, `:quit`, `:clear`, `:vars`, `:reset`

**Example Session**:

```
╔══════════════════════════════════════════════════════╗
║          Ruff REPL v0.5.0 - Interactive Shell        ║
╚══════════════════════════════════════════════════════╝

  Welcome! Use :help for commands or :quit
  Tip: Multi-line input: End with unclosed braces

ruff> let x := 42
ruff> x * 2
=> 84

ruff> func factorial(n) {
....>     if n <= 1 {
....>         return 1
....>     }
....>     return n * factorial(n - 1)
....> }

ruff> factorial(5)
=> 120

ruff> let names := ["Alice", "Bob", "Charlie"]
ruff> names
=> ["Alice", "Bob", "Charlie"]

ruff> :help
# Shows available commands

ruff> :quit
Goodbye!
```

**Tips**:
- Type `:help` to see all available commands
- Press Ctrl+C to interrupt current input
- Press Ctrl+D or type `:quit` to exit
- Leave braces unclosed for multi-line input
- Any expression automatically prints its result

---

## Writing `.ruff` Scripts

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

### Enhanced Error Handling (v0.4.0)

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

### HTTP Server & Client (v0.5.0)

Build HTTP servers and make HTTP requests with ease:

**HTTP Client**:
```ruff
# Make HTTP GET request
result := http_get("https://api.example.com/users")
if result.is_ok {
    data := result.value
    print("Status:", data["status"])
    print("Body:", data["body"])
}

# POST request with JSON body
user_data := {"name": "Alice", "email": "alice@example.com"}
result := http_post("https://api.example.com/users", user_data)

# PUT and DELETE requests
http_put("https://api.example.com/users/1", {"name": "Bob"})
http_delete("https://api.example.com/users/1")
```

**HTTP Server**:
```ruff
# Create HTTP server on port 8080
server := http_server(8080)

# Register GET route
server.route("GET", "/hello", func(request) {
    return http_response(200, "Hello, World!")
})

# Register POST route with JSON response
server.route("POST", "/api/data", func(request) {
    data := {"received": request.body, "method": request.method}
    return json_response(200, data)
})

# Start server (blocks and handles requests)
print("Server listening on http://localhost:8080")
server.listen()
```

**REST API Example**:
```ruff
# In-memory data store
todos := []

server := http_server(8080)

# GET all todos
server.route("GET", "/todos", func(request) {
    return json_response(200, todos)
})

# POST new todo
server.route("POST", "/todos", func(request) {
    todo := parse_json(request.body)
    push(todos, todo)
    return json_response(201, todo)
})

print("REST API running on http://localhost:8080")
server.listen()
```

### Binary File Operations (v0.6.0)

Download and work with binary files like images, PDFs, and archives:

```ruff
# Download binary file from URL
image_data := http_get_binary("https://example.com/photo.jpg")
write_binary_file("photo.jpg", image_data)
print("Downloaded:", len(image_data), "bytes")

# Read binary file
file_bytes := read_binary_file("document.pdf")

# Base64 encoding for API transfers
base64_str := encode_base64(file_bytes)  # Also accepts strings
decoded := decode_base64(base64_str)

# Example: Download AI-generated image
# After calling image generation API...
generated_url := api_response["image_url"]
image := http_get_binary(generated_url)
write_binary_file("ai_generated.png", image)

# Embed in JSON
json_payload := {
    "image": encode_base64(image),
    "format": "png"
}
```

**HTTP Headers** (v0.5.1):
```ruff
# Add individual headers
response := http_response(200, "Success")
response := set_header(response, "X-API-Version", "1.0")
response := set_header(response, "Cache-Control", "max-age=3600")

# Set multiple headers at once
headers := {
    "X-Request-ID": "abc-123",
    "X-Rate-Limit": "1000",
    "Access-Control-Allow-Origin": "*"
}
response := set_headers(response, headers)

# JSON responses automatically include Content-Type
json := json_response(200, {"status": "success"})  # Includes Content-Type: application/json

# Redirects with custom headers
redirect_headers := {"X-Redirect-Reason": "Moved permanently"}
redirect := redirect_response("https://new-url.com", redirect_headers)

# Access request headers in route handlers
server.route("POST", "/api/upload", func(request) {
    content_type := request.headers["Content-Type"]
    auth_token := request.headers["Authorization"]
    
    if content_type != "application/json" {
        return http_response(400, "Invalid Content-Type")
    }
    
    response := json_response(200, {"uploaded": true})
    return set_header(response, "X-Upload-ID", "upload-123")
})
```

**Key Features**:
- HTTP methods: GET, POST, PUT, DELETE
- Path-based routing with exact matching
- JSON request/response handling
- Automatic request body parsing
- Request body parsing (JSON)
- Built-in response helpers
- Error handling with proper status codes
- Full header control:
  - Custom response headers
  - Automatic headers (Content-Type for JSON)
  - Request header access
  - CORS support
  - Security headers

See [examples/http_server_simple.ruff](examples/http_server_simple.ruff), [examples/http_rest_api.ruff](examples/http_rest_api.ruff), [examples/http_client.ruff](examples/http_client.ruff), [examples/http_webhook.ruff](examples/http_webhook.ruff), and [examples/http_headers_demo.ruff](examples/http_headers_demo.ruff) for complete examples.

### SQLite Database (v0.5.0) ✨

Ruff includes built-in SQLite database support for persistent data storage:

```ruff
# Connect to database (creates file if not exists)
db := db_connect("myapp.db")

# Create table
db_execute(db, "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)", [])

# Insert data with parameterized queries (prevents SQL injection)
db_execute(db, "INSERT INTO users (name, email) VALUES (?, ?)", ["Alice", "alice@example.com"])
db_execute(db, "INSERT INTO users (name, email) VALUES (?, ?)", ["Bob", "bob@example.com"])

# Query data - returns array of dicts
results := db_query(db, "SELECT * FROM users", [])
for user in results {
    print("User: " + user["name"] + " - " + user["email"])
}

# Query with parameters
user := db_query(db, "SELECT * FROM users WHERE name = ?", ["Alice"])
if len(user) > 0 {
    print("Found: " + user[0]["email"])
}

# Update and delete
db_execute(db, "UPDATE users SET email = ? WHERE name = ?", ["alice@newmail.com", "Alice"])
db_execute(db, "DELETE FROM users WHERE name = ?", ["Bob"])

# Close connection
db_close(db)
```

See [examples/projects/url_shortener.ruff](examples/projects/url_shortener.ruff) for a complete example using SQLite with an HTTP server.

### String Interpolation (v0.3.0)

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

### Comments (v0.3.0)

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

## Running Tests

Place test files in the `tests/` directory. Each `.ruff` file can have a matching `.out` file for expected output:

```bash
cargo run -- test
```

To regenerate expected `.out` snapshots:

```bash
cargo run -- test --update
```

---

## Language Features

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

## Roadmap

See [ROADMAP.md](ROADMAP.md) for detailed feature plans.

---

## Contributing

View the [CONTRIBUTING](CONTRIBUTING.md) document.