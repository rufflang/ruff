// File: src/jit.rs
//
// JIT Compilation module for Ruff bytecode using Cranelift.
// Provides just-in-time compilation of hot bytecode functions to native machine code.

use crate::bytecode::BytecodeChunk;
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
    
    /// Builder context for function creation
    builder_context: FunctionBuilderContext,
    
    /// Code generation context
    ctx: codegen::Context,
    
    /// Execution counter for hot path detection
    execution_counts: HashMap<usize, usize>,
    
    /// Cache of compiled functions (bytecode offset -> native function)
    compiled_cache: HashMap<usize, CompiledFn>,
    
    /// JIT enabled/disabled flag
    enabled: bool,
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
        
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        
        let module = JITModule::new(builder);
        
        Ok(JitCompiler {
            module,
            builder_context: FunctionBuilderContext::new(),
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
    pub fn get_compiled(&self, offset: usize) -> Option<CompiledFn> {
        self.compiled_cache.get(&offset).copied()
    }
    
    /// Compile a bytecode chunk to native code
    pub fn compile(&mut self, _chunk: &BytecodeChunk, offset: usize) -> Result<CompiledFn, String> {
        // Clear previous context
        self.ctx.clear();
        
        // Create function signature: fn(*mut Value) -> i64
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(types::I64)); // Stack pointer
        sig.returns.push(AbiParam::new(types::I64)); // Return value (0 = success)
        
        let func_id = self.module
            .declare_function(&format!("ruff_jit_{}", offset), Linkage::Local, &sig)
            .map_err(|e| format!("Failed to declare function: {}", e))?;
        
        self.ctx.func.signature = sig;
        
        // Build the function
        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
            
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);
            
            let _stack_ptr = builder.block_params(entry_block)[0];
            
            // TODO: Translate bytecode instructions to Cranelift IR
            // For now, just return success
            let zero = builder.ins().iconst(types::I64, 0);
            builder.ins().return_(&[zero]);
            
            builder.finalize();
        }
        
        // Compile the function
        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| format!("Failed to define function: {}", e))?;
        
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions()
            .map_err(|e| format!("Failed to finalize: {}", e))?;
        
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
}
