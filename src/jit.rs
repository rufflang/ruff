// File: src/jit.rs
//
// JIT Compilation module for Ruff bytecode using Cranelift.
// Provides just-in-time compilation of hot bytecode functions to native machine code.

use crate::bytecode::{BytecodeChunk, Constant, OpCode};
use crate::interpreter::Value;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use std::collections::HashMap;

/// JIT compilation threshold - number of executions before compiling
const JIT_THRESHOLD: usize = 100;

/// Compiled function type: takes stack pointer, returns result
type CompiledFn = unsafe extern "C" fn(*mut Value) -> i64;

/// JIT compiler for Ruff bytecode
pub struct JitCompiler {
    /// Cranelift JIT module
    module: JITModule,

    /// Code generation context
    ctx: codegen::Context,

    /// Execution counter for hot path detection
    execution_counts: HashMap<usize, usize>,

    /// Cache of compiled functions (bytecode offset -> native function)
    compiled_cache: HashMap<usize, CompiledFn>,

    /// JIT enabled/disabled flag
    enabled: bool,
}

/// Bytecode translator - converts bytecode to Cranelift IR
struct BytecodeTranslator {
    /// Stack simulation - maps stack depth to Cranelift values
    value_stack: Vec<cranelift::prelude::Value>,
    /// Variable storage - maps variable names to Cranelift values (reserved for future use)
    variables: HashMap<String, cranelift::prelude::Value>,
    /// Blocks for control flow - maps bytecode PC to Cranelift blocks
    blocks: HashMap<usize, Block>,
}

impl BytecodeTranslator {
    fn new() -> Self {
        Self { value_stack: Vec::new(), variables: HashMap::new(), blocks: HashMap::new() }
    }

    /// Pre-create blocks for all jump targets
    fn create_blocks(
        &mut self,
        builder: &mut FunctionBuilder,
        instructions: &[OpCode],
    ) -> Result<(), String> {
        // Create a block for each instruction that could be a jump target
        for (pc, instruction) in instructions.iter().enumerate() {
            match instruction {
                OpCode::Jump(target)
                | OpCode::JumpIfFalse(target)
                | OpCode::JumpIfTrue(target)
                | OpCode::JumpBack(target) => {
                    // Create block for the target if it doesn't exist
                    if !self.blocks.contains_key(target) {
                        self.blocks.insert(*target, builder.create_block());
                    }
                    // Also create block for the instruction after the jump
                    let next_pc = pc + 1;
                    if next_pc < instructions.len() && !self.blocks.contains_key(&next_pc) {
                        self.blocks.insert(next_pc, builder.create_block());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Translate a bytecode instruction to Cranelift IR
    fn translate_instruction(
        &mut self,
        builder: &mut FunctionBuilder,
        pc: usize,
        instruction: &OpCode,
        constants: &[Constant],
    ) -> Result<bool, String> {
        // Returns Ok(true) if this instruction terminates the block
        
        match instruction {
            // Arithmetic operations
            OpCode::Add => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                let result = builder.ins().iadd(a, b);
                self.push_value(result);
            }

            OpCode::Sub => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                let result = builder.ins().isub(a, b);
                self.push_value(result);
            }

            OpCode::Mul => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                let result = builder.ins().imul(a, b);
                self.push_value(result);
            }

            OpCode::Div => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                let result = builder.ins().sdiv(a, b);
                self.push_value(result);
            }

            OpCode::Mod => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                let result = builder.ins().srem(a, b);
                self.push_value(result);
            }

            OpCode::Negate => {
                let a = self.pop_value()?;
                let result = builder.ins().ineg(a);
                self.push_value(result);
            }

            // Comparison operations
            OpCode::Equal => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                let result = builder.ins().icmp(IntCC::Equal, a, b);
                let extended = builder.ins().uextend(types::I64, result);
                self.push_value(extended);
            }

            OpCode::NotEqual => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                let result = builder.ins().icmp(IntCC::NotEqual, a, b);
                let extended = builder.ins().uextend(types::I64, result);
                self.push_value(extended);
            }

