# 🎉 Phase 7 Steps 1 & 2 COMPLETE - Start Step 3!

## Session Date: 2026-01-28

## Current Status

✅ **Step 1: Function Call Tracking Infrastructure - COMPLETE!**
✅ **Step 2: Function Body Compilation Infrastructure - COMPLETE!**

Both steps are fully implemented, tested, committed, and pushed to main.

## 🎯 NEXT STEP: Step 3 - Call Opcode JIT Support

### What's Working Now

1. **Function Call Tracking** (Step 1):
   - VM tracks how many times each function is called
   - Compilation triggered at 100-call threshold
   - Compiled functions cached for reuse
   - Fast execution path for JIT-compiled functions

2. **Function Compilation** (Step 2):
   - `compile_function()` compiles function bodies to native code
   - `can_compile_function()` validates bytecode is compilable
   - All supported opcodes translate correctly
   - Clean error handling and debug logging

### What's Missing (Step 3 Will Fix)

- ❌ Call opcode not supported in JIT-compiled code
- ❌ Functions can't call other functions from JIT code
- ❌ Arguments not passed to compiled functions
- ❌ Return values not properly handled

### Step 3 Implementation Plan

**Goal**: Enable JIT-compiled functions to call other functions (both JIT'd and interpreted)

**Time Estimate**: 2-3 days

#### A. Understand Current Call Mechanism

First, study how Call opcode works in the VM:
- Location: `src/vm.rs`, around line 570-680
- How arguments are passed (via stack)
- How return values are handled
- How function lookup works

#### B. Implement Call Opcode Translation

**Location**: `src/jit.rs`, in `BytecodeTranslator::translate_instruction`

**Current State**: Call opcode returns error (unsupported)

**Required Changes**:

1. **Add Call case to translate_instruction**:
   ```rust
   OpCode::Call(arg_count) => {
       // 1. Pop function from stack
       // 2. Pop arguments from stack (arg_count times)
       // 3. Check if function is JIT-compiled
       // 4. If JIT: Call native function pointer
       // 5. If not: Fall back to VM interpreter
       // 6. Push return value to stack
       false // doesn't terminate block
   }
   ```

2. **Strategy Options**:

   **Option A: VM Callback (Simpler, Start Here)**
   - Create a runtime helper: `jit_call_function`
   - Takes function value + args from stack
   - Calls back into VM to execute function
   - VM handles JIT vs interpreter decision
   - Returns result to JIT code
   - **Pros**: Reuses all VM logic, safer, easier to debug
   - **Cons**: Some overhead from callbacks

   **Option B: Direct Native Calls (More Complex)**
   - JIT code directly calls other JIT-compiled functions
   - Requires function pointer lookup in JIT
   - Need to handle argument marshaling
   - Need to handle JIT/interpreter mixing
   - **Pros**: Faster (no VM callback)
   - **Cons**: More complex, more potential bugs

   **Recommendation**: Start with Option A, optimize to Option B later

#### C. Implementation Steps (Option A)

**Step 3.1: Add Runtime Helper** (1-2 hours)

Add to `src/jit.rs`, near other runtime helpers:

```rust
/// Runtime helper: Call a function from JIT code
/// Returns the result value as i64 (tagged integer)
#[no_mangle]
pub extern "C" fn jit_call_function(
    ctx: *mut VMContext,
    func_value_ptr: *const Value,
    arg_count: i64,
) -> i64 {
    unsafe {
        let ctx_ref = &mut *ctx;
        let stack = &mut *ctx_ref.stack_ptr;
        
        // Get the function value
        let func_value = &*func_value_ptr;
        
        // Pop arguments from stack
        let mut args = Vec::new();
        for _ in 0..arg_count {
            if let Some(arg) = stack.pop() {
                args.push(arg);
            }
        }
        args.reverse(); // Stack is LIFO
        
        // TODO: Call the function through VM
        // For now, return 0 (will be implemented in Step 3.2)
        0
    }
}
```

**Step 3.2: Declare Helper in JitCompiler::new** (5 minutes)

In `JitCompiler::new()`, add symbol:
```rust
builder.symbol("jit_call_function", jit_call_function as *const u8);
```

**Step 3.3: Declare in compile() Method** (15 minutes)

In `JitCompiler::compile()`, add signature:
```rust
let mut call_func_sig = self.module.make_signature();
call_func_sig.params.push(AbiParam::new(types::I64)); // ctx
call_func_sig.params.push(AbiParam::new(types::I64)); // func_value_ptr
call_func_sig.params.push(AbiParam::new(types::I64)); // arg_count
call_func_sig.returns.push(AbiParam::new(types::I64)); // result

let call_func_id = self.module
    .declare_function("jit_call_function", Linkage::Import, &call_func_sig)
    .map_err(|e| format!("Failed to declare jit_call_function: {}", e))?;
```

Import into function scope:
```rust
let call_func_ref = self.module.declare_func_in_func(call_func_id, builder.func);
```

Pass to translator:
```rust
translator.set_call_function(call_func_ref);
```

**Step 3.4: Add to BytecodeTranslator** (30 minutes)

Add field:
```rust
struct BytecodeTranslator {
    // ... existing fields ...
    call_func: Option<FuncRef>,
}
```

Add setter:
```rust
fn set_call_function(&mut self, call_func: FuncRef) {
    self.call_func = Some(call_func);
}
```

**Step 3.5: Implement Call Translation** (2-3 hours)

In `translate_instruction`, add Call case:
```rust
OpCode::Call(arg_count) => {
    let call_func = self.call_func
        .ok_or("Call function not set")?;
    
    // For now, just call the helper
    // It will pop args and function from stack internally
    let ctx = self.ctx_param.ok_or("Context not set")?;
    let arg_count_val = builder.ins().iconst(types::I64, *arg_count as i64);
    
    // TODO: Get function pointer from stack top
    // For now, pass null and let runtime figure it out
    let null_ptr = builder.ins().iconst(types::I64, 0);
    
    let call = builder.ins().call(
        call_func,
        &[ctx, null_ptr, arg_count_val]
    );
    
    // Push result to stack
    let result = builder.inst_results(call)[0];
    self.push_value(result);
    
    Ok(false) // doesn't terminate block
}
```

**Step 3.6: Update is_supported_opcode** (2 minutes)

In `JitCompiler::is_supported_opcode`, change:
```rust
// OLD:
_ => false,

// NEW:
OpCode::Call(_) => true,
_ => false,
```

#### D. Testing Steps

**Test 3.1: Compilation Test** (30 minutes)
```bash
cargo build
# Should compile without errors
```

**Test 3.2: Simple Function Call** (1 hour)

Create `test_jit_call.ruff`:
```ruff
func add(a, b) {
    return a + b
}

func test() {
    return add(2, 3)
}

# Call test() 150 times to trigger JIT
for i in range(150) {
    result := test()
}

print("Result:", result)
```

Run:
```bash
DEBUG_JIT=1 cargo run --release -- run test_jit_call.ruff
```

Expected output:
- "Function 'test' hit threshold"
- "Successfully compiled function 'test'"
- "Calling compiled function: test"
- No crashes

**Test 3.3: All Tests Pass** (15 minutes)
```bash
cargo test --lib
# All tests should still pass
```

#### E. Common Issues

1. **"Call function not set" error**:
   - Make sure you called `translator.set_call_function()`
   - Check that call_func_ref is passed correctly

2. **Segmentation fault**:
   - Check pointer validity in runtime helper
   - Make sure VMContext is properly constructed
   - Verify stack operations are safe

3. **Wrong results**:
   - Check argument order (LIFO vs FIFO)
   - Verify return value is pushed correctly
   - Test with simple cases first

#### F. Step 3 Success Criteria

- ✅ Call opcode compiles in JIT-compiled functions
- ✅ Simple function calls work (even if slow)
- ✅ No crashes or segfaults
- ✅ All existing tests pass
- ✅ Debug output shows Call opcode translating
- ✅ Functions can call other functions

#### G. What NOT to Implement Yet

- ❌ Argument marshaling (Step 4)
- ❌ Optimized direct calls (later optimization)
- ❌ Tail call optimization (Step 6)
- ❌ Recursive function optimization (Step 6)

Just focus on making basic function calls work, even if slow.

### Files to Modify in Step 3

1. `src/jit.rs`:
   - Add `jit_call_function` runtime helper
   - Add Call opcode translation
   - Update `is_supported_opcode`
   - Declare Call function in compile()
   - Add `call_func` field to BytecodeTranslator

2. `test_jit_call.ruff`:
   - New test file for function calls

### Expected Timeline

- **Day 1** (4-6 hours):
  - Implement runtime helper
  - Wire up declarations
  - Add Call translation (basic version)
  
- **Day 2** (3-4 hours):
  - Debug and fix issues
  - Get basic calls working
  - Test thoroughly

- **Day 3** (2-3 hours):
  - Polish and cleanup
  - Ensure all tests pass
  - Write documentation
  - Commit Step 3

**Total**: 9-13 hours over 2-3 days

### After Step 3

Once Step 3 is working:
- Functions can call functions (basic)
- JIT coverage expands significantly
- Ready for Step 4: Argument passing optimization
- Fibonacci should start to see improvements

### Quick Start Commands

```bash
# 1. Verify Steps 1-2 still work
cargo build && cargo test --lib

# 2. Study current Call implementation
grep -A 30 "OpCode::Call" src/vm.rs

# 3. Create new test file
cat > test_jit_call.ruff << 'EOF'
func add(a, b) {
    return a + b
}
for i in range(150) {
    result := add(i, 1)
}
print("Result:", result)
EOF

# 4. Start implementing Step 3
# (follow the plan above)
```

### Reference Documents

- This file - Implementation guide for Step 3
- `SESSION_SUMMARY_2026-01-28_STEP2.md` - What was done in Step 2
- `ROADMAP.md` Phase 7 - Overall plan
- `src/vm.rs` lines 570-680 - Current Call implementation
- `src/jit.rs` - Where you'll make changes

### Remember

- Take it one step at a time
- Test incrementally
- Use DEBUG_JIT=1 liberally
- Don't try to optimize prematurely
- Get it working first, then optimize

**Good luck with Step 3!** 🚀

---

## Summary

- ✅ Steps 1-2 complete and working
- 🎯 Step 3 is next: Call opcode support
- 📋 Clear implementation plan provided
- ⏱️ Estimated 2-3 days
- 🎉 Momentum is strong - keep going!
