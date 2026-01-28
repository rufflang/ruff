# Phase 7: Function-Level JIT - Next Session Start Guide

## Current Progress Summary

### ✅ COMPLETED: Step 1 - Function Call Tracking Infrastructure

All code has been written for Step 1. The following files have been modified:

1. **src/vm.rs**:
   - Added `function_call_counts` HashMap to track call frequency
   - Added `compiled_functions` cache for JIT-compiled functions
   - Added `JIT_FUNCTION_THRESHOLD` constant (100 calls)
   - Modified `OpCode::Call` handler to count calls and execute JIT when available
   - Fast path for JIT-compiled functions fully implemented

2. **src/jit.rs**:
   - Exported `CompiledFn` type as `pub type`

3. **test_function_jit_simple.ruff**:
   - Created test file to validate call tracking

### ⚠️ IMPORTANT: Compilation Not Verified

Due to bash command execution failures (pty_posix_spawn errors), the code has NOT been compiled or tested yet.

**FIRST ACTION IN NEXT SESSION:**
```bash
cd /Users/robertdevore/2026/ruff
cargo build
cargo test --lib vm
```

If there are compilation errors, fix them before proceeding.

## 🎯 NEXT STEP: Step 2 - Function Body Compilation Infrastructure

### Implementation Plan

#### A. Add `compile_function()` to JitCompiler

**Location**: `src/jit.rs`

Add this method to the `JitCompiler` impl block:

```rust
/// Compile an entire function to native code
/// Returns a compiled function pointer that can be called directly
pub fn compile_function(
    &mut self, 
    chunk: &BytecodeChunk, 
    name: &str
) -> Result<CompiledFn, String> {
    // 1. Check if function is compilable
    if !self.can_compile_function(chunk) {
        return Err(format!("Function '{}' contains unsupported opcodes", name));
    }
    
    // 2. Create Cranelift function signature
    //    Takes VMContext pointer, returns status code (i64)
    let mut sig = self.module.make_signature();
    sig.params.push(AbiParam::new(self.pointer_type));
    sig.returns.push(AbiParam::new(types::I64));
    
    // 3. Declare function in module
    let func_id = self.module
        .declare_function(name, Linkage::Export, &sig)
        .map_err(|e| format!("Failed to declare function: {}", e))?;
    
    // 4. Create function builder context
    let mut ctx = self.module.make_context();
    ctx.func.signature = sig;
    
    // 5. Build function body
    {
        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut self.func_ctx);
        let entry_block = bcx.create_block();
        bcx.append_block_params_for_function_params(entry_block);
        bcx.switch_to_block(entry_block);
        bcx.seal_block(entry_block);
        
        // Get VMContext parameter
        let vm_context_param = bcx.block_params(entry_block)[0];
        
        // Create BytecodeTranslator for this function
        let mut translator = BytecodeTranslator::new(
            &mut bcx,
            vm_context_param,
            &self.var_names,
        );
        
        // Translate all instructions from function body
        // (from index 0 to first Return opcode)
        for (idx, instr) in chunk.instructions.iter().enumerate() {
            match instr {
                OpCode::Return | OpCode::ReturnNone => {
                    // End of function - return success
                    let zero = translator.builder.ins().iconst(types::I64, 0);
                    translator.builder.ins().return_(&[zero]);
                    break;
                }
                _ => {
                    translator.translate_instruction(instr)?;
                }
            }
        }
        
        // Finalize the function
        translator.builder.finalize();
    }
    
    // 6. Compile the function
    self.module
        .define_function(func_id, &mut ctx)
        .map_err(|e| format!("Failed to define function: {}", e))?;
    
    self.module.clear_context(&mut ctx);
    self.module.finalize_definitions();
    
    // 7. Get function pointer
    let code_ptr = self.module.get_finalized_function(func_id);
    
    // 8. Cast to our function type
    let compiled_fn: CompiledFn = unsafe {
        std::mem::transmute(code_ptr)
    };
    
    Ok(compiled_fn)
}
```

#### B. Add `can_compile_function()` Check

**Location**: `src/jit.rs`

