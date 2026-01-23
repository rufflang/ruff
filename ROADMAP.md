# Ruff Language - Development Roadmap

This roadmap outlines planned features and improvements for future versions of the Ruff programming language. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.5.1  
> **Next Planned Release**: v0.6.0

---

## Priority Levels

- **P1 (High)**: Core language features needed for real-world applications
- **P2 (Medium)**: Quality-of-life improvements and developer experience
- **P3 (Low)**: Nice-to-have features for advanced use cases

---

## v0.6.0 - Core Language Improvements

### 1. Closures & Variable Capturing (P1)

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
```

---

### 2. Method Chaining & Fluent APIs (P1)

**Status**: Planned  
**Estimated Effort**: Medium (1 week)

**Description**:  
Enhanced syntax for chainable operations and null-safe access.

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

# Null coalescing
value := user?.name ?? "Anonymous"  # Use "Anonymous" if name is null
```

---

### 3. Advanced Collections (P2)

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
```

---

### 4. Serialization Formats (P2)

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
```

---

## v0.7.0 - Production Database Support

### 5. PostgreSQL & MySQL (P1)

**Status**: Planned  
**Estimated Effort**: Large (3-4 weeks)

**Description**:  
Production database support for large-scale applications (restaurants, blogs, forums, e-commerce, etc.).

**Planned Features**:

**Unified Database Interface**:
```ruff
# PostgreSQL connection
db := db_connect("postgres", "host=localhost dbname=myapp user=admin password=secret")

# MySQL connection  
db := db_connect("mysql", "mysql://user:pass@localhost:3306/myapp")

# SQLite connection (existing)
db := db_connect("sqlite", "app.db")

# Same API works for all databases
db_execute(db, "INSERT INTO users (name) VALUES (?)", ["Alice"])
users := db_query(db, "SELECT * FROM users", [])
```

**Connection Pooling** (for high-traffic applications):
```ruff
# Create connection pool
pool := db_pool("postgres", "host=localhost dbname=myapp", {
    "min_connections": 5,
    "max_connections": 20
})

# Get connection from pool
db := pool.acquire()
users := db_query(db, "SELECT * FROM users", [])
pool.release(db)
```

**Transactions**:
```ruff
db_begin(db)
try {
    db_execute(db, "INSERT INTO orders ...", [order_data])
    db_execute(db, "UPDATE inventory ...", [inventory_data])
    db_commit(db)
} except err {
    db_rollback(db)
    throw err
}
```

**Target Use Cases**:
- 🍽️ Restaurant menu management systems
- 📝 Blog platforms with user accounts
- 💬 Forums and community sites
- 🛒 E-commerce applications
- 📊 Analytics dashboards
- 🏢 Business management tools

---

### 6. Concurrency & Async (P1)

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
```

**Use Cases**:
- Parallel processing
- Web servers handling multiple requests
- Background tasks
- Non-blocking I/O

---

## v0.8.0 - Developer Experience

### 7. LSP (Language Server Protocol) (P1)

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

### 8. Testing Enhancements (P2)

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
}

# Mocking
mock_api := mock({
    get_user: func(id) { return {"id": id, "name": "Test User"} }
})

test "User service" {
    service := UserService { api: mock_api }
    user := service.fetch_user(123)
    assert(user.name == "Test User")
}

# Code coverage
ruff test --coverage
# Output: 85% coverage (120/140 lines)
```

---

### 9. Package Manager (P2)

**Status**: Planned  
**Estimated Effort**: Large (2-3 weeks)

**Features**:
- `ruff.toml` project configuration
- Dependency management
- Package registry
- Semantic versioning
- CLI commands: `ruff init`, `ruff add`, `ruff install`

---

## v0.9.0+ - Advanced Features

### 10. Advanced Type System (P2)

**Status**: Research Phase  
**Estimated Effort**: Large (2-3 weeks)

**Planned Features**:
- Generic types: `Array<T>`, `Option<T>`, `Result<T, E>`
- Union types: `int | string | null`
- Type aliases: `type UserId = int`
- Null safety with `Option<T>`

---

### 11. Macros & Metaprogramming (P3)

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

### 12. Foreign Function Interface (FFI) (P3)

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
```

---

### 13. Memory Management (P3)

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

### 14. Graphics & GUI (P3)

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
    text: "Click Me",
    on_click: func() { print("Clicked!") }
}

window.add(button, 10, 5)
app.run()
```

**2D Graphics**:
```ruff
import graphics

canvas := graphics.Canvas(800, 600)
canvas.set_color(255, 0, 0)  # Red
canvas.draw_rect(100, 100, 200, 150)
canvas.draw_circle(400, 300, 50)
canvas.save("output.png")
```

---

### 15. Compilation Targets (P3)

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
- Advanced collections (Set, Queue, Stack)

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
