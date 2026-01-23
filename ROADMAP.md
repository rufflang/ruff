# Ruff Language - Development Roadmap

This roadmap outlines planned features and improvements for future versions of the Ruff programming language. For completed features, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.5.0 (in development)  
> **Next Planned Release**: v0.5.0

---

## 📦 High Priority (v0.4.0) - COMPLETED

### 1. Operator Overloading ✅

**Status**: **COMPLETED**  
**Completed**: January 2026

See [CHANGELOG.md](CHANGELOG.md#040---2026-01-23) for full details.

**Summary**: Structs can define custom behavior for operators using `op_` methods. Supports arithmetic (+, -, *, /, %) and comparison (==, !=, <, >, <=, >=) operators.

### 2. Struct Method Self Parameter ✅

**Status**: **COMPLETED**  
**Completed**: January 2026

See [CHANGELOG.md](CHANGELOG.md#unreleased) for full details.

**Summary**: Methods can now use explicit `self` parameter for method composition and builder patterns. Fully backward compatible with existing code.

---

## 🌟 Medium Priority (v0.5.0)

### 1. Unary Operator Overloading

**Status**: Planned  
**Estimated Effort**: Small (1-2 days)

**Description**:  
Extend operator overloading to support unary operators (prefix operators that work on a single operand).

**Planned Syntax**:
```ruff
struct Vector {
    x: float,
    y: float,
    
    # Unary negation: -v
    func op_neg(self) {
        return Vector { x: -self.x, y: -self.y };
    }
}

struct Flag {
    enabled: bool,
    
    # Logical not: !flag
    func op_not(self) {
        return Flag { enabled: !self.enabled };
    }
}

# Usage
v1 := Vector { x: 3.0, y: 4.0 };
v2 := -v1;  # Calls v1.op_neg(), result: Vector { x: -3.0, y: -4.0 }

flag := Flag { enabled: true };
negated := !flag;  # Calls flag.op_not(), result: Flag { enabled: false }
```

**Operators to Support**:
- `op_neg` for unary minus (`-value`)
- `op_not` for logical not (`!value`)

**Benefits**:
- Complete operator overloading support
- Natural syntax for mathematical types (vectors, matrices, complex numbers)
- Boolean flag types with custom negation logic

**Implementation Notes**:
- Constants and mapping function already defined in `ast.rs`
- Need to add unary expression evaluation in interpreter
- Parser already handles unary operators in expressions

---

### 2. HTTP Server & Networking

**Status**: Planned  
**Estimated Effort**: Large (2-3 weeks)

**Description**:  
Add HTTP server capabilities and networking functions to enable web APIs and services.

**Planned Functions**:
```ruff
# Create HTTP server
server := http_server(8080)

# Define routes
server.route("GET", "/", func(req) {
    return http_response(200, "Hello, World!")
})

server.route("GET", "/todos", func(req) {
    todos := load_todos()
    return json_response(200, todos)
})

server.route("POST", "/todos", func(req) {
    body := parse_json(req.body)
    save_todo(body)
    return json_response(201, {"success": true})
})

# Start listening
server.listen()

# HTTP client functions
response := http_get("https://api.example.com/data")
result := http_post("https://api.example.com/submit", {"key": "value"})
```

**Use Cases**:
- REST APIs
- Webhooks
- Microservices
- Web applications
- API integrations

---

### 3. REPL (Interactive Shell)

**Status**: Planned  
**Estimated Effort**: Medium (3-4 days)

**Features**:
- Interactive Read-Eval-Print Loop
- Multi-line input support
- Command history (up/down arrows)
- Tab completion
- Special commands (`:help`, `:clear`, `:quit`)

---

### 4. Concurrency & Async

**Status**: Planned  
**Estimated Effort**: Large (3-4 weeks)

**Description**:  
Lightweight concurrency with goroutine-style threads and channels.

**Planned Features**:
```ruff
# Spawn lightweight threads
spawn {
    print("Running in background")
    heavy_computation()
}

# Channels for communication
chan := channel()

spawn {
    result := fetch_data()
    chan.send(result)
}

data := chan.receive()  # Block until data received
print(data)

# Async/await syntax
async func fetch_user(id: int) {
    response := await http_get("/api/users/" + id)
    return parse_json(response.body)
}

user := await fetch_user(123)
print(user.name)

# Mutex for shared state
mut := mutex()
counter := 0

func increment() {
    mut.lock()
    counter := counter + 1
    mut.unlock()
}

# Spawn multiple workers
for i in range(10) {
    spawn { increment() }
}
```

**Use Cases**:
- Parallel processing
- Web servers handling multiple requests
- Background tasks
- Non-blocking I/O

---

### 6. Advanced Collections

**Status**: Planned  
**Estimated Effort**: Medium (2 weeks)

**Description**:  
Additional data structures beyond arrays and dictionaries.

**Planned Types**:
```ruff
# Sets - unique values
set := Set{1, 2, 3, 3, 2}  # {1, 2, 3}
set.add(4)
set.has(2)  # true
set.remove(1)

# Set operations
a := Set{1, 2, 3}
b := Set{2, 3, 4}
union := a.union(b)         # {1, 2, 3, 4}
intersect := a.intersect(b) # {2, 3}
diff := a.difference(b)     # {1}

# Queue - FIFO
queue := Queue{}
queue.enqueue("first")
queue.enqueue("second")
item := queue.dequeue()  # "first"

# Stack - LIFO
stack := Stack{}
stack.push(1)
stack.push(2)
top := stack.pop()  # 2

# Linked List
list := LinkedList{}
list.append(10)
list.prepend(5)
list.insert(1, 7)  # Insert at index 1

# Priority Queue
pq := PriorityQueue{}
pq.insert(5, "low priority")
pq.insert(10, "high priority")
highest := pq.pop()  # Returns "high priority"
```

---

### 7. Method Chaining & Fluent APIs

**Status**: Planned  
**Estimated Effort**: Medium (1 week)

**Description**:  
Enhanced syntax for chainable operations and optional access.

**Planned Features**:
```ruff
# Optional chaining - safely access nested properties
user := get_user(123)
email := user?.profile?.email  # Returns null if any part is null

# Pipe operator for data transformation
result := data
    |> filter(func(x) { return x > 10 })
    |> map(func(x) { return x * 2 })
    |> reduce(0, func(acc, x) { return acc + x })

# Builder pattern support
query := QueryBuilder()
    .select(["name", "email"])
    .from("users")
    .where("age", ">", 18)
    .order_by("name")
    .limit(10)
    .build()

# Null coalescing
value := user?.name ?? "Anonymous"  # Use "Anonymous" if name is null
```

---

### 8. Closures & Capturing

**Status**: Planned  
**Estimated Effort**: Medium (1 week)

**Description**:  
Proper closure support with variable capturing from outer scopes.

**Planned Features**:
```ruff
# Counter closure
func make_counter() {
    let count := 0
    return func() {
        count := count + 1
        return count
    }
}

counter1 := make_counter()
print(counter1())  # 1
print(counter1())  # 2
print(counter1())  # 3

counter2 := make_counter()
print(counter2())  # 1 (independent state)

# Partial application
func multiply(a, b) {
    return a * b
}

double := func(x) { return multiply(2, x) }
triple := func(x) { return multiply(3, x) }

print(double(5))  # 10
print(triple(5))  # 15

# Event handlers with closures
buttons := []
for i in range(5) {
    button := create_button()
    button.on_click(func() {
        print("Button ${i} clicked")  # Captures current i
    })
    buttons.push(button)
}
```

---

## 🎓 Professional Features (v0.6.0+)

### 9. Advanced Type System Features

**Status**: Research Phase  
**Estimated Effort**: Large (2-3 weeks)

**Planned Features**:
- Generic types: `Array<T>`, `Option<T>`, `Result<T, E>`
- Union types: `int | string | null`
- Type aliases: `type UserId = int`
- Optional chaining: `user?.profile?.email`
- Null safety with `Option<T>`

---

### 10. LSP (Language Server Protocol)

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

### 11. Macros & Metaprogramming

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

# Template expansion
macro create_getter_setter(field) {
    func get_${field}(self) {
        return self.${field}
    }
    
    func set_${field}(self, value) {
        self.${field} := value
    }
}

struct User {
    name: string
    email: string
}

create_getter_setter!(name)
create_getter_setter!(email)

# Domain-specific language support
macro html(content) {
    # Compile HTML-like syntax to function calls
}

page := html! {
    <div class="container">
        <h1>Welcome</h1>
        <p>Hello, World!</p>
    </div>
}
```

---

### 12. Database Support

**Status**: Planned  
**Estimated Effort**: Large (2-3 weeks)

**Description**:  
Built-in database connectivity starting with SQLite.

**Planned Functions**:
```ruff
# Connect to database
db := db_connect("sqlite:///data/app.db")

# Execute queries
db.exec("""
    CREATE TABLE users (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        email TEXT UNIQUE
    )
""")

# Query with results
users := db.query("SELECT * FROM users WHERE age > ?", [18])
for user in users {
    print(user["name"])
}

# Prepared statements
stmt := db.prepare("INSERT INTO users (name, email) VALUES (?, ?)")
stmt.exec(["Alice", "alice@example.com"])
stmt.exec(["Bob", "bob@example.com"])

# Transactions
db.begin()
try {
    db.exec("INSERT INTO users ...")
    db.exec("UPDATE accounts ...")
    db.commit()
} except err {
    db.rollback()
    throw err
}

# ORM-style interface
struct User {
    id: int
    name: string
    email: string
}

users := User.find_all()
user := User.find(123)
user.name := "Updated Name"
user.save()
```

**Supported Databases**:
- SQLite (built-in)
- PostgreSQL (extension)
- MySQL (extension)
- MongoDB (extension)

---

### 13. Serialization Formats

**Status**: Planned  
**Estimated Effort**: Medium (1-2 weeks)

**Description**:  
Support for multiple data serialization formats beyond JSON.

**Planned Formats**:
```ruff
# TOML
config := parse_toml(read_file("config.toml"))
toml_str := to_toml(config)

# YAML
data := parse_yaml(read_file("data.yaml"))
yaml_str := to_yaml(data)

# CSV
rows := parse_csv(read_file("data.csv"))
csv_str := to_csv(rows)

# XML
doc := parse_xml(read_file("data.xml"))
xml_str := to_xml(doc)

# MessagePack (binary)
bytes := to_msgpack(data)
data := from_msgpack(bytes)

# Custom serialization
struct User {
    name: string
    email: string
}

func User.serialize(self) {
    return {"n": self.name, "e": self.email}
}

func User.deserialize(data) {
    return User { name: data["n"], email: data["e"] }
}
```

---

### 14. Testing Enhancements

**Status**: Planned  
**Estimated Effort**: Medium (1-2 weeks)

**Description**:  
Advanced testing capabilities beyond basic test runner.

**Planned Features**:
```ruff
# Benchmarking
benchmark "Array operations" {
    setup {
        arr := range(1000)
    }
    
    test "map" {
        result := map(arr, func(x) { return x * 2 })
    }
    
    test "filter" {
        result := filter(arr, func(x) { return x % 2 == 0 })
    }
}

# Property-based testing
property "Addition is commutative" {
    forall a: int, b: int {
        assert(a + b == b + a)
    }
}

# Mocking
mock_api := mock({
    get_user: func(id) { return {"id": id, "name": "Test User"} }
})

test "User service" {
    service := UserService { api: mock_api }
    user := service.fetch_user(123)
    assert(user.name == "Test User")
    assert(mock_api.get_user.called_with(123))
}

# Code coverage
ruff test --coverage
# Output: 85% coverage (120/140 lines)
```

---

## 🏗️ Infrastructure (v0.7.0+)

### 15. Package Manager

**Status**: Planned  
**Estimated Effort**: Large (2-3 weeks)

**Features**:
- `ruff.toml` project configuration
- Dependency management
- Package registry
- Semantic versioning
- CLI commands: `ruff init`, `ruff add`, `ruff install`

---

### 16. Memory Management

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
- Manual memory hints: `free()`, `retain()`, `release()`

```ruff
# Automatic cleanup
func process_large_file() {
    data := read_file("huge.txt")  # Allocates memory
    result := process(data)
    return result
}  # data is automatically freed here

# Memory profiling
mem_before := memory_used()
process_data()
mem_after := memory_used()
print("Memory used: ${mem_after - mem_before} bytes")

# Leak detection
ruff run --detect-leaks program.ruff
```

---

### 17. Foreign Function Interface (FFI)

**Status**: Research Phase  
**Estimated Effort**: Large (3-4 weeks)

**Description**:  
Call external C libraries and system functions from Ruff.

**Planned Features**:
```ruff
# Load C library
lib := load_library("libmath.so")

# Declare external function
extern func cos(x: float) -> float from "libm.so"

# Call external function
result := cos(3.14)

# Rust integration
extern struct RustString from "librust_helper.so"
extern func rust_process_string(s: RustString) -> RustString

# System calls
extern func getpid() -> int from "libc.so"
pid := getpid()
print("Process ID: ${pid}")

# Callback functions
extern func qsort(arr: array, size: int, compare: func) from "libc.so"
```

---

### 18. Graphics & GUI

**Status**: Research Phase  
**Estimated Effort**: Very Large (2-3 months)

**Description**:  
Graphics and GUI capabilities for visual applications.

**Planned Features**:

**Terminal UI**:
```ruff
# Terminal-based interfaces
import tui

app := tui.App()
window := app.create_window(80, 24)

button := tui.Button { 
    text: "Click Me",
    on_click: func() { print("Clicked!") }
}

input := tui.TextInput { placeholder: "Enter name" }

window.add(button, 10, 5)
window.add(input, 10, 8)
app.run()
```

**2D Graphics**:
```ruff
# Simple 2D graphics primitives
import graphics

canvas := graphics.Canvas(800, 600)
canvas.set_color(255, 0, 0)  # Red
canvas.draw_rect(100, 100, 200, 150)
canvas.draw_circle(400, 300, 50)
canvas.draw_line(0, 0, 800, 600)
canvas.save("output.png")
```

**GUI Framework**:
```ruff
# Desktop application
import gui

app := gui.Application()
window := app.create_window("My App", 640, 480)

button := gui.Button("Click Me")
button.on_click(func() {
    gui.alert("Button clicked!")
})

layout := gui.VBox()
layout.add(button)
window.set_layout(layout)
app.run()
```

---

### 19. Compilation Targets

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
| JSON Support | High | v0.3.0 | Planned |
| Standard Library | Medium | v0.4.0 | ✅ Complete (see CHANGELOG) |
| Regular Expressions | Medium | v0.4.0 | ✅ Complete (see CHANGELOG) |
| Error Handling | Medium | v0.4.0 | ✅ Complete (see CHANGELOG) |
| Operator Overloading | Medium | v0.4.0 | Planned |
| HTTP/Networking | High | v0.5.0 | Planned |
| REPL | Medium | v0.5.0 | Planned |
| Concurrency/Async | High | v0.5.0 | Planned |
| Advanced Collections | Medium | v0.5.0 | Planned |
| Method Chaining | Medium | v0.5.0 | Planned |
| Closures | Medium | v0.5.0 | Planned |
| Advanced Types | Long Term | v0.6.0 | Research |
| LSP | Long Term | v0.6.0 | Planned |
| Macros | Long Term | v0.6.0 | Research |
| Database Support | Long Term | v0.6.0 | Planned |
| Serialization | Long Term | v0.6.0 | Planned |
| Testing Enhancements | Long Term | v0.6.0 | Planned |
| Package Manager | Long Term | v0.7.0 | Planned |
| Memory Management | Long Term | v0.7.0 | Research |
| FFI | Long Term | v0.7.0 | Research |
| Graphics/GUI | Long Term | v0.7.0+ | Research |
| Compilation Targets | Long Term | v0.8.0+ | Research |

---

## 🎯 Version Milestones

**v0.3.0 - "Functional"** (Current - Q1 2026)
- JSON support
- Enhanced comment types

**v0.4.0 - "Practical"** (Target: Q2 2026)
- Standard library enhancements (random, date/time, system, paths)
- Regular expressions
- Improved error handling with stack traces
- Operator overloading for structs

**v0.5.0 - "Concurrent"** (Target: Q3 2026)
- HTTP server & networking support
- Concurrency & async operations
- REPL implementation
- Advanced collections (Set, Queue, Stack, etc.)
- Method chaining and pipe operators
- Closures with proper variable capturing

**v0.6.0 - "Professional"** (Target: Q4 2026)
- LSP support with VS Code extension
- Advanced type system (generics, union types, null safety)
- Macros and metaprogramming
- Database support (SQLite, PostgreSQL)
- Multiple serialization formats (TOML, YAML, CSV, XML)
- Testing enhancements (benchmarks, property tests, mocking)

**v0.7.0 - "Ecosystem"** (Target: Q1 2027)
- Package manager with dependency management
- Memory management improvements
- Foreign Function Interface (FFI)
- Graphics and GUI capabilities

**v0.8.0 - "Performance"** (Target: Q2 2027)
- Compilation targets (bytecode, WASM, native, JIT)
- Performance optimizations
- Production hardening

---

## 🤝 Contributing

Want to help implement these features? Check out [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Good First Issues** (Small effort, high impact):
- Multi-line comments
- Random number generation
- Date/time formatting
- Path operations

**Medium Complexity** (Great learning opportunities):
- JSON support
- Regular expressions
- Operator overloading
- Advanced collections (Set, Queue, Stack)

**Advanced Projects** (For experienced contributors):
- REPL implementation
- Concurrency & async
- LSP support
- Database integration

---

*Last Updated: January 22, 2026*