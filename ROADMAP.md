# Ruff Language - Development Roadmap

This roadmap outlines planned features and improvements for future versions of the Ruff programming language. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.7.0 (Core Language Completion - RELEASED! 🎉)  
> **Next Planned Release**: v0.8.0 (Performance & Modern Syntax)  
> **Path to v1.0**: See [PATH_TO_PRODUCTION.md](PATH_TO_PRODUCTION.md) for comprehensive roadmap

---

## Priority Levels

- **P1 (High)**: Core language features needed for real-world applications
- **P2 (Medium)**: Quality-of-life improvements and developer experience
- **P3 (Low)**: Nice-to-have features for advanced use cases

---

## v0.8.0 - Performance & Error Handling

**Focus**: Speed improvements, modern error handling, and essential language ergonomics  
**Timeline**: Q2 2026 (2-3 months)  
**See**: [PATH_TO_PRODUCTION.md](PATH_TO_PRODUCTION.md) for details

### 18. Destructuring (P1)

**Status**: ✅ Complete (v0.8.0)  
**Estimated Effort**: Medium (2-3 weeks)

**Why Critical**: Expected by developers from JavaScript, Python, Rust. Huge quality-of-life improvement.

**Features**:
```ruff
# Array destructuring
[first, second, ...rest] := [1, 2, 3, 4, 5]
# first=1, second=2, rest=[3,4,5]

# Ignore values with _
[x, _, z] := [1, 2, 3]  # x=1, z=3

# Dict destructuring  
{name, email} := user
# Extracts user["name"] and user["email"]

# With defaults
{name, role="guest"} := user

# Function returns
[status, data] := http_get("api.com/users")
[ok, result] := divide(10, 2)

# Nested destructuring
{user: {name, age}, posts: [first_post, ...]} := response

# In function parameters
func process_user({name, email, age}) {
    print("Processing ${name}")
}
```

**Implementation**: Requires parser updates for destructuring patterns, interpreter support for pattern binding

---

### 19. Spread Operator (P1)

**Status**: ✅ Complete (v0.8.0)  
**Estimated Effort**: Medium (1-2 weeks)

**Why Critical**: Essential for modern functional programming patterns

**Features**:
```ruff
# Array spread
arr1 := [1, 2, 3]
arr2 := [...arr1, 4, 5, 6]  # [1, 2, 3, 4, 5, 6]
combined := [...arr1, ...arr2]  # Merge arrays

# Dict spread (merge with override)
defaults := {"timeout": 30, "retry": 3}
config := {...defaults, "timeout": 60}  # {"timeout": 60, "retry": 3}
merged := {...dict1, ...dict2, ...dict3}

# Function arguments
args := [1, 2, 3]
result := some_function(...args)  # Spread as arguments
```

**Implementation**: Add `...` operator to parser, handle in array/dict literals and function calls

---

### 20. Enhanced Error Messages (P1)

**Status**: ✅ Complete (v0.8.0)  
**Estimated Effort**: Medium (2-3 weeks)

**Why Critical**: Developer experience - helps newcomers learn faster

**Features**:
```
Error: Undefined variable 'usrname'
  --> script.ruff:15:10
   |
15 |     print(usrname)
   |           ^^^^^^^ not found in this scope
   |
   = help: Did you mean 'username'?

Error: Type mismatch in function call
  --> script.ruff:23:5
   |
23 |     calculate("hello")
   |     ^^^^^^^^^^^^^^^^^ expected number, found string
   |
   = note: Function 'calculate' expects numeric argument
   = help: Try converting with to_int() or to_float()
```

**Implementation**: ✅ Complete
- ✅ Levenshtein distance algorithm for "Did you mean?" suggestions
- ✅ Context-aware help messages for common errors
- ✅ Multiple error reporting (type checker collects all errors)
- ✅ Structured error display with source location, code context, and suggestions
- ✅ Helpful guidance for type mismatches, undefined functions, and other errors

---

### 21. Bytecode Compiler & VM (P1)

**Status**: Planned  
**Estimated Effort**: Large (6-8 weeks)

**Goal**: **10-20x performance improvement** over tree-walking interpreter

**Architecture**:
- Compile AST to bytecode instructions
- Stack-based virtual machine
- Register-based optimization passes

