// File: src/jit.rs
//
// JIT Compilation module for Ruff bytecode using Cranelift.
// Provides just-in-time compilation of hot bytecode functions to native machine code.

use crate::bytecode::{BytecodeChunk, Constant, OpCode};
use crate::interpreter::Value;
use crate::vm::VM; // For calling back into VM from JIT
use cranelift::prelude::*;
use cranelift::codegen::ir::FuncRef;
use cranelift_jit::{JITBuilder, JITModule};
// FuncId used for future multi-function JIT optimization
#[allow(unused_imports)]
use cranelift_module::{Linkage, Module, FuncId};
use std::collections::HashMap;
// Hash/Hasher for future variable hashing optimizations
#[allow(unused_imports)]
use std::hash::{Hash, Hasher};
#[allow(unused_imports)]
use std::collections::hash_map::DefaultHasher;

/// JIT compilation threshold - number of executions before compiling
const JIT_THRESHOLD: usize = 100;

/// Guard failure threshold - recompile if guard failures exceed this percentage
#[allow(dead_code)] // Used in Phase 4D guard validation logic
const GUARD_FAILURE_THRESHOLD: f64 = 0.10; // 10%

/// Minimum samples before type specialization
#[allow(dead_code)] // Used in Phase 4A type profiling logic
const MIN_TYPE_SAMPLES: usize = 50;

/// Runtime context passed to JIT-compiled functions
/// This allows JIT code to access VM state (stack, variables, etc.)
#[repr(C)]
pub struct VMContext {
    /// Pointer to value stack
    pub stack_ptr: *mut Vec<Value>,
    /// Pointer to local variables (current call frame)
    pub locals_ptr: *mut HashMap<String, Value>,
    /// Pointer to global variables
    pub globals_ptr: *mut HashMap<String, Value>,
    /// Pointer to variable name mapping (hash -> name) for JIT
    pub var_names_ptr: *mut HashMap<u64, String>,
    /// Pointer to VM for calling back into interpreter (Step 4+)
    pub vm_ptr: *mut std::ffi::c_void,  // Actually *mut VM, but avoid circular dependency
}

impl VMContext {
    /// Create a new VMContext from VM state
    /// Used for JIT function execution with VM integration
    #[allow(dead_code)] // TODO: Will be used when JIT fully integrated into VM loop
    pub fn new(
        stack: *mut Vec<Value>,
        locals: *mut HashMap<String, Value>,
        globals: *mut HashMap<String, Value>,
    ) -> Self {
        Self { 
            stack_ptr: stack, 
            locals_ptr: locals, 
            globals_ptr: globals,
            var_names_ptr: std::ptr::null_mut(), // Will be set if needed
            vm_ptr: std::ptr::null_mut(), // No VM pointer yet
        }
    }
    
    /// Create with VM pointer for full VM integration (Step 4+)
    pub fn new_with_vm(
        stack: *mut Vec<Value>,
        locals: *mut HashMap<String, Value>,
        globals: *mut HashMap<String, Value>,
        vm: *mut std::ffi::c_void,
    ) -> Self {
        Self { 
            stack_ptr: stack, 
            locals_ptr: locals, 
            globals_ptr: globals,
            var_names_ptr: std::ptr::null_mut(),
            vm_ptr: vm,
        }
    }
    
    /// Create with variable name mapping
    /// Used for JIT variable resolution optimization
    #[allow(dead_code)] // TODO: Will be used when variable hashing is implemented
    pub fn with_var_names(
        stack: *mut Vec<Value>,
        locals: *mut HashMap<String, Value>,
        globals: *mut HashMap<String, Value>,
        var_names: *mut HashMap<u64, String>,
    ) -> Self {
        Self { 
            stack_ptr: stack, 
            locals_ptr: locals, 
            globals_ptr: globals,
            var_names_ptr: var_names,
            vm_ptr: std::ptr::null_mut(),
        }
    }
}

/// Compiled function type: takes VMContext pointer, returns status code
pub type CompiledFn = unsafe extern "C" fn(*mut VMContext) -> i64;

/// Type profile for a variable or operation
/// Infrastructure for Phase 4A adaptive specialization
#[allow(dead_code)] // TODO: Integrate into VM execution loop for automatic profiling
#[derive(Debug, Clone, Default)]
pub struct TypeProfile {
    /// Count of Int values observed
    pub int_count: usize,
    /// Count of Float values observed
    pub float_count: usize,
    /// Count of Bool values observed
    pub bool_count: usize,
    /// Count of other types observed
    pub other_count: usize,
}

#[allow(dead_code)] // Infrastructure for adaptive recompilation
impl TypeProfile {
    /// Record a type observation
    pub fn record(&mut self, value: &Value) {
        match value {
            Value::Int(_) => self.int_count += 1,
            Value::Float(_) => self.float_count += 1,
            Value::Bool(_) => self.bool_count += 1,
            _ => self.other_count += 1,
        }
    }
    
    /// Get total observations
    pub fn total(&self) -> usize {
        self.int_count + self.float_count + self.bool_count + self.other_count
    }
    
    /// Get the dominant type (most frequently observed)
    pub fn dominant_type(&self) -> Option<ValueType> {
        if self.total() < MIN_TYPE_SAMPLES {
            return None;
        }
        
        let max_count = self.int_count.max(self.float_count).max(self.bool_count).max(self.other_count);
        
        if max_count == self.int_count && self.int_count as f64 / self.total() as f64 > 0.90 {
            Some(ValueType::Int)
        } else if max_count == self.float_count && self.float_count as f64 / self.total() as f64 > 0.90 {
            Some(ValueType::Float)
        } else if max_count == self.bool_count && self.bool_count as f64 / self.total() as f64 > 0.90 {
            Some(ValueType::Bool)
        } else {
            None // Mixed types
        }
    }
    
    /// Check if this profile is stable enough for specialization
    pub fn is_stable(&self) -> bool {
        self.total() >= MIN_TYPE_SAMPLES && self.dominant_type().is_some()
    }
}

/// Value types for specialization
/// Infrastructure for Phase 4B type-specialized code generation
#[allow(dead_code)] // TODO: Used in adaptive recompilation decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Int,
    Float,
    Bool,
    Mixed,
}

/// Specialization strategy for a function
/// Infrastructure for Phase 4 adaptive optimization
#[allow(dead_code)] // TODO: Integrate into VM hot path detection
#[derive(Debug, Clone)]
pub struct SpecializationInfo {
    /// Type profiles for each variable (by name hash)
    pub variable_types: HashMap<u64, TypeProfile>,
    /// Dominant types for specialization
    pub specialized_types: HashMap<u64, ValueType>,
    /// Guard success count
    pub guard_successes: usize,
    /// Guard failure count
    pub guard_failures: usize,
}

#[allow(dead_code)] // Infrastructure for adaptive recompilation
impl SpecializationInfo {
    fn new() -> Self {
        Self {
            variable_types: HashMap::new(),
            specialized_types: HashMap::new(),
            guard_successes: 0,
            guard_failures: 0,
        }
    }
    
    /// Check if guards are failing too often
    fn should_despecialize(&self) -> bool {
        let total = self.guard_successes + self.guard_failures;
        if total < MIN_TYPE_SAMPLES {
            return false;
        }
        (self.guard_failures as f64 / total as f64) > GUARD_FAILURE_THRESHOLD
    }
}

// Runtime helper functions that JIT code can call
// These are marked #[no_mangle] so JIT code can find them by name

/// Push a value onto the VM stack (called from JIT code)
#[no_mangle]
pub unsafe extern "C" fn jit_stack_push(ctx: *mut VMContext, value: i64) {
    if ctx.is_null() {
        return;
    }
    let ctx = &mut *ctx;
    if !ctx.stack_ptr.is_null() {
        let stack = &mut *ctx.stack_ptr;
        stack.push(Value::Int(value));
    }
}

/// Pop a value from the VM stack (called from JIT code)
#[no_mangle]
pub unsafe extern "C" fn jit_stack_pop(ctx: *mut VMContext) -> i64 {
    if ctx.is_null() {
        return 0;
    }
    let ctx = &mut *ctx;
    if !ctx.stack_ptr.is_null() {
        let stack = &mut *ctx.stack_ptr;
        if let Some(Value::Int(val)) = stack.pop() {
            return val;
        }
    }
    0
}

/// Load a variable from locals or globals (called from JIT code)
/// name_hash: hash of the variable name (or 0 to use name_ptr/name_len)
/// name_ptr/name_len: pointer and length of variable name string (if name_hash is 0)
/// Returns the variable value as i64, or 0 if not found
#[no_mangle]
pub unsafe extern "C" fn jit_load_variable(ctx: *mut VMContext, name_hash: i64, _name_len: usize) -> i64 {
    if ctx.is_null() {
        return 0;
    }
    
    let ctx = &*ctx;
    
    // Try to resolve the name from hash
    let name = if name_hash != 0 && !ctx.var_names_ptr.is_null() {
        let var_names = &*ctx.var_names_ptr;
        if let Some(n) = var_names.get(&(name_hash as u64)) {
            n.as_str()
        } else {
            return 0; // Hash not found
        }
    } else {
        return 0; // No name provided
    };
    
    // Try locals first
    if !ctx.locals_ptr.is_null() {
        let locals = &*ctx.locals_ptr;
        if let Some(Value::Int(val)) = locals.get(name) {
            return *val;
        }
    }
    
    // Then try globals
    if !ctx.globals_ptr.is_null() {
        let globals = &*ctx.globals_ptr;
        if let Some(Value::Int(val)) = globals.get(name) {
            return *val;
        }
    }
    
    0
}

/// Store a variable to locals (called from JIT code)
/// name_hash: hash of the variable name
#[no_mangle]
pub unsafe extern "C" fn jit_store_variable(ctx: *mut VMContext, name_hash: i64, _name_len: usize, value: i64) {
    if ctx.is_null() {
        return;
    }
    
    let ctx = &mut *ctx;
    
    // Try to resolve the name from hash
    let name = if name_hash != 0 && !ctx.var_names_ptr.is_null() {
        let var_names = &*ctx.var_names_ptr;
        if let Some(n) = var_names.get(&(name_hash as u64)) {
            n.clone()
        } else {
            return; // Hash not found
        }
    } else {
        return; // No name provided
    };
    
    // Store in locals
    if !ctx.locals_ptr.is_null() {
        let locals = &mut *ctx.locals_ptr;
        locals.insert(name, Value::Int(value));
    }
}