            OpCode::LessThan => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                let result = builder.ins().icmp(IntCC::SignedLessThan, a, b);
                let extended = builder.ins().uextend(types::I64, result);
                self.push_value(extended);
            }

            OpCode::GreaterThan => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                let result = builder.ins().icmp(IntCC::SignedGreaterThan, a, b);
                let extended = builder.ins().uextend(types::I64, result);
                self.push_value(extended);
            }

            OpCode::LessEqual => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                // Less or equal: a <= b is !(a > b)
                let result = builder.ins().icmp(IntCC::SignedGreaterThan, a, b);
                let inverted = builder.ins().bnot(result);
                let extended = builder.ins().uextend(types::I64, inverted);
                self.push_value(extended);
            }

            OpCode::GreaterEqual => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                // Greater or equal: a >= b is !(a < b)
                let result = builder.ins().icmp(IntCC::SignedLessThan, a, b);
                let inverted = builder.ins().bnot(result);
                let extended = builder.ins().uextend(types::I64, inverted);
                self.push_value(extended);
            }

            // Logical operations
            OpCode::Not => {
                let a = self.pop_value()?;
                let zero = builder.ins().iconst(types::I64, 0);
                let result = builder.ins().icmp(IntCC::Equal, a, zero);
                let extended = builder.ins().uextend(types::I64, result);
                self.push_value(extended);
            }

            OpCode::And => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                let result = builder.ins().band(a, b);
                self.push_value(result);
            }

            OpCode::Or => {
                let b = self.pop_value()?;
                let a = self.pop_value()?;
                let result = builder.ins().bor(a, b);
                self.push_value(result);
            }

            // Stack operations
            OpCode::Pop => {
                self.pop_value()?;
            }

            OpCode::Dup => {
                let val = self.peek_value()?;
                self.push_value(val);
            }

            // Constant loading
            OpCode::LoadConst(index) => {
                if let Some(constant) = constants.get(*index) {
                    match constant {
                        Constant::Int(i) => {
                            let val = builder.ins().iconst(types::I64, *i);
                            self.push_value(val);
                        }
                        Constant::Bool(b) => {
                            let val = builder.ins().iconst(types::I64, if *b { 1 } else { 0 });
                            self.push_value(val);
                        }
                        // Other constant types need runtime support
                        _ => {
                            return Err(format!(
                                "Unsupported constant type for JIT: {:?}",
                                constant
                            ))
                        }
                    }
                } else {
                    return Err(format!("Invalid constant index: {}", index));
                }
            }

            // Control flow with proper block handling
            OpCode::Jump(target) => {
                if let Some(&target_block) = self.blocks.get(target) {
                    builder.ins().jump(target_block, &[]);
                    return Ok(true); // Terminates block
                } else {
                    return Err(format!("Jump to undefined block at PC {}", target));
                }
            }

            OpCode::JumpIfFalse(target) => {
                let condition = self.pop_value()?;
                let zero = builder.ins().iconst(types::I64, 0);
                let is_false = builder.ins().icmp(IntCC::Equal, condition, zero);

                if let Some(&target_block) = self.blocks.get(target) {
                    // Get or create the fallthrough block
                    let next_pc = pc + 1;
                    let fallthrough_block = *self.blocks.get(&next_pc)
                        .ok_or_else(|| format!("No fallthrough block after JumpIfFalse at PC {}", pc))?;

                    builder.ins().brif(is_false, target_block, &[], fallthrough_block, &[]);
                    return Ok(true); // Terminates block
                } else {
                    return Err(format!("JumpIfFalse to undefined block at PC {}", target));
                }
            }

            OpCode::JumpIfTrue(target) => {
                let condition = self.pop_value()?;
                let zero = builder.ins().iconst(types::I64, 0);
                let is_true = builder.ins().icmp(IntCC::NotEqual, condition, zero);

                if let Some(&target_block) = self.blocks.get(target) {
                    // Get or create the fallthrough block
                    let next_pc = pc + 1;
                    let fallthrough_block = *self.blocks.get(&next_pc)
                        .ok_or_else(|| format!("No fallthrough block after JumpIfTrue at PC {}", pc))?;

                    builder.ins().brif(is_true, target_block, &[], fallthrough_block, &[]);
                    return Ok(true); // Terminates block
                } else {
                    return Err(format!("JumpIfTrue to undefined block at PC {}", target));
                }
            }

            OpCode::JumpBack(target) => {
                // JumpBack is like Jump but backwards (for loops)
                if let Some(&target_block) = self.blocks.get(target) {
                    builder.ins().jump(target_block, &[]);
                    return Ok(true); // Terminates block
                } else {
                    return Err(format!("JumpBack to undefined block at PC {}", target));
                }
            }

            OpCode::Return => {
                if self.value_stack.last().is_some() {
                    // Return the value (for now, just return 0 for success)
                    let zero = builder.ins().iconst(types::I64, 0);
                    builder.ins().return_(&[zero]);
                } else {
                    let zero = builder.ins().iconst(types::I64, 0);
                    builder.ins().return_(&[zero]);
                }
                return Ok(true); // Terminates block
            }

            OpCode::ReturnNone => {
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero]);
                return Ok(true); // Terminates block
            }

            // Unsupported operations fall back to interpreter
            _ => {
                return Err(format!("Unsupported opcode for JIT: {:?}", instruction));
            }
        }

        Ok(false) // Doesn't terminate block
    }

    fn push_value(&mut self, val: cranelift::prelude::Value) {
        self.value_stack.push(val);
    }

    fn pop_value(&mut self) -> Result<cranelift::prelude::Value, String> {
        self.value_stack.pop().ok_or_else(|| "Stack underflow".to_string())
    }

    fn peek_value(&self) -> Result<cranelift::prelude::Value, String> {
        self.value_stack.last().copied().ok_or_else(|| "Stack empty".to_string())
    }
}

