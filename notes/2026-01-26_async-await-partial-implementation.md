# Async/Await Implementation Session - 2026-01-26

## Session Goal
Implement async/await syntax and Promise-based concurrency for Ruff v0.8.0

## Status: ⚠️ PARTIAL IMPLEMENTATION

### Completed ✅
1. **Lexer**: Added `async` and `await` as reserved keywords
2. **AST**: Extended with `is_async` field on FuncDef/Function, added `Expr::Await`
3. **Parser**: Full syntax support for `async func name()` and `await expr`
4. **Value Types**: Added `AsyncFunction` and `Promise` enum variants
5. **Syntax Compilation**: All code compiles with syntax support

### Blocked ⚠️
**Runtime Implementation Blocked by Rust Send/Sync Requirements**

The current `Value` enum contains `Rc<RefCell<Environment>>` which is NOT `Send`.
This prevents passing Values through channels or spawning threads with Value arguments.

**Error**: 
```
error[E0277]: `Rc<RefCell<Environment>>` cannot be sent between threads safely
```

**Root Cause**:
- `Value::Function` and `Value::AsyncFunction` contain `Option<Rc<RefCell<Environment>>>` for closures
- `Rc` is not thread-safe (not `Send`)
- Async functions need to execute in threads and return Values through channels
- mpsc::Sender<Value> requires Value to be `Send`

### Solutions (for future work):

1. **Replace Rc with Arc** (Major refactor - 2-3 days):
   - Change `Rc<RefCell<Environment>>` to `Arc<Mutex<Environment>>` throughout
   - Makes Value Send + Sync
   - Allows proper async/await implementation
   - Affects ~100+ lines across interpreter

2. **Serialize return values** (Medium effort - 1 day):
   - Create `SendableValue` enum with only Send types
   - Convert Value → SendableValue before sending to thread
   - Convert back after receiving
   - Limitation: Can't return functions/closures from async functions

3. **Use tokio properly** (Large effort - 1 week):
   - Integrate tokio runtime throughout interpreter
   - Use tokio::spawn instead of std::thread::spawn
   - Async/await at Rust level, not just Ruff level
   - Most correct solution but requires deep integration

### Recommendation
Implement Solution #2 (Serializable return values) as it:
- Unblocks async/await MVP functionality
- Doesn't require massive refactor
- Matches common async patterns (most async functions return data, not functions)
- Can be upgraded to Solution #1 or #3 later

## Technical Decisions Made

### Architecture
- Async functions spawn threads (like `spawn {}` blocks)
- Each async function executes in isolated environment
- Promises use channels for result communication
- await blocks until promise resolves

### Design Pattern
```ruff
async func fetch_data(url) {
    # Executes in background thread
    return http_get(url)
}

result := await fetch_data("https://api.example.com")
# Blocks here until fetch_data completes
```

## Code Changes

### Files Modified
1. `src/lexer.rs` - Added async/await keywords
2. `src/ast.rs` - Added is_async field, Expr::Await
3. `src/parser.rs` - Parse async func and await expr
4. `src/interpreter.rs` - AsyncFunction value type, Promise handling
5. `src/builtins.rs` - Debug formatting for new types
6. `src/type_checker.rs` - Type checking for async constructs  
7. `src/compiler.rs` - Error messages for unsupported bytecode async

### New Types
```rust
// Value enum additions
AsyncFunction(Vec<String>, LeakyFunctionBody, Option<Rc<RefCell<Environment>>>),
Promise {
    receiver: Arc<Mutex<std::sync::mpsc::Receiver<Result<Value, String>>>>,
    is_polled: Arc<Mutex<bool>>,
    cached_result: Arc<Mutex<Option<Result<Value, String>>>>,
},

// AST additions  
is_async: bool  // on FuncDef and Function
Expr::Await(Box<Expr>)
```

## Testing
⚠️ No tests created yet - blocked on runtime implementation

## Next Steps (When Unblocked)

1. Choose and implement one of the three solutions above
2. Implement Promise.all([promises]) built-in
3. Implement Promise.race([promises]) built-in  
4. Create comprehensive test suite (30+ tests)
5. Write example programs demonstrating async/await
6. Update CHANGELOG, ROADMAP, README
7. Create session notes in `notes/GOTCHAS.md`

## Time Investment
- **Syntax Implementation**: 3 hours
- **Runtime Attempt**: 2 hours (blocked)
- **Total**: 5 hours
- **Estimated Remaining**: 8-12 hours (depending on solution chosen)

## Lessons Learned

### Critical Gotcha: Rc<RefCell<>> is not Send
**Problem**: Using Rc<RefCell<>> for environments prevents threading
**Symptom**: Compile error when trying to send Value through channels or spawn threads
**Solution**: Either use Arc<Mutex<>> everywhere, or serialize values for thread communication
**Prevention**: Consider Send/Sync requirements when designing types that need threading

### Async/Await Requires Deep Integration
**Problem**: Async/await isn't just syntax - it requires runtime support
**Reality**: Either commit to full tokio integration OR accept limitations (no closures in async returns)
**Implication**: "Simple" async/await is actually one of the most complex features

### Thread Spawning vs Async Runtime
**Problem**: std::thread::spawn has different guarantees than tokio::spawn
**Trade-off**: Threads are simpler but less efficient; tokio is complex but proper async
**Decision**: Started with threads for MVP, should migrate to tokio for production

## Status Summary
✅ **Syntax Complete**: Can parse async/await code
⚠️ **Runtime Blocked**: Cannot execute async functions due to Send trait
📝 **Documentation**: This session note
🔜 **Next**: Choose solution strategy and implement runtime

**Blocking Issue**: Value enum must be made Send-compatible before async/await can work.
