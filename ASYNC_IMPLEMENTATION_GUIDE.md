# Implementation Guide: Make Async/Await Actually Work in Ruff

## Problem Statement

Ruff currently has `async func` and `await` **syntax**, but async functions execute **synchronously**. They don't provide any concurrency or parallelism. This makes I/O-bound workloads extremely slow (SSG benchmark: 55 seconds for 10K files vs 0.05s in Python).

**Goal**: Make async/await truly asynchronous using Tokio runtime for real concurrency.

---

## Current State Analysis

### What Works (Syntax)
```ruff
async func fetch_data(id) {
    let result := "Data for " + id
    return result
}

let promise := fetch_data(1)
let result := await promise
```

### What's Wrong (Implementation)

From `src/vm.rs` lines 1025-1110:

```rust
// Note: Async functions execute synchronously in the VM.
// The return value will be wrapped in a Promise by the Return opcode
```

The async functions:
1. Execute **synchronously** (blocking) - no concurrency at all
2. Return a Promise that's **already resolved** (no async work happens)
3. `await` just unwraps the Promise immediately (no polling, no yielding)

This is currently just syntactic sugar with zero performance benefit.

---

## Implementation Plan

### Phase 1: Add Tokio Runtime Integration

**Files to Modify:**
- `src/vm.rs` - VM execution loop
- `src/main.rs` - Initialize tokio runtime
- `Cargo.toml` - Add tokio dependencies

**Tasks:**

1. **Add Tokio to Cargo.toml**
```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
```

2. **Wrap VM execution in Tokio runtime**

In `src/main.rs`:
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // existing main code
}
```

3. **Make VM execution async-aware**

Add field to VM struct:
```rust
pub struct VM {
    // ... existing fields
    runtime_handle: tokio::runtime::Handle,
    async_tasks: Vec<tokio::task::JoinHandle<Result<Value, String>>>,
}
```

---

### Phase 2: Make Async Functions Actually Async

**Current Code (synchronous):**
```rust
// In OpCode::Call
if is_async_function {
    self.call_bytecode_function(function, args)?;
    // Wraps result in already-resolved Promise
}
```

**Target Code (asynchronous):**
```rust
// In OpCode::Call
if is_async_function {
    // Spawn async task on tokio runtime
    let handle = self.runtime_handle.spawn(async move {
        // Execute function in background
        self.call_bytecode_function_async(function, args).await
    });
    
    // Return Promise that will be resolved when task completes
    let promise = Value::Promise {
        handle: Some(handle),
        is_polled: Arc::new(Mutex::new(false)),
        cached_result: Arc::new(Mutex::new(None)),
    };
    self.stack.push(promise);
}
```

**Key Changes:**
1. Don't execute async functions immediately
2. Spawn them as tokio tasks
3. Return Promise with JoinHandle
4. Let await actually poll the task

---

### Phase 3: Make Await Actually Wait

**Current Code:**
```rust
OpCode::Await => {
    let promise = self.stack.pop()?;
    // Just unwraps already-resolved Promise
    let result = promise.unwrap_promise_value()?;
    self.stack.push(result);
}
```

**Target Code:**
```rust
OpCode::Await => {
    let promise = self.stack.pop()?;
    
    match promise {
        Value::Promise { handle: Some(h), .. } => {
            // Actually wait for task to complete
            let result = self.runtime_handle.block_on(h)?;
            self.stack.push(result);
        }
        Value::Promise { cached_result, .. } => {
            // Already resolved - use cached
            self.stack.push(cached_result.lock().unwrap().clone()?);
        }
        _ => return Err("Cannot await non-promise".to_string()),
    }
}
```

---

### Phase 4: Make Built-in I/O Operations Async

**File Operations to Convert:**

```rust
// Current (blocking):
pub fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| e.to_string())
}

// Target (async):
pub async fn read_file(path: &str) -> Result<String, String> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| e.to_string())
}
```

**Functions to Convert:**
- `read_file()` → tokio::fs::read_to_string
- `write_file()` → tokio::fs::write
- `list_dir()` → tokio::fs::read_dir
- HTTP requests (if any) → tokio-based client

---

### Phase 5: Add Task Spawning Support

**New Syntax:**
```ruff
# Option A: spawn keyword
spawn {
    process_file("file1.md")
}

# Option B: async block
let task := async {
    process_file("file1.md")
}
```

**Implementation:**

Add `spawn()` built-in function:
```rust
pub fn spawn_task(vm: &mut VM, closure: Value) -> Result<Value, String> {
    let handle = vm.runtime_handle.spawn(async move {
        // Execute closure
        vm.execute_value(closure).await
    });
    
    Ok(Value::Promise { handle: Some(handle), ... })
}
```

---

### Phase 6: Add Parallel Utilities

**Helper Functions:**

```ruff
# Built-in: await_all
let results := await_all([task1, task2, task3])