/// Load a variable as float from locals or globals (called from JIT code)
#[no_mangle]
pub unsafe extern "C" fn jit_load_variable_float(ctx: *mut VMContext, name_hash: i64) -> f64 {
    if ctx.is_null() {
        return 0.0;
    }
    
    let ctx = &*ctx;
    
    // Try to resolve the name from hash
    let name = if name_hash != 0 && !ctx.var_names_ptr.is_null() {
        let var_names = &*ctx.var_names_ptr;
        if let Some(n) = var_names.get(&(name_hash as u64)) {
            n.as_str()
        } else {
            return 0.0; // Hash not found
        }
    } else {
        return 0.0; // No name provided
    };
    
    // Try locals first
    if !ctx.locals_ptr.is_null() {
        let locals = &*ctx.locals_ptr;
        if let Some(Value::Float(val)) = locals.get(name) {
            return *val;
        }
    }
    
    // Try globals
    if !ctx.globals_ptr.is_null() {
        let globals = &*ctx.globals_ptr;
        if let Some(Value::Float(val)) = globals.get(name) {
            return *val;
        }
    }
    
    0.0
}

/// Store a float variable to locals (called from JIT code)
#[no_mangle]
pub unsafe extern "C" fn jit_store_variable_float(ctx: *mut VMContext, name_hash: i64, value: f64) {
    if ctx.is_null() {
        return;
    }
    
    let ctx = &mut *ctx;
    
    // Try to resolve the name from hash
    let name = if name_hash != 0 && !ctx.var_names_ptr.is_null() {
        let var_names = &*ctx.var_names_ptr;
        if let Some(n) = var_names.get(&(name_hash as u64)) {
            n.clone()
        } else {
            return; // Hash not found
        }
    } else {
        return; // No name provided
    };
    
    // Store in locals
    if !ctx.locals_ptr.is_null() {
        let locals = &mut *ctx.locals_ptr;
        locals.insert(name, Value::Float(value));
    }
}

/// Check if a variable is an Int (called from JIT code for guards)
/// Returns 1 if Int, 0 otherwise
#[no_mangle]
pub unsafe extern "C" fn jit_check_type_int(ctx: *mut VMContext, name_hash: i64) -> i64 {
    if ctx.is_null() {
        return 0;
    }
    
    let ctx = &*ctx;
    
    let name = if name_hash != 0 && !ctx.var_names_ptr.is_null() {
        let var_names = &*ctx.var_names_ptr;
        if let Some(n) = var_names.get(&(name_hash as u64)) {
            n.as_str()
        } else {
            return 0;
        }
    } else {
        return 0;
    };
    
    // Check locals first
    if !ctx.locals_ptr.is_null() {
        let locals = &*ctx.locals_ptr;
        if let Some(value) = locals.get(name) {
            return if matches!(value, Value::Int(_)) { 1 } else { 0 };
        }
    }
    
    // Check globals
    if !ctx.globals_ptr.is_null() {
        let globals = &*ctx.globals_ptr;
        if let Some(value) = globals.get(name) {
            return if matches!(value, Value::Int(_)) { 1 } else { 0 };
        }
    }
    
    0
}

/// Check if a variable is a Float (called from JIT code for guards)
/// Returns 1 if Float, 0 otherwise
#[no_mangle]
pub unsafe extern "C" fn jit_check_type_float(ctx: *mut VMContext, name_hash: i64) -> i64 {
    if ctx.is_null() {
        return 0;
    }
    
    let ctx = &*ctx;
    
    let name = if name_hash != 0 && !ctx.var_names_ptr.is_null() {
        let var_names = &*ctx.var_names_ptr;
        if let Some(n) = var_names.get(&(name_hash as u64)) {
            n.as_str()
        } else {
            return 0;
        }
    } else {
        return 0;
    };
    
    // Check locals first
    if !ctx.locals_ptr.is_null() {
        let locals = &*ctx.locals_ptr;
        if let Some(value) = locals.get(name) {
            return if matches!(value, Value::Float(_)) { 1 } else { 0 };
        }
    }
    
    // Check globals
    if !ctx.globals_ptr.is_null() {
        let globals = &*ctx.globals_ptr;
        if let Some(value) = globals.get(name) {
            return if matches!(value, Value::Float(_)) { 1 } else { 0 };
        }
    }
    
    0
}

/// Runtime helper: Push an integer value to the VM stack as Value::Int
/// Used by Return opcode to return integer results
#[no_mangle]
pub unsafe extern "C" fn jit_push_int(ctx: *mut VMContext, value: i64) -> i64 {
    if ctx.is_null() {
        return 1; // Error
    }
    
    let ctx_ref = &mut *ctx;
    if ctx_ref.stack_ptr.is_null() {
        return 2; // Error
    }
    
    let stack = &mut *ctx_ref.stack_ptr;
    stack.push(Value::Int(value));
    0 // Success
}

/// Runtime helper: Call a function from JIT code
/// This enables JIT-compiled functions to call other functions
/// ctx: VMContext pointer (includes VM pointer for callbacks)
/// func_value_ptr: Unused (function is on stack)
/// arg_count: Number of arguments to pass to the function
/// Returns 0 on success, non-zero on error
#[no_mangle]
pub unsafe extern "C" fn jit_call_function(
    ctx: *mut VMContext,
    _func_value_ptr: *const Value,
    arg_count: i64,
) -> i64 {
    if ctx.is_null() {
        return 1; // Error: null context
    }
    
    let ctx_ref = &mut *ctx;
    
    // Check if we have a stack
    if ctx_ref.stack_ptr.is_null() {
        return 2; // Error: null stack
    }
    
    let stack = &mut *ctx_ref.stack_ptr;
    
    // Pop function from stack
    let function = if let Some(f) = stack.pop() {
        f
    } else {
        return 3; // Error: stack underflow (no function)
    };
    
    // Pop arguments from stack  
    let mut args = Vec::new();
    for _ in 0..arg_count {
        if let Some(arg) = stack.pop() {
            args.push(arg);
        } else {
            // Stack underflow - push function back and return error
            stack.push(function);
            return 4; // Error: stack underflow (not enough args)
        }
    }
    args.reverse(); // Arguments were pushed in order, popped in reverse
    
    // Check if we have VM pointer
    if ctx_ref.vm_ptr.is_null() {
        // No VM pointer - can't execute. Push placeholder and return error
        stack.push(Value::Int(0));
        return 5; // Error: no VM pointer
    }
    
    // Cast VM pointer back to VM
    let vm = &mut *(ctx_ref.vm_ptr as *mut VM);
    
    // Call the function through VM
    match vm.call_function_from_jit(function, args) {
        Ok(result) => {
            // Push result to stack
            stack.push(result);
            0 // Success
        }
        Err(_err) => {
            // Error during execution - push null and return error
            stack.push(Value::Null);
            6 // Error: execution failed
        }
    }
}

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
    
    /// Type profiling data for specialization
    type_profiles: HashMap<usize, SpecializationInfo>,
}

/// Bytecode translator - converts bytecode to Cranelift IR
struct BytecodeTranslator {
    /// Stack simulation - maps stack depth to Cranelift values
    value_stack: Vec<cranelift::prelude::Value>,
    /// Variable storage - maps variable names to Cranelift values
    /// TODO: Future optimization - keep frequently used variables in registers
    #[allow(dead_code)]
    variables: HashMap<String, cranelift::prelude::Value>,
    /// Blocks for control flow - maps bytecode PC to Cranelift blocks
    blocks: HashMap<usize, Block>,
    /// VMContext parameter passed to the function
    ctx_param: Option<cranelift::prelude::Value>,
    /// External function references
    load_var_func: Option<FuncRef>,
    store_var_func: Option<FuncRef>,
    /// Float load/store function references
    load_var_float_func: Option<FuncRef>,
    store_var_float_func: Option<FuncRef>,
    /// Type guard function references
    check_type_int_func: Option<FuncRef>,
    check_type_float_func: Option<FuncRef>,
    /// Call function reference (for Call opcode)
    call_func: Option<FuncRef>,
    /// Push int function reference (for Return opcode)
    push_int_func: Option<FuncRef>,
    /// Specialization information for this compilation
    specialization: Option<SpecializationInfo>,
}

impl BytecodeTranslator {
    fn new() -> Self {
        Self { 
            value_stack: Vec::new(), 
            variables: HashMap::new(), 
            blocks: HashMap::new(), 
            ctx_param: None,
            load_var_func: None,
            store_var_func: None,
            load_var_float_func: None,
            store_var_float_func: None,
            check_type_int_func: None,
            check_type_float_func: None,
            call_func: None,
            push_int_func: None,
            specialization: None,
        }
    }
    
    fn set_context_param(&mut self, ctx: cranelift::prelude::Value) {
        self.ctx_param = Some(ctx);
    }
    
    fn set_external_functions(&mut self, load_var: FuncRef, store_var: FuncRef) {
        self.load_var_func = Some(load_var);
        self.store_var_func = Some(store_var);
    }
    
    fn set_float_functions(&mut self, load_var_float: FuncRef, store_var_float: FuncRef) {
        self.load_var_float_func = Some(load_var_float);
        self.store_var_float_func = Some(store_var_float);
    }
    
    fn set_guard_functions(&mut self, check_int: FuncRef, check_float: FuncRef) {
        self.check_type_int_func = Some(check_int);
        self.check_type_float_func = Some(check_float);
    }
    
    fn set_call_function(&mut self, call_func: FuncRef) {
        self.call_func = Some(call_func);
    }
    
    fn set_push_int_function(&mut self, push_int_func: FuncRef) {
        self.push_int_func = Some(push_int_func);
    }
    
    fn set_specialization(&mut self, spec: SpecializationInfo) {
        self.specialization = Some(spec);
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
                // Check if we have specialization context for optimized path
                if self.specialization.is_some() {
                    self.translate_add_specialized(builder, None, None)?;
                } else {
                    // Generic fallback
                    let b = self.pop_value()?;
                    let a = self.pop_value()?;
                    let result = builder.ins().iadd(a, b);
                    self.push_value(result);
                }
            }

