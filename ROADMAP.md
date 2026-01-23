# Ruff Language - Development Roadmap

This roadmap outlines planned features and improvements for future versions of the Ruff programming language. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.5.1 (HTTP headers support added)  
> **Next Planned Release**: v0.6.0 (Closures, method chaining, and binary file support completed)

---

## Priority Levels

- **P1 (High)**: Core language features needed for real-world applications
- **P2 (Medium)**: Quality-of-life improvements and developer experience
- **P3 (Low)**: Nice-to-have features for advanced use cases

---

## v0.6.0 - Core Language Improvements

### Completed Features ✅

For detailed information about implemented features, see [CHANGELOG.md](CHANGELOG.md):
- **Closures & Variable Capturing** (P1) - Completed 2026-01-23
- **Method Chaining & Fluent APIs** (P1) - Completed 2026-01-23  
- **Binary File Support & HTTP Downloads** (P1) - Completed 2026-01-23

---

### Remaining Features for v0.6.0

### 1. Advanced Collections (P2)

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

### 2. Image Processing (P2)

**Status**: Planned  
**Estimated Effort**: Medium (1-2 weeks)

**Description**:  
Built-in image manipulation capabilities for common operations without external dependencies.

**Planned Features**:
```ruff
# Load and basic operations
img := load_image("photo.jpg")
width := img.width
height := img.height
format := img.format  # "jpeg", "png", "webp"

# Resize
resized := img.resize(800, 600)
thumbnail := img.resize(200, 200, "fit")  # Maintain aspect ratio

# Crop
cropped := img.crop(100, 100, 400, 400)  # x, y, width, height

# Rotate and flip
rotated := img.rotate(90)
flipped := img.flip("horizontal")

# Format conversion
img.save("output.png")  # Auto-converts JPEG -> PNG
img.save("output.webp", {"quality": 85})

# Filters and adjustments
brightened := img.adjust_brightness(1.2)
contrasted := img.adjust_contrast(1.1)
grayscale := img.to_grayscale()
blurred := img.blur(5)

# Watermarking
watermarked := img.add_text("© 2026", {
    "position": "bottom-right",
    "font": "Arial",
    "size": 20,
    "color": "white"
})

# Composite images
logo := load_image("logo.png")
final := img.overlay(logo, 10, 10)  # x, y position

# Batch operations
images := ["img1.jpg", "img2.jpg", "img3.jpg"]
for path in images {
    img := load_image(path)
    thumb := img.resize(200, 200, "fit")
    thumb.save("thumbs/" + basename(path))
}
```

**Use Cases**:
- AI image generation pipelines (resize, crop, watermark outputs)
- Thumbnail generation for galleries
- Image optimization for web (format conversion, compression)
- Social media image preparation (specific dimensions)
- Batch processing for e-commerce product photos

---

### 3. HTTP Authentication & Streaming (P1)

**Status**: Planned  
**Estimated Effort**: Medium (1-2 weeks)

**Description**:  
Advanced HTTP features for API integrations - OAuth, JWT, and streaming responses.

**Planned Features**:
```ruff
# OAuth 2.0 helper
oauth := OAuth2Client({
    "client_id": env("CLIENT_ID"),
    "client_secret": env("CLIENT_SECRET"),
    "auth_url": "https://provider.com/oauth/authorize",
    "token_url": "https://provider.com/oauth/token"
})

access_token := oauth.get_token()
response := http_get(url, {"Authorization": "Bearer " + access_token})

# JWT encoding/decoding
jwt_payload := {"user_id": 123, "exp": now() + 3600}
token := jwt_encode(jwt_payload, secret_key)
decoded := jwt_decode(token, secret_key)

# HTTP streaming for large responses
stream := http_get_stream("https://api.example.com/large-file")
while stream.has_data() {
    chunk := stream.read(8192)
    process_chunk(chunk)
}
stream.close()

# Server-Sent Events (SSE) for real-time updates
server.route("GET", "/events", func(request) {
    stream := sse_response()
    stream.send({"event": "message", "data": "Hello"})
    return stream
})
```

**Use Cases**:
- **AI APIs**: Authenticate with OpenAI, Anthropic, DeepSeek
- **Streaming**: Handle large AI responses without memory issues
- **Real-time**: Live updates for chat applications
- **Security**: JWT tokens for stateless authentication

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

### 5. Concurrency & Async/Await (P1) 🚀

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
