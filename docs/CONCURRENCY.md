# Ruff Language - Concurrency Model

This document describes Ruff's concurrency primitives and their internal implementation.

**Last Updated**: January 27, 2026  
**Version**: v0.9.0

---

## Table of Contents

1. [Overview](#overview)
2. [Threading Model](#threading-model)
3. [Async/Await Architecture](#asyncawait-architecture)
4. [Promises](#promises)
5. [Channels](#channels)
6. [Spawn Blocks](#spawn-blocks)
7. [Generators](#generators)
8. [Concurrency Patterns](#concurrency-patterns)
9. [Best Practices](#best-practices)
10. [Performance Considerations](#performance-considerations)

---

## Overview

Ruff provides multiple concurrency primitives to handle different use cases:

- **Async/Await**: Promise-based asynchronous execution (currently synchronous, see Phase 5 in ROADMAP)
- **Spawn Blocks**: True parallel execution with OS threads
- **Channels**: Thread-safe message passing between concurrent tasks
- **Generators**: Lazy evaluation with cooperative multitasking (yield/resume)

### Concurrency vs Parallelism

| Feature | Type | OS Threads | Use Case |
|---------|------|------------|----------|
| Async/Await | Concurrent | No (currently) | I/O-bound tasks |
| Spawn Blocks | Parallel | Yes | CPU-bound tasks |
| Generators | Concurrent | No | Lazy sequences |
| Channels | Either | Yes | Message passing |

---

## Threading Model

Ruff uses Rust's standard library threading model with **Arc<Mutex<>>** for shared mutable state.

### Thread Safety

All shared data uses **Arc** (Atomic Reference Counting) for safe cross-thread ownership:

```rust
// Environment shared across threads
pub struct Environment {
    parent: Option<Arc<Mutex<Environment>>>,
    variables: HashMap<String, Value>,
    functions: HashMap<String, Value>,
}

// Captured closure environment
Value::Function(
    params,
    body,
    Some(Arc::Mutex<Environment>>) // Shared environment
)
```

### Data Structures with Thread Safety

- **Channel**: `Arc<Mutex<(Sender<Value>, Receiver<Value>)>>`
- **Promise**: `Arc<Mutex<Receiver<Result<Value, String>>>>`
- **Generator**: `Arc<Mutex<Environment>>` for state
- **Database Connection**: `Arc<Mutex<Connection>>`

---

## Async/Await Architecture

### Current Implementation (v0.9.0)

**Important**: Async/await is currently **synchronous** - it wraps results in Promises but doesn't provide true concurrent I/O. Phase 5 (Tokio integration) will add true asynchronous execution.

### Async Function Definition

**Syntax**:
```ruff
async func fetch_data(url) {
    # Function body
    return data
}
```

**AST Representation**:
```rust
// src/ast.rs
pub enum Stmt {
    FuncDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        is_async: bool, // Marks async functions
    },
    // ...
}
```

**Runtime Value**:
```rust
// src/interpreter/value.rs
pub enum Value {
    AsyncFunction(
        Vec<String>,           // Parameters
        LeakyFunctionBody,     // Function body
        Option<Arc<Mutex<Environment>>>, // Captured closure env
    ),
    // ...
}
```

### Await Expression

**Syntax**:
```ruff
result := await fetch_data("https://api.example.com")
```

**Evaluation** (`src/interpreter/mod.rs:3876-3920`):
```rust
Expr::Await(promise_expr) => {
    let promise_value = self.eval_expr(promise_expr);
    
    match promise_value {
        Value::Promise { receiver, is_polled, cached_result } => {
            // Check cache first (promises are single-use)
            {
                let cached = cached_result.lock().unwrap();
                if let Some(ref result) = *cached {
                    return match result {
                        Ok(v) => v.clone(),
                        Err(e) => Value::Error(e.clone()),
                    };
                }
            }
            
            // Poll the promise (blocks until result available)
            let mut polled = is_polled.lock().unwrap();
            if !*polled {
                *polled = true;
                drop(polled);
                
                let result = receiver.lock().unwrap().recv();
                match result {
                    Ok(Ok(value)) => {
                        // Cache result
                        *cached_result.lock().unwrap() = Some(Ok(value.clone()));
                        value
                    }
                    Ok(Err(err)) => {
                        *cached_result.lock().unwrap() = Some(Err(err.clone()));
                        Value::Error(err)
                    }
                    Err(_) => Value::Error("Promise channel closed".to_string()),
                }
            } else {
                // Already polled - return cached result
                let cached = cached_result.lock().unwrap();
                match &*cached {
                    Some(Ok(v)) => v.clone(),
                    Some(Err(e)) => Value::Error(e.clone()),
                    None => Value::Error("Promise already consumed".to_string()),
                }
            }
        }
        other => other, // Not a promise, return as-is
    }
}
```

---

## Promises

Promises represent the eventual result of an asynchronous operation.

### Promise Structure

```rust
// src/interpreter/value.rs:367-371
Value::Promise {
    receiver: Arc<Mutex<std::sync::mpsc::Receiver<Result<Value, String>>>>,
    is_polled: Arc<Mutex<bool>>,
    cached_result: Arc<Mutex<Option<Result<Value, String>>>>,
}
```

**Fields**:
- `receiver`: Channel to receive async result
- `is_polled`: Ensures promise is only polled once
- `cached_result`: Stores result after polling (promises are single-use)

### Promise Creation

When calling an async function:

```ruff
# Define async function
async func compute() {
    return 42
}

# Call returns a Promise
promise := compute()  # Returns Promise immediately

# Await to get result
result := await promise  # Blocks until result available
```

**Implementation**:
```rust
// When calling async function
let (sender, receiver) = std::sync::mpsc::channel();

// Spawn thread to execute async body
std::thread::spawn(move || {
    let result = execute_async_body(body, env);
    let _ = sender.send(Ok(result));
});

// Return promise immediately
Value::Promise {
    receiver: Arc::new(Mutex::new(receiver)),
    is_polled: Arc::new(Mutex::new(false)),
    cached_result: Arc::new(Mutex::new(None)),
}
```

### Promise State Machine

```
┌─────────────┐
│  Created    │ ◄─── async function called
└──────┬──────┘
       │
       ▼ await expression
┌─────────────┐
│  Polling    │ ◄─── blocking on channel
└──────┬──────┘
       │
       ▼ result received
┌─────────────┐
│  Resolved   │ ◄─── result cached
└─────────────┘
```

---

## Channels

Channels provide thread-safe message passing using Rust's `std::sync::mpsc` (multi-producer, single-consumer).

### Channel Creation

**Syntax**:
```ruff
ch := channel()  # Create new channel
```

**Implementation** (`src/interpreter/native_functions/concurrency.rs:10-17`):
```rust
use std::sync::mpsc;

let (sender, receiver) = mpsc::channel();
let channel = Arc::new(Mutex::new((sender, receiver)));
Value::Channel(channel)
```

### Sending Messages

**Syntax**:
```ruff
ch.send(42)
ch.send("hello")
ch.send([1, 2, 3])
```

**Implementation**:
```rust
// Extract sender from channel
let (sender, _) = channel_mutex.lock().unwrap();
sender.send(value).map_err(|_| "Channel closed")?;
```

### Receiving Messages

**Syntax**:
```ruff
value := ch.receive()  # Blocks until message available
```

**Implementation**:
```rust
// Extract receiver from channel
let (_, receiver) = channel_mutex.lock().unwrap();
match receiver.recv() {
    Ok(value) => value,
    Err(_) => Value::Error("Channel closed".to_string()),
}
```

### Channel Example

```ruff
# Create channel
ch := channel()

# Producer thread
spawn {
    for i in range(5) {
        ch.send(i)
    }
}

# Consumer (main thread)
for i in range(5) {
    value := ch.receive()
    print("Received: ${value}")
}
```

**Output**:
```
Received: 0
Received: 1
Received: 2
Received: 3
Received: 4
```

---

## Spawn Blocks

Spawn blocks execute code in **true OS threads**, enabling CPU-bound parallelism.

### Syntax

```ruff
spawn {
    # Code executes in separate thread
    print("Hello from thread!")
}
```

### AST Representation

```rust
// src/ast.rs
pub enum Stmt {
    Spawn { body: Vec<Stmt> },
    // ...
}
```

### Implementation

**Evaluation** (`src/interpreter/mod.rs:2391-2420`):
```rust
Stmt::Spawn { body } => {
    let body_clone = body.clone();
    
    // Spawn OS thread
    std::thread::spawn(move || {
        // Create isolated interpreter for thread
        let mut thread_interpreter = Interpreter::new();
        
        // Execute body statements
        for stmt in body_clone.iter() {
            thread_interpreter.eval_stmt(stmt);
        }
    });
    
    // Main thread continues immediately (non-blocking)
}
```

### Important Characteristics

1. **Isolation**: Spawned code runs in isolated environment (no access to parent scope)
2. **Non-blocking**: Main thread continues immediately
3. **No return value**: Spawn blocks don't return values (use channels for communication)
4. **OS threads**: Each spawn creates a real OS thread (not green threads)

### Spawn with Channels

```ruff
ch := channel()

# Background computation
spawn {
    result := expensive_computation()
    ch.send(result)
}

# Main thread does other work
do_other_work()

# Get result when ready
result := ch.receive()
```

---

## Generators

Generators provide **lazy evaluation** with cooperative multitasking using `yield`.

### Generator Definition

**Syntax**:
```ruff
gen range(n) {
    i := 0
    while i < n {
        yield i
        i := i + 1
    }
}
```

### AST Representation

```rust
// src/ast.rs
pub enum Stmt {
    GeneratorDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    // ...
}

pub enum Expr {
    Yield(Option<Box<Expr>>),
    // ...
}
```

### Generator State

**Generator Definition** (before calling):
```rust
Value::GeneratorDef(
    Vec<String>,        // Parameters
    LeakyFunctionBody,  // Body with yield points
)
```

**Generator Instance** (after calling):
```rust
Value::Generator {
    params: Vec<String>,
    body: LeakyFunctionBody,
    env: Arc<Mutex<Environment>>, // Persistent state
    pc: usize,          // Program counter (resume position)
    is_exhausted: bool, // No more values to yield
}
```

### Yield Expression

**Evaluation**:
```rust
Expr::Yield(value_expr) => {
    let yielded = if let Some(expr) = value_expr {
        self.eval_expr(expr)
    } else {
        Value::Null
    };
    
    // Signal to generator executor to pause and return value
    Value::Return(Box::new(yielded))
}
```

### Generator Execution Flow

```
┌─────────────┐
│  Created    │ ◄─── gen_instance := generator()
└──────┬──────┘
       │
       ▼ .next() called
┌─────────────┐
│  Running    │ ◄─── Execute until yield
└──────┬──────┘
       │
       ▼ yield value
┌─────────────┐
│  Suspended  │ ◄─── Save pc and env
└──────┬──────┘
       │
       ▼ .next() called again
┌─────────────┐
│  Resumed    │ ◄─── Continue from pc
└──────┬──────┘
       │
       ▼ No more yields
┌─────────────┐
│  Exhausted  │ ◄─── is_exhausted = true
└─────────────┘
```

### Generator Usage

```ruff
gen := range(5)

loop {
    value := gen.next()
    if value == null {
        break
    }
    print(value)
}
```

**Output**:
```
0
1
2
3
4
```

---

## Concurrency Patterns

### Pattern 1: Fan-Out/Fan-In

**Use Case**: Distribute work across multiple threads, collect results.

```ruff
func fan_out_fan_in(items) {
    ch := channel()
    
    # Fan-out: Spawn worker threads
    for item in items {
        spawn {
            result := process(item)
            ch.send(result)
        }
    }
    
    # Fan-in: Collect results
    results := []
    for i in range(len(items)) {
        results.push(ch.receive())
    }
    
    return results
}
```

### Pattern 2: Pipeline

**Use Case**: Chain processing stages with channels.

```ruff
func pipeline(data) {
    ch1 := channel()
    ch2 := channel()
    
    # Stage 1: Input
    spawn {
        for item in data {
            ch1.send(item)
        }
    }
    
    # Stage 2: Transform
    spawn {
        loop {
            item := ch1.receive()
            if item == null { break }
            ch2.send(transform(item))
        }
    }
    
    # Stage 3: Output
    results := []
    for i in range(len(data)) {
        results.push(ch2.receive())
    }
    
    return results
}
```

### Pattern 3: Async Map

**Use Case**: Apply async function to array items concurrently.

```ruff
async func async_map(items, async_fn) {
    promises := []
    
    for item in items {
        promise := async_fn(item)
        promises.push(promise)
    }
    
    results := []
    for promise in promises {
        results.push(await promise)
    }
    
    return results
}

# Usage
async func fetch(url) {
    return http_get(url)
}

urls := ["https://api1.com", "https://api2.com", "https://api3.com"]
results := await async_map(urls, fetch)
```

### Pattern 4: Worker Pool

**Use Case**: Limit concurrent tasks to avoid resource exhaustion.

```ruff
func worker_pool(tasks, num_workers) {
    task_ch := channel()
    result_ch := channel()
    
    # Spawn workers
    for i in range(num_workers) {
        spawn {
            loop {
                task := task_ch.receive()
                if task == null { break }
                
                result := execute_task(task)
                result_ch.send(result)
            }
        }
    }
    
    # Send tasks
    spawn {
        for task in tasks {
            task_ch.send(task)
        }
    }
    
    # Collect results
    results := []
    for i in range(len(tasks)) {
        results.push(result_ch.receive())
    }
    
    return results
}
```

---

## Best Practices

### 1. Use Appropriate Concurrency Primitive

- **Async/Await**: I/O-bound operations (HTTP, files, databases)
- **Spawn**: CPU-bound operations (computation, image processing)
- **Generators**: Lazy sequences, infinite streams
- **Channels**: Communication between concurrent tasks

### 2. Avoid Shared Mutable State

**Bad** (prone to race conditions):
```ruff
counter := 0  # Shared between threads

spawn {
    counter := counter + 1  # Race condition!
}

spawn {
    counter := counter + 1  # Race condition!
}
```

**Good** (use channels):
```ruff
ch := channel()

spawn {
    ch.send(1)
}

spawn {
    ch.send(1)
}

counter := ch.receive() + ch.receive()  # Safe: 2
```

### 3. Always Close Channels

Ensure receivers don't block forever:

```ruff
ch := channel()

spawn {
    for i in range(10) {
        ch.send(i)
    }
    ch.send(null)  # Signal completion
}

loop {
    value := ch.receive()
    if value == null { break }
    process(value)
}
```

### 4. Handle Promise Errors

```ruff
async func fetch_data(url) {
    result := http_get(url)
    if type(result) == "Error" {
        return Err(result)
    }
    return Ok(result)
}

promise := fetch_data("https://api.example.com")
result := await promise

match result {
    Ok(data) => print("Success: ${data}"),
    Err(error) => print("Failed: ${error}"),
}
```

### 5. Limit Concurrent Tasks

Don't spawn unbounded threads:

```ruff
# Bad: Could spawn 1000 threads!
for i in range(1000) {
    spawn {
        process(i)
    }
}

# Good: Use worker pool pattern
results := worker_pool(range(1000), 10)  # Max 10 workers
```

---

## Performance Considerations

### Thread Creation Overhead

- Creating OS threads is expensive (~100µs per thread)
- Consider worker pools for many small tasks
- Generators have near-zero overhead (no threads)

### Channel Performance

- `mpsc` channels are lock-free and very fast (~50ns per message)
- Prefer channels over shared mutable state
- Batch messages to reduce overhead

### Async/Await Current Limitations

**v0.9.0**: Async functions execute **synchronously** - no true concurrency benefit yet.

**Workaround**: Use `spawn` for parallel I/O:
```ruff
ch := channel()

spawn {
    result := http_get("https://api1.com")
    ch.send(result)
}

spawn {
    result := http_get("https://api2.com")
    ch.send(result)
}

# Now truly concurrent
result1 := ch.receive()
result2 := ch.receive()
```

**Future (Phase 5)**: Tokio integration will provide true async I/O (10-100x faster for I/O-bound workloads).

### Generator Performance

- Generators have minimal overhead (just function call + state save)
- Perfect for large or infinite sequences
- No memory for unyielded values (lazy evaluation)

**Example**:
```ruff
# Eager: Allocates 1 million integers in memory
nums := range(1000000)

# Lazy: Generates one at a time
gen nums := range(1000000)
```

---

## Debugging Concurrency Issues

### Common Issues

1. **Deadlocks**: Two threads waiting for each other
2. **Race Conditions**: Shared mutable state without synchronization
3. **Channel Leaks**: Receiver blocked forever (sender died)

### Debugging Tips

**Add logging**:
```ruff
spawn {
    print("[Thread ${thread_id()}] Starting work")
    result := do_work()
    print("[Thread ${thread_id()}] Completed: ${result}")
    ch.send(result)
}
```

**Use timeouts** (future feature):
```ruff
# Will be added in Phase 5
result := ch.receive_timeout(5000)  # 5 second timeout
if result == null {
    print("ERROR: Timeout waiting for result")
}
```

**Test with smaller workloads**:
```ruff
# Debug with 10 items first
# worker_pool(range(1000000), 100)
worker_pool(range(10), 2)
```

---

## Future Improvements (Roadmap)

### Phase 5: True Async Runtime (v0.9.0)

- Tokio integration for true async I/O
- Non-blocking HTTP, file, database operations
- 10-100x faster for I/O-bound workloads
- Async timeout support

### Potential Future Features

- **Select expressions**: Wait on multiple channels
- **Async iterators**: `async for item in async_stream { ... }`
- **Cancellation**: Cancel async tasks mid-execution
- **Task priorities**: High/low priority task scheduling

---

## Further Reading

- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture overview
- [MEMORY.md](MEMORY.md) - Memory management and ownership
- [ROADMAP.md](../ROADMAP.md) - Planned concurrency features
- [examples/concurrency/](../examples/concurrency/) - Concurrency examples

---

**Questions?** Open an issue on GitHub or check the documentation.