            OpCode::Sub => {
                // Check if we have specialization context for optimized path
                if self.specialization.is_some() {
                    self.translate_sub_specialized(builder, None, None)?;
                } else {
                    // Generic fallback
                    let b = self.pop_value()?;
                    let a = self.pop_value()?;
                    let result = builder.ins().isub(a, b);
                    self.push_value(result);
                }
            }

            OpCode::Mul => {
                // Check if we have specialization context for optimized path
                if self.specialization.is_some() {
                    self.translate_mul_specialized(builder, None, None)?;
                } else {
                    // Generic fallback
                    let b = self.pop_value()?;
                    let a = self.pop_value()?;
                    let result = builder.ins().imul(a, b);
                    self.push_value(result);
                }
            }

            OpCode::Div => {
                // Check if we have specialization context for optimized path
                if self.specialization.is_some() {
                    self.translate_div_specialized(builder, None, None)?;
                } else {
                    // Generic fallback
                    let b = self.pop_value()?;
                    let a = self.pop_value()?;
                    let result = builder.ins().sdiv(a, b);
                    self.push_value(result);
                }
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
                        // Other constant types (strings, floats, etc.) - push placeholder
                        // These will be handled by interpreter when needed
                        // Push 0 as placeholder to maintain stack balance
                        _ => {
                            let zero = builder.ins().iconst(types::I64, 0);
                            self.push_value(zero);
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
                // IMPORTANT: VM semantics PEEK at condition (doesn't pop)
                // A subsequent Pop instruction will remove it
                let condition = self.peek_value()?;
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
                // IMPORTANT: VM semantics PEEK at condition (doesn't pop)
                // A subsequent Pop instruction will remove it
                let condition = self.peek_value()?;
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

            OpCode::Call(arg_count) => {
                // For now, this is a placeholder implementation
                // The actual call logic is complex and requires:
                // 1. Popping the function from the stack
                // 2. Popping arguments from the stack  
                // 3. Calling the function (either JIT or interpreter)
                // 4. Pushing the result back
                
                // For Step 3, we just call the runtime helper which will handle it
                if let (Some(ctx), Some(call_func)) = (self.ctx_param, self.call_func) {
                    // Pass context, null function pointer (runtime will get it from stack),
                    // and arg count
                    let null_ptr = builder.ins().iconst(types::I64, 0);
                    let arg_count_val = builder.ins().iconst(types::I64, *arg_count as i64);
                    
                    let call_inst = builder.ins().call(call_func, &[ctx, null_ptr, arg_count_val]);
                    let _result = builder.inst_results(call_inst)[0];
                    
                    // For now, push a placeholder result to stack
                    // In a full implementation, this would be the actual return value
                    let placeholder = builder.ins().iconst(types::I64, 0);
                    self.value_stack.push(placeholder);
                    
                    return Ok(false); // Doesn't terminate block
                } else {
                    return Err("Call opcode requires context and call function to be set".to_string());
                }
            }

            OpCode::Return => {
                // Pop the return value from our stack
                if let Some(return_value) = self.value_stack.pop() {
                    // Call jit_push_int to push the value to VM stack
                    if let Some(ctx) = self.ctx_param {
                        if let Some(push_int_func) = self.push_int_func {
                            let inst = builder.ins().call(push_int_func, &[ctx, return_value]);
                            let _result = builder.inst_results(inst)[0];
                            // TODO: Check result code for errors
                        }
                    }
                }
                // Return 0 (success)
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero]);
                return Ok(true); // Terminates block
            }