**Expected Performance**: Move from ~50-100x slower than Python to competitive speeds

---

### 22. Result & Option Types (P1)

**Status**: ✅ Complete (v0.8.0)  
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

### 23. Standard Library Expansion (P1)

**Status**: Planned  
**Estimated Effort**: Large (3 months)

**Core Modules** (First Priority):
- `os` - Operating system interface (getcwd, chdir, mkdir, environ)
- `path` - Path manipulation (join, absolute, exists, is_dir)
- `io` - Buffered I/O and binary operations
- `net` - TCP/UDP sockets beyond HTTP
- `crypto` - Hashing (SHA256, MD5) and encryption (AES)
**Essential Built-in Functions** (High Priority):
```ruff
# Command-line argument parsing
parser := arg_parser()
parser.add_argument("--verbose", "-v", type="bool", help="Enable verbose output")
parser.add_argument("--config", type="string", required=true)
args := parser.parse()

# Environment variable helpers
db_host := env_or("DB_HOST", "localhost")  # Get with default
db_port := env_int("DB_PORT")  # Parse as int
api_key := env_required("API_KEY")  # Error if missing

# Compression & Archives
archive := zip_create("backup.zip")
zip_add_file(archive, "data.txt")
zip_add_dir(archive, "documents/")
zip_close(archive)

files := unzip("archive.zip", "output/")

# Hashing & Crypto
sha := sha256("my data")  # SHA-256 hash
md5hash := md5_file("document.pdf")  # MD5 of file
password := hash_password("secret123")  # bcrypt
valid := verify_password(input, password)

# Process management
proc := spawn_process(["python", "script.py"])
output := proc.wait_output()  # Blocking wait
exitcode := proc.exitcode()
proc.kill()  # Force terminate

# Process with pipes
result := pipe_commands([
    ["cat", "data.txt"],
    ["grep", "error"],
    ["wc", "-l"]
])
```
**See**: [PATH_TO_PRODUCTION.md](PATH_TO_PRODUCTION.md) Section 3.1 for complete module list

---

### 24. Enhanced Collection Methods (P2)

**Status**: ✅ Complete (v0.8.0)  
**Estimated Effort**: Medium (1-2 weeks)

**Why Important**: Complete the collections API to match Python/JavaScript/Rust expectations

**Implementation**: All methods implemented and tested

**Array Methods**:
```ruff
# Advanced transformations
chunk(arr, n)         # [[1,2,3,4,5]].chunk(2) → [[1,2], [3,4], [5]]
flatten(arr)          # [[1,2], [3,4]] → [1,2,3,4]
zip(arr1, arr2)       # zip([1,2], [3,4]) → [[1,3], [2,4]]
enumerate(arr)        # ["a", "b"] → [[0, "a"], [1, "b"]]
take(arr, n)          # First n elements
skip(arr, n)          # Skip n elements
windows(arr, n)       # Sliding window: [1,2,3,4].windows(2) → [[1,2], [2,3], [3,4]]
```

**Dict Methods**:
```ruff
# Advanced operations
invert(dict)          # {a:1, b:2} → {1:a, 2:b}
update(dict1, dict2)  # Merge dict2 into dict1 (returns new)
get_default(dict, key, default)  # Get value or return default if missing
```

**String Methods**:
```ruff
# Case and formatting
pad_left(str, width, char)     # "5".pad_left(3, "0") → "005"
pad_right(str, width, char)    # "a".pad_right(3, "-") → "a--"
lines(str)                     # Split on any newline \n, \r\n, \r
words(str)                     # Split on whitespace
str_reverse(str)               # "hello" → "olleh"
slugify(str)                   # "Hello World!" → "hello-world"
truncate(str, len, suffix)     # "Hello World".truncate(8, "...") → "Hello..."
to_camel_case(str)             # "hello_world" → "helloWorld"
to_snake_case(str)             # "helloWorld" → "hello_world"
to_kebab_case(str)             # "helloWorld" → "hello-world"
```

**Note**: Some advanced methods like `filter_map`, `partition`, `map_values`, and `map_keys` are deferred to future releases as they require higher-order function support enhancements.

---

### 25. Async/Await (P1)

