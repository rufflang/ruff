# Ruff Language - Architecture Documentation

This document provides a high-level overview of the Ruff programming language's architecture, design decisions, and internal structure.

**Last Updated**: January 26, 2026  
**Version**: v0.8.0 (v0.9.0 modularization in progress)

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture Diagram](#architecture-diagram)
3. [Data Flow](#data-flow)
4. [Core Components](#core-components)
5. [Interpreter Module Structure](#interpreter-module-structure)
6. [Execution Models](#execution-models)
7. [Memory Management](#memory-management)
8. [Concurrency Model](#concurrency-model)
9. [Design Decisions](#design-decisions)
10. [Future Architecture](#future-architecture)

---

## Overview

Ruff is a dynamically-typed, interpreted programming language implemented in Rust. It features:

- **Tree-walking interpreter** (current primary execution path)
- **Bytecode VM** (experimental, not yet default)
- **Async/await** with promise-based concurrency
- **Generators** and iterators
- **Lexical scoping** with closures
- **Pattern matching** and destructuring
- **Result/Option types** for error handling

The interpreter executes programs by traversing an Abstract Syntax Tree (AST) constructed from the source code.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Ruff Program (.ruff)                     │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
             ┌────────────────┐
             │     Lexer      │ ──────────► Tokens
             │  (lexer.rs)    │
             └────────┬───────┘
                      │
                      ▼
             ┌────────────────┐
             │     Parser     │ ──────────► AST (Stmt, Expr)
             │  (parser.rs)   │
             └────────┬───────┘
                      │
                ┌─────┴──────┐
                │            │
                ▼            ▼
       ┌────────────┐  ┌──────────────┐
       │Type Checker│  │   Compiler   │
       │(optional)  │  │(experimental)│
       └────────────┘  └──────┬───────┘
                              │
                              ▼
                       ┌─────────────┐
                       │  Bytecode   │
                       │  (vm.rs)    │
                       └─────────────┘
                      
                ┌─────────────────────┐
                │   Interpreter       │ ◄─── Current Default
                │ (interpreter/mod.rs)│
                └─────────┬───────────┘
                          │
                          ▼
                 ┌────────────────┐
                 │   Execution    │
                 │    (Value)     │
                 └────────────────┘
```

---

## Data Flow

### Source → Execution Pipeline

1. **Lexical Analysis (Lexer)**:
   - Input: Raw source code string
   - Output: Token stream
   - File: `src/lexer.rs`
   - Process: Character-by-character scanning, keyword recognition, operator identification

2. **Parsing (Parser)**:
   - Input: Token stream
   - Output: Abstract Syntax Tree (AST)
   - Files: `src/parser.rs`, `src/ast.rs`
   - Process: Recursive descent parsing with precedence climbing for operators
   - AST Nodes: `Stmt` (statements), `Expr` (expressions), `Pattern` (destructuring)

3. **Optional Type Checking**:
   - Input: AST
   - Output: Type errors or validated AST
   - File: `src/type_checker.rs`
   - Status: Optional, not enforced by default
   - Note: Type checker and runtime function registry are separate

4. **Interpretation (Tree-Walking)**:
   - Input: AST
   - Output: Program execution / side effects
   - Files: `src/interpreter/mod.rs` (14,802 lines, being modularized)
   - Process: 
     - `eval_stmt()` - Execute statements
     - `eval_expr()` - Evaluate expressions to Values
     - Environment management for scoping

5. **Alternative: Bytecode Compilation (Experimental)**:
   - Input: AST
   - Output: Bytecode instructions
   - Files: `src/compiler.rs`, `src/bytecode.rs`, `src/vm.rs`
   - Status: Implemented but not default execution path (marked with `#[allow(dead_code)]`)
   - Future: Will become primary execution path in v0.9.0+

---

## Core Components

### 1. Lexer (`src/lexer.rs`)

**Responsibility**: Convert source code into tokens

**Key Features**:
- Character-by-character scanning
- Multi-character operator recognition (`:=`, `==`, `...`, etc.)
- String interpolation support
- Line number tracking for error reporting

**Important**: 
- Keywords like `Ok`, `Err`, `Some`, `None` are identifiers, NOT keywords
- This allows them to be used in pattern matching
- Reserved keywords (if, else, func, etc.) cannot be function names

### 2. Parser (`src/parser.rs`)

**Responsibility**: Build AST from token stream

**Techniques**:
- Recursive descent parsing
- Pratt parser for expression precedence
- Lookahead for method call vs field access disambiguation

**AST Structure** (`src/ast.rs`):
```rust
pub enum Stmt {
    Let { pattern: Pattern, value: Box<Expr> },
    Assignment { target: Box<Expr>, value: Box<Expr> },
    FuncDef { name: String, params: Vec<String>, body: Vec<Stmt>, is_generator: bool },
    Return(Box<Expr>),
    If { condition: Box<Expr>, then_block: Vec<Stmt>, else_block: Option<Vec<Stmt>> },
    While { condition: Box<Expr>, body: Vec<Stmt> },
    For { var: String, iterable: Box<Expr>, body: Vec<Stmt> },
    Match { value: Box<Expr>, cases: Vec<(Pattern, Vec<Stmt>)> },
    // ... more variants
}

pub enum Expr {
    Number(f64),
    Str(String),
    Identifier(String),
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Call { func: Box<Expr>, args: Vec<Expr> },
    MethodCall { object: Box<Expr>, method: String, args: Vec<Expr> },
    // ... more variants
}
```

### 3. Interpreter (`src/interpreter/mod.rs`)

**Responsibility**: Execute AST and manage runtime state

**Current Structure** (Being Modularized in v0.9.0):
- **14,802 lines** in single file (too large)
- Contains: Value enum, Environment, Interpreter, builtins, test runner
- Target: Split into focused modules (~500-2000 lines each)

**Key Components**:

#### Value Enum (Runtime Values)
```rust
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
    Array(Vec<Value>),
    Dict(HashMap<String, Value>),
    Function(Vec<String>, LeakyFunctionBody, Option<Arc<Mutex<Environment>>>),
    AsyncFunction(...),
    Generator {...},
    Iterator {...},
    Promise {...},
    Result { is_ok: bool, value: Box<Value> },
    Option { is_some: bool, value: Box<Value> },
    // ... 30+ variants total
}
```

#### Environment (Scoping)
```rust
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}
```
- Stack-based lexical scoping
- `push_scope()` / `pop_scope()` for blocks
- `get()` searches from inner to outer scopes
- `set()` updates existing variable or creates in current scope

#### Interpreter Struct
```rust
pub struct Interpreter {
    pub env: Environment,
    return_value: Option<Value>,
    // ... builtin functions, async runtime, module loader, etc.
}
```

**Execution Methods**:
- `eval_stmt(&mut self, stmt: &Stmt)` - Execute a statement
- `eval_expr(&mut self, expr: &Expr) -> Value` - Evaluate expression
- `register_builtins(&mut self)` - Register native functions (~600 lines)

### 4. Built-in Functions (`src/builtins.rs`)

**Responsibility**: Provide standard library functions

**Categories**:
- I/O: `read_file`, `write_file`, `http_get`, `http_post`
- Collections: `map`, `filter`, `reduce`, `zip`, `chunk`
- Strings: `split`, `join`, `trim`, `replace`, `regex_match`
- Math: `sqrt`, `pow`, `abs`, `floor`, `ceil`
- System: `env`, `args`, `exec`, `spawn_process`
- Database: `db_connect`, `db_query`, `db_transaction`
- Async: `sleep`, `timeout`, `race`
- Testing: `assert_equal`, `assert_true`, `run_tests`

**Integration**: Called via `Value::NativeFunction(name)` with dispatch in interpreter

### 5. Module System (`src/module.rs`)

**Responsibility**: Load and manage imported modules

**Features**:
- `import "path/to/module.ruff"` statement
- Caches loaded modules to prevent re-execution
- Module exports tracked in environment
- Circular import detection

---

## Interpreter Module Structure

### Previous Structure (v0.8.0 and earlier)
```
src/
├── interpreter.rs          (14,802 lines - MONOLITHIC)
├── lexer.rs
├── parser.rs
├── ast.rs
├── builtins.rs
├── compiler.rs (experimental)
├── bytecode.rs (experimental)
└── vm.rs (experimental)
```

### Current Structure (v0.9.0 - Phase 2 Complete, January 26, 2026)
```
src/interpreter/
├── mod.rs              (~14,071 lines) - Core Interpreter + call_native_function_impl + register_builtins
├── value.rs            (~497 lines)    - Value enum with 30+ variants, DB types
├── environment.rs      (~109 lines)    - Environment struct with lexical scoping
├── control_flow.rs     (~22 lines)     - ControlFlow enum for break/continue
├── test_runner.rs      (~230 lines)    - TestRunner, TestCase, TestResult, TestReport
└── legacy_full.rs      (~14,754 lines) - Backup of original monolithic file
```

**Progress Summary**:
- ✅ Phase 1 (Jan 26): Extracted Value enum (500 lines) and Environment struct (110 lines) → -517 lines
- ✅ Phase 2 (Jan 26): Extracted ControlFlow enum (22 lines) and test framework (230 lines) → -214 lines
- ✅ **Total reduction**: 14,802 → 14,071 lines (-731 lines, ~5% reduction)
- ✅ Zero compilation errors, minimal warnings
- ✅ All functionality preserved, tests passing

**Key Design Constraints**:
- The 5,700-line `call_native_function_impl` method must remain in mod.rs due to Rust's requirement that methods with `&mut self` stay in the same impl context
- The 564-line `register_builtins` method remains in mod.rs as it directly mutates `self.env`
- These methods are well-organized with category comments and are not extractable without significant architectural refactoring

**Benefits Achieved**:
- Easier navigation with IDE "Go to File"
- Parallel compilation of modules
- Clear separation of concerns
- Easier code review (smaller diffs)
- Reduced mental overhead per file
- Improved onboarding for new contributors

**Status**: ✅ Phase 2 Complete - Further modularization requires architectural changes to interpreter design

---

## Execution Models

### 1. Tree-Walking Interpreter (Current Default)

**How It Works**:
1. Parse source to AST
2. Recursively traverse AST nodes
3. Execute statements, evaluate expressions
4. Return values bubble up the tree

**Pros**:
- Simple to implement and debug
- Direct mapping from source to execution
- Easy to add new features

**Cons**:
- **Slow**: 100-500x slower than compiled languages
- Repeated AST traversal overhead
- No optimization passes

**Performance**: Acceptable for scripts, not production workloads

### 2. Bytecode VM (Experimental, Future Default)

**How It Works**:
1. Parse source to AST
2. Compile AST to bytecode instructions
3. VM executes bytecode with stack machine
4. Optimizations during compilation

**Pros**:
- 10-50x faster than tree-walking
- Optimization opportunities (constant folding, dead code elimination)
- Smaller memory footprint (no AST at runtime)

**Cons**:
- More complex implementation
- Harder to debug
- Startup overhead for compilation

**Status**: 
- Compiler, bytecode, and VM exist (`src/compiler.rs`, `src/bytecode.rs`, `src/vm.rs`)
- Marked with `#[allow(dead_code)]` - not actively used
- **Roadmap**: Complete in v0.9.0 Task #28

### 3. JIT Compilation (Future)

**Planned Features** (v0.9.0+):
- Detect hot code paths at runtime
- Compile bytecode to native machine code (Cranelift or LLVM)
- Type specialization for common types
- Expected: 100-500x faster than tree-walking, competitive with Go

---

## Memory Management

### Ownership Model

Ruff uses **Rust's ownership** for memory safety:
- Values are `Clone` - copied when needed
- Environments wrapped in `Arc<Mutex<>>` for shared access
- Function closures capture environments via `Arc<Mutex<Environment>>`

### Value Lifecycle

```rust
let x := [1, 2, 3]     // Array allocated
let y := x             // Array cloned (deep copy)
// x and y are independent
```

**Important**: Ruff does NOT use reference counting for user-visible references. Values are copied on assignment.

### LeakyFunctionBody Issue

**Problem**: 
- Function bodies are `Vec<Stmt>`
- Statements contain nested `Vec<Stmt>` (in loops, conditionals)
- Recursive Drop causes stack overflow on deeply nested ASTs

**Current Workaround**:
```rust
pub struct LeakyFunctionBody(ManuallyDrop<Arc<Vec<Stmt>>>);
```
- Intentionally leaks memory to avoid stack overflow
- Memory reclaimed by OS at program exit
- **Not ideal** but prevents crashes

**Future Fix** (Roadmap Task #29):
- Implement iterative Drop traversal
- Or use arena allocation
- Or flatten statement structures

### Arc<Mutex<>> Refactor (v0.8.0)

**Change**: Replaced `Rc<RefCell<>>` with `Arc<Mutex<>>`

**Reason**: Enable thread-safe async/await

**Impact**:
- All environments, channels, connections now thread-safe
- Enables true concurrency (not just cooperative multitasking)
- Required for async functions running in separate threads

---

## Concurrency Model

### 1. Async/Await (v0.8.0+)

**Architecture**:
```rust
async func fetch_data(id) {
    // Runs in separate thread
    let result := http_get("/api/data/${id}")
    return result
}

let promise := fetch_data(42)    // Returns immediately
let data := await promise         // Blocks until complete
```

**Implementation**:
- Async functions return `Value::Promise`
- Promise wraps `mpsc::Receiver<Result<Value, String>>`
- Async body executes in new thread via `std::thread::spawn`
- `await` blocks on receiver until result available

**Thread Safety**: All shared state uses `Arc<Mutex<>>`

### 2. Spawn Blocks

```ruff
spawn {
    // Runs in separate thread
    process_large_file("data.txt")
}
```

**Implementation**: Creates detached thread, no return value

### 3. Channels

```ruff
let (tx, rx) := channel()

spawn {
    tx.send("Hello from thread!")
}

let msg := rx.recv()  // Blocks until message
```

**Implementation**: Wraps `std::sync::mpsc::channel()`

### 4. Generators

```ruff
func* numbers(n) {
    for i in range(n) {
        yield i
    }
}

for num in numbers(5) {
    print(num)
}
```

**Implementation**:
- `Value::Generator` stores: params, body, environment, program counter
- `yield` returns value and saves state
- Next call resumes from program counter
- **Status**: Syntax parses, execution needs refinement

---

## Design Decisions

### 1. Why Tree-Walking Interpreter First?

**Decision**: Implement tree-walking interpreter before bytecode VM

**Rationale**:
- Faster to implement (get language working sooner)
- Easier to debug and add features
- Establish language semantics before optimizing
- Prove language viability before investing in performance

**Trade-off**: Slow execution, but that's acceptable for v0.x

### 2. Why Dynamic Typing?

**Decision**: No compile-time type checking by default

**Rationale**:
- Simpler for scripting and prototyping
- Faster iteration during development
- Python/Ruby/JavaScript model familiar to users

**Future**: Optional type annotations (v0.10.0+) for performance and tooling

### 3. Why Lexical Scoping?

**Decision**: Inner scopes can access outer scope variables

**Rationale**:
- Matches JavaScript, Python behavior
- Enables closures naturally
- Intuitive for most programmers

**Implementation**: Environment scope stack

### 4. Why LeakyFunctionBody?

**Decision**: Accept memory leak to prevent stack overflow

**Rationale**:
- Stack overflow is worse than leak (crashes vs leaks at exit)
- Temporary workaround until proper fix (Task #29)
- OS reclaims memory at program exit anyway

**Future**: Eliminate with iterative drop or arena allocation

### 5. Why Separate Async Runtime?

**Decision**: Thread-per-async-function model, not async runtime like Tokio

**Rationale**:
- Simpler implementation (std::thread)
- True parallelism on multi-core
- No need for executor, no event loop complexity

**Trade-off**: More memory per task, but acceptable for Ruff's use cases

---

## Future Architecture

### v0.9.0: Modularization + VM

**Goals**:
1. Split interpreter into focused modules
2. Make bytecode VM the default execution path
3. Improve error messages with source locations
4. Comprehensive architecture documentation

**Timeline**: Q2 2026 (2-3 months)

### v0.10.0+: Optional Typing + JIT

**Goals**:
1. Add type annotation syntax
2. Optional runtime type checking
3. JIT compilation for typed code (Cranelift or LLVM)
4. Performance competitive with Go

**Timeline**: TBD (exploratory)

### v1.0: Production-Ready

**Success Criteria**:
- Performance within 2-5x of Go
- Comprehensive standard library
- Excellent error messages
- Professional tooling (LSP, formatter, linter)
- Battle-tested in real applications

---

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- How to add a new built-in function
- How to add a new language feature
- Code style guidelines
- Testing requirements

---

## Further Reading

- [ROADMAP.md](../ROADMAP.md) - Planned features and timeline
- [CHANGELOG.md](../CHANGELOG.md) - Version history
- [notes/GOTCHAS.md](../notes/GOTCHAS.md) - Common pitfalls and lessons learned

---

**Questions?** Open an issue on GitHub or reach out to the maintainers.