impl JitCompiler {
    /// Create a new JIT compiler instance
    pub fn new() -> Result<Self, String> {
        let mut flag_builder = settings::builder();
        flag_builder.set("opt_level", "speed").map_err(|e| e.to_string())?;
        flag_builder.set("is_pic", "false").map_err(|e| e.to_string())?;

        let isa_builder = cranelift_native::builder()
            .map_err(|e| format!("Failed to create ISA builder: {}", e))?;
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| format!("Failed to create ISA: {}", e))?;

        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        let module = JITModule::new(builder);

        Ok(JitCompiler {
            module,
            ctx: codegen::Context::new(),
            execution_counts: HashMap::new(),
            compiled_cache: HashMap::new(),
            enabled: true,
        })
    }

    /// Check if JIT should compile this function based on execution count
    pub fn should_compile(&mut self, offset: usize) -> bool {
        if !self.enabled {
            return false;
        }

        let count = self.execution_counts.entry(offset).or_insert(0);
        *count += 1;

        *count >= JIT_THRESHOLD && !self.compiled_cache.contains_key(&offset)
    }

    /// Get compiled function from cache
    #[allow(dead_code)] // Will be used when executing compiled code
    pub fn get_compiled(&self, offset: usize) -> Option<CompiledFn> {
        self.compiled_cache.get(&offset).copied()
    }

    /// Compile a bytecode chunk to native code
    pub fn compile(&mut self, chunk: &BytecodeChunk, offset: usize) -> Result<CompiledFn, String> {
        // Clear previous context
        self.ctx.clear();

        // Create function signature: fn(*mut Value) -> i64
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(types::I64)); // Stack pointer
        sig.returns.push(AbiParam::new(types::I64)); // Return value (0 = success)

        let func_id = self
            .module
            .declare_function(&format!("ruff_jit_{}", offset), Linkage::Local, &sig)
            .map_err(|e| format!("Failed to declare function: {}", e))?;

        self.ctx.func.signature = sig;

        // Build the function with a fresh builder context
        {
            let mut builder_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut builder_ctx);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);

            let _stack_ptr = builder.block_params(entry_block)[0];

            // Translate bytecode instructions to Cranelift IR
            let mut translator = BytecodeTranslator::new();
            
            // First pass: create blocks for all jump targets
            translator.create_blocks(&mut builder, &chunk.instructions)?;
            
            // Add entry block to the map
            translator.blocks.insert(0, entry_block);
            
            // Track sealed blocks to avoid double-sealing
            let mut sealed_blocks = std::collections::HashSet::new();
            
            // Second pass: translate instructions
            let mut current_block = entry_block;
            let mut block_terminated = false;
            
            for (pc, instruction) in chunk.instructions.iter().enumerate() {
                // If this PC has a block (it's a jump target), switch to it
                if let Some(&block) = translator.blocks.get(&pc) {
                    if block != current_block {
                        // If current block not terminated, add a fallthrough jump
                        if !block_terminated {
                            builder.ins().jump(block, &[]);
                        }
                        
                        // Seal the previous block before switching
                        if !sealed_blocks.contains(&current_block) {
                            builder.seal_block(current_block);
                            sealed_blocks.insert(current_block);
                        }
                        builder.switch_to_block(block);
                        current_block = block;
                        block_terminated = false;
                    }
                }
                
                // Skip instruction if block is already terminated
                if block_terminated {
                    continue;
                }
                
                match translator.translate_instruction(&mut builder, pc, instruction, &chunk.constants) {
                    Ok(terminates_block) => {
                        if terminates_block {
                            // Block is terminated, seal it
                            if !sealed_blocks.contains(&current_block) {
                                builder.seal_block(current_block);
                                sealed_blocks.insert(current_block);
                            }
                            block_terminated = true;
                        }
                    }
                    Err(e) => {
                        // If translation fails, we can't JIT compile this function
                        // This is expected for complex operations
                        return Err(format!("Translation failed at PC {}: {}", pc, e));
                    }
                }
            }
            
            // If the last block is not terminated, add a return
            if !block_terminated {
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero]);
                if !sealed_blocks.contains(&current_block) {
                    builder.seal_block(current_block);
                }
            }

            builder.finalize();
        }

        // Compile the function
        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| format!("Failed to define function: {}", e))?;

        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions().map_err(|e| format!("Failed to finalize: {}", e))?;

        // Get the compiled function pointer
        let code_ptr = self.module.get_finalized_function(func_id);
        let compiled_fn: CompiledFn = unsafe { std::mem::transmute(code_ptr) };

        // Cache it
        self.compiled_cache.insert(offset, compiled_fn);

        Ok(compiled_fn)
    }

    /// Enable or disable JIT compilation
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get JIT statistics
    pub fn stats(&self) -> JitStats {
        JitStats {
            total_functions: self.execution_counts.len(),
            compiled_functions: self.compiled_cache.len(),
            enabled: self.enabled,
        }
    }
}