**Status**: Planned  
**Estimated Effort**: Very Large (6-8 weeks)

**Why Critical**: THE feature that defines modern languages. Essential for I/O-heavy applications.

**Features**:
```ruff
# Async function declaration
async func fetch_user(id) {
    response := await http_get("api.com/users/${id}")
    return response.body
}

# Concurrent execution
func fetch_all_users() {
    # Run requests concurrently
    users := await Promise.all([
        fetch_user(1),
        fetch_user(2),
        fetch_user(3)
    ])
    
    return users
}

# Race condition (first to finish)
fastest := await Promise.race([
    http_get("api1.com/data"),
    http_get("api2.com/data")
])

# Error handling with async
async func safe_fetch(url) {
    try {
        response := await http_get(url)
        return Ok(response)
    } except error {
        return Err(error)
    }
}

# Async iteration
async for chunk in stream_large_file("data.csv") {
    process(chunk)
}
```

**Integration with existing concurrency**:
- Build on top of `spawn` and channels
- Async runtime with work-stealing scheduler
- Compatible with existing blocking I/O (auto-wrap in async)

---

### 26. Iterators & Generators (P1)

**Status**: Planned  
**Estimated Effort**: Large (3-4 weeks)

**Why Important**: Lazy evaluation, memory efficiency, and functional programming patterns

**Features**:
```ruff
# Generator functions with yield
func* fibonacci() {
    let a := 0
    let b := 1
    loop {
        yield a
        [a, b] := [b, a + b]
    }
}

# Use with for-in
for num in fibonacci().take(10) {
    print(num)  # First 10 Fibonacci numbers
}

# Iterator chaining (like Rust)
result := range(100)
    .filter(func(n) { return n % 2 == 0 })  # Even numbers
    .map(func(n) { return n * n })          # Square them
    .take(5)                                # First 5
    .collect()  # [0, 4, 16, 36, 64]

# Lazy evaluation (no intermediate arrays!)
first_match := range(1000000)
    .filter(func(n) { return n % 37 == 0 })
    .take(1)  # Only checks until first match

# Custom iterators
struct RangeIter {
    current: int,
    end: int,
    step: int,
    
    func next(self) {
        if self.current >= self.end {
            return None  # Iterator exhausted
        }
        value := self.current
        self.current := self.current + self.step
        return Some(value)
    }
}
```

**Implementation**: Iterator protocol with `next()` method, generator syntax `func*` and `yield` keyword

---

## v0.9.0 - Developer Experience

**Focus**: World-class tooling for productivity  
**Timeline**: Q3 2026 (3 months)  
**See**: [PATH_TO_PRODUCTION.md](PATH_TO_PRODUCTION.md) Pillar 4

### 27. Built-in Testing Framework (P1)

**Status**: Planned  
**Estimated Effort**: Medium (2-3 weeks)

**Why Critical**: Every modern language needs testing built-in, not as afterthought

**Features**:
```ruff
# Test blocks with built-in assertions
test "array operations work correctly" {
    arr := [1, 2, 3]
    arr2 := push(arr, 4)
    assert_equal(arr2, [1, 2, 3, 4])
    assert_true(contains(arr2, 3))
    assert_false(contains(arr2, 99))
}

test "http requests return 200" {
    response := http_get("example.com")
    assert_equal(response.status, 200)
    assert_true(len(response.body) > 0)
}

# Setup/teardown
test_setup {
    db := db_connect("sqlite", ":memory:")
    db_execute(db, "CREATE TABLE users (id INTEGER, name TEXT)", [])
}

test_teardown {
    db_close(db)
}

# Test groups
test_group "database operations" {
    test "insert works" { ... }
    test "query works" { ... }
    test "delete works" { ... }
}
```

**CLI**:
```bash
ruff test file.ruff              # Run all tests
ruff test file.ruff --verbose    # Show details
ruff test --watch                # Run on file changes
```

**Implementation**: Add `test` keyword, assertion functions, test runner in CLI

---

### 28. REPL Improvements (P2)

**Status**: Planned  
**Estimated Effort**: Medium (1-2 weeks)

**Current Gaps**:
- ❌ No tab completion
- ❌ No syntax highlighting
- ❌ No multi-line editing help
- ❌ No import from previous sessions
- ❌ No `.help <function>` documentation