```rust
/// Check if a function can be JIT-compiled
/// Returns true if all opcodes in the function are supported
pub fn can_compile_function(&self, chunk: &BytecodeChunk) -> bool {
    for instr in &chunk.instructions {
        if !self.is_supported_opcode(instr) {
            return false;
        }
        
        // Stop at Return - that's the end of the function body
        match instr {
            OpCode::Return | OpCode::ReturnNone => break,
            _ => {}
        }
    }
    true
}
```

#### C. Wire Up Compilation in VM

**Location**: `src/vm.rs`, around line 635

Replace the TODO comment:
```rust
// Check if we should JIT-compile this function
if *count == JIT_FUNCTION_THRESHOLD {
    if std::env::var("DEBUG_JIT").is_ok() {
        eprintln!(
            "JIT: Function '{}' hit threshold ({} calls), attempting compilation...",
            func_name, JIT_FUNCTION_THRESHOLD
        );
    }
    
    // Attempt to compile the function
    match self.jit_compiler.compile_function(chunk, func_name) {
        Ok(compiled_fn) => {
            // Successfully compiled!
            if std::env::var("DEBUG_JIT").is_ok() {
                eprintln!("JIT: Successfully compiled function '{}'", func_name);
            }
            self.compiled_functions.insert(func_name.to_string(), compiled_fn);
        }
        Err(e) => {
            // Compilation failed - just log and continue with interpreter
            if std::env::var("DEBUG_JIT").is_ok() {
                eprintln!("JIT: Failed to compile function '{}': {}", func_name, e);
            }
        }
    }
}
```

### Testing Step 2

After implementation:

```bash
# Build
cargo build

# Run test file with JIT debugging
DEBUG_JIT=1 cargo run --release -- run test_function_jit_simple.ruff

# Expected output:
# JIT: Function 'add' hit threshold (100 calls), attempting compilation...
# JIT: Successfully compiled function 'add'
# JIT: Calling compiled function: add
# (repeated 50 more times)
# Final result: 150
# Function called 150 times - should have triggered JIT compilation
```

### Common Issues to Watch For

1. **Module/Linkage errors**: Make sure `Linkage` is imported from cranelift_module
2. **Signature mismatches**: VMContext pointer must match between caller and callee
3. **Block sealing**: All blocks must be sealed before finalization
4. **Return handling**: Every code path must end with a return instruction
5. **Call opcode**: Function body cannot contain Call opcodes yet (Phase 3)

### Estimated Time
- Implementation: 3-4 hours
- Testing: 1 hour
- Debugging: 1-2 hours
- **Total**: 5-7 hours (one work day)

### Success Criteria for Step 2
- ✅ `compile_function()` successfully compiles simple functions
- ✅ `can_compile_function()` correctly identifies compilable functions
- ✅ Test file runs without crashes
- ✅ DEBUG_JIT shows compilation and execution messages
- ✅ Simple add() function executes correctly via JIT
- ✅ Performance is at least as fast as interpreter (ideally faster)

### What NOT to Implement Yet
- ❌ Call opcode translation (Step 4)
- ❌ Recursive functions (Step 6)
- ❌ Complex control flow (Step 10)
- ❌ Argument passing (Step 3)

Just focus on compiling simple, self-contained functions that:
- Take arguments (but don't validate them yet)
- Do arithmetic
- Return a value
- Don't call other functions

### After Step 2 is Complete

Commit with message:
```
:package: NEW: implement function body JIT compilation

- Add compile_function() method to JitCompiler
- Add can_compile_function() opcode checking
- Wire up compilation trigger in VM OpCode::Call handler
- Simple functions now JIT-compile after 100 calls
- Tested with add() function - successful compilation

Part of Phase 7 Step 2: Function body compilation infrastructure
Next: Implement Call opcode translation and argument passing
```

Then proceed to Step 3: Full Call opcode support with argument passing.

## 📁 Files to Modify in Step 2
1. `src/jit.rs` - Add compile_function() and can_compile_function()
2. `src/vm.rs` - Replace TODO with actual compilation call
3. `test_function_jit_simple.ruff` - Already created, use for testing

## 🔍 Debugging Tips

Enable full JIT debugging:
```bash
export DEBUG_JIT=1
export RUST_BACKTRACE=1
cargo run --release -- run test_function_jit_simple.ruff
```

Check for compilation warnings:
```bash
cargo build 2>&1 | grep -i warning
```

Run specific tests:
```bash
cargo test --lib jit
cargo test --lib vm
```
