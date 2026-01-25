# 🐾 Ruff Programming Language

**Ruff** is a purpose-built, correctness-first execution language designed for tooling, automation, and AI-assisted development.

> **Status**: v0.7.0 (Released January 2026) - **Core Language Complete!** 🎉 A fully-featured language with production-ready databases, comprehensive string/array/dict methods, type system, testing utilities, HTTP server/client, concurrency, and 100+ built-in functions.

**Quick Links**: [Installation](#installation) • [Getting Started](#getting-started) • [REPL](#interactive-repl-v050-) • [Examples](#writing-ruff-scripts) • [Features](#project-status) • [Changelog](CHANGELOG.md) • [Roadmap](ROADMAP.md)

---

## Project Status

### Implemented Features (v0.7.0)

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
  - **NEW**: Integers (i64) and Floats (f64) - Separate types (v0.7.0)
    - Integer literals: `42`, `-10`, `0`
    - Float literals: `3.14`, `-2.5`, `0.0`
    - Type preservation: `5 + 3` → `8` (int), `5.0 + 3.0` → `8.0` (float)
    - Integer division truncates: `10 / 3` → `3`
    - Mixed operations promote to float: `5 + 2.5` → `7.5`
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
  - **NEW**: Advanced Collections (v0.6.0)
    - **Set**: Unique value collections with union, intersection, difference operations
      ```ruff
      users := Set(["alice", "bob", "alice"])  # {"alice", "bob"}
      users := set_add(users, "charlie")
      has_bob := set_has(users, "bob")  # true
      ```
    - **Queue**: FIFO (First-In-First-Out) data structure for task processing
      ```ruff
      tasks := Queue([])
      tasks := queue_enqueue(tasks, "task1")
      result := queue_dequeue(tasks)  # [modified_queue, "task1"]
      ```
    - **Stack**: LIFO (Last-In-First-Out) data structure for undo/history
      ```ruff
      history := Stack([])
      history := stack_push(history, "page1")
      result := stack_pop(history)  # [modified_stack, "page1"]
      ```

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
  - **Math**: `abs()`, `sqrt()`, `pow()`, `floor()`, `ceil()`, `round()`, `min()`, `max()`, `sin()`, `cos()`, `tan()`, `log()` (v0.7.0), `exp()` (v0.7.0), constants `PI` and `E`
  - **Random** (v0.4.0): `random()`, `random_int(min, max)`, `random_choice(array)` - Random number generation
  - **Range** (v0.7.0): `range(stop)`, `range(start, stop)`, `range(start, stop, step)` - Generate number sequences for loops
    ```ruff
    for i in range(5) { print(i) }  # 0, 1, 2, 3, 4
    for i in range(1, 10, 2) { print(i) }  # 1, 3, 5, 7, 9
    for i in range(10, 0, 0 - 2) { print(i) }  # 10, 8, 6, 4, 2
    ```
  - **Format String** (v0.7.0): `format(template, ...args)` - sprintf-style string formatting with `%s`, `%d`, `%f`
    ```ruff
    format("Hello %s", "world")  # "Hello world"
    format("%s has %d apples", "Alice", 5)  # "Alice has 5 apples"
    format("Pi is %.2f", 3.14159)  # "Pi is 3.14"
    ```
  - **Strings**: `len()`, `to_upper()`, `to_lower()`, `trim()`, `substring()`, `contains()`, `replace_str()`, `starts_with()`, `ends_with()`, `index_of()`, `repeat()`, `split()`, `join()`
  - **String Methods** (v0.7.0): `upper()`, `lower()`, `capitalize()`, `trim_start()`, `trim_end()`, `char_at(index)`, `is_empty()`, `count_chars()` - Enhanced string manipulation
    ```ruff
    capitalize("hello world")  # "Hello world"
    char_at("ruff", 1)  # "u"
    is_empty("")  # true
    ```
  - **Regex** (v0.4.0): `regex_match()`, `regex_find_all()`, `regex_replace()`, `regex_split()` - Pattern matching and text processing
  - **Arrays**: `push()`, `pop()`, `slice()`, `concat()`, `len()`
  - **Array Higher-Order**: `map()`, `filter()`, `reduce()`, `find()` (v0.3.0)
  - **Array Utilities** (v0.7.0): `sort()`, `reverse()`, `unique()`, `sum()`, `any()`, `all()` - Essential operations for data processing and analysis
  - **Array Mutation** (v0.7.0): `insert(array, index, item)`, `remove(array, item)`, `remove_at(array, index)`, `clear(array)`, `index_of(array, item)`, `contains(array, item)` - In-place array operations
    ```ruff
    arr := [1, 2, 4]
    arr2 := insert(arr, 2, 3)  # [1, 2, 3, 4]
    arr3 := remove(arr2, 3)  # [1, 2, 4]
    has := contains(arr3, 2)  # true
    ```
  - **Dicts**: `keys()`, `values()`, `has_key()`, `remove()`, `len()`
  - **Dict Methods** (v0.7.0): `items(dict)`, `get(dict, key, default)`, `merge(dict1, dict2)`, `clear(dict)` - Enhanced dictionary operations
    ```ruff
    user := {"name": "Alice", "age": 30}
    pairs := items(user)  # [["name", "Alice"], ["age", 30]]
    email := get(user, "email", "N/A")  # "N/A" (not found)
    combined := merge(user, {"city": "NYC"})  # {"name": "Alice", "age": 30, "city": "NYC"}
    ```
  - **JSON**: `parse_json()`, `to_json()` - Parse and serialize JSON data (v0.3.0)
  - **TOML** (v0.6.0): `parse_toml()`, `to_toml()` - Parse and serialize TOML configuration files
  - **YAML** (v0.6.0): `parse_yaml()`, `to_yaml()` - Parse and serialize YAML documents
  - **CSV** (v0.6.0): `parse_csv()`, `to_csv()` - Parse and serialize CSV data files
  - **Date/Time** (v0.4.0): `now()`, `format_date()`, `parse_date()`, `current_timestamp()` (v0.7.0), `performance_now()` (v0.7.0), `time_us()` (v0.7.0), `time_ns()` (v0.7.0), `format_duration()` (v0.7.0), `elapsed()` (v0.7.0) - Complete timing suite with microsecond/nanosecond precision for robust benchmarking
  - **System** (v0.4.0): `env()`, `args()`, `exit()`, `sleep()`, `execute()` - System operations
  - **Paths** (v0.4.0): `join_path()`, `dirname()`, `basename()`, `path_exists()` - Path manipulation
  - **HTTP** (v0.5.0): `http_get()`, `http_post()`, `http_put()`, `http_delete()`, `http_server()`, `http_response()`, `json_response()`, `redirect_response()`, `set_header()` (v0.5.1), `set_headers()` (v0.5.1) - HTTP client and server with full header control
  - **Concurrency & Parallelism** (v0.6.0): `spawn { }`, `parallel_http()`, `channel()`, `chan.send()`, `chan.receive()` - Background tasks and parallel HTTP requests for 3x faster API calls
  - **HTTP Authentication** (v0.6.0): `jwt_encode()`, `jwt_decode()` - JWT token encoding/decoding for API authentication
  - **OAuth2** (v0.6.0): `oauth2_auth_url()`, `oauth2_get_token()` - OAuth2 flow helpers for third-party authentication
  - **HTTP Streaming** (v0.6.0): `http_get_stream()` - Memory-efficient downloads for large files
  - **Binary Files** (v0.6.0): `http_get_binary()`, `read_binary_file()`, `write_binary_file()`, `encode_base64()`, `decode_base64()` - Download and work with binary data (images, PDFs, archives)
  - **Image Processing** (v0.6.0): `load_image()`, `img.resize()`, `img.crop()`, `img.rotate()`, `img.flip()`, `img.to_grayscale()`, `img.blur()`, `img.adjust_brightness()`, `img.adjust_contrast()`, `img.save()` - Load, manipulate, and save images (JPEG, PNG, WebP, GIF, BMP)
  - **Database** (v0.6.0): `db_connect(db_type, connection_string)` - Unified database API supporting SQLite ✅, PostgreSQL ✅, and MySQL ✅, `db_execute()`, `db_query()`, `db_close()`, `db_begin()`, `db_commit()`, `db_rollback()` - Full CRUD operations with transactions and connection pooling
  - **I/O**: `print()`, `input()`
  - **Type Conversion**: `parse_int()`, `parse_float()`
  - **Type Introspection** (v0.7.0): `type()`, `is_int()`, `is_float()`, `is_string()`, `is_array()`, `is_dict()`, `is_bool()`, `is_null()`, `is_function()` - Runtime type checking for defensive coding and validation
  - **Assert & Debug** (v0.7.0): `assert(condition, message?)`, `debug(...args)` - Runtime assertions and detailed debug output for testing and troubleshooting
  - **File I/O**: `read_file()`, `write_file()`, `append_file()`, `file_exists()`, `read_lines()`, `list_dir()`, `create_dir()`, `file_size()`, `delete_file()`, `rename_file()`, `copy_file()`
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

### Concurrency & Parallelism (v0.6.0)

Run code concurrently for faster AI tools, batch processing, and non-blocking operations:

**Parallel HTTP Requests** (3x faster for AI model comparison):
```ruff
# Query multiple AI providers simultaneously
urls := [
    "https://api.openai.com/v1/chat/completions",
    "https://api.anthropic.com/v1/messages",
    "https://api.deepseek.com/v1/chat/completions"
]

# All 3 requests happen in parallel - returns in ~2s instead of 6s!
results := parallel_http(urls)

for result in results {
    print("Status: " + result["status"])
    print("Response: " + result["body"])
}
```

**Background Tasks with spawn**:
```ruff
# Fire and forget - main thread continues immediately
spawn {
    print("Processing in background...")
    process_large_file()
}

print("Main thread continues without waiting")
```

**Thread Communication with Channels**:
```ruff
chan := channel()

spawn {
    result := expensive_computation()
    chan.send(result)
}

# Receive result from background thread
value := chan.receive()
print("Got result: " + value)
```

**Key Features**:
- **3x faster** parallel HTTP requests for AI model comparison
- Non-blocking background tasks with `spawn`
- Thread-safe communication with channels
- Perfect for batch processing and data pipelines

### Image Processing (v0.6.0)

Load, manipulate, and save images with built-in support for JPEG, PNG, WebP, GIF, and BMP:

```ruff
# Load and inspect image
img := load_image("photo.jpg")
print("Size: " + img.width + "x" + img.height)
print("Format: " + img.format)

# Create thumbnail (maintain aspect ratio)
thumb := img.resize(200, 200, "fit")
thumb.save("thumbnail.jpg")

# Exact dimensions
resized := img.resize(800, 600)
resized.save("resized.jpg")

# Crop region (x, y, width, height)
cropped := img.crop(100, 100, 400, 300)
cropped.save("cropped.jpg")

# Transformations
rotated := img.rotate(90)  # 90, 180, or 270 degrees
flipped := img.flip("horizontal")  # or "vertical"

# Filters and adjustments
gray := img.to_grayscale()
blurred := img.blur(5.0)  # sigma value
brighter := img.adjust_brightness(1.2)  # 20% brighter
contrast := img.adjust_contrast(1.3)  # More contrast

# Method chaining for complex workflows
enhanced := img
    .resize(800, 600)
    .adjust_brightness(1.1)
    .adjust_contrast(1.15)
    .to_grayscale()

enhanced.save("enhanced.jpg")

# Format conversion (auto-detects from extension)
img.save("output.png")  # JPEG -> PNG
img.save("output.webp")  # JPEG -> WebP

# Batch processing
images := ["img1.jpg", "img2.jpg", "img3.jpg"]
for path in images {
    img := load_image(path)
    thumb := img.resize(200, 200, "fit")
    
    # Extract filename
    parts := split(path, ".")
    name := parts[0]
    
    thumb.save("thumbs/" + name + "_thumb.jpg")
}

# Social media image prep
func prepare_instagram_post(image_path) {
    img := load_image(image_path)
    
    # Instagram: 1080x1080
    resized := img.resize(1080, 1080, "fit")
    
    # Enhance for social media
    enhanced := resized
        .adjust_brightness(1.1)
        .adjust_contrast(1.15)
    
    enhanced.save("instagram_post.jpg")
}
```

**Supported Operations**:
- Load/save: JPEG, PNG, WebP, GIF, BMP
- Resize: exact dimensions or maintain aspect ratio
- Transform: crop, rotate (90/180/270), flip (h/v)
- Filters: grayscale, blur (Gaussian)
- Adjust: brightness, contrast
- Properties: width, height, format
- Method chaining for complex workflows
- Error handling for missing/invalid files

- Built-in response helpers
- Error handling with proper status codes
- Full header control:
  - Custom response headers
  - Automatic headers (Content-Type for JSON)
  - Request header access
  - CORS support
  - Security headers

See [examples/http_server_simple.ruff](examples/http_server_simple.ruff), [examples/http_rest_api.ruff](examples/http_rest_api.ruff), [examples/http_client.ruff](examples/http_client.ruff), [examples/http_webhook.ruff](examples/http_webhook.ruff), and [examples/http_headers_demo.ruff](examples/http_headers_demo.ruff) for complete examples.

### Unified Database API (v0.7.0) 🗄️

Ruff includes a unified database API that works across different database backends. Currently supports **SQLite**, **PostgreSQL**, and **MySQL**:

```ruff
# SQLite - Perfect for local apps and embedded databases
db := db_connect("sqlite", "myapp.db")
db_execute(db, "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)", [])
db_execute(db, "INSERT INTO users (name, email) VALUES (?, ?)", ["Alice", "alice@example.com"])
users := db_query(db, "SELECT * FROM users", [])

# PostgreSQL - Perfect for production web applications  
db := db_connect("postgres", "host=localhost dbname=myapp user=admin password=secret")
db_execute(db, "CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY, name TEXT, email TEXT)", [])
db_execute(db, "INSERT INTO users (name, email) VALUES ($1, $2)", ["Alice", "alice@example.com"])
users := db_query(db, "SELECT * FROM users", [])

# MySQL - Perfect for traditional web applications
db := db_connect("mysql", "mysql://root@localhost:3306/myapp")
db_execute(db, "CREATE TABLE IF NOT EXISTS users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), email VARCHAR(100))", [])
db_execute(db, "INSERT INTO users (name, email) VALUES (?, ?)", ["Alice", "alice@example.com"])
users := db_query(db, "SELECT * FROM users", [])

# Same Ruff code works with all databases!
for user in users {
    print("User: " + user["name"] + " - " + user["email"])
}

# Query with parameters (prevents SQL injection)
bob := db_query(db, "SELECT * FROM users WHERE name = $1", ["Bob"])  # PostgreSQL uses $1, $2
# bob := db_query(db, "SELECT * FROM users WHERE name = ?", ["Bob"])   # SQLite & MySQL use ?

# Update and delete
db_execute(db, "UPDATE users SET email = $1 WHERE name = $2", ["alice@newmail.com", "Alice"])
db_execute(db, "DELETE FROM users WHERE name = $1", ["Bob"])

# Close connection
db_close(db)
```

**Key Features**:
- **Unified API**: Same `db_connect()`, `db_execute()`, and `db_query()` functions work across all databases
- **SQLite Support**: `?` placeholders for parameters
- **PostgreSQL Support**: `$1, $2, $3` placeholders for parameters
- **MySQL Support**: `?` placeholders for parameters (async driver with transparent blocking)
- **Parameter Binding**: Prevents SQL injection attacks
- **Type Safety**: Returns proper Null values for NULL database fields
- **Type Support**: Integers, floats, strings, booleans, and NULL
- **Multi-Backend Ready**: Switch databases by changing connection string only

**Database-Specific Syntax Notes**:
- **SQLite**: Uses `?` for parameters: `INSERT INTO users VALUES (?, ?)`
- **PostgreSQL**: Uses `$1, $2` for parameters: `INSERT INTO users VALUES ($1, $2)`
- **MySQL**: Uses `?` for parameters: `INSERT INTO users VALUES (?, ?)`
- Everything else is the same Ruff code!

See `examples/database_unified.ruff` for comprehensive SQLite examples, `examples/database_postgres.ruff` for PostgreSQL examples, `examples/database_mysql.ruff` for MySQL examples, and `examples/projects/url_shortener.ruff` for a complete URL shortener using SQLite with an HTTP server.

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