**Features**:
```
$ ruff repl
>>> range<TAB>
range(stop)  range(start, stop)  range(start, stop, step)

>>> .help range
range(stop) - Generate sequence from 0 to stop-1
range(start, stop) - Generate sequence from start to stop-1  
range(start, stop, step) - Generate sequence with custom step

Examples:
  range(5) → [0, 1, 2, 3, 4]
  range(1, 10, 2) → [1, 3, 5, 7, 9]

>>> arr := [1, 2, 3]
[1, 2, 3]

>>> # Syntax highlighting for code
>>> func double(x) {
...     return x * 2
... }
<function double>

>>> .history
1: arr := [1, 2, 3]
2: func double(x) { return x * 2 }

>>> .save session.ruff  # Save session to file
```

**Implementation**: Enhanced rustyline integration, documentation database, completion provider

---

### 29. Documentation Generator (P2)

**Status**: Planned  
**Estimated Effort**: Medium (2-3 weeks)

**Features**:
```ruff
/// Calculates the square of a number.
/// 
/// # Examples
/// ```ruff
/// result := square(5)  # 25
/// result := square(10) # 100
/// ```
/// 
/// # Parameters
/// - n: The number to square (int or float)
/// 
/// # Returns
/// The square of the input (same type as input)
/// 
/// # Errors
/// None - this function cannot fail
func square(n) {
    return n * n
}
```

**CLI**:
```bash
ruff doc                    # Generate docs to ./docs
ruff doc --output ./api     # Custom output dir
ruff doc --serve            # Live preview on localhost:8080
ruff doc --format markdown  # or html, json
```

**Output**: Beautiful HTML documentation like Rust's docs.rs

---

### 30. Language Server Protocol (LSP) (P1)

**Status**: Planned  
**Estimated Effort**: Large (4-6 weeks)

**Why Critical**: Professional IDE support is non-negotiable for developer adoption

**Features**:
- **Autocomplete**: Built-ins, variables, functions, imports, struct fields
- **Go to definition**: Jump to function/struct/variable definitions
- **Find references**: Show all usages of a symbol
- **Hover documentation**: Show function signatures and doc comments
- **Real-time diagnostics**: Errors and warnings as you type
- **Rename refactoring**: Rename symbols across entire project
- **Code actions**: Quick fixes, import organization, extract function
- **Inlay hints**: Show inferred types and parameter names
- **Semantic highlighting**: Context-aware syntax coloring
- **Workspace symbols**: Jump to any symbol in project
- **IDE support**: VS Code (primary), IntelliJ, Vim, Emacs, Sublime

**Implementation**: Use `tower-lsp` Rust framework

---

### 31. Code Formatter (ruff-fmt) (P1)

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

### 32. Linter (ruff-lint) (P1)

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

### 33. Package Manager (P1)

**Status**: Planned  
**Estimated Effort**: Large (8-12 weeks)

**Why Critical**: No language succeeds without a package ecosystem

**Features**:
- `ruff.toml` project configuration
- Dependency management with semver
- Package registry (like npm, crates.io)
- CLI commands: `ruff init`, `ruff add`, `ruff install`, `ruff publish`, `ruff remove`
- Lock files for reproducible builds
- Private registry support
- Workspace support (monorepos)

**Example ruff.toml**:
```toml
[package]
name = "my-web-app"
version = "1.0.0"
authors = ["Alice <alice@example.com>"]
license = "MIT"

[dependencies]
http-server = "0.5.0"
json-schema = "1.2.0"
logger = "^2.0"  # Caret for compatible versions

[dev-dependencies]
test-utils = "0.1.0"

[scripts]
start = "ruff run server.ruff"
test = "ruff test tests/"
build = "ruff build --release"
```

---

### 34. Debugger (P2)

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

### 35. Profiler (P2)

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

### 36. Hot Reload (P2)

**Status**: Planned  
**Estimated Effort**: Medium (2-3 weeks)

**Why Important**: Rapid development feedback loop

**Features**:
```bash
# Watch mode for development
ruff watch server.ruff          # Auto-restart on changes
ruff watch --exec "test"        # Run tests on changes
ruff watch --debounce 500       # Wait 500ms after last change
```

**Implementation**: File watcher + process management

---

### 37. Standard Patterns Library (P2)

**Status**: Planned  
**Estimated Effort**: Medium (2-3 weeks)

**Why Important**: Common patterns as built-in utilities save developers time

**Features**:
```ruff
import patterns

