# Ruff Language - Development Roadmap

This roadmap outlines planned features and improvements for future versions of the Ruff programming language. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.6.0 (Advanced Collections completed)  
> **Next Planned Release**: v0.6.0 (Production database support and advanced features)

---

## Priority Levels

- **P1 (High)**: Core language features needed for real-world applications
- **P2 (Medium)**: Quality-of-life improvements and developer experience
- **P3 (Low)**: Nice-to-have features for advanced use cases

---

## v0.6.0 - HTTP Authentication & Streaming ✅ COMPLETED

All features for v0.6.0 HTTP Authentication & Streaming have been completed! See [CHANGELOG.md](CHANGELOG.md) for full details:

- **JWT Encoding & Decoding** (P1) - Completed 2026-01-23
- **OAuth2 Authorization Flow** (P1) - Completed 2026-01-23  
- **HTTP Streaming for Large Files** (P1) - Completed 2026-01-23
- **HTML Response with Content-Type Header** (P1) - Completed 2026-01-23
- **Advanced Collections (Set, Queue, Stack)** (P2) - Completed 2026-01-23

---

## v0.6.0 - Serialization Formats ✅ COMPLETED

**Status**: Completed 2026-01-23

All serialization format features have been implemented! See [CHANGELOG.md](CHANGELOG.md) for full details:

- **TOML Support** (P2) - Completed 2026-01-23
- **YAML Support** (P2) - Completed 2026-01-23
- **CSV Support** (P2) - Completed 2026-01-23

---

## v0.6.0 - Image Processing ✅ COMPLETED

### 1. Image Processing (P2) ✅ COMPLETED

**Status**: Completed 2026-01-23  
**Effort**: Medium (1-2 weeks)

**Description**:  
Built-in image manipulation capabilities for common operations without external dependencies.

**Implemented Features**:
- ✅ `load_image(path)` - Load images from files (JPEG, PNG, WebP, GIF, BMP)
- ✅ `img.width`, `img.height`, `img.format` - Image properties
- ✅ `img.resize(width, height)` - Resize to exact dimensions
- ✅ `img.resize(width, height, "fit")` - Resize maintaining aspect ratio
- ✅ `img.crop(x, y, width, height)` - Crop rectangular region
- ✅ `img.rotate(degrees)` - Rotate 90, 180, or 270 degrees
- ✅ `img.flip(direction)` - Flip horizontal or vertical
- ✅ `img.to_grayscale()` - Convert to grayscale
- ✅ `img.blur(sigma)` - Apply Gaussian blur
- ✅ `img.adjust_brightness(factor)` - Adjust brightness
- ✅ `img.adjust_contrast(factor)` - Adjust contrast
- ✅ `img.save(path)` - Save with automatic format conversion
- ✅ Method chaining support for complex workflows
- ✅ Comprehensive test suite in `tests/image_processing_test.ruff`
- ✅ Example code in `examples/image_processing.ruff`

**Use Cases**:
- AI image generation pipelines (resize, crop, watermark outputs)
- Thumbnail generation for galleries and listings
- Image optimization for web (format conversion, compression)
- Social media image preparation (specific dimensions, filters)
- Batch processing for e-commerce product photos
- Automated image enhancement workflows

### 2. Concurrency & Async/Await (P1) 🚀

**Status**: Planned  
**Estimated Effort**: Large (3-4 weeks)

**Description**:  
Lightweight concurrency for parallel API calls, background tasks, and non-blocking I/O operations.

**⚠️ CRITICAL FOR AI TOOLS**: This feature is essential for production-ready AI applications. Without it, multi-model comparisons are 3x slower and batch generation is impractical.

**Planned Features**:
```ruff
# Async/await for non-blocking operations
async func fetch_data(url) {
    response := await http_get(url)
    return parse_json(response.body)
}

# Parallel API calls - critical for AI tools!
async func compare_models(prompt) {
    # All three calls happen simultaneously
    gpt_task := fetch_data("https://api.openai.com/v1/chat/completions")
    claude_task := fetch_data("https://api.anthropic.com/v1/messages")
    deepseek_task := fetch_data("https://api.deepseek.com/v1/chat/completions")
    
    # Wait for all to complete
    results := await all([gpt_task, claude_task, deepseek_task])
    return results
}

# Spawn lightweight threads for background work
spawn {
    print("Processing in background...")
    process_large_file()
}

# Channels for thread communication
chan := channel()

spawn {
    result := expensive_computation()
    chan.send(result)
}

data := chan.receive()  # Block until data received

# Timeout for async operations
try {
    result := await timeout(fetch_data(url), 5000)  # 5 second timeout
} except TimeoutError {
    print("Request timed out")
}
```

**Use Cases**:
- **AI Tools**: Parallel API calls to multiple providers (OpenAI, DeepSeek, Claude)
- **Batch Processing**: Generate 100+ pieces of content simultaneously
- **Web Servers**: Handle multiple HTTP requests concurrently
- **Background Tasks**: Process files while accepting user input
- **Data Pipelines**: Parallel data fetching and processing

**Why P1 for v0.6.0**:
This is CRITICAL for AI tool development - without it, multi-model comparison takes 3x longer, batch generation is slow, and the tools are not production-ready.

**Implementation Priority**:
1. Basic async/await syntax
2. Parallel HTTP requests (for AI APIs)
3. Background tasks (spawn)
4. Channels (for thread communication)
5. Timeout handling

---

## v0.7.0 - Production Database Support

### 9. PostgreSQL & MySQL (P1)

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

## v0.8.0 - Developer Experience

### 10. LSP (Language Server Protocol) (P1)

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

### 11. Testing Enhancements (P2)

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

### 12. Package Manager (P2)

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

### 13. Advanced Type System (P2)

**Status**: Research Phase  
**Estimated Effort**: Large (2-3 weeks)

**Planned Features**:
- Generic types: `Array<T>`, `Option<T>`, `Result<T, E>`
- Union types: `int | string | null`
- Type aliases: `type UserId = int`
- Null safety with `Option<T>`

---

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
extern func cos(x: float) -> float from "libm.so"

# Call external function
result := cos(3.14)
```

---

### 16. Memory Management (P3)

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