# Built-in: par_map
let results := par_map(files, func(f) {
    return process_file(f)
})
```

**Implementation:**
```rust
pub async fn await_all(vm: &mut VM, promises: Vec<Value>) -> Result<Vec<Value>, String> {
    let handles = promises.into_iter()
        .map(|p| extract_handle(p))
        .collect();
    
    let results = futures::future::join_all(handles).await;
    Ok(results)
}
```

---

## Testing Strategy

### Unit Tests

**Test 1: Async Function Returns Promise**
```ruff
async func test() { return 42 }
let p := test()
assert(type(p) == "Promise")
```

**Test 2: Await Resolves Promise**
```ruff
async func test() { return 42 }
let result := await test()
assert(result == 42)
```

**Test 3: Concurrent Execution**
```ruff
async func slow_task(id) {
    # Simulated delay
    return id
}

let start := performance_now()
let tasks := [slow_task(1), slow_task(2), slow_task(3)]
let results := await_all(tasks)
let elapsed := performance_now() - start

# Should be faster than sequential (< 3x single task time)
assert(len(results) == 3)
```

**Test 4: File I/O Concurrency**
```ruff
async func read_all_files(files) {
    let tasks := []
    for file in files {
        push(tasks, read_file(file))
    }
    return await_all(tasks)
}

let start := performance_now()
let contents := await read_all_files(["f1.txt", "f2.txt", "f3.txt"])
let elapsed := performance_now() - start

# Should be faster than sequential reads
```

### Integration Test: SSG Benchmark

**Target**: Process 10,000 files in <1 second (vs current 55s)

```ruff
async func process_all_files(files) {
    let tasks := []
    for file in files {
        push(tasks, process_file(file))
    }
    return await_all(tasks)
}

let start := performance_now()
let results := await process_all_files(list_dir("content"))
let elapsed := (performance_now() - start) / 1000.0

print("Processed " + to_string(len(results)) + " files in " + to_string(elapsed) + "s")
# Expected: ~0.5-1.0s on 8-core machine
```

---

## Success Criteria

### Performance Targets

- [ ] SSG benchmark: 10,000 files in <1 second (vs current 55s)
- [ ] 100 concurrent tasks spawned in <10ms
- [ ] File I/O operations don't block other tasks
- [ ] CPU utilization reaches 80%+ on multicore systems

### Functional Requirements

- [ ] `async func` spawns real async tasks
- [ ] `await` actually waits for task completion
- [ ] Multiple async tasks run concurrently
- [ ] Built-in I/O functions are non-blocking
- [ ] `await_all()` waits for multiple promises efficiently
- [ ] No deadlocks or race conditions

### Compatibility

- [ ] Existing sync code still works
- [ ] All 198+ tests still pass
- [ ] Async syntax doesn't break non-async code

---

## Example Agent Prompt

**Use this to instruct the AI:**

```
I need you to implement true async/await concurrency in Ruff. Currently, async functions 
execute synchronously (they're just syntactic sugar). 

Follow the implementation guide in ASYNC_IMPLEMENTATION_GUIDE.md:

1. Add tokio runtime integration to the VM
2. Make async functions spawn real tokio tasks instead of executing synchronously
3. Make `await` actually poll tasks instead of just unwrapping resolved promises
4. Convert file I/O operations to use tokio::fs (async)
5. Add `await_all()` utility for concurrent task execution

The goal is to make the SSG benchmark (examples/ssg/) run in <1 second instead of 55 seconds
by enabling true parallel file processing across multiple CPU cores.

Key files to modify:
- src/vm.rs (VM execution and async handling)
- src/interpreter/native_functions/*.rs (file I/O operations)
- Cargo.toml (add tokio dependencies)

Success criteria:
- SSG benchmark completes in <1 second (current: 55s)
- All existing tests still pass
- Async functions run concurrently, not sequentially

Reference the detailed implementation plan in ASYNC_IMPLEMENTATION_GUIDE.md for specific
code changes needed.
```

---

## Additional Resources

### Relevant Code Locations

- **Async execution**: `src/vm.rs` lines 1025-1110
- **Promise structure**: `src/interpreter/value.rs` (Value::Promise)
- **File I/O**: `src/interpreter/native_functions/file.rs`
- **Examples**: `examples/async_*.ruff`

### Tokio Resources

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Tokio Spawning](https://tokio.rs/tokio/tutorial/spawning)
- [Tokio Channels](https://tokio.rs/tokio/tutorial/channels)

### Similar Implementations

- **Deno**: JavaScript runtime with async built on Tokio
- **Rustlang**: Native async/await with tokio
- **Python asyncio**: Event loop-based concurrency

---

## Estimated Effort

- **Phase 1-3** (Core async): 3-5 days
- **Phase 4** (Async I/O): 2-3 days  
- **Phase 5-6** (Utilities): 2-3 days
- **Testing**: 2-3 days

**Total**: 2-3 weeks for complete implementation

---

## Notes

- This is P0 CRITICAL for v0.9.0 release
- Without this, Ruff appears slow on real-world I/O workloads despite having fast JIT
- The syntax already exists - just need to make it actually work!
- This will enable 8-10x speedup on multicore systems for I/O-bound tasks