# Retry with exponential backoff
result := patterns.retry(
    func() { return http_get("flaky-api.com") },
    max_attempts=5,
    backoff="exponential",  # or "linear", "constant"
    initial_delay=100  # milliseconds
)

# Rate limiting
limiter := patterns.rate_limit(100, "per_minute")  # 100 calls per minute
for request in requests {
    limiter.wait()  # Blocks if rate exceeded
    process(request)
}

# Circuit breaker (prevent cascading failures)
breaker := patterns.circuit_breaker(
    failure_threshold=5,    # Open after 5 failures
    timeout=60,             # Try again after 60 seconds
    success_threshold=2     # Close after 2 successes
)

result := breaker.call(func() { 
    return external_api_call() 
})

if breaker.is_open() {
    print("Service degraded, using fallback")
}

# Memoization/caching
cached_fn := patterns.memoize(expensive_function)
result1 := cached_fn(10)  # Computed
result2 := cached_fn(10)  # Cached (instant)

# Debounce/throttle
throttled := patterns.throttle(api_call, 1000)  # Max once per second
debounced := patterns.debounce(search, 300)     # Wait 300ms after last call
```

---

### 38. HTTP Testing & Mocking (P2)

**Status**: Planned  
**Estimated Effort**: Small (1 week)

**Why Important**: Essential for testing HTTP-dependent code

**Features**:
```ruff
import http.testing

# Create mock server
mock := http_mock()
mock.on_get("/users", {
    status: 200, 
    body: [{"id": 1, "name": "Alice"}]
})
mock.on_post("/users", func(request) {
    # Dynamic response based on request
    return {status: 201, body: {"id": 2}}
})

# Use mock in tests
test "user service fetches users" {
    result := http_get("http://mock/users")
    assert_equal(result.status, 200)
    assert_equal(len(result.body), 1)
}

# Request assertions
mock.assert_called("/users", times=3)
mock.assert_called_with("/users", method="GET", headers={"Auth": "Bearer token"})

# Record/replay
recorder := http_recorder()
recorder.record(func() {
    http_get("real-api.com/data")
})
recorder.save("fixtures/api_response.json")

# Later, replay
replayer := http_replay("fixtures/api_response.json")
```

---

### 39. Language Server Protocol (LSP) (P1)

**Status**: Planned  
**Estimated Effort**: Large (4-6 weeks)

**Why Critical**: Professional IDE support is non-negotiable for developer adoption

**Features**:
- **Autocomplete**: Built-ins, variables, functions, imports, struct fields
- **Go to definition**: Jump to function/struct/variable definitions
- **Find references**: Show all usages of a symbol
- **Hover documentation**: Show function signatures and doc comments
- **Real-time diagnostics**: Errors and warnings as you type
- **Rename refactoring**: Rename symbols across entire project
- **Code actions**: Quick fixes, import organization, extract function
- **Inlay hints**: Show inferred types and parameter names
- **Semantic highlighting**: Context-aware syntax coloring
- **Workspace symbols**: Jump to any symbol in project
- **IDE support**: VS Code (primary), IntelliJ, Vim, Emacs, Sublime

**Implementation**: Use `tower-lsp` Rust framework

---

## v1.0.0 - Production Ready

**Focus**: Polish, documentation, community  
**Timeline**: Q4 2026 (3 months)  
**Goal**: Production-ready language competitive with other popular languages

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

## v1.0.0+ - Advanced Features

### 40. Enums with Methods (P2)

**Status**: Planned  
**Estimated Effort**: Medium (2-3 weeks)

**Why Important**: Enums are more powerful when they have behavior

**Features**:
```ruff
enum Status {
    Pending,
    Active { user_id: int, started_at: int },
    Completed { result: string, finished_at: int },
    Failed { error: string }
    