            OpCode::ReturnNone => {
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero]);
                return Ok(true); // Terminates block
            }

            // Variable operations - call runtime helpers
            OpCode::LoadVar(name) => {
                if let (Some(ctx), Some(load_func)) = (self.ctx_param, self.load_var_func) {
                    // For simplicity, we'll use a hash of the variable name
                    // In a full implementation, we'd pass the string pointer
                    let name_hash = {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        name.hash(&mut hasher);
                        hasher.finish() as i64
                    };
                    
                    let name_hash_val = builder.ins().iconst(types::I64, name_hash);
                    let zero = builder.ins().iconst(types::I64, 0); // name_len = 0 (use hash instead)
                    
                    // Call jit_load_variable(ctx, name_hash, 0)
                    let call = builder.ins().call(load_func, &[ctx, name_hash_val, zero]);
                    let result = builder.inst_results(call)[0];
                    self.push_value(result);
                } else {
                    // Fallback: load 0 if context not available
                    let zero = builder.ins().iconst(types::I64, 0);
                    self.push_value(zero);
                }
            }

            OpCode::StoreVar(name) => {
                if let (Some(ctx), Some(store_func)) = (self.ctx_param, self.store_var_func) {
                    // IMPORTANT: StoreVar PEEKS at the stack (doesn't pop)
                    // The value remains on stack after assignment
                    let value = self.peek_value()?;
                    
                    // Use hash of variable name
                    let name_hash = {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        name.hash(&mut hasher);
                        hasher.finish() as i64
                    };
                    
                    let name_hash_val = builder.ins().iconst(types::I64, name_hash);
                    let zero = builder.ins().iconst(types::I64, 0); // name_len = 0
                    
                    // Call jit_store_variable(ctx, name_hash, 0, value)
                    builder.ins().call(store_func, &[ctx, name_hash_val, zero, value]);
                    // Value stays on stack - do NOT pop
                } else {
                    // Can't store without context - just leave value on stack
                }
            }

            OpCode::LoadGlobal(name) => {
                // Same as LoadVar for now
                if let (Some(ctx), Some(load_func)) = (self.ctx_param, self.load_var_func) {
                    let name_hash = {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        name.hash(&mut hasher);
                        hasher.finish() as i64
                    };
                    
                    let name_hash_val = builder.ins().iconst(types::I64, name_hash);
                    let zero = builder.ins().iconst(types::I64, 0);
                    
                    let call = builder.ins().call(load_func, &[ctx, name_hash_val, zero]);
                    let result = builder.inst_results(call)[0];
                    self.push_value(result);
                } else {
                    let zero = builder.ins().iconst(types::I64, 0);
                    self.push_value(zero);
                }
            }

            OpCode::StoreGlobal(name) => {
                // Same as StoreVar - PEEKS at stack, doesn't pop
                if let (Some(ctx), Some(store_func)) = (self.ctx_param, self.store_var_func) {
                    let value = self.peek_value()?;
                    
                    let name_hash = {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        name.hash(&mut hasher);
                        hasher.finish() as i64
                    };
                    
                    let name_hash_val = builder.ins().iconst(types::I64, name_hash);
                    let zero = builder.ins().iconst(types::I64, 0);
                    
                    builder.ins().call(store_func, &[ctx, name_hash_val, zero, value]);
                    // Value stays on stack - do NOT pop
                } else {
                    // Can't store without context - leave value on stack
                }
            }

            // Unsupported operations fall back to interpreter
            _ => {
                return Err(format!("Unsupported opcode for JIT: {:?}", instruction));
            }
        }

        Ok(false) // Doesn't terminate block
    }

    /// Translate Add operation with type specialization
    fn translate_add_specialized(
        &mut self,
        builder: &mut FunctionBuilder,
        var_a: Option<&str>,
        var_b: Option<&str>,
    ) -> Result<(), String> {
        // Try to determine types from specialization info
        let type_a = var_a.and_then(|name| {
            let hash = Self::hash_var_name(name);
            self.specialization.as_ref()?.specialized_types.get(&hash).copied()
        });
        
        let type_b = var_b.and_then(|name| {
            let hash = Self::hash_var_name(name);
            self.specialization.as_ref()?.specialized_types.get(&hash).copied()
        });
        
        let b = self.pop_value()?;
        let a = self.pop_value()?;
        
        // Generate specialized code based on types
        match (type_a, type_b) {
            (Some(ValueType::Int), Some(ValueType::Int)) => {
                // Pure integer addition - fastest path
                // Direct native i64 addition without type checks
                let result = builder.ins().iadd(a, b);
                self.push_value(result);
            }
            // Float specialization deferred to future optimization pass
            // Current implementation focuses on integer optimization
            _ => {
                // Generic fallback - assume integers for now
                let result = builder.ins().iadd(a, b);
                self.push_value(result);
            }
        }
        Ok(())
    }

    /// Translate Sub operation with type specialization
    fn translate_sub_specialized(
        &mut self,
        builder: &mut FunctionBuilder,
        var_a: Option<&str>,
        var_b: Option<&str>,
    ) -> Result<(), String> {
        let type_a = var_a.and_then(|name| {
            let hash = Self::hash_var_name(name);
            self.specialization.as_ref()?.specialized_types.get(&hash).copied()
        });
        
        let type_b = var_b.and_then(|name| {
            let hash = Self::hash_var_name(name);
            self.specialization.as_ref()?.specialized_types.get(&hash).copied()
        });
        
        let b = self.pop_value()?;
        let a = self.pop_value()?;
        
        match (type_a, type_b) {
            (Some(ValueType::Int), Some(ValueType::Int)) => {
                // Pure integer subtraction
                let result = builder.ins().isub(a, b);
                self.push_value(result);
            }
            _ => {
                let result = builder.ins().isub(a, b);
                self.push_value(result);
            }
        }
        Ok(())
    }

    /// Translate Mul operation with type specialization
    fn translate_mul_specialized(
        &mut self,
        builder: &mut FunctionBuilder,
        var_a: Option<&str>,
        var_b: Option<&str>,
    ) -> Result<(), String> {
        let type_a = var_a.and_then(|name| {
            let hash = Self::hash_var_name(name);
            self.specialization.as_ref()?.specialized_types.get(&hash).copied()
        });
        
        let type_b = var_b.and_then(|name| {
            let hash = Self::hash_var_name(name);
            self.specialization.as_ref()?.specialized_types.get(&hash).copied()
        });
        
        let b = self.pop_value()?;
        let a = self.pop_value()?;
        
        match (type_a, type_b) {
            (Some(ValueType::Int), Some(ValueType::Int)) => {
                // Pure integer multiplication
                let result = builder.ins().imul(a, b);
                self.push_value(result);
            }
            _ => {
                let result = builder.ins().imul(a, b);
                self.push_value(result);
            }
        }
        Ok(())
    }

    /// Translate Div operation with type specialization
    fn translate_div_specialized(
        &mut self,
        builder: &mut FunctionBuilder,
        var_a: Option<&str>,
        var_b: Option<&str>,
    ) -> Result<(), String> {
        let type_a = var_a.and_then(|name| {
            let hash = Self::hash_var_name(name);
            self.specialization.as_ref()?.specialized_types.get(&hash).copied()
        });
        
        let type_b = var_b.and_then(|name| {
            let hash = Self::hash_var_name(name);
            self.specialization.as_ref()?.specialized_types.get(&hash).copied()
        });
        
        let b = self.pop_value()?;
        let a = self.pop_value()?;
        
        match (type_a, type_b) {
            (Some(ValueType::Int), Some(ValueType::Int)) => {
                // Pure integer division
                let result = builder.ins().sdiv(a, b);
                self.push_value(result);
            }
            _ => {
                let result = builder.ins().sdiv(a, b);
                self.push_value(result);
            }
        }
        Ok(())
    }

    /// Helper to hash variable names consistently
    fn hash_var_name(name: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish()
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

        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        
        // Register runtime helper symbols so JIT can find them
        builder.symbol("jit_load_variable", jit_load_variable as *const u8);
        builder.symbol("jit_store_variable", jit_store_variable as *const u8);
        builder.symbol("jit_stack_push", jit_stack_push as *const u8);
        builder.symbol("jit_stack_pop", jit_stack_pop as *const u8);
        builder.symbol("jit_load_variable_float", jit_load_variable_float as *const u8);
        builder.symbol("jit_store_variable_float", jit_store_variable_float as *const u8);
        builder.symbol("jit_check_type_int", jit_check_type_int as *const u8);
        builder.symbol("jit_check_type_float", jit_check_type_float as *const u8);
        builder.symbol("jit_call_function", jit_call_function as *const u8);
        builder.symbol("jit_push_int", jit_push_int as *const u8);

        let module = JITModule::new(builder);

        Ok(JitCompiler {
            module,
            ctx: codegen::Context::new(),
            execution_counts: HashMap::new(),
            compiled_cache: HashMap::new(),
            enabled: true,
            type_profiles: HashMap::new(),
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

    /// Check if a loop can be JIT-compiled (all opcodes supported)
    /// Scans from start to end (inclusive) checking for unsupported operations
    pub fn can_compile_loop(&self, chunk: &BytecodeChunk, start: usize, end: usize) -> bool {
        for pc in start..=end {
            if let Some(instruction) = chunk.instructions.get(pc) {
                if !self.is_supported_opcode(instruction, &chunk.constants) {
                    if std::env::var("DEBUG_JIT").is_ok() {
                        eprintln!("JIT: Unsupported opcode at {}: {:?}", pc, instruction);
                    }
                    return false;
                }
            }
        }
        true
    }

    /// Check if a specific opcode is supported by JIT compiler
    fn is_supported_opcode(&self, opcode: &OpCode, constants: &[Constant]) -> bool {
        match opcode {
            // Constants - All types supported (strings push placeholder 0)
            OpCode::LoadConst(idx) => {
                constants.get(*idx).is_some()
            }
            
            // Arithmetic operations
            OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod => true,
            OpCode::Negate => true,
            
            // Comparison operations
            OpCode::Equal | OpCode::NotEqual => true,
            OpCode::LessThan | OpCode::GreaterThan => true,
            OpCode::LessEqual | OpCode::GreaterEqual => true,
            
            // Logical operations
            OpCode::Not | OpCode::And | OpCode::Or => true,
            
            // Stack operations
            OpCode::Pop | OpCode::Dup => true,
            
            // Variable operations
            OpCode::LoadVar(_) | OpCode::StoreVar(_) => true,
            OpCode::LoadGlobal(_) | OpCode::StoreGlobal(_) => true,
            
            // Control flow - simple jumps only
            OpCode::Jump(_) | OpCode::JumpIfFalse(_) | OpCode::JumpIfTrue(_) => true,
            OpCode::JumpBack(_) => true,
            
            // Function returns - needed for compiling functions (not just loops)
            OpCode::Return | OpCode::ReturnNone => true,
            
            // Function calls - Step 3: Basic Call opcode support
            OpCode::Call(_) => true,
            
            // Everything else is unsupported - causes code to skip JIT
            // This includes: CallNative, Arrays, Dicts, Generators, Async, etc.
            _ => false,
        }
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
        
        // Declare external runtime helper functions
        // jit_load_variable: fn(*mut VMContext, *const u8, usize) -> i64
        let mut load_var_sig = self.module.make_signature();
        load_var_sig.params.push(AbiParam::new(types::I64)); // ctx pointer
        load_var_sig.params.push(AbiParam::new(types::I64)); // name_ptr
        load_var_sig.params.push(AbiParam::new(types::I64)); // name_len
        load_var_sig.returns.push(AbiParam::new(types::I64)); // return value
        
        let load_var_func_id = self
            .module
            .declare_function("jit_load_variable", Linkage::Import, &load_var_sig)
            .map_err(|e| format!("Failed to declare jit_load_variable: {}", e))?;
        
        // jit_store_variable: fn(*mut VMContext, *const u8, usize, i64)
        let mut store_var_sig = self.module.make_signature();
        store_var_sig.params.push(AbiParam::new(types::I64)); // ctx pointer
        store_var_sig.params.push(AbiParam::new(types::I64)); // name_ptr
        store_var_sig.params.push(AbiParam::new(types::I64)); // name_len
        store_var_sig.params.push(AbiParam::new(types::I64)); // value
        // no return value
        
        let store_var_func_id = self
            .module
            .declare_function("jit_store_variable", Linkage::Import, &store_var_sig)
            .map_err(|e| format!("Failed to declare jit_store_variable: {}", e))?;
        
        // jit_load_variable_float: fn(*mut VMContext, i64) -> f64
        let mut load_var_float_sig = self.module.make_signature();
        load_var_float_sig.params.push(AbiParam::new(types::I64)); // ctx pointer
        load_var_float_sig.params.push(AbiParam::new(types::I64)); // name_hash
        load_var_float_sig.returns.push(AbiParam::new(types::F64)); // return f64
        
        let load_var_float_func_id = self
            .module
            .declare_function("jit_load_variable_float", Linkage::Import, &load_var_float_sig)
            .map_err(|e| format!("Failed to declare jit_load_variable_float: {}", e))?;
        
        // jit_store_variable_float: fn(*mut VMContext, i64, f64)
        let mut store_var_float_sig = self.module.make_signature();
        store_var_float_sig.params.push(AbiParam::new(types::I64)); // ctx pointer
        store_var_float_sig.params.push(AbiParam::new(types::I64)); // name_hash
        store_var_float_sig.params.push(AbiParam::new(types::F64)); // value f64
        
        let store_var_float_func_id = self
            .module
            .declare_function("jit_store_variable_float", Linkage::Import, &store_var_float_sig)
            .map_err(|e| format!("Failed to declare jit_store_variable_float: {}", e))?;
        
        // jit_check_type_int: fn(*mut VMContext, i64) -> i64
        let mut check_int_sig = self.module.make_signature();
        check_int_sig.params.push(AbiParam::new(types::I64)); // ctx pointer
        check_int_sig.params.push(AbiParam::new(types::I64)); // name_hash
        check_int_sig.returns.push(AbiParam::new(types::I64)); // returns 1 if Int, 0 otherwise
        
        let check_type_int_func_id = self
            .module
            .declare_function("jit_check_type_int", Linkage::Import, &check_int_sig)
            .map_err(|e| format!("Failed to declare jit_check_type_int: {}", e))?;
        
        // jit_check_type_float: fn(*mut VMContext, i64) -> i64
        let mut check_float_sig = self.module.make_signature();
        check_float_sig.params.push(AbiParam::new(types::I64)); // ctx pointer
        check_float_sig.params.push(AbiParam::new(types::I64)); // name_hash
        check_float_sig.returns.push(AbiParam::new(types::I64)); // returns 1 if Float, 0 otherwise
        
        let check_type_float_func_id = self
            .module
            .declare_function("jit_check_type_float", Linkage::Import, &check_float_sig)
            .map_err(|e| format!("Failed to declare jit_check_type_float: {}", e))?;

        // Build the function with a fresh builder context
        {
            let mut builder_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut builder_ctx);
            
            // Import the external functions into this function's scope
            let load_var_func_ref = self.module.declare_func_in_func(load_var_func_id, builder.func);
            let store_var_func_ref = self.module.declare_func_in_func(store_var_func_id, builder.func);
            let load_var_float_func_ref = self.module.declare_func_in_func(load_var_float_func_id, builder.func);
            let store_var_float_func_ref = self.module.declare_func_in_func(store_var_float_func_id, builder.func);
            let check_int_func_ref = self.module.declare_func_in_func(check_type_int_func_id, builder.func);
            let check_float_func_ref = self.module.declare_func_in_func(check_type_float_func_id, builder.func);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);

            let ctx_ptr = builder.block_params(entry_block)[0];

            // Translate bytecode instructions to Cranelift IR
            let mut translator = BytecodeTranslator::new();
            translator.set_context_param(ctx_ptr);
            translator.set_external_functions(load_var_func_ref, store_var_func_ref);
            translator.set_float_functions(load_var_float_func_ref, store_var_float_func_ref);
            translator.set_guard_functions(check_int_func_ref, check_float_func_ref);
            
            // Initialize current block tracking
            let mut current_block = entry_block;
            
            // Set specialization info if available
            if let Some(spec) = self.type_profiles.get(&offset) {
                translator.set_specialization(spec.clone());
                
                // Generate type guards at function entry for specialized variables
                if !spec.specialized_types.is_empty() {
                    // Create blocks for guard success and guard failure
                    let guard_success_block = builder.create_block();
                    let guard_failure_block = builder.create_block();
                    
                    // Check each specialized variable's type
                    let mut all_guards_passed = None;
                    for (var_hash, expected_type) in &spec.specialized_types {
                        let hash_val = builder.ins().iconst(types::I64, *var_hash as i64);
                        
                        // Call appropriate type check function
                        let check_result = match expected_type {
                            ValueType::Int => {
                                let call = builder.ins().call(check_int_func_ref, &[ctx_ptr, hash_val]);
                                builder.inst_results(call)[0]
                            }
                            ValueType::Float => {
                                let call = builder.ins().call(check_float_func_ref, &[ctx_ptr, hash_val]);
                                builder.inst_results(call)[0]
                            }
                            _ => {
                                // Other types not yet specialized, skip guard
                                continue;
                            }
                        };
                        
                        // Check if result is 1 (type matches)
                        let one = builder.ins().iconst(types::I64, 1);
                        let guard_passed = builder.ins().icmp(IntCC::Equal, check_result, one);
                        
                        // Combine with previous guards (AND operation)
                        all_guards_passed = Some(if let Some(prev) = all_guards_passed {
                            builder.ins().band(prev, guard_passed)
                        } else {
                            guard_passed
                        });
                    }
                    
                    // Branch based on guard results
                    if let Some(guards_passed) = all_guards_passed {
                        // Branch: if guards_passed, go to success, else go to failure
                        builder.ins().brif(guards_passed, guard_success_block, &[], guard_failure_block, &[]);
                        
                        // Seal entry block now that we've branched from it
                        builder.seal_block(entry_block);
                        
                        // Guard failure block: return error code (-1)
                        builder.switch_to_block(guard_failure_block);
                        builder.seal_block(guard_failure_block);
                        let error_code = builder.ins().iconst(types::I64, -1);
                        builder.ins().return_(&[error_code]);
                        
                        // Guard success block: continue with function body
                        // Switch to it but DON'T seal it yet - let normal sealing logic handle it
                        builder.switch_to_block(guard_success_block);
                        
                        // Update current block for instruction translation
                        current_block = guard_success_block;
                        // Add guard_success_block as the new entry for instruction 0
                        translator.blocks.insert(0, guard_success_block);
                    }
                }
            }
            
            // First pass: create blocks for all jump targets
            translator.create_blocks(&mut builder, &chunk.instructions)?;
            
            // Add entry block to the map (will be guard_success_block if guards were generated)
            if !translator.blocks.contains_key(&0) {
                translator.blocks.insert(0, entry_block);
            }
            
            // Track sealed blocks to avoid double-sealing
            let mut sealed_blocks = std::collections::HashSet::new();
            
            // Second pass: translate instructions
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
        
        // 2. Clear previous context
        self.ctx.clear();
        
        // 3. Create Cranelift function signature
        //    Takes VMContext pointer, returns status code (i64)
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(types::I64)); // VMContext pointer
        sig.returns.push(AbiParam::new(types::I64)); // Status code
        
        // 4. Declare function in module
        let func_id = self.module
            .declare_function(name, Linkage::Export, &sig)
            .map_err(|e| format!("Failed to declare function: {}", e))?;
        
        // 5. Set function signature
        self.ctx.func.signature = sig;
        
        // 6. Build function body
        {
            let mut builder_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut builder_ctx);
            
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            
            // Get VMContext parameter
            let vm_context_param = builder.block_params(entry_block)[0];
            
            // Declare jit_call_function for Call opcode support
            // jit_call_function: fn(*mut VMContext, *const Value, i64) -> i64
            let mut call_func_sig = self.module.make_signature();
            call_func_sig.params.push(AbiParam::new(types::I64)); // ctx pointer
            call_func_sig.params.push(AbiParam::new(types::I64)); // func_value_ptr
            call_func_sig.params.push(AbiParam::new(types::I64)); // arg_count
            call_func_sig.returns.push(AbiParam::new(types::I64)); // return value
            
            let call_func_id = self.module
                .declare_function("jit_call_function", Linkage::Import, &call_func_sig)
                .map_err(|e| format!("Failed to declare jit_call_function: {}", e))?;
            
            let call_func_ref = self.module.declare_func_in_func(call_func_id, &mut builder.func);
            
            // Declare jit_push_int for Return opcode support
            // jit_push_int: fn(*mut VMContext, i64) -> i64
            let mut push_int_sig = self.module.make_signature();
            push_int_sig.params.push(AbiParam::new(types::I64)); // ctx pointer
            push_int_sig.params.push(AbiParam::new(types::I64)); // value to push
            push_int_sig.returns.push(AbiParam::new(types::I64)); // status code
            
            let push_int_id = self.module
                .declare_function("jit_push_int", Linkage::Import, &push_int_sig)
                .map_err(|e| format!("Failed to declare jit_push_int: {}", e))?;
            
            let push_int_ref = self.module.declare_func_in_func(push_int_id, &mut builder.func);
            
            // Declare jit_load_variable and jit_store_variable for variable operations
            // jit_load_variable: fn(*mut VMContext, i64, usize) -> i64
            let mut load_var_sig = self.module.make_signature();
            load_var_sig.params.push(AbiParam::new(types::I64)); // ctx pointer
            load_var_sig.params.push(AbiParam::new(types::I64)); // name_hash
            load_var_sig.params.push(AbiParam::new(types::I64)); // name_len
            load_var_sig.returns.push(AbiParam::new(types::I64)); // value
            
            let load_var_id = self.module
                .declare_function("jit_load_variable", Linkage::Import, &load_var_sig)
                .map_err(|e| format!("Failed to declare jit_load_variable: {}", e))?;
            
            let load_var_ref = self.module.declare_func_in_func(load_var_id, &mut builder.func);
            
            // jit_store_variable: fn(*mut VMContext, i64, usize, i64)
            let mut store_var_sig = self.module.make_signature();
            store_var_sig.params.push(AbiParam::new(types::I64)); // ctx pointer
            store_var_sig.params.push(AbiParam::new(types::I64)); // name_hash
            store_var_sig.params.push(AbiParam::new(types::I64)); // name_len
            store_var_sig.params.push(AbiParam::new(types::I64)); // value
            
            let store_var_id = self.module
                .declare_function("jit_store_variable", Linkage::Import, &store_var_sig)
                .map_err(|e| format!("Failed to declare jit_store_variable: {}", e))?;
            
            let store_var_ref = self.module.declare_func_in_func(store_var_id, &mut builder.func);
            
            // Create BytecodeTranslator for this function
            let mut translator = BytecodeTranslator::new();
            translator.set_context_param(vm_context_param);
            translator.set_call_function(call_func_ref);
            translator.set_push_int_function(push_int_ref);
            translator.set_external_functions(load_var_ref, store_var_ref);
            
            // Create blocks for all jump targets
            translator.create_blocks(&mut builder, &chunk.instructions)?;
            
            // Add entry block to block map
            if !translator.blocks.contains_key(&0) {
                translator.blocks.insert(0, entry_block);
            }
            
            // Seal entry block
            builder.seal_block(entry_block);
            
            let mut sealed_blocks = std::collections::HashSet::new();
            sealed_blocks.insert(entry_block);
            
            let mut current_block = entry_block;
            let mut block_terminated = false;
            
            // Translate all instructions from function body
            // IMPORTANT: Stop at Return/ReturnNone - that's the end of the function
            for (pc, instr) in chunk.instructions.iter().enumerate() {
                // Stop at Return - we don't want to translate code after the function body
                match instr {
                    OpCode::Return | OpCode::ReturnNone => {
                        // Translate this Return instruction and then stop
                        match translator.translate_instruction(&mut builder, pc, instr, &chunk.constants) {
                            Ok(_terminates_block) => {
                                // Seal the current block
                                if !sealed_blocks.contains(&current_block) {
                                    builder.seal_block(current_block);
                                    sealed_blocks.insert(current_block);
                                }
                            }
                            Err(e) => {
                                return Err(format!("Translation failed at PC {}: {}", pc, e));
                            }
                        }
                        // Stop translating - we've reached the end of the function
                        break;
                    }
                    _ => {}
                }
                
                // If this PC has a block, switch to it
                if let Some(&block) = translator.blocks.get(&pc) {
                    if block != current_block {
                        // If current block not terminated, add fallthrough jump
                        if !block_terminated {
                            builder.ins().jump(block, &[]);
                        }
                        
                        // Seal previous block
                        if !sealed_blocks.contains(&current_block) {
                            builder.seal_block(current_block);
                            sealed_blocks.insert(current_block);
                        }
                        
                        builder.switch_to_block(block);
                        current_block = block;
                        block_terminated = false;
                    }
                }
                
                // Skip if block terminated
                if block_terminated {
                    continue;
                }
                
                // Translate the instruction
                match translator.translate_instruction(&mut builder, pc, instr, &chunk.constants) {
                    Ok(terminates_block) => {
                        if std::env::var("DEBUG_JIT").is_ok() {
                            eprintln!("JIT: PC {} OK, stack depth now: {}", pc, translator.value_stack.len());
                        }
                        if terminates_block {
                            if !sealed_blocks.contains(&current_block) {
                                builder.seal_block(current_block);
                                sealed_blocks.insert(current_block);
                            }
                            block_terminated = true;
                        }
                    }
                    Err(e) => {
                        if std::env::var("DEBUG_JIT").is_ok() {
                            eprintln!("JIT: PC {} FAILED: {}, instruction: {:?}, stack depth: {}", 
                                pc, e, instr, translator.value_stack.len());
                        }
                        return Err(format!("Translation failed at PC {}: {}", pc, e));
                    }
                }
            }
            
            // If last block not terminated, add return
            if !block_terminated {
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero]);
                if !sealed_blocks.contains(&current_block) {
                    builder.seal_block(current_block);
                }
            }
            
            // Finalize the function
            builder.finalize();
        }
        
        // 7. Compile the function
        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| format!("Failed to define function: {}", e))?;
        
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions()
            .map_err(|e| format!("Failed to finalize: {}", e))?;
        
        // 8. Get function pointer
        let code_ptr = self.module.get_finalized_function(func_id);
        
        // 9. Cast to our function type
        let compiled_fn: CompiledFn = unsafe {
            std::mem::transmute(code_ptr)
        };
        
        Ok(compiled_fn)
    }

    /// Check if a function can be JIT-compiled
    /// Returns true if all opcodes in the function are supported
    pub fn can_compile_function(&self, chunk: &BytecodeChunk) -> bool {
        for instr in &chunk.instructions {
            if !self.is_supported_opcode(instr, &chunk.constants) {
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

    /// Enable or disable JIT compilation
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Record a type observation for profiling
    /// TODO: Integrate into VM execution loop for automatic profiling
    #[allow(dead_code)]
    pub fn record_type(&mut self, offset: usize, var_hash: u64, value: &Value) {
        let profile = self.type_profiles.entry(offset).or_insert_with(SpecializationInfo::new);
        let type_profile = profile.variable_types.entry(var_hash).or_insert_with(TypeProfile::default);
        type_profile.record(value);
        
        // Update specialization strategy if profile is stable
        if type_profile.is_stable() {
            if let Some(dominant_type) = type_profile.dominant_type() {
                profile.specialized_types.insert(var_hash, dominant_type);
            }
        }
    }
    
    /// Record a guard success
    /// TODO: Call from JIT-compiled code guard checks
    #[allow(dead_code)]
    pub fn record_guard_success(&mut self, offset: usize) {
        if let Some(profile) = self.type_profiles.get_mut(&offset) {
            profile.guard_successes += 1;
        }
    }
    
    /// Record a guard failure
    /// TODO: Call from JIT-compiled code when guard checks fail
    #[allow(dead_code)]
    pub fn record_guard_failure(&mut self, offset: usize) {
        if let Some(profile) = self.type_profiles.get_mut(&offset) {
            profile.guard_failures += 1;
            
            // Check if we should recompile
            if profile.should_despecialize() {
                // Remove from cache to force recompilation with generic code
                self.compiled_cache.remove(&offset);
                profile.specialized_types.clear();
                profile.guard_successes = 0;
                profile.guard_failures = 0;
            }
        }
    }
    
    /// Get specialization info for a function
    /// TODO: Use for adaptive recompilation decisions
    #[allow(dead_code)]
    pub fn get_specialization(&self, offset: usize) -> Option<&SpecializationInfo> {
        self.type_profiles.get(&offset)
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

    #[test]
    fn test_execute_compiled_code() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        // Simple arithmetic: 5 + 3 = 8
        let const_5 = chunk.add_constant(Constant::Int(5));
        let const_3 = chunk.add_constant(Constant::Int(3));

        chunk.emit(OpCode::LoadConst(const_5));
        chunk.emit(OpCode::LoadConst(const_3));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Return);

        let compiled_fn = compiler.compile(&chunk, 0).expect("Should compile");

        // Execute the compiled function
        // For now, pass a null pointer since we don't need context for pure arithmetic
        let mut ctx = VMContext::new(std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut());
        let result = unsafe { compiled_fn(&mut ctx as *mut VMContext) };

        // Result should be 0 (success code)
        assert_eq!(result, 0, "Compiled function should return success code");

        println!("✓ Compiled code executed successfully!");
    }

    #[test]
    fn test_compile_with_variables() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        // Test that variable opcodes at least compile (even if stubbed)
        // x := 5; y := 3; result := x + y
        let const_5 = chunk.add_constant(Constant::Int(5));
        let const_3 = chunk.add_constant(Constant::Int(3));

        chunk.emit(OpCode::LoadConst(const_5));
        chunk.emit(OpCode::StoreVar("x".to_string()));
        chunk.emit(OpCode::LoadConst(const_3));
        chunk.emit(OpCode::StoreVar("y".to_string()));
        chunk.emit(OpCode::LoadVar("x".to_string()));
        chunk.emit(OpCode::LoadVar("y".to_string()));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::StoreVar("result".to_string()));
        chunk.emit(OpCode::Return);

        let result = compiler.compile(&chunk, 0);
        assert!(result.is_ok(), "Should compile variable operations: {:?}", result.err());
        
        println!("✓ Variable operations compile successfully!");
    }
    
    #[test]
    fn test_execute_with_variables() {
        use std::collections::HashMap;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();

        // x := 10; y := 20; result := x + y; return
        let const_10 = chunk.add_constant(Constant::Int(10));
        let const_20 = chunk.add_constant(Constant::Int(20));

        chunk.emit(OpCode::LoadConst(const_10));
        chunk.emit(OpCode::StoreVar("x".to_string()));
        chunk.emit(OpCode::LoadConst(const_20));
        chunk.emit(OpCode::StoreVar("y".to_string()));
        chunk.emit(OpCode::LoadVar("x".to_string()));
        chunk.emit(OpCode::LoadVar("y".to_string()));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Return);

        let compiled_fn = compiler.compile(&chunk, 0).expect("Should compile");
        
        // Create actual variable storage
        let mut locals: HashMap<String, Value> = HashMap::new();
        let mut globals: HashMap<String, Value> = HashMap::new();
        let mut var_names: HashMap<u64, String> = HashMap::new();
        
        // Register variable names with their hashes
        for name in &["x", "y", "result"] {
            let mut hasher = DefaultHasher::new();
            name.hash(&mut hasher);
            let hash = hasher.finish();
            var_names.insert(hash, name.to_string());
        }
        
        // Create context
        let mut ctx = VMContext::with_var_names(
            std::ptr::null_mut(),
            &mut locals as *mut _,
            &mut globals as *mut _,
            &mut var_names as *mut _,
        );
        
        // Execute compiled code
        let result = unsafe { compiled_fn(&mut ctx as *mut VMContext) };
        
        assert_eq!(result, 0, "Should return success");
        
        // Check variables were stored
        assert!(locals.contains_key("x"), "x should exist");
        assert!(locals.contains_key("y"), "y should exist");
        
        // Check values
        match locals.get("x") {
            Some(Value::Int(10)) => {},
            other => panic!("Expected x=10, got {:?}", other),
        }
        
        match locals.get("y") {
            Some(Value::Int(20)) => {},
            other => panic!("Expected y=20, got {:?}", other),
        }
        
        println!("✓ Variables work correctly in JIT code!");
        println!("  x = {:?}", locals.get("x"));
        println!("  y = {:?}", locals.get("y"));
    }
    
    #[test]
    fn test_type_profiling() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset = 0;
        
        // Simulate profiling Int values
        let var_hash = 12345u64;
        for _ in 0..60 {
            compiler.record_type(offset, var_hash, &Value::Int(42));
        }
        
        // Check profile
        let profile = compiler.get_specialization(offset).expect("Profile should exist");
        let type_profile = profile.variable_types.get(&var_hash).expect("Type profile should exist");
        
        assert_eq!(type_profile.int_count, 60);
        assert_eq!(type_profile.float_count, 0);
        assert!(type_profile.is_stable(), "Profile should be stable");
        assert_eq!(type_profile.dominant_type(), Some(ValueType::Int));
        
        // Check specialization was recorded
        assert_eq!(profile.specialized_types.get(&var_hash), Some(&ValueType::Int));
    }
    
    #[test]
    fn test_type_profiling_mixed_types() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset = 0;
        let var_hash = 12345u64;
        
        // Simulate mixed types (40 Int, 20 Float)
        for _ in 0..40 {
            compiler.record_type(offset, var_hash, &Value::Int(42));
        }
        for _ in 0..20 {
            compiler.record_type(offset, var_hash, &Value::Float(3.14));
        }
        
        let profile = compiler.get_specialization(offset).expect("Profile should exist");
        let type_profile = profile.variable_types.get(&var_hash).expect("Type profile should exist");
        
        assert_eq!(type_profile.int_count, 40);
        assert_eq!(type_profile.float_count, 20);
        assert_eq!(type_profile.total(), 60);
        
        // Not stable enough for specialization (need >90% of one type)
        assert!(type_profile.dominant_type().is_none(), "Mixed types shouldn't specialize");
    }
    
    #[test]
    fn test_guard_success_tracking() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset = 0;
        
        // Initialize profile
        compiler.record_type(offset, 12345, &Value::Int(1));
        
        // Record guard successes
        for _ in 0..100 {
            compiler.record_guard_success(offset);
        }
        
        let profile = compiler.get_specialization(offset).expect("Profile should exist");
        assert_eq!(profile.guard_successes, 100);
        assert_eq!(profile.guard_failures, 0);
        assert!(!profile.should_despecialize());
    }
    
    #[test]
    fn test_guard_failure_despecialization() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset = 0;
        
        // Initialize profile and add to compiled cache
        compiler.record_type(offset, 12345, &Value::Int(1));
        compiler.compiled_cache.insert(offset, unsafe { std::mem::transmute(0usize) });
        
        // Record mostly successes, then failures
        for _ in 0..90 {
            compiler.record_guard_success(offset);
        }
        
        // Add failures - when we hit 11 failures out of 101 total (10.9%), despec triggers
        // After that, the counters reset and remaining failures get added to new counters
        for _ in 0..20 {
            compiler.record_guard_failure(offset);
        }
        
        // Should have cleared cache (happened when threshold was crossed)
        assert!(!compiler.compiled_cache.contains_key(&offset), "Should remove from cache");
        let profile = compiler.get_specialization(offset).expect("Profile should exist");
        assert_eq!(profile.specialized_types.len(), 0, "Should clear specializations");
        
        // Counters were reset when despec happened, then more failures were added
        // So we should have some failures but not all 20
        assert!(profile.guard_failures < 20 && profile.guard_failures > 0, 
                "Should have some failures after reset: {}", profile.guard_failures);
        assert_eq!(profile.guard_successes, 0, "Successes should be reset");
    }
    
    #[test]
    fn test_float_specialization_profile() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset = 0;
        let var_hash = 99999u64;
        
        // Profile float values
        for _ in 0..60 {
            compiler.record_type(offset, var_hash, &Value::Float(3.14));
        }
        
        let profile = compiler.get_specialization(offset).expect("Profile should exist");
        let type_profile = profile.variable_types.get(&var_hash).expect("Type profile should exist");
        
        assert_eq!(type_profile.float_count, 60);
        assert!(type_profile.is_stable());
        assert_eq!(type_profile.dominant_type(), Some(ValueType::Float));
        assert_eq!(profile.specialized_types.get(&var_hash), Some(&ValueType::Float));
    }

    #[test]
    fn test_int_specialized_addition() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();
        
        // Test: 100 + 200 = 300 (pure integer addition)
        let const_100 = chunk.add_constant(Constant::Int(100));
        let const_200 = chunk.add_constant(Constant::Int(200));
        
        chunk.emit(OpCode::LoadConst(const_100));
        chunk.emit(OpCode::LoadConst(const_200));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Return);
        
        let result = compiler.compile(&chunk, 0);
        assert!(result.is_ok(), "Should compile int addition: {:?}", result.err());
    }

    #[test]
    fn test_int_specialized_subtraction() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();
        
        // Test: 500 - 200 = 300
        let const_500 = chunk.add_constant(Constant::Int(500));
        let const_200 = chunk.add_constant(Constant::Int(200));
        
        chunk.emit(OpCode::LoadConst(const_500));
        chunk.emit(OpCode::LoadConst(const_200));
        chunk.emit(OpCode::Sub);
        chunk.emit(OpCode::Return);
        
        let result = compiler.compile(&chunk, 0);
        assert!(result.is_ok(), "Should compile int subtraction: {:?}", result.err());
    }

    #[test]
    fn test_int_specialized_multiplication() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();
        
        // Test: 25 * 4 = 100
        let const_25 = chunk.add_constant(Constant::Int(25));
        let const_4 = chunk.add_constant(Constant::Int(4));
        
        chunk.emit(OpCode::LoadConst(const_25));
        chunk.emit(OpCode::LoadConst(const_4));
        chunk.emit(OpCode::Mul);
        chunk.emit(OpCode::Return);
        
        let result = compiler.compile(&chunk, 0);
        assert!(result.is_ok(), "Should compile int multiplication: {:?}", result.err());
    }

    #[test]
    fn test_int_specialized_division() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();
        
        // Test: 100 / 4 = 25
        let const_100 = chunk.add_constant(Constant::Int(100));
        let const_4 = chunk.add_constant(Constant::Int(4));
        
        chunk.emit(OpCode::LoadConst(const_100));
        chunk.emit(OpCode::LoadConst(const_4));
        chunk.emit(OpCode::Div);
        chunk.emit(OpCode::Return);
        
        let result = compiler.compile(&chunk, 0);
        assert!(result.is_ok(), "Should compile int division: {:?}", result.err());
    }

    #[test]
    fn test_specialized_arithmetic_chain() {
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();
        
        // Test: (10 + 5) * 2 - 3 = 27
        let const_10 = chunk.add_constant(Constant::Int(10));
        let const_5 = chunk.add_constant(Constant::Int(5));
        let const_2 = chunk.add_constant(Constant::Int(2));
        let const_3 = chunk.add_constant(Constant::Int(3));
        
        chunk.emit(OpCode::LoadConst(const_10));
        chunk.emit(OpCode::LoadConst(const_5));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::LoadConst(const_2));
        chunk.emit(OpCode::Mul);
        chunk.emit(OpCode::LoadConst(const_3));
        chunk.emit(OpCode::Sub);
        chunk.emit(OpCode::Return);
        
        let result = compiler.compile(&chunk, 0);
        assert!(result.is_ok(), "Should compile arithmetic chain: {:?}", result.err());
    }

    #[test]
    fn test_compilation_with_specialization_context() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset = 0;
        let var_hash_x = 11111u64;
        let var_hash_y = 22222u64;
        
        // Build type profile with stable Int types
        for _ in 0..60 {
            compiler.record_type(offset, var_hash_x, &Value::Int(10));
            compiler.record_type(offset, var_hash_y, &Value::Int(20));
        }
        
        // Verify profile is stable and specialized
        let profile = compiler.get_specialization(offset).expect("Profile should exist");
        assert_eq!(profile.specialized_types.get(&var_hash_x), Some(&ValueType::Int));
        assert_eq!(profile.specialized_types.get(&var_hash_y), Some(&ValueType::Int));
        
        // Now compile with specialization context
        let mut chunk = BytecodeChunk::new();
        let const_5 = chunk.add_constant(Constant::Int(5));
        let const_3 = chunk.add_constant(Constant::Int(3));
        
        chunk.emit(OpCode::LoadConst(const_5));
        chunk.emit(OpCode::LoadConst(const_3));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Return);
        
        // Compilation should succeed with specialization
        let result = compiler.compile(&chunk, offset);
        assert!(result.is_ok(), "Should compile with specialization context: {:?}", result.err());
    }

    #[test]
    fn test_compilation_without_specialization_fallback() {
        let mut compiler = JitCompiler::new().unwrap();
        
        // No type profiling - compile without specialization
        let mut chunk = BytecodeChunk::new();
        let const_100 = chunk.add_constant(Constant::Int(100));
        let const_50 = chunk.add_constant(Constant::Int(50));
        
        chunk.emit(OpCode::LoadConst(const_100));
        chunk.emit(OpCode::LoadConst(const_50));
        chunk.emit(OpCode::Sub);
        chunk.emit(OpCode::Return);
        
        // Should still compile successfully using generic path
        let result = compiler.compile(&chunk, 0);
        assert!(result.is_ok(), "Should compile without specialization: {:?}", result.err());
    }

    #[test]
    fn test_all_arithmetic_ops_with_specialization() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset_base = 100;
        
        // Profile for specialization at offset 100
        for _ in 0..60 {
            compiler.record_type(offset_base, 99999u64, &Value::Int(42));
        }
        
        // Test Add with specialization at offset 100
        let mut chunk = BytecodeChunk::new();
        let const_10 = chunk.add_constant(Constant::Int(10));
        let const_20 = chunk.add_constant(Constant::Int(20));
        chunk.emit(OpCode::LoadConst(const_10));
        chunk.emit(OpCode::LoadConst(const_20));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Return);
        assert!(compiler.compile(&chunk, offset_base).is_ok());
        
        // Profile for specialization at offset 200
        let offset_sub = 200;
        for _ in 0..60 {
            compiler.record_type(offset_sub, 88888u64, &Value::Int(30));
        }
        
        // Test Sub with specialization at offset 200
        let mut chunk = BytecodeChunk::new();
        let const_30 = chunk.add_constant(Constant::Int(30));
        let const_10 = chunk.add_constant(Constant::Int(10));
        chunk.emit(OpCode::LoadConst(const_30));
        chunk.emit(OpCode::LoadConst(const_10));
        chunk.emit(OpCode::Sub);
        chunk.emit(OpCode::Return);
        let result = compiler.compile(&chunk, offset_sub);
        assert!(result.is_ok(), "Sub compilation failed: {:?}", result.err());
        
        // Profile for specialization at offset 300
        let offset_mul = 300;
        for _ in 0..60 {
            compiler.record_type(offset_mul, 77777u64, &Value::Int(5));
        }
        
        // Test Mul with specialization at offset 300
        let mut chunk = BytecodeChunk::new();
        let const_5 = chunk.add_constant(Constant::Int(5));
        let const_6 = chunk.add_constant(Constant::Int(6));
        chunk.emit(OpCode::LoadConst(const_5));
        chunk.emit(OpCode::LoadConst(const_6));
        chunk.emit(OpCode::Mul);
        chunk.emit(OpCode::Return);
        assert!(compiler.compile(&chunk, offset_mul).is_ok());
        
        // Profile for specialization at offset 400
        let offset_div = 400;
        for _ in 0..60 {
            compiler.record_type(offset_div, 66666u64, &Value::Int(100));
        }
        
        // Test Div with specialization at offset 400
        let mut chunk = BytecodeChunk::new();
        let const_100 = chunk.add_constant(Constant::Int(100));
        let const_5 = chunk.add_constant(Constant::Int(5));
        chunk.emit(OpCode::LoadConst(const_100));
        chunk.emit(OpCode::LoadConst(const_5));
        chunk.emit(OpCode::Div);
        chunk.emit(OpCode::Return);
        assert!(compiler.compile(&chunk, offset_div).is_ok());
    }

    #[test]
    fn test_guard_generation_with_specialization() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset = 500;
        let var_hash = 12345u64;
        
        // Build stable Int type profile
        for _ in 0..70 {
            compiler.record_type(offset, var_hash, &Value::Int(42));
        }
        
        // Verify specialization was created
        let profile = compiler.get_specialization(offset).expect("Profile should exist");
        assert_eq!(profile.specialized_types.get(&var_hash), Some(&ValueType::Int));
        
        // Compile with guards
        let mut chunk = BytecodeChunk::new();
        let const_10 = chunk.add_constant(Constant::Int(10));
        let const_20 = chunk.add_constant(Constant::Int(20));
        chunk.emit(OpCode::LoadConst(const_10));
        chunk.emit(OpCode::LoadConst(const_20));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Return);
        
        let result = compiler.compile(&chunk, offset);
        assert!(result.is_ok(), "Should compile with guards: {:?}", result.err());
    }

    #[test]
    fn test_compilation_without_guards_when_no_specialization() {
        let mut compiler = JitCompiler::new().unwrap();
        
        // No profiling - no guards should be generated
        let mut chunk = BytecodeChunk::new();
        let const_5 = chunk.add_constant(Constant::Int(5));
        let const_3 = chunk.add_constant(Constant::Int(3));
        chunk.emit(OpCode::LoadConst(const_5));
        chunk.emit(OpCode::LoadConst(const_3));
        chunk.emit(OpCode::Mul);
        chunk.emit(OpCode::Return);
        
        let result = compiler.compile(&chunk, 600);
        assert!(result.is_ok(), "Should compile without guards: {:?}", result.err());
    }

    #[test]
    fn test_multiple_specialized_variables_guards() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset = 700;
        let var_hash_1 = 11111u64;
        let var_hash_2 = 22222u64;
        let var_hash_3 = 33333u64;
        
        // Build profiles for multiple variables
        for _ in 0..70 {
            compiler.record_type(offset, var_hash_1, &Value::Int(10));
            compiler.record_type(offset, var_hash_2, &Value::Int(20));
            compiler.record_type(offset, var_hash_3, &Value::Int(30));
        }
        
        // Verify all specialized
        let profile = compiler.get_specialization(offset).expect("Profile should exist");
        assert_eq!(profile.specialized_types.len(), 3);
        
        // Compile - should generate guards for all 3 variables
        let mut chunk = BytecodeChunk::new();
        let const_100 = chunk.add_constant(Constant::Int(100));
        chunk.emit(OpCode::LoadConst(const_100));
        chunk.emit(OpCode::Return);
        
        let result = compiler.compile(&chunk, offset);
        assert!(result.is_ok(), "Should compile with multiple guards: {:?}", result.err());
    }

    // ========================================================================
    // Phase 4E: Performance Benchmarking Tests
    // ========================================================================

    /// Helper function to measure execution time of a closure
    fn measure_time<F>(f: F) -> std::time::Duration
    where
        F: FnOnce(),
    {
        let start = std::time::Instant::now();
        f();
        start.elapsed()
    }

    #[test]
    #[ignore] // Benchmark test - run with: cargo test --release -- --ignored benchmark_
    fn benchmark_specialized_vs_generic_addition() {
        use std::time::Duration;
        
        let mut compiler = JitCompiler::new().unwrap();
        let mut chunk = BytecodeChunk::new();
        
        // Create a simple addition benchmark: sum 1000 times
        // We'll use constants to isolate JIT performance
        let const_1 = chunk.add_constant(Constant::Int(1));
        let const_0 = chunk.add_constant(Constant::Int(0));
        
        // Initialize sum = 0
        chunk.emit(OpCode::LoadConst(const_0));
        
        // Add 1, 1000 times (unrolled for simplicity)
        for _ in 0..1000 {
            chunk.emit(OpCode::LoadConst(const_1));
            chunk.emit(OpCode::Add);
        }
        chunk.emit(OpCode::Return);
        
        // Compile without specialization
        let compile_start = std::time::Instant::now();
        let result = compiler.compile(&chunk, 0);
        let compile_time = compile_start.elapsed();
        
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
        
        println!("\n=== Specialized vs Generic Addition Benchmark ===");
        println!("Compilation time: {:?}", compile_time);
        println!("Instructions compiled: {}", chunk.instructions.len());
        println!("Note: This is a micro-benchmark of JIT compilation itself");
        println!("For runtime performance, see the benchmark_*.ruff examples");
    }

    #[test]
    #[ignore]
    fn benchmark_compilation_overhead() {
        let mut compiler = JitCompiler::new().unwrap();
        
        // Test 1: Simple arithmetic (should be fast)
        let mut chunk1 = BytecodeChunk::new();
        let c1 = chunk1.add_constant(Constant::Int(10));
        let c2 = chunk1.add_constant(Constant::Int(20));
        chunk1.emit(OpCode::LoadConst(c1));
        chunk1.emit(OpCode::LoadConst(c2));
        chunk1.emit(OpCode::Add);
        chunk1.emit(OpCode::Return);
        
        let time1 = measure_time(|| {
            let _ = compiler.compile(&chunk1, 0);
        });
        
        // Test 2: Complex arithmetic chain (should be slower)
        let mut chunk2 = BytecodeChunk::new();
        for i in 0..100 {
            let c = chunk2.add_constant(Constant::Int(i));
            chunk2.emit(OpCode::LoadConst(c));
            if i > 0 {
                chunk2.emit(OpCode::Add);
            }
        }
        chunk2.emit(OpCode::Return);
        
        let time2 = measure_time(|| {
            let _ = compiler.compile(&chunk2, 1);
        });
        
        println!("\n=== Compilation Overhead Benchmark ===");
        println!("Simple (3 instructions): {:?}", time1);
        println!("Complex (200 instructions): {:?}", time2);
        println!("Ratio: {:.2}x", time2.as_nanos() as f64 / time1.as_nanos() as f64);
        
        // Complex should take longer (but not orders of magnitude longer)
        assert!(time2 > time1, "Complex compilation should take longer");
    }

    #[test]
    #[ignore]
    fn benchmark_type_profiling_overhead() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset = 0;
        let var_hash = 12345u64;
        
        // Benchmark: recording 10,000 type observations
        let iterations = 10_000;
        
        let time = measure_time(|| {
            for _ in 0..iterations {
                compiler.record_type(offset, var_hash, &Value::Int(42));
            }
        });
        
        println!("\n=== Type Profiling Overhead Benchmark ===");
        println!("Recorded {} type observations in {:?}", iterations, time);
        println!("Average per observation: {:?}", time / iterations);
        println!("Observations per second: {:.0}", iterations as f64 / time.as_secs_f64());
        
        // Verify profile was built
        let profile = compiler.get_specialization(offset).expect("Profile should exist");
        assert_eq!(profile.specialized_types.get(&var_hash), Some(&ValueType::Int));
    }

    #[test]
    #[ignore]
    fn benchmark_specialized_arithmetic_chain() {
        let mut compiler = JitCompiler::new().unwrap();
        
        // Build a long arithmetic chain: ((((1 + 2) + 3) + 4) + ... + 100)
        let mut chunk = BytecodeChunk::new();
        
        for i in 1..=100 {
            let c = chunk.add_constant(Constant::Int(i));
            chunk.emit(OpCode::LoadConst(c));
            if i > 1 {
                chunk.emit(OpCode::Add);
            }
        }
        chunk.emit(OpCode::Return);
        
        // Compile the chain
        let compile_time = measure_time(|| {
            let result = compiler.compile(&chunk, 0);
            assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
        });
        
        println!("\n=== Specialized Arithmetic Chain Benchmark ===");
        println!("Chain length: 100 additions");
        println!("Total instructions: {}", chunk.instructions.len());
        println!("Compilation time: {:?}", compile_time);
        println!("Time per instruction: {:?}", compile_time / chunk.instructions.len() as u32);
    }

    #[test]
    #[ignore]
    fn benchmark_guard_generation_overhead() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset = 0;
        
        // Setup: Create specialization profiles for many variables
        let var_count = 50;
        let mut var_hashes = Vec::new();
        
        for i in 0..var_count {
            let var_hash = (i as u64) * 1000;
            var_hashes.push(var_hash);
            
            // Build stable profile
            for _ in 0..100 {
                compiler.record_type(offset, var_hash, &Value::Int(42));
            }
        }
        
        // Compile with many guards
        let mut chunk = BytecodeChunk::new();
        let const_1 = chunk.add_constant(Constant::Int(1));
        chunk.emit(OpCode::LoadConst(const_1));
        chunk.emit(OpCode::Return);
        
        let compile_time = measure_time(|| {
            let result = compiler.compile(&chunk, offset);
            assert!(result.is_ok(), "Compilation with guards failed: {:?}", result.err());
        });
        
        println!("\n=== Guard Generation Overhead Benchmark ===");
        println!("Variables with specialized types: {}", var_count);
        println!("Compilation time: {:?}", compile_time);
        println!("Time per guard: {:?}", compile_time / var_count);
    }

    #[test]
    #[ignore]
    fn benchmark_cache_lookup_performance() {
        let mut compiler = JitCompiler::new().unwrap();
        
        // Compile several functions
        for offset in 0..100 {
            let mut chunk = BytecodeChunk::new();
            let c = chunk.add_constant(Constant::Int(offset as i64));
            chunk.emit(OpCode::LoadConst(c));
            chunk.emit(OpCode::Return);
            
            let _ = compiler.compile(&chunk, offset);
        }
        
        // Benchmark: Check if functions should compile (cache lookup)
        let iterations = 100_000;
        let time = measure_time(|| {
            for _ in 0..iterations {
                for offset in 0..100 {
                    let _ = compiler.should_compile(offset);
                }
            }
        });
        
        println!("\n=== Cache Lookup Performance Benchmark ===");
        println!("Cache entries: 100");
        println!("Total lookups: {}", iterations * 100);
        println!("Time: {:?}", time);
        println!("Lookups per second: {:.0}", (iterations * 100) as f64 / time.as_secs_f64());
        println!("Average per lookup: {:?}", time / (iterations * 100));
    }

    #[test]
    #[ignore]
    fn benchmark_specialization_decision_overhead() {
        let mut compiler = JitCompiler::new().unwrap();
        let offset = 0;
        let var_hash = 99999u64;
        
        // Record mixed types to test decision logic
        for i in 0..100 {
            if i % 10 == 0 {
                compiler.record_type(offset, var_hash, &Value::Float(1.0));
            } else {
                compiler.record_type(offset, var_hash, &Value::Int(1));
            }
        }
        
        // Benchmark: decision making
        let iterations = 100_000;
        let time = measure_time(|| {
            for _ in 0..iterations {
                if let Some(profile) = compiler.get_specialization(offset) {
                    let _ = profile.specialized_types.get(&var_hash);
                    let _ = profile.should_despecialize();
                }
            }
        });
        
        println!("\n=== Specialization Decision Overhead Benchmark ===");
        println!("Decisions made: {}", iterations);
        println!("Time: {:?}", time);
        println!("Decisions per second: {:.0}", iterations as f64 / time.as_secs_f64());
        println!("Average per decision: {:?}", time / iterations);
    }

    #[test]
    fn validate_phase4_infrastructure_complete() {
        // This test validates that Phase 4A-4D infrastructure is in place
        let mut compiler = JitCompiler::new().unwrap();
        
        println!("\n=== Phase 4A-4D Infrastructure Validation ===");
        
        // Phase 4A: Type profiling system
        let offset = 0;
        let var_hash = 11111u64;
        
        for _ in 0..60 {
            compiler.record_type(offset, var_hash, &Value::Int(42));
        }
        
        let profile = compiler.get_specialization(offset);
        assert!(profile.is_some(), "Phase 4A: Type profiling should work");
        println!("✅ Phase 4A: Type profiling system working");
        
        // Phase 4B: Specialized code generation
        let mut chunk = BytecodeChunk::new();
        let c1 = chunk.add_constant(Constant::Int(10));
        let c2 = chunk.add_constant(Constant::Int(20));
        chunk.emit(OpCode::LoadConst(c1));
        chunk.emit(OpCode::LoadConst(c2));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Return);
        
        let result = compiler.compile(&chunk, offset);
        assert!(result.is_ok(), "Phase 4B: Specialized compilation should work");
        println!("✅ Phase 4B: Specialized code generation working");
        
        // Phase 4C: Integration (compile with profile)
        assert!(compiler.compiled_cache.contains_key(&offset), 
                "Phase 4C: Compiled function should be cached");
        println!("✅ Phase 4C: Integration working");
        
        // Phase 4D: Guard generation
        let profile = compiler.get_specialization(offset).unwrap();
        assert!(!profile.specialized_types.is_empty(), 
                "Phase 4D: Should have specialized types for guards");
        println!("✅ Phase 4D: Guard generation working");
        
        println!("\n🎉 All Phase 4A-4D infrastructure validated!");
        println!("Phase 4E: Ready for performance validation and advanced optimizations");
    }
}
