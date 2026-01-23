# Ruff Language - Development Roadmap

This roadmap outlines planned features and improvements for future versions of the Ruff programming language. For completed features, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.3.0-dev (January 2026)  
> **Next Planned Release**: v0.3.0

---

## 🔥 High Priority (v0.3.0)

### 1. Array Higher-Order Functions

**Status**: ✅ Complete (see CHANGELOG)  
**Completed**: January 22, 2026

**Description**:  
Functional programming operations on arrays.

**Implemented Functions**:
```ruff
# Map - transform each element
squared := map([1, 2, 3], func(x) { return x * x })  # [1, 4, 9]

# Filter - select elements
evens := filter([1, 2, 3, 4], func(x) { return x % 2 == 0 })  # [2, 4]

# Reduce - accumulate
sum := reduce([1, 2, 3], 0, func(acc, x) { return acc + x })  # 6

# Find - first matching element
first_even := find([1, 2, 3, 4], func(x) { return x % 2 == 0 })  # 2
```

---

### 2. Multi-Line and Doc Comments

**Status**: Planned  
**Estimated Effort**: Small (1-2 days)

**Syntax**:
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

---

### 3. JSON Support

**Status**: Planned  
**Estimated Effort**: Medium (3-4 days)

**Planned Functions**:
```ruff
# Parse JSON string to Ruff value
data := parse_json('{"name": "Alice", "age": 30}')
print(data["name"])  # Alice

# Convert Ruff value to JSON string
person := {"name": "Bob", "score": 95}
json_str := to_json(person)  # '{"name":"Bob","score":95}'
```

---

### 4. HTTP Server & Networking

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

## 🚀 Long Term (v0.5.0+)

### 5. REPL (Interactive Shell)

**Status**: Planned  
**Estimated Effort**: Medium (3-4 days)

**Features**:
- Interactive Read-Eval-Print Loop
- Multi-line input support
- Command history (up/down arrows)
- Tab completion
- Special commands (`:help`, `:clear`, `:quit`)

---

### 6. Advanced Type System Features

**Status**: Research Phase  
**Estimated Effort**: Large (2-3 weeks)

**Planned Features**:
- Generic types: `Array<T>`, `Option<T>`, `Result<T, E>`
- Union types: `int | string | null`
- Type aliases: `type UserId = int`
- Optional chaining: `user?.profile?.email`
- Null safety with `Option<T>`

---

### 7. LSP (Language Server Protocol)

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

### 8. Package Manager

**Status**: Planned  
**Estimated Effort**: Large (2-3 weeks)

**Features**:
- `ruff.toml` project configuration
- Dependency management
- Package registry
- Semantic versioning
- CLI commands: `ruff init`, `ruff add`, `ruff install`

---

### 12. Compilation Targets

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
| Boolean Type | High | v0.3.0 | ✅ Complete (see CHANGELOG) |
| Loop Control | High | v0.3.0 | ✅ Complete (see CHANGELOG) |
| String Interpolation | High | v0.3.0 | ✅ Complete (see CHANGELOG) |
| Array Higher-Order Fns | High | v0.3.0 | ✅ Complete (see CHANGELOG) |
| Enhanced Strings | Medium | v0.4.0 | ✅ Complete (see CHANGELOG) |
| Multi-line Comments | Low | v0.4.0 | Planned |
| JSON Support | Medium | v0.4.0 | Planned |
| HTTP/Networking | High | v0.5.0 | Planned |
| REPL | Long Term | v0.5.0 | Planned |
| Advanced Types | Long Term | v0.6.0 | Research |
| LSP | Long Term | v0.6.0 | Planned |
| Package Manager | Long Term | v0.7.0 | Planned |
| Compilation | Long Term | v0.8.0+ | Research |

---

## 🎯 Version Milestones

**v0.3.0 - "Practical"** (Target: Q1 2026)
- Boolean type as first-class value
- Loop control (break/continue/while)
- String interpolation

**v0.4.0 - "Expressive"** (Target: Q2 2026)
- Array higher-order functions
- JSON support
- Enhanced string functions

**v0.5.0 - "Interactive"** (Target: Q3 2026)
- HTTP server & networking support
- REPL implementation
- Improved error messages
- Better debugging tools

**v0.6.0 - "Professional"** (Target: Q4 2026)
- LSP support
- Advanced type system
- Comprehensive standard library

**v0.7.0 - "Ecosystem"** (2027)
- Package manager
- Community packages
- Documentation generator

---

## 🤝 Contributing

Want to help implement these features? Check out [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Priority features for community contributions:
- Enhanced string functions
- Array higher-order functions
- Multi-line comments
- JSON support

---

*Last Updated: January 22, 2026*