    # Methods on enums!
    func is_done(self) {
        return match self {
            case Status::Completed: true
            case Status::Failed: true
            case _: false
        }
    }
    
    func get_message(self) {
        return match self {
            case Status::Pending: "Waiting to start..."
            case Status::Active{user_id}: "User ${user_id} is working"
            case Status::Completed{result}: "Done: ${result}"
            case Status::Failed{error}: "Error: ${error}"
        }
    }
}

# Usage
status := Status::Active { user_id: 123, started_at: now() }
print(status.get_message())  # "User 123 is working"
if status.is_done() {
    finalize()
}
```

---

### 41. Generic Types (P2)

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

# Generic structs
struct Container<T> {
    value: T
    
    func get(self) -> T {
        return self.value
    }
}

# Type constraints
func process<T: Serializable>(item: T) {
    data := item.serialize()
}

# Multiple type parameters
func zip<A, B>(arr1: Array<A>, arr2: Array<B>) -> Array<[A, B]> {
    result := []
    for i in range(min(len(arr1), len(arr2))) {
        result := push(result, [arr1[i], arr2[i]])
    }
    return result
}
```

---

### 42. Union Types (P2)

**Status**: Research Phase  
**Estimated Effort**: Medium (2-3 weeks)

**Features**:
```ruff
# Union type annotations
func process(value: int | string | null) {
    match type(value) {
        case "int": print("Number: ${value}")
        case "string": print("Text: ${value}")
        case "null": print("Empty")
    }
}

# Type aliases
type UserID = int
type Handler = func(Request) -> Response
type JSONValue = int | float | string | bool | null | Array<JSONValue> | Dict<string, JSONValue>
```

---

### 43. Macros & Metaprogramming (P3)

**Status**: Research Phase  
**Estimated Effort**: Large (3-4 weeks)

**Why Interesting**: Compile-time code generation enables DSLs and zero-cost abstractions

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

### 44. Foreign Function Interface (FFI) (P3)

**Status**: Research Phase  
**Estimated Effort**: Large (3-4 weeks)

**Description**:  
Call external C libraries and system functions from Ruff.

**Planned Features**:
```ruff
# Load C library
lib := load_library("libmath.so")

# Declare external function
extern func cos(x: float) -> float from lib

# Call C function from Ruff
result := cos(3.14159)
```

---

### 45. AI/ML Built-in (P3)

**Status**: Research Phase  
**Estimated Effort**: Very Large (2-3 months)

**Why Unique**: Differentiate Ruff as "AI-native" language - ML without heavy dependencies

**Planned Features**:
```ruff
import ml

# Simple linear regression
model := ml.linear_regression()
model.train(x_train, y_train)
predictions := model.predict(x_test)
mse := model.evaluate(x_test, y_test)

# Neural network (basic)
nn := ml.neural_net(
    layers=[784, 128, 64, 10],
    activation="relu",
    output_activation="softmax"
)

nn.train(
    x_train, 
    y_train, 
    epochs=10, 
    batch_size=32,
    learning_rate=0.001
)

accuracy := nn.evaluate(x_test, y_test)

# Common ML tasks
data := ml.normalize(raw_data)  # Feature scaling
[x_train, x_test, y_train, y_test] := ml.train_test_split(x, y, test_size=0.2)
confusion := ml.confusion_matrix(y_true, y_pred)

# Clustering
kmeans := ml.kmeans(n_clusters=3)
labels := kmeans.fit_predict(data)

# Decision trees
tree := ml.decision_tree(max_depth=5)
tree.train(x_train, y_train)
predictions := tree.predict(x_test)
```

**Implementation**: Embed lightweight ML library (maybe SmartCore or linfa for Rust)

---

### 46. Additional Compilation Targets (P3)

**Status**: Research Phase  
**Estimated Effort**: Very Large (1-2 months per target)

**Options** (after bytecode VM in v0.8.0):
1. **WebAssembly** - Compile to WASM for browser/embedded use
2. **Native Code** - AOT compilation to native executables via LLVM
3. **JIT Compilation** - Just-in-time compilation for hot paths (100x+ speedup)

---

### 47. Automatic Memory Management (P3)
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

### 48. Graphics & GUI (P3)

**Status**: Research Phase  
**Estimated Effort**: Very Large (2-3 months)