/// JIT compilation statistics
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields accessed via Debug formatting and will be used in future performance monitoring
pub struct JitStats {
    pub total_functions: usize,
    pub compiled_functions: usize,
    pub enabled: bool,
}

impl Default for JitCompiler {
    fn default() -> Self {
        Self::new().expect("Failed to create JIT compiler")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_compiler_creation() {
        let compiler = JitCompiler::new();
        assert!(compiler.is_ok(), "JIT compiler should be created successfully");
    }

    #[test]
    fn test_hot_path_detection() {
        let mut compiler = JitCompiler::new().unwrap();

        // Should not compile initially
        for i in 0..JIT_THRESHOLD - 1 {
            assert!(!compiler.should_compile(0), "Should not compile at iteration {}", i);
        }

        // Should compile after threshold
        assert!(compiler.should_compile(0), "Should compile after threshold");

        // Mark as compiled by adding a dummy entry to cache
        compiler.compiled_cache.insert(0, unsafe { std::mem::transmute(0usize) });

        // Should not try to compile again (already in cache)
        assert!(!compiler.should_compile(0), "Should not recompile");
    }

    #[test]
    fn test_jit_enable_disable() {
        let mut compiler = JitCompiler::new().unwrap();

        assert!(compiler.enabled, "JIT should be enabled by default");

        compiler.set_enabled(false);
        assert!(!compiler.should_compile(0), "Should not compile when disabled");

        compiler.set_enabled(true);
        // Increment counter to threshold
        for _ in 0..JIT_THRESHOLD {
            compiler.should_compile(0);
        }
    }

    #[test]
    fn test_jit_stats() {
        let compiler = JitCompiler::new().unwrap();
        let stats = compiler.stats();

        assert_eq!(stats.total_functions, 0);
        assert_eq!(stats.compiled_functions, 0);
        assert!(stats.enabled);
    }

    #[test]
    fn test_compile_simple_arithmetic() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        // Create a simple program: 5 + 3
        let const_5 = chunk.add_constant(Constant::Int(5));
        let const_3 = chunk.add_constant(Constant::Int(3));

        chunk.emit(OpCode::LoadConst(const_5));
        chunk.emit(OpCode::LoadConst(const_3));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Return);

