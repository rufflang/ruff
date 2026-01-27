// File: src/jit.rs
//
// JIT Compilation module for Ruff bytecode using Cranelift.
// Provides just-in-time compilation of hot bytecode functions to native machine code.

use crate::bytecode::{BytecodeChunk, Constant, OpCode};
use crate::interpreter::Value;
use cranelift::prelude::*;
use cranelift::codegen::ir::FuncRef;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module, FuncId};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// JIT compilation threshold - number of executions before compiling
const JIT_THRESHOLD: usize = 100;

/// Guard failure threshold - recompile if guard failures exceed this percentage
const GUARD_FAILURE_THRESHOLD: f64 = 0.10; // 10%

/// Minimum samples before type specialization
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
}

impl VMContext {
    /// Create a new VMContext from VM state
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
        }
    }
    
    /// Create with variable name mapping
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
        }
    }
}

/// Compiled function type: takes VMContext pointer, returns status code
type CompiledFn = unsafe extern "C" fn(*mut VMContext) -> i64;

/// Type profile for a variable or operation
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Int,
    Float,
    Bool,
    Mixed,
}

/// Specialization strategy for a function
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
    /// Variable storage - maps variable names to Cranelift values (reserved for future use)
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
                    let value = self.pop_value()?;
                    
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
                } else {
                    // Fallback: just pop the value
                    let _val = self.pop_value()?;
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
                // Same as StoreVar for now
                if let (Some(ctx), Some(store_func)) = (self.ctx_param, self.store_var_func) {
                    let value = self.pop_value()?;
                    
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
                } else {
                    let _val = self.pop_value()?;
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
            
            // Set specialization info if available
            if let Some(spec) = self.type_profiles.get(&offset) {
                translator.set_specialization(spec.clone());
            }
            
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
    
    /// Record a type observation for profiling
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
    pub fn record_guard_success(&mut self, offset: usize) {
        if let Some(profile) = self.type_profiles.get_mut(&offset) {
            profile.guard_successes += 1;
        }
    }
    
    /// Record a guard failure
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
}