**Description**:  
Graphics and GUI capabilities for visual applications.

**Terminal UI**:
```ruff
import tui

app := tui.App()
window := app.create_window(80, 24)

button := tui.Button {
    label: "Click Me",
    on_click: func() { print("Clicked!") }
}
window.add(button)
app.run()
```

**Canvas Drawing**:
```ruff
import graphics

canvas := graphics.Canvas(800, 600)
canvas.set_color(255, 0, 0)  # Red
canvas.draw_rect(100, 100, 200, 150)
canvas.draw_circle(400, 300, 50)
canvas.save("output.png")
```

---

### 49. WebAssembly Compilation (P3)

**Status**: Research Phase  
**Estimated Effort**: Very Large (2-3 months)

**Why Interesting**: Run Ruff in browsers, serverless, embedded systems

**Features**:
```bash
ruff build --target wasm script.ruff  # Compile to WASM
```

```html
<!-- Use in browser -->
<script type="module">
  import init, { run_ruff } from './script.wasm';
  await init();
  run_ruff();
</script>
```

---

## 🤝 Contributing

**Good First Issues** (v0.7.0):
- Timing functions (`current_timestamp`, `performance_now`)
- Type introspection (`type()`, `is_string()`, etc.)
- String formatting (`format()` function)
- Array utilities (`sort()`, `reverse()`, `unique()`)

**Medium Complexity** (v0.8.0):
- Destructuring
- Spread operator
- Enhanced error messages
- Standard library modules (arg parsing, compression, crypto)
- Result/Option types
- Bytecode instruction design

**Advanced Projects** (v0.9.0+):
- Async/await runtime
- Iterators & generators
- Language Server Protocol (LSP)
- Package manager & registry
- Code formatter and linter
- Debugger implementation
- Testing framework

---

## Version Strategy

**Current Approach**:
- **v0.6.0**: Production database support, HTTP streaming, collections ✅
- **v0.7.0**: Core language completion (foundation features + P2 quality-of-life) ✅
- **v0.8.0**: Performance (bytecode, 10x speedup) + modern syntax (destructuring, async)
- **v0.9.0**: Developer experience (LSP, package manager, tooling)
- **v1.0.0**: Production-ready, and competitive with other popular programming languages 🎉

**Philosophy**: Build the foundation first (language features), then performance, then tooling. This ensures LSP autocomplete and package manager are built on a complete, stable language.

**See Also**:
- [CHANGELOG.md](CHANGELOG.md) - Completed features and release history

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

**Good First Issues** (v0.7.0):
- String padding methods (`pad_left`, `pad_right`)
- String case conversion (`to_camel_case`, `to_snake_case`, `slugify`)
- Array methods (`take`, `skip`, `chunk`, `enumerate`)

**Medium Complexity** (v0.8.0):
- Destructuring
- Spread operator  
- Enhanced error messages ("Did you mean?")
- Standard library modules (arg parsing, compression, crypto)
- Result/Option types
- Bytecode instruction design

**Advanced Projects** (v0.9.0+):
- Async/await runtime
- Iterators & generators
- Language Server Protocol (LSP)
- Package manager & registry
- Code formatter and linter
- Debugger implementation
- Testing framework

---

## Version Strategy

**Current Approach**:
- **v0.6.0**: Production database support, HTTP streaming, collections ✅
- **v0.7.0**: Core language completion (foundation features + P2 quality-of-life) ✅
- **v0.8.0**: Performance (bytecode, 10x speedup) + modern syntax (destructuring, async)
- **v0.9.0**: Developer experience (LSP, package manager, tooling)
- **v1.0.0**: Production-ready, Go/Python competitive 🎉

**Philosophy**: Build the foundation first (language features), then performance, then tooling. This ensures LSP autocomplete and package manager are built on a complete, stable language.

**See Also**:
- [CORE_FEATURES_NEEDED.md](CORE_FEATURES_NEEDED.md) - v0.7.0 implementation guide
- [PATH_TO_PRODUCTION.md](PATH_TO_PRODUCTION.md) - Complete roadmap to world-class language
- [CHANGELOG.md](CHANGELOG.md) - Completed features and release history

---

*Last Updated: January 25, 2026*