        let result = compiler.compile(&chunk, 0);
        // Compilation should succeed for simple arithmetic
        assert!(result.is_ok(), "Should compile simple arithmetic: {:?}", result.err());
    }

    #[test]
    fn test_compile_comparisons() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        // Create a simple program: 10 < 20
        let const_10 = chunk.add_constant(Constant::Int(10));
        let const_20 = chunk.add_constant(Constant::Int(20));

        chunk.emit(OpCode::LoadConst(const_10));
        chunk.emit(OpCode::LoadConst(const_20));
        chunk.emit(OpCode::LessThan);
        chunk.emit(OpCode::Return);

        let result = compiler.compile(&chunk, 0);
        assert!(result.is_ok(), "Should compile comparison operations: {:?}", result.err());
    }

    #[test]
    fn test_compile_logical_ops() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        // Create a simple program: true && false
        let const_true = chunk.add_constant(Constant::Bool(true));
        let const_false = chunk.add_constant(Constant::Bool(false));

        chunk.emit(OpCode::LoadConst(const_true));
        chunk.emit(OpCode::LoadConst(const_false));
        chunk.emit(OpCode::And);
        chunk.emit(OpCode::Return);

        let result = compiler.compile(&chunk, 0);
        assert!(result.is_ok(), "Should compile logical operations: {:?}", result.err());
    }

    #[test]
    fn test_compile_stack_operations() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        // Test dup and pop
        let const_42 = chunk.add_constant(Constant::Int(42));

        chunk.emit(OpCode::LoadConst(const_42));
        chunk.emit(OpCode::Dup);
        chunk.emit(OpCode::Pop);
        chunk.emit(OpCode::Return);

        let result = compiler.compile(&chunk, 0);
        assert!(result.is_ok(), "Should compile stack operations: {:?}", result.err());
    }

    #[test]
    fn test_compile_simple_loop() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        // Simple loop: counter from 0 to 10
        // counter := 0
        // loop_start:
        //   counter := counter + 1
        //   if counter < 10 then goto loop_start
        //   return

        let const_0 = chunk.add_constant(Constant::Int(0));
        let const_1 = chunk.add_constant(Constant::Int(1));
        let const_10 = chunk.add_constant(Constant::Int(10));

        // Initialize counter to 0
        chunk.emit(OpCode::LoadConst(const_0)); // 0: load 0

        // loop_start (PC 1):
        let loop_start = chunk.instructions.len();
        chunk.emit(OpCode::Dup); // 1: duplicate counter
        chunk.emit(OpCode::LoadConst(const_1)); // 2: load 1
        chunk.emit(OpCode::Add); // 3: counter + 1

        // Check if counter < 10
        chunk.emit(OpCode::Dup); // 4: duplicate new counter
        chunk.emit(OpCode::LoadConst(const_10)); // 5: load 10
        chunk.emit(OpCode::LessThan); // 6: counter < 10

        // If true, jump back to loop_start
        let jump_if_true = chunk.emit(OpCode::JumpIfTrue(0)); // 7: conditional jump (will be patched)
        chunk.set_jump_target(jump_if_true, loop_start);

        // Exit loop
        chunk.emit(OpCode::Return); // 8: return

        let result = compiler.compile(&chunk, 0);
        assert!(result.is_ok(), "Should compile simple loop: {:?}", result.err());
    }
}
