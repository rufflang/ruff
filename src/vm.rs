// File: src/vm.rs
//
// Virtual Machine for executing Ruff bytecode.
// Stack-based VM with support for function calls, closures, and all Ruff features.

use crate::ast::Pattern;
use crate::bytecode::{BytecodeChunk, Constant, OpCode};
use crate::interpreter::{Environment, Interpreter, Value};
use crate::jit::{JitCompiler, CompiledFn, CompiledFnInfo};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// JIT compilation threshold for functions
/// A function will be JIT-compiled after being called this many times
const JIT_FUNCTION_THRESHOLD: usize = 100;

/// Upvalue: heap-allocated captured variable for closures
#[derive(Debug, Clone)]
/// Upvalue - captured variable for closures
/// Infrastructure for closure variable capture
#[allow(dead_code)] // TODO: Full closure upvalue implementation
struct Upvalue {
    /// The captured value
    value: Arc<Mutex<Value>>,

    /// Whether the upvalue is still on the stack (false) or has been closed (true)
    is_closed: bool,

    /// If still on stack, the stack index
    stack_index: Option<usize>,
}

/// Virtual Machine for executing bytecode
#[allow(dead_code)] // VM not yet integrated into execution path
pub struct VM {
    /// Value stack for computation
    stack: Vec<Value>,

    /// Call frames for function calls
    call_frames: Vec<CallFrame>,

    /// Global environment (must be Mutex for interior mutability)
    globals: Arc<Mutex<Environment>>,

    /// Current instruction pointer
    ip: usize,

    /// Current bytecode chunk
    chunk: BytecodeChunk,

    /// Interpreter instance for calling native functions
    interpreter: Interpreter,

    /// Upvalues (captured variables) - indexed by upvalue ID
    /// These are heap-allocated shared references to captured variables
    upvalues: Vec<Upvalue>,

    /// Exception handler stack for try/catch blocks
    /// Tracks nested try blocks and their catch handlers
    exception_handlers: Vec<ExceptionHandlerFrame>,

    /// JIT compiler for hot code paths
    jit_compiler: JitCompiler,

    /// JIT enabled flag (can be disabled for debugging)
    jit_enabled: bool,

    /// Function call stack for error reporting (tracks function names)
    function_call_stack: Vec<String>,

    /// Function call counts for JIT compilation threshold
    /// Maps function name to number of times it has been called
    function_call_counts: HashMap<String, usize>,

    /// Cache of JIT-compiled functions
    /// Maps function name to compiled native code
    compiled_functions: HashMap<String, CompiledFn>,
    
    /// Enhanced cache with direct-arg variants for recursive functions
    /// Maps function name to CompiledFnInfo (includes both standard and direct-arg)
    compiled_fn_info: HashMap<String, CompiledFnInfo>,
    
    /// Cache of var_names for JIT-compiled functions
    /// This avoids re-computing hash mappings on every call
    jit_var_names_cache: HashMap<String, HashMap<u64, String>>,
    
    /// Current recursion depth (for optimization and debugging)
    recursion_depth: usize,
    
    /// Maximum recursion depth observed (for profiling)
    max_recursion_depth: usize,
    
    /// Inline cache for function calls at specific call sites
    /// Key: (chunk_id, instruction_pointer) uniquely identifies a call site
    /// Value: Cached function pointer and metadata for fast dispatch
    inline_cache: HashMap<CallSiteId, InlineCacheEntry>,
    
    /// Tokio runtime handle for spawning async tasks
    /// This allows the VM to spawn truly concurrent async tasks
    runtime_handle: tokio::runtime::Handle,
}

/// Unique identifier for a call site (location in bytecode where a Call occurs)
/// Used as key for inline cache lookups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CallSiteId {
    /// Unique identifier for the chunk/function containing this call site
    /// We use the chunk name's hash for stability across executions
    chunk_id: u64,
    /// Instruction pointer within the chunk where the Call opcode is
    ip: usize,
}

impl CallSiteId {
    fn new(chunk_name: Option<&str>, ip: usize) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let chunk_id = match chunk_name {
            Some(name) => {
                let mut hasher = DefaultHasher::new();
                name.hash(&mut hasher);
                hasher.finish()
            }
            None => 0, // Anonymous/top-level chunk
        };
        
        Self { chunk_id, ip }
    }
}

/// Cached information for a call site to enable fast function dispatch
#[derive(Clone)]
struct InlineCacheEntry {
    /// The expected function name at this call site
    /// Used to validate cache hit (guard against function reassignment)
    expected_func_name: String,
    
    /// Cached JIT-compiled function pointer (if available)
    /// None if function is not JIT-compiled yet
    compiled_fn: Option<CompiledFn>,
    
    /// Cached var_names HashMap for this function (avoids rebuilding on every call)
    var_names: HashMap<u64, String>,
    
    /// Cache hit count (for profiling and debugging)
    hit_count: usize,
    
    /// Cache miss count (for profiling - indicates polymorphic call sites)
    miss_count: usize,
}

impl InlineCacheEntry {
    fn new(func_name: &str, compiled_fn: Option<CompiledFn>, var_names: HashMap<u64, String>) -> Self {
        Self {
            expected_func_name: func_name.to_string(),
            compiled_fn,
            var_names,
            hit_count: 0,
            miss_count: 0,
        }
    }
}

/// Exception handler frame for active try blocks
#[derive(Debug, Clone)]
struct ExceptionHandlerFrame {
    /// Instruction pointer for catch block
    catch_ip: usize,

    /// Stack size when entering try block (for unwinding)
    stack_offset: usize,

    /// Call frame depth when entering try block (for unwinding)
    frame_offset: usize,
}

/// Generator state for suspended execution
/// Infrastructure for generator resume functionality
#[allow(dead_code)] // TODO: Full generator state restoration
#[derive(Debug, Clone)]
pub struct GeneratorState {
    /// Instruction pointer where generator yielded
    pub ip: usize,

    /// Stack snapshot at yield point
    pub stack: Vec<Value>,

    /// Call frame stack at yield point (stored as separate values to avoid circular dependency)
    pub call_frames_data: Vec<CallFrameData>,

    /// Bytecode chunk being executed
    pub chunk: BytecodeChunk,

    /// Local variables at yield point
    pub locals: HashMap<String, Value>,

    /// Captured variables at yield point
    pub captured: HashMap<String, Arc<Mutex<Value>>>,

    /// Whether the generator has finished
    pub is_exhausted: bool,
}

/// Serializable call frame data for generator state
#[derive(Debug, Clone)]
pub struct CallFrameData {
    pub return_ip: usize,
    pub stack_offset: usize,
    pub locals: HashMap<String, Value>,
    pub captured: HashMap<String, Arc<Mutex<Value>>>,
}

/// Call frame for function calls
#[derive(Debug, Clone)]
#[allow(dead_code)] // CallFrame not yet used - nested calls incomplete
struct CallFrame {
    /// Return address (instruction pointer)
    return_ip: usize,

    /// Stack offset for this frame
    stack_offset: usize,

    /// Local environment for this frame (parameters and local variables)
    locals: HashMap<String, Value>,

    /// Captured variables (upvalues) with shared mutable state
    captured: HashMap<String, Arc<Mutex<Value>>>,

    /// Previous chunk (for returning)
    prev_chunk: Option<BytecodeChunk>,

    /// Whether this function is async (for wrapping return values in Promises)
    is_async: bool,
}

#[allow(dead_code)] // VM not yet integrated into execution path
impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            call_frames: Vec::new(),
            globals: Arc::new(Mutex::new(Environment::new())),
            ip: 0,
            chunk: BytecodeChunk::new(),
            interpreter: Interpreter::new(),
            upvalues: Vec::new(),
            exception_handlers: Vec::new(),
            jit_compiler: JitCompiler::new().unwrap_or_else(|e| {
                eprintln!("Warning: Failed to initialize JIT compiler: {}", e);
                eprintln!("Falling back to interpreter-only mode");
                JitCompiler::default()
            }),
            jit_enabled: true,
            function_call_stack: Vec::new(),
            function_call_counts: HashMap::new(),
            compiled_functions: HashMap::new(),
            compiled_fn_info: HashMap::new(),
            jit_var_names_cache: HashMap::new(),
            recursion_depth: 0,
            max_recursion_depth: 0,
            inline_cache: HashMap::new(),
            runtime_handle: tokio::runtime::Handle::try_current()
                .unwrap_or_else(|_| {
                    // If not in a tokio runtime, create one
                    crate::interpreter::AsyncRuntime::runtime().handle().clone()
                }),
        }
    }

    /// Set the global environment (for accessing built-in functions)
    pub fn set_globals(&mut self, env: Arc<Mutex<Environment>>) {
        self.globals = env.clone();
        // Also set the interpreter's environment so it can resolve native functions
        self.interpreter.set_env(env);
    }

    /// Enable or disable JIT compilation
    pub fn set_jit_enabled(&mut self, enabled: bool) {
        self.jit_enabled = enabled;
        self.jit_compiler.set_enabled(enabled);
    }

    /// Get JIT compilation statistics
    pub fn jit_stats(&self) -> crate::jit::JitStats {
        self.jit_compiler.stats()
    }

    /// Get the VM's function call stack for error reporting
    pub fn get_call_stack(&self) -> Vec<String> {
        self.function_call_stack.clone()
    }

    /// Execute a bytecode chunk
    pub fn execute(&mut self, chunk: BytecodeChunk) -> Result<Value, String> {
        self.chunk = chunk;
        self.ip = 0;
        self.stack.clear();

        loop {
            if self.ip >= self.chunk.instructions.len() {
                // Reached end of program
                return Ok(Value::Null);
            }

            // Check if we should JIT compile this hot path
            if self.jit_enabled {
                // For loops (backward jumps), check if we should compile
                if let Some(OpCode::JumpBack(jump_target)) = self.chunk.instructions.get(self.ip) {
                    if self.jit_compiler.should_compile(self.ip) {
                        // PRE-SCAN: Check if loop contains only supported opcodes
                        // This prevents compilation failures and maintains correctness
                        if self.jit_compiler.can_compile_loop(&self.chunk, *jump_target, self.ip) {
                            // Try to compile this hot loop
                            // IMPORTANT: Compile from the loop START (jump_target), not from the JumpBack!
                            // The JumpBack just marks the end of the loop
                            match self.jit_compiler.compile(&self.chunk, *jump_target) {
                                Ok(compiled_fn) => {
                                    // Successfully compiled! Now EXECUTE the compiled function
                                    if std::env::var("DEBUG_JIT").is_ok() {
                                        eprintln!(
                                            "JIT: Successfully compiled hot loop starting at offset {} - EXECUTING NOW!",
                                            jump_target
                                        );
                                    }

                                    // Get VM pointer early (before any borrows)
                                    let vm_ptr: *mut std::ffi::c_void = self as *mut _ as *mut std::ffi::c_void;

                                    // Execute the JIT-compiled function
                                    // Get mutable pointers to VM state for VMContext
                                    let stack_ptr: *mut Vec<Value> = &mut self.stack;
                                    
                                    // Get globals - lock and get mutable reference to the first scope
                                    let mut globals_guard = self.globals.lock().unwrap();
                                    let globals_ptr: *mut HashMap<String, Value> = &mut globals_guard.scopes[0];
                                    
                                    // Get locals from current call frame, or use globals if at top level
                                    let locals_ptr: *mut HashMap<String, Value> = if let Some(frame) = self.call_frames.last_mut() {
                                        &mut frame.locals
                                    } else {
                                        // Top-level: use globals as locals
                                        globals_ptr
                                    };
                                    
                                    // Create VMContext with VM pointer for Call opcode support
                                    let mut vm_context = crate::jit::VMContext::new_with_vm(
                                        stack_ptr,
                                        locals_ptr,
                                        globals_ptr,
                                        vm_ptr,
                                    );
                                    
                                    // Execute the compiled function!
                                    let result_code = unsafe {
                                        (compiled_fn)(&mut vm_context)
                                    };
                                    
                                    // Drop the globals lock
                                    drop(globals_guard);
                                    
                                    if result_code != 0 {
                                        return Err(format!("JIT execution failed with code: {}", result_code));
                                    }
                                    
                                    if std::env::var("DEBUG_JIT").is_ok() {
                                        eprintln!("JIT: Execution completed successfully!");
                                    }
                                    
                                    // The JIT function executed the loop completely
                                    // Skip past the JumpBack instruction to avoid re-executing the loop
                                    self.ip += 1;
                                    continue;
                                }
                                Err(e) => {
                                    // Compilation failed - this shouldn't happen if pre-scan worked
                                    if std::env::var("DEBUG_JIT").is_ok() {
                                        eprintln!(
                                            "JIT: Unexpected compilation failure at offset {}: {}",
                                            jump_target, e
                                        );
                                    }
                                }
                            }
                        } else {
                            // Loop contains unsupported opcodes - skip JIT compilation
                            // This is normal for loops with function calls, strings, etc.
                            if std::env::var("DEBUG_JIT").is_ok() {
                                eprintln!(
                                    "JIT: Loop at offset {} contains unsupported opcodes, using interpreter",
                                    jump_target
                                );
                            }
                        }
                    }
                }
            }

            let instruction = self.chunk.instructions[self.ip].clone();
            self.ip += 1;

            match instruction {
                OpCode::LoadConst(index) => {
                    let constant = &self.chunk.constants[index];
                    let value = self.constant_to_value(constant)?;
                    self.stack.push(value);
                }

                OpCode::LoadVar(name) => {
                    // Look in current call frame first - check captured variables (Arc<Mutex<Value>>) first, then locals
                    let value = if let Some(frame) = self.call_frames.last() {
                        if std::env::var("DEBUG_VM").is_ok() {
                            eprintln!("LoadVar('{}'):  checking frame captured ({} entries) and locals ({} entries)", 
                                name, frame.captured.len(), frame.locals.len());
                        }

                        // Check captured variables first (these are shared mutable references)
                        if let Some(captured_ref) = frame.captured.get(&name) {
                            if std::env::var("DEBUG_VM").is_ok() {
                                eprintln!("LoadVar('{}'): found in captured", name);
                            }
                            Some(captured_ref.lock().unwrap().clone())
                        } else {
                            // Fall back to locals
                            frame.locals.get(&name).cloned()
                        }
                    } else {
                        if std::env::var("DEBUG_VM").is_ok() {
                            eprintln!("LoadVar('{}'): no call frame", name);
                        }
                        None
                    };

                    let value = value
                        .or_else(|| {
                            let global_val = self.globals.lock().unwrap().get(&name);
                            if std::env::var("DEBUG_VM").is_ok() {
                                eprintln!(
                                    "LoadVar('{}'): checking globals -> {:?}",
                                    name,
                                    global_val.is_some()
                                );
                            }
                            global_val
                        })
                        .ok_or_else(|| {
                            if std::env::var("DEBUG_VM").is_ok() {
                                eprintln!(
                                    "LoadVar('{}'): FAILED - not in captured, locals or globals",
                                    name
                                );
                                eprintln!(
                                    "  Current frame captured: {:?}",
                                    self.call_frames
                                        .last()
                                        .map(|f| f.captured.keys().collect::<Vec<_>>())
                                );
                                eprintln!(
                                    "  Current frame locals: {:?}",
                                    self.call_frames
                                        .last()
                                        .map(|f| f.locals.keys().collect::<Vec<_>>())
                                );
                            }
                            format!("Undefined variable: {}", name)
                        })?;

                    self.stack.push(value);
                }

                OpCode::LoadGlobal(name) => {
                    let value = self
                        .globals
                        .lock()
                        .unwrap()
                        .get(&name)
                        .ok_or_else(|| format!("Undefined global: {}", name))?;
                    self.stack.push(value);
                }

                OpCode::StoreVar(name) => {
                    let value = self.stack.last().ok_or("Stack underflow")?.clone();

                    if let Some(frame) = self.call_frames.last_mut() {
                        // Check if this is a captured variable first
                        if let Some(captured_ref) = frame.captured.get(&name) {
                            if std::env::var("DEBUG_VM").is_ok() {
                                eprintln!("StoreVar('{}'): updating captured variable", name);
                            }
                            *captured_ref.lock().unwrap() = value;
                        } else {
                            // Store in local variables
                            if std::env::var("DEBUG_VM").is_ok() {
                                eprintln!("StoreVar('{}'): storing in frame locals", name);
                            }
                            frame.locals.insert(name, value);
                        }
                    } else {
                        if std::env::var("DEBUG_VM").is_ok() {
                            eprintln!("StoreVar('{}'): storing in globals (no frame)", name);
                        }
                        // Store in global
                        self.globals.lock().unwrap().set(name, value);
                    }
                }

                OpCode::StoreGlobal(name) => {
                    let value = self.stack.last().ok_or("Stack underflow")?.clone();

                    self.globals.lock().unwrap().set(name, value);
                }

                OpCode::Pop => {
                    self.stack.pop().ok_or("Stack underflow")?;
                }

                OpCode::Dup => {
                    let value = self.stack.last().ok_or("Stack underflow")?.clone();
                    self.stack.push(value);
                }

                // Arithmetic operations
                OpCode::Add => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = self.binary_op(&left, "+", &right)?;
                    self.stack.push(result);
                }

                OpCode::Sub => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = self.binary_op(&left, "-", &right)?;
                    self.stack.push(result);
                }

                OpCode::Mul => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = self.binary_op(&left, "*", &right)?;
                    self.stack.push(result);
                }

                OpCode::Div => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = self.binary_op(&left, "/", &right)?;
                    self.stack.push(result);
                }

                OpCode::Mod => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = self.binary_op(&left, "%", &right)?;
                    self.stack.push(result);
                }

                OpCode::Negate => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let result = self.unary_op("-", &value)?;
                    self.stack.push(result);
                }

                // Comparison operations
                OpCode::Equal => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = Value::Bool(self.values_equal(&left, &right));
                    self.stack.push(result);
                }

                OpCode::NotEqual => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = Value::Bool(!self.values_equal(&left, &right));
                    self.stack.push(result);
                }

                OpCode::LessThan => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = self.compare_op(&left, "<", &right)?;
                    self.stack.push(result);
                }

                OpCode::GreaterThan => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = self.compare_op(&left, ">", &right)?;
                    self.stack.push(result);
                }

                OpCode::LessEqual => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = self.compare_op(&left, "<=", &right)?;
                    self.stack.push(result);
                }

                OpCode::GreaterEqual => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = self.compare_op(&left, ">=", &right)?;
                    self.stack.push(result);
                }

                // Logical operations
                OpCode::Not => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let result = Value::Bool(!self.is_truthy(&value));
                    self.stack.push(result);
                }

                OpCode::And => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = Value::Bool(self.is_truthy(&left) && self.is_truthy(&right));
                    self.stack.push(result);
                }

                OpCode::Or => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = Value::Bool(self.is_truthy(&left) || self.is_truthy(&right));
                    self.stack.push(result);
                }

                // Control flow
                OpCode::Jump(target) => {
                    self.ip = target;
                }

                OpCode::JumpIfFalse(target) => {
                    let condition = self.stack.last().ok_or("Stack underflow")?;
                    if !self.is_truthy(condition) {
                        self.ip = target;
                    }
                }

                OpCode::JumpIfTrue(target) => {
                    let condition = self.stack.last().ok_or("Stack underflow")?;
                    if self.is_truthy(condition) {
                        self.ip = target;
                    }
                }

                OpCode::JumpBack(target) => {
                    self.ip = target;
                }

                // Function operations
                OpCode::Call(arg_count) => {
                    // Create call site ID for inline cache lookup
                    // This identifies where in the bytecode this call occurs
                    let call_site_id = CallSiteId::new(self.chunk.name.as_deref(), self.ip);
                    
                    // Function is on top of stack, then arguments below it
                    // Stack layout: [... arg1, arg2, ..., argN, function]
                    let function = self.stack.pop().ok_or("Stack underflow in Call")?;

                    // Collect arguments
                    let mut args = Vec::new();
                    for _ in 0..arg_count {
                        args.push(self.stack.pop().ok_or("Stack underflow in Call args")?);
                    }
                    args.reverse(); // Arguments were pushed in order

                    // Check if this is a bytecode function or native function
                    match &function {
                        Value::BytecodeFunction { chunk, captured } => {
                            // Track function calls for JIT compilation
                            if self.jit_enabled && !chunk.is_generator {
                                let func_name = chunk.name.as_deref().unwrap_or("<anonymous>");
                                
                                // Get VM pointer early (before any borrows)
                                let vm_ptr: *mut std::ffi::c_void = self as *mut _ as *mut std::ffi::c_void;
                                
                                // === INLINE CACHE FAST PATH ===
                                // Check inline cache for this specific call site
                                // This is faster than HashMap lookups because:
                                // 1. We cache the compiled_fn pointer directly (no string hash)
                                // 2. We cache var_names to avoid rebuilding on every call
                                // 3. We validate with a simple string comparison (guard)
                                if let Some(cache_entry) = self.inline_cache.get_mut(&call_site_id) {
                                    // Cache hit! Validate that function hasn't changed (guard)
                                    if cache_entry.expected_func_name == func_name {
                                        cache_entry.hit_count += 1;
                                        
                                        // If we have a compiled function, use it directly
                                        if let Some(compiled_fn) = cache_entry.compiled_fn {
                                            // Create locals HashMap for the function parameters
                                            let mut func_locals = HashMap::new();
                                            
                                            // Check if we can use fast arg passing (≤4 integer args)
                                            let use_fast_args = args.len() <= 4 && args.iter().all(|a| matches!(a, Value::Int(_)));
                                            
                                            // Bind arguments to parameter names
                                            for (i, param_name) in chunk.params.iter().enumerate() {
                                                if let Some(arg) = args.get(i) {
                                                    func_locals.insert(param_name.clone(), arg.clone());
                                                }
                                            }
                                            
                                            // OPTIMIZATION: Get mutable pointer directly to cached var_names
                                            // This avoids HashMap clone on every call!
                                            let var_names_ptr: *mut HashMap<u64, String> = &mut cache_entry.var_names;
                                            
                                            // Save stack size to detect return value
                                            let stack_size_before = self.stack.len();
                                            
                                            // Execute the JIT-compiled function
                                            let stack_ptr: *mut Vec<Value> = &mut self.stack;
                                            
                                            // Get globals - drop lock before execution
                                            let globals_ptr: *mut HashMap<String, Value> = {
                                                let mut globals_guard = self.globals.lock().unwrap();
                                                let ptr = &mut globals_guard.scopes[0] as *mut HashMap<String, Value>;
                                                drop(globals_guard);
                                                ptr
                                            };
                                            
                                            let locals_ptr: *mut HashMap<String, Value> = &mut func_locals;
                                            // var_names_ptr already created above from cache entry
                                            
                                            // Create VMContext with fast arg fields
                                            let mut vm_context = crate::jit::VMContext {
                                                stack_ptr,
                                                locals_ptr,
                                                globals_ptr,
                                                var_names_ptr,
                                                vm_ptr,
                                                return_value: 0,
                                                has_return_value: false,
                                                arg0: if use_fast_args && args.len() > 0 { 
                                                    if let Value::Int(n) = args[0] { n } else { 0 }
                                                } else { 0 },
                                                arg1: if use_fast_args && args.len() > 1 { 
                                                    if let Value::Int(n) = args[1] { n } else { 0 }
                                                } else { 0 },
                                                arg2: if use_fast_args && args.len() > 2 { 
                                                    if let Value::Int(n) = args[2] { n } else { 0 }
                                                } else { 0 },
                                                arg3: if use_fast_args && args.len() > 3 { 
                                                    if let Value::Int(n) = args[3] { n } else { 0 }
                                                } else { 0 },
                                                arg_count: args.len() as i64,
                                            };
                                            
                                            let result_code = unsafe {
                                                compiled_fn(&mut vm_context)
                                            };
                                            
                                            if result_code != 0 {
                                                return Err(format!("JIT execution failed with code: {}", result_code));
                                            }
                                            
                                            if vm_context.has_return_value {
                                                self.stack.push(Value::Int(vm_context.return_value));
                                            } else if self.stack.len() > stack_size_before {
                                                // Return value was pushed to stack
                                            } else {
                                                return Err("JIT-compiled function did not return a value".to_string());
                                            }
                                            
                                            continue; // Skip to next instruction
                                        }
                                    } else {
                                        // Cache miss - function at this call site changed (polymorphic)
                                        cache_entry.miss_count += 1;
                                    }
                                }
                                
                                // === SLOW PATH - populate cache and execute ===
                                // PHASE 7 STEP 12: Check for direct-arg version FIRST for single-int-arg calls
                                // This is the key optimization for recursive functions - avoids FFI on each call
                                if args.len() == 1 {
                                    if let Value::Int(arg_val) = args[0] {
                                        if let Some(fn_info) = self.compiled_fn_info.get(func_name) {
                                            if let Some(direct_fn) = fn_info.fn_with_arg {
                                                // ULTRA-FAST PATH: Use direct-arg JIT variant!
                                                // This function takes an i64 directly and returns the result
                                                // WITHOUT going through VMContext arg fields or FFI for recursion
                                                
                                                // Create minimal VMContext for the function
                                                let stack_ptr: *mut Vec<Value> = &mut self.stack;
                                                let globals_ptr: *mut HashMap<String, Value> = {
                                                    let mut globals_guard = self.globals.lock().unwrap();
                                                    let ptr = &mut globals_guard.scopes[0] as *mut HashMap<String, Value>;
                                                    drop(globals_guard);
                                                    ptr
                                                };
                                                let mut func_locals = HashMap::new();
                                                let locals_ptr: *mut HashMap<String, Value> = &mut func_locals;
                                                
                                                let mut vm_context = crate::jit::VMContext {
                                                    stack_ptr,
                                                    locals_ptr,
                                                    globals_ptr,
                                                    var_names_ptr: std::ptr::null_mut(),
                                                    vm_ptr,
                                                    return_value: 0,
                                                    has_return_value: false,
                                                    arg0: arg_val,
                                                    arg1: 0,
                                                    arg2: 0,
                                                    arg3: 0,
                                                    arg_count: 1,
                                                };
                                                
                                                // Execute direct-arg function - result is returned directly!
                                                let result = unsafe {
                                                    direct_fn(&mut vm_context, arg_val)
                                                };
                                                
                                                if std::env::var("DEBUG_JIT").is_ok() {
                                                    eprintln!("JIT: Interpreter direct-arg call to '{}' with arg {} returned {}", 
                                                        func_name, arg_val, result);
                                                }
                                                
                                                self.stack.push(Value::Int(result));
                                                continue; // Skip to next instruction
                                            }
                                        }
                                    }
                                }
                                
                                // Check if we have a JIT-compiled version (standard path)
                                if let Some(compiled_fn) = self.compiled_functions.get(func_name) {
                                    // Fast path: Call JIT-compiled version
                                    
                                    // Create locals HashMap for the function parameters
                                    let mut func_locals = HashMap::new();
                                    
                                    // Bind arguments to parameter names
                                    for (i, param_name) in chunk.params.iter().enumerate() {
                                        if let Some(arg) = args.get(i) {
                                            func_locals.insert(param_name.clone(), arg.clone());
                                        }
                                    }
                                    
                                    // Get or create var_names from cache
                                    let func_name_owned = func_name.to_string();
                                    let var_names = if let Some(cached) = self.jit_var_names_cache.get(&func_name_owned) {
                                        cached.clone()
                                    } else {
                                        // Build var_names once and cache it
                                        let mut cached_var_names = HashMap::new();
                                        
                                        // Register parameter names
                                        for param_name in &chunk.params {
                                            use std::collections::hash_map::DefaultHasher;
                                            use std::hash::{Hash, Hasher};
                                            let mut hasher = DefaultHasher::new();
                                            param_name.hash(&mut hasher);
                                            let hash = hasher.finish();
                                            cached_var_names.insert(hash, param_name.clone());
                                        }
                                        
                                        // Register all LoadVar names
                                        for instr in &chunk.instructions {
                                            if let OpCode::LoadVar(name) = instr {
                                                use std::collections::hash_map::DefaultHasher;
                                                use std::hash::{Hash, Hasher};
                                                let mut hasher = DefaultHasher::new();
                                                name.hash(&mut hasher);
                                                let hash = hasher.finish();
                                                cached_var_names.insert(hash, name.clone());
                                            }
                                        }
                                        
                                        self.jit_var_names_cache.insert(func_name_owned.clone(), cached_var_names.clone());
                                        cached_var_names
                                    };
                                    
                                    // === POPULATE INLINE CACHE ===
                                    // Store in inline cache for faster lookup next time
                                    self.inline_cache.insert(
                                        call_site_id,
                                        InlineCacheEntry::new(func_name, Some(*compiled_fn), var_names.clone()),
                                    );
                                    
                                    let mut var_names_mut = var_names;
                                    
                                    // Save stack size to detect return value
                                    let stack_size_before = self.stack.len();
                                    
                                    // Execute the JIT-compiled function
                                    // Get mutable pointers to VM state for VMContext
                                    let stack_ptr: *mut Vec<Value> = &mut self.stack;
                                    
                                    // Get globals - drop lock before execution to avoid deadlock on recursive calls
                                    let globals_ptr: *mut HashMap<String, Value> = {
                                        let mut globals_guard = self.globals.lock().unwrap();
                                        let ptr = &mut globals_guard.scopes[0] as *mut HashMap<String, Value>;
                                        drop(globals_guard);
                                        ptr
                                    };
                                    
                                    // Use the function's locals (with bound parameters)
                                    let locals_ptr: *mut HashMap<String, Value> = &mut func_locals;
                                    
                                    // Set up var_names for JIT variable resolution
                                    let var_names_ptr: *mut HashMap<u64, String> = &mut var_names_mut;
                                    
                                    // Check if we can use fast arg passing (≤4 integer args)
                                    let use_fast_args = args.len() <= 4 && args.iter().all(|a| matches!(a, Value::Int(_)));
                                    
                                    // Create VMContext with fast arg fields
                                    let mut vm_context = crate::jit::VMContext {
                                        stack_ptr,
                                        locals_ptr,
                                        globals_ptr,
                                        var_names_ptr,
                                        vm_ptr,
                                        return_value: 0,
                                        has_return_value: false,
                                        arg0: if use_fast_args && args.len() > 0 { 
                                            if let Value::Int(n) = args[0] { n } else { 0 }
                                        } else { 0 },
                                        arg1: if use_fast_args && args.len() > 1 { 
                                            if let Value::Int(n) = args[1] { n } else { 0 }
                                        } else { 0 },
                                        arg2: if use_fast_args && args.len() > 2 { 
                                            if let Value::Int(n) = args[2] { n } else { 0 }
                                        } else { 0 },
                                        arg3: if use_fast_args && args.len() > 3 { 
                                            if let Value::Int(n) = args[3] { n } else { 0 }
                                        } else { 0 },
                                        arg_count: args.len() as i64,
                                    };
                                    
                                    // Execute the compiled function!
                                    // Lock is NOT held during execution to allow recursive calls
                                    let result_code = unsafe {
                                        (*compiled_fn)(&mut vm_context)
                                    };
                                    
                                    if result_code != 0 {
                                        return Err(format!("JIT execution failed with code: {}", result_code));
                                    }
                                    
                                    // Check for return value - prefer optimized VMContext.return_value
                                    // This is the FAST PATH from Phase 7 Step 8 optimization
                                    if vm_context.has_return_value {
                                        // Use the optimized return value directly
                                        self.stack.push(Value::Int(vm_context.return_value));
                                    } else if self.stack.len() > stack_size_before {
                                        // Fallback: return value was pushed to stack (old path)
                                        // No action needed - value is already on stack
                                    } else {
                                        return Err("JIT-compiled function did not return a value".to_string());
                                    }
                                    
                                    // Skip the normal bytecode execution
                                    continue;
                                }
                                
                                // Increment call counter
                                let count = self.function_call_counts
                                    .entry(func_name.to_string())
                                    .or_insert(0);
                                *count += 1;
                                
                                // Check if we should JIT-compile this function
                                if *count == JIT_FUNCTION_THRESHOLD {
                                    if std::env::var("DEBUG_JIT").is_ok() {
                                        eprintln!(
                                            "JIT: Function '{}' hit threshold ({} calls), attempting compilation...",
                                            func_name, JIT_FUNCTION_THRESHOLD
                                        );
                                        // Dump bytecode for debugging
                                        eprintln!("JIT: Bytecode for '{}':", func_name);
                                        for (pc, instr) in chunk.instructions.iter().enumerate() {
                                            eprintln!("  {:3}: {:?}", pc, instr);
                                        }
                                        // Dump constants
                                        eprintln!("JIT: Constants for '{}':", func_name);
                                        for (idx, constant) in chunk.constants.iter().enumerate() {
                                            eprintln!("  {:3}: {:?}", idx, constant);
                                        }
                                    }
                                    
                                    // Attempt to compile the function with enhanced info
                                    // This creates both standard and direct-arg variants for recursion
                                    match self.jit_compiler.compile_function_with_info(chunk, func_name) {
                                        Ok(fn_info) => {
                                            // Successfully compiled!
                                            if std::env::var("DEBUG_JIT").is_ok() {
                                                eprintln!("JIT: Successfully compiled function '{}' (direct_recursion={})", 
                                                    func_name, fn_info.supports_direct_recursion);
                                            }
                                            // Store both the standard pointer and the enhanced info
                                            self.compiled_functions.insert(func_name.to_string(), fn_info.fn_ptr);
                                            self.compiled_fn_info.insert(func_name.to_string(), fn_info);
                                        }
                                        Err(e) => {
                                            // Compilation failed - just log and continue with interpreter
                                            if std::env::var("DEBUG_JIT").is_ok() {
                                                eprintln!("JIT: Failed to compile function '{}': {}", func_name, e);
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Check if this is a generator function
                            if chunk.is_generator {
                                // Generator functions don't execute immediately
                                // Instead, create a generator instance
                                let mut locals = HashMap::new();

                                // Bind arguments to parameters
                                for (i, param_name) in chunk.params.iter().enumerate() {
                                    if let Some(arg) = args.get(i) {
                                        locals.insert(param_name.clone(), arg.clone());
                                    }
                                }

                                // Create generator state (not yet started, IP at 0)
                                let gen_state = GeneratorState {
                                    ip: 0,
                                    stack: Vec::new(),
                                    call_frames_data: Vec::new(),
                                    chunk: chunk.clone(),
                                    locals,
                                    captured: captured.clone(),
                                    is_exhausted: false,
                                };

                                let generator = Value::BytecodeGenerator {
                                    state: Arc::new(Mutex::new(gen_state)),
                                };

                                self.stack.push(generator);
                            } else {
                                // Regular or async function - set up call frame and switch context
                                // Return value will be pushed by Return opcode
                                // Note: Async functions execute synchronously in the VM.
                                // The return value will be wrapped in a Promise by the Return opcode
                                // if needed, based on the chunk.is_async flag.
                                self.call_bytecode_function(function, args)?;
                                // Don't push anything - Return will do it
                            }
                        }
                        Value::NativeFunction(_) => {
                            // Native functions return synchronously
                            let result = self.call_native_function_vm(function, args)?;
                            self.stack.push(result);
                        }
                        _ => return Err("Cannot call non-function".to_string()),
                    }
                }

                OpCode::Return => {
                    let return_value = self.stack.pop().ok_or("Stack underflow")?;

                    if let Some(frame) = self.call_frames.pop() {
                        // Pop from function call stack for error reporting
                        self.function_call_stack.pop();
                        
                        // Decrement recursion depth
                        if self.recursion_depth > 0 {
                            self.recursion_depth -= 1;
                        }
                        
                        // Restore previous state
                        self.ip = frame.return_ip;
                        if let Some(prev_chunk) = frame.prev_chunk {
                            self.chunk = prev_chunk;
                        }

                        // Clear stack to frame offset
                        self.stack.truncate(frame.stack_offset);

                        // If this was an async function, wrap the return value in a Promise
                        let value_to_push = if frame.is_async {
                            // Create a tokio oneshot channel with the result already available
                            let (tx, rx) = tokio::sync::oneshot::channel();
                            tx.send(Ok(return_value))
                                .map_err(|_| "Failed to send to promise channel")?;

                            Value::Promise {
                                receiver: Arc::new(Mutex::new(rx)),
                                is_polled: Arc::new(Mutex::new(false)),
                                cached_result: Arc::new(Mutex::new(None)),
                task_handle: None,
                            }
                        } else {
                            return_value
                        };

                        // Push return value (or promise)
                        self.stack.push(value_to_push);
                    } else {
                        // Top-level return
                        return Ok(return_value);
                    }
                }

                OpCode::ReturnNone => {
                    if let Some(frame) = self.call_frames.pop() {
                        // Decrement recursion depth
                        if self.recursion_depth > 0 {
                            self.recursion_depth -= 1;
                        }
                        
                        self.ip = frame.return_ip;
                        if let Some(prev_chunk) = frame.prev_chunk {
                            self.chunk = prev_chunk;
                        }
                        self.stack.truncate(frame.stack_offset);

                        // If this was an async function, wrap None in a Promise
                        let value_to_push = if frame.is_async {
                            let (tx, rx) = tokio::sync::oneshot::channel();
                            tx.send(Ok(Value::Null))
                                .map_err(|_| "Failed to send to promise channel")?;

                            Value::Promise {
                                receiver: Arc::new(Mutex::new(rx)),
                                is_polled: Arc::new(Mutex::new(false)),
                                cached_result: Arc::new(Mutex::new(None)),
                task_handle: None,
                            }
                        } else {
                            Value::Null
                        };

                        self.stack.push(value_to_push);
                    } else {
                        return Ok(Value::Null);
                    }
                }

                OpCode::MakeClosure(func_index) => {
                    let constant = &self.chunk.constants[func_index];
                    if let Constant::Function(chunk) = constant {
                        // Capture upvalues listed in the function's chunk
                        let mut captured = HashMap::new();

                        if std::env::var("DEBUG_VM").is_ok() {
                            eprintln!(
                                "MakeClosure: function has {} upvalues: {:?}",
                                chunk.upvalues.len(),
                                chunk.upvalues
                            );
                            eprintln!("  Call stack depth: {}", self.call_frames.len());
                            if let Some(frame) = self.call_frames.last() {
                                eprintln!(
                                    "  Current frame has {} locals: {:?}, {} captured: {:?}",
                                    frame.locals.len(),
                                    frame.locals.keys().collect::<Vec<_>>(),
                                    frame.captured.len(),
                                    frame.captured.keys().collect::<Vec<_>>()
                                );
                            } else {
                                eprintln!("  No current frame!");
                            }
                        }

                        for upvalue_name in &chunk.upvalues {
                            // Find the variable in current scope (locals only - NOT globals)
                            // Globals/built-ins will be resolved at runtime
                            let value = if let Some(frame) = self.call_frames.last() {
                                frame.locals.get(upvalue_name).cloned()
                            } else {
                                None
                            };

                            if let Some(val) = value {
                                if std::env::var("DEBUG_VM").is_ok() {
                                    eprintln!(
                                        "  Captured '{}' from locals = {:?}",
                                        upvalue_name, val
                                    );
                                }
                                // Wrap in Arc<Mutex<>> for shared mutable state
                                captured.insert(upvalue_name.clone(), Arc::new(Mutex::new(val)));
                            } else {
                                if std::env::var("DEBUG_VM").is_ok() {
                                    eprintln!(
                                        "  Skipped '{}' (not in locals, will resolve at runtime)",
                                        upvalue_name
                                    );
                                }
                                // Variable not in locals - it's either a global or undefined
                                // Don't capture it - let it be resolved at runtime
                            }
                        }

                        // Create a closure value with captured variables
                        let value = Value::BytecodeFunction { chunk: (**chunk).clone(), captured };
                        self.stack.push(value);
                    } else {
                        return Err("Expected function constant".to_string());
                    }
                }

                // Collection operations
                OpCode::MakeArray(count) => {
                    // Collect elements from stack
                    // If the bottom-most element is ArrayMarker, collect until marker
                    // Otherwise, collect exactly 'count' elements
                    let mut elements = Vec::new();
                    let mut found_marker = false;

                    for _ in 0..count {
                        let value = self.stack.pop().ok_or("Stack underflow in MakeArray")?;
                        if matches!(value, Value::ArrayMarker) {
                            found_marker = true;
                            break;
                        }
                        elements.push(value);
                    }

                    // If we found a marker, that's it
                    // Otherwise we need to check if there are more elements (from spreads)
                    if found_marker {
                        // Collect any remaining elements until we reach the marker
                        // Actually, we already hit the marker, so we're done
                    }

                    elements.reverse();
                    self.stack.push(Value::Array(Arc::new(elements)));
                }

                OpCode::PushArrayMarker => {
                    self.stack.push(Value::ArrayMarker);
                }

                OpCode::MakeDict(count) => {
                    let mut dict = HashMap::new();
                    for _ in 0..count {
                        let value = self.stack.pop().ok_or("Stack underflow")?;
                        let key = self.stack.pop().ok_or("Stack underflow")?;

                        let key_str = match key {
                            Value::Str(s) => s.as_ref().clone(),
                            _ => return Err("Dict keys must be strings".to_string()),
                        };

                        dict.insert(key_str, value);
                    }
                    self.stack.push(Value::Dict(Arc::new(dict)));
                }

                OpCode::IndexGet => {
                    let index = self.stack.pop().ok_or("Stack underflow")?;
                    let object = self.stack.pop().ok_or("Stack underflow")?;

                    let result = match (&object, &index) {
                        (Value::Array(arr), Value::Int(i)) => {
                            let idx =
                                if *i < 0 { (arr.len() as i64 + i) as usize } else { *i as usize };
                            arr.get(idx)
                                .cloned()
                                .ok_or_else(|| format!("Index out of bounds: {}", i))?
                        }
                        (Value::Dict(dict), Value::Str(key)) => {
                            dict.get(key.as_ref()).cloned().unwrap_or(Value::Null)
                        }
                        (Value::Dict(dict), Value::Int(i)) => {
                            // Support integer keys by converting to string
                            dict.get(&i.to_string()).cloned().unwrap_or(Value::Null)
                        }
                        (Value::Str(s), Value::Int(i)) => {
                            let idx =
                                if *i < 0 { (s.len() as i64 + i) as usize } else { *i as usize };
                            s.chars()
                                .nth(idx)
                                .map(|c| Value::Str(Arc::new(c.to_string())))
                                .ok_or_else(|| format!("Index out of bounds: {}", i))?
                        }
                        _ => return Err("Invalid index operation".to_string()),
                    };

                    self.stack.push(result);
                }

                OpCode::IndexSet => {
                    let index = self.stack.pop().ok_or("Stack underflow")?;
                    let object = self.stack.pop().ok_or("Stack underflow")?;
                    let value = self.stack.pop().ok_or("Stack underflow")?;

                    match (object, index) {
                        (Value::Array(arr), Value::Int(i)) => {
                            let mut arr_clone = Arc::try_unwrap(arr).unwrap_or_else(|arc| (*arc).clone());
                            let idx =
                                if i < 0 { (arr_clone.len() as i64 + i) as usize } else { i as usize };

                            if idx < arr_clone.len() {
                                arr_clone[idx] = value;
                                self.stack.push(Value::Array(Arc::new(arr_clone)));
                            } else {
                                return Err(format!("Index out of bounds: {}", i));
                            }
                        }
                        (Value::Dict(dict), Value::Str(key)) => {
                            let mut dict_clone = Arc::try_unwrap(dict).unwrap_or_else(|arc| (*arc).clone());
                            dict_clone.insert(key.as_ref().clone(), value);
                            self.stack.push(Value::Dict(Arc::new(dict_clone)));
                        }
                        (Value::Dict(dict), Value::Int(i)) => {
                            let mut dict_clone = Arc::try_unwrap(dict).unwrap_or_else(|arc| (*arc).clone());
                            // Support integer keys by converting to string
                            dict_clone.insert(i.to_string(), value);
                            self.stack.push(Value::Dict(Arc::new(dict_clone)));
                        }
                        _ => return Err("Invalid index assignment".to_string()),
                    }
                }

                OpCode::IndexGetInPlace(var_name) => {
                    // Pop index from stack
                    let index = self.stack.pop().ok_or("Stack underflow")?;

                    // Get reference to local variable without cloning
                    let frame = self
                        .call_frames
                        .last_mut()
                        .ok_or("IndexGetInPlace requires call frame")?;

                    // Check if variable exists in captured or locals
                    let value_opt = if let Some(captured_ref) = frame.captured.get(&var_name) {
                        Some(captured_ref.lock().unwrap().clone())
                    } else {
                        frame.locals.get(&var_name).cloned()
                    };

                    let object =
                        value_opt.ok_or_else(|| format!("Undefined variable: {}", var_name))?;

                    // Perform the index access
                    let result = match (&object, &index) {
                        (Value::Array(arr), Value::Int(i)) => {
                            let idx =
                                if *i < 0 { (arr.len() as i64 + i) as usize } else { *i as usize };
                            arr.get(idx)
                                .cloned()
                                .ok_or_else(|| format!("Index out of bounds: {}", i))?
                        }
                        (Value::Dict(dict), Value::Str(key)) => {
                            dict.get(key.as_ref()).cloned().unwrap_or(Value::Null)
                        }
                        (Value::Dict(dict), Value::Int(i)) => {
                            dict.get(&i.to_string()).cloned().unwrap_or(Value::Null)
                        }
                        (Value::Str(s), Value::Int(i)) => {
                            let idx =
                                if *i < 0 { (s.len() as i64 + i) as usize } else { *i as usize };
                            s.chars()
                                .nth(idx)
                                .map(|c| Value::Str(Arc::new(c.to_string())))
                                .ok_or_else(|| format!("Index out of bounds: {}", i))?
                        }
                        _ => return Err("Invalid index operation".to_string()),
                    };

                    self.stack.push(result);
                }

                OpCode::IndexSetInPlace(var_name) => {
                    // Stack layout: [... value, index] (index on top)
                    // Pop index and value from stack
                    let index = self.stack.pop().ok_or("Stack underflow")?;
                    let value = self.stack.pop().ok_or("Stack underflow")?;

                    // Get mutable reference to local variable
                    let frame = self
                        .call_frames
                        .last_mut()
                        .ok_or("IndexSetInPlace requires call frame")?;

                    // Check if variable is in captured or locals and modify in-place
                    if let Some(captured_ref) = frame.captured.get(&var_name) {
                        // Modify captured variable in-place
                        let mut object = captured_ref.lock().unwrap();
                        match (&mut *object, index) {
                            (Value::Array(arr), Value::Int(i)) => {
                                let arr_mut = Arc::make_mut(arr);
                                let idx = if i < 0 {
                                    (arr_mut.len() as i64 + i) as usize
                                } else {
                                    i as usize
                                };
                                if idx < arr_mut.len() {
                                    arr_mut[idx] = value;
                                } else {
                                    return Err(format!("Index out of bounds: {}", i));
                                }
                            }
                            (Value::Dict(dict), Value::Str(key)) => {
                                let dict_mut = Arc::make_mut(dict);
                                dict_mut.insert(key.as_ref().clone(), value);
                            }
                            (Value::Dict(dict), Value::Int(i)) => {
                                let dict_mut = Arc::make_mut(dict);
                                dict_mut.insert(i.to_string(), value);
                            }
                            _ => return Err("Invalid index assignment".to_string()),
                        }
                    } else if let Some(object) = frame.locals.get_mut(&var_name) {
                        // Modify local variable in-place
                        match (object, index) {
                            (Value::Array(arr), Value::Int(i)) => {
                                let arr_mut = Arc::make_mut(arr);
                                let idx = if i < 0 {
                                    (arr_mut.len() as i64 + i) as usize
                                } else {
                                    i as usize
                                };
                                if idx < arr_mut.len() {
                                    arr_mut[idx] = value;
                                } else {
                                    return Err(format!("Index out of bounds: {}", i));
                                }
                            }
                            (Value::Dict(dict), Value::Str(key)) => {
                                let dict_mut = Arc::make_mut(dict);
                                dict_mut.insert(key.as_ref().clone(), value);
                            }
                            (Value::Dict(dict), Value::Int(i)) => {
                                let dict_mut = Arc::make_mut(dict);
                                dict_mut.insert(i.to_string(), value);
                            }
                            _ => return Err("Invalid index assignment".to_string()),
                        }
                    } else {
                        return Err(format!("Undefined variable: {}", var_name));
                    }

                    // Push a null value to keep stack balanced (will be popped by following Pop instruction)
                    self.stack.push(Value::Null);
                }

                OpCode::FieldGet(field) => {
                    let object = self.stack.pop().ok_or("Stack underflow")?;

                    let result = match object {
                        Value::Struct { fields, .. } => fields
                            .get(&field)
                            .cloned()
                            .ok_or_else(|| format!("Field not found: {}", field))?,
                        Value::Dict(dict) => dict.get(&field).cloned().unwrap_or(Value::Null),
                        _ => return Err("Cannot access field on non-struct".to_string()),
                    };

                    self.stack.push(result);
                }

                OpCode::FieldSet(field) => {
                    let object = self.stack.pop().ok_or("Stack underflow")?;
                    let value = self.stack.pop().ok_or("Stack underflow")?;

                    match object {
                        Value::Struct { name, mut fields } => {
                            fields.insert(field, value);
                            self.stack.push(Value::Struct { name, fields });
                        }
                        Value::Dict(dict) => {
                            let mut dict_clone = Arc::try_unwrap(dict).unwrap_or_else(|arc| (*arc).clone());
                            dict_clone.insert(field, value);
                            self.stack.push(Value::Dict(Arc::new(dict_clone)));
                        }
                        _ => return Err("Cannot set field on non-struct".to_string()),
                    }
                }

                // Spread operations
                OpCode::SpreadArray => {
                    let array = self.stack.pop().ok_or("Stack underflow")?;

                    match array {
                        Value::Array(arr) => {
                            for elem in arr.iter() {
                                self.stack.push(elem.clone());
                            }
                        }
                        _ => return Err("Can only spread arrays".to_string()),
                    }
                }

                OpCode::SpreadDict => {
                    let dict = self.stack.pop().ok_or("Stack underflow")?;

                    match dict {
                        Value::Dict(d) => {
                            for (key, value) in d.iter() {
                                self.stack.push(Value::Str(Arc::new(key.clone())));
                                self.stack.push(value.clone());
                            }
                        }
                        _ => return Err("Can only spread dicts".to_string()),
                    }
                }

                OpCode::SpreadArgs => {
                    // Similar to SpreadArray but for function arguments
                    let array = self.stack.pop().ok_or("Stack underflow")?;

                    match array {
                        Value::Array(arr) => {
                            for elem in arr.iter() {
                                self.stack.push(elem.clone());
                            }
                        }
                        _ => return Err("Can only spread arrays as arguments".to_string()),
                    }
                }

                // Pattern matching
                OpCode::MatchPattern(pattern_index) => {
                    let constant = self.chunk.constants[pattern_index].clone();
                    if let Constant::Pattern(pattern) = constant {
                        let value = self.stack.last().ok_or("Stack underflow")?.clone();
                        let success = self.match_pattern(&pattern, &value)?;
                        self.stack.push(Value::Bool(success));
                    } else {
                        return Err("Expected pattern constant".to_string());
                    }
                }

                OpCode::BeginCase | OpCode::EndCase => {
                    // These are markers for debugging/disassembly
                }

                // Result/Option operations
                OpCode::MakeOk => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(Value::Result { is_ok: true, value: Box::new(value) });
                }

                OpCode::MakeErr => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(Value::Result { is_ok: false, value: Box::new(value) });
                }

                OpCode::MakeSome => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(Value::Option { is_some: true, value: Box::new(value) });
                }

                OpCode::MakeNone => {
                    self.stack.push(Value::Option { is_some: false, value: Box::new(Value::Null) });
                }

                OpCode::TryUnwrap => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;

                    match value {
                        Value::Result { is_ok, value } => {
                            if is_ok {
                                self.stack.push(*value);
                            } else {
                                // Early return with error
                                return Ok(Value::Result { is_ok: false, value });
                            }
                        }
                        Value::Option { is_some, value } => {
                            if is_some {
                                self.stack.push(*value);
                            } else {
                                // Early return with None
                                return Ok(Value::Option {
                                    is_some: false,
                                    value: Box::new(Value::Null),
                                });
                            }
                        }
                        _ => return Err("Try operator requires Result or Option".to_string()),
                    }
                }

                // Struct operations
                OpCode::MakeStruct(name, fields) => {
                    let mut field_map = HashMap::new();

                    for field_name in fields.iter().rev() {
                        let value = self.stack.pop().ok_or("Stack underflow")?;
                        field_map.insert(field_name.clone(), value);
                    }

                    self.stack.push(Value::Struct { name, fields: field_map });
                }

                // Environment management
                OpCode::PushScope | OpCode::PopScope => {
                    // These are handled by call frames
                }

                // Iterator operations
                OpCode::MakeIterator => {
                    let collection = self.stack.pop().ok_or("Stack underflow")?;
                    // Call interpreter's built-in iterator function
                    let result = self.interpreter.call_native_function_impl("iter", &[collection]);
                    self.stack.push(result);
                }

                OpCode::IteratorNext => {
                    let iterator = self.stack.pop().ok_or("Stack underflow")?;
                    // Call next() on the iterator
                    let result = self
                        .interpreter
                        .call_native_function_impl("iterator_next", &[iterator.clone()]);
                    self.stack.push(iterator); // Keep iterator on stack
                    self.stack.push(result); // Push result (Some/None)
                }

                OpCode::IteratorHasNext => {
                    let iterator = self.stack.last().ok_or("Stack underflow")?.clone();
                    // Check if iterator has more values
                    let has_next = match &iterator {
                        Value::Iterator { index, source, .. } => match source.as_ref() {
                            Value::Array(arr) => *index < arr.len(),
                            _ => false,
                        },
                        _ => false,
                    };
                    self.stack.push(Value::Bool(has_next));
                }

                // Generator operations
                OpCode::MakeGenerator => {
                    // Pop the function from stack and convert it to a generator
                    let function = self.stack.pop().ok_or("Stack underflow in MakeGenerator")?;

                    if let Value::BytecodeFunction { chunk, captured } = function {
                        // Create initial generator state (not yet started)
                        let state = GeneratorState {
                            ip: 0,
                            stack: Vec::new(),
                            call_frames_data: Vec::new(),
                            chunk: chunk.clone(),
                            locals: HashMap::new(),
                            captured: captured.clone(),
                            is_exhausted: false,
                        };

                        let generator =
                            Value::BytecodeGenerator { state: Arc::new(Mutex::new(state)) };

                        self.stack.push(generator);
                    } else {
                        return Err("MakeGenerator requires a BytecodeFunction".to_string());
                    }
                }

                OpCode::Yield => {
                    // Yield is handled specially in generator_next() method
                    // This opcode serves as a marker for the generator execution loop
                    // When we reach here in normal execution, it's an error
                    return Err("Yield can only be used inside generator functions".to_string());
                }

                OpCode::ResumeGenerator => {
                    // Resume generator by calling generator_next()
                    // This pops the generator from stack and pushes the result (Some(value) or None)
                    let generator = self.stack.pop().ok_or("Stack underflow in ResumeGenerator")?;
                    let result = self.generator_next(generator)?;
                    self.stack.push(result);
                }

                // Async/await operations
                OpCode::Await => {
                    // Pop promise from stack and await it
                    let promise = self.stack.pop().ok_or("Stack underflow in Await")?;

                    match promise {
                        Value::Promise { receiver, is_polled, cached_result, .. } => {
                            // Check if we've already polled this promise
                            {
                                let polled = is_polled.lock().unwrap();
                                let cached = cached_result.lock().unwrap();

                                if *polled {
                                    // Use cached result
                                    match cached.as_ref() {
                                        Some(Ok(val)) => {
                                            self.stack.push(val.clone());
                                            continue;
                                        }
                                        Some(Err(err)) => {
                                            return Err(format!("Promise rejected: {}", err));
                                        }
                                        None => {
                                            return Err(
                                                "Promise polled but no result cached".to_string()
                                            );
                                        }
                                    }
                                }
                            }

                            // Poll the promise using tokio runtime - blocks until result is ready
                            let result = {
                                let mut recv_guard = receiver.lock().unwrap();
                                // Take ownership by replacing with a dummy closed channel
                                let (dummy_tx, dummy_rx) = tokio::sync::oneshot::channel();
                                drop(dummy_tx); // Close immediately
                                let actual_rx = std::mem::replace(&mut *recv_guard, dummy_rx);
                                drop(recv_guard); // Release lock before blocking
                                
                                // Debug logging
                                if std::env::var("DEBUG_ASYNC").is_ok() {
                                    eprintln!("VM Await: about to block_on receiver");
                                }
                                
                                // Use the runtime handle to block on the receiver
                                // This works even when we're already inside a tokio runtime
                                let result = self.runtime_handle.block_on(actual_rx);
                                
                                if std::env::var("DEBUG_ASYNC").is_ok() {
                                    eprintln!("VM Await: block_on completed with result: {:?}", 
                                        match &result {
                                            Ok(Ok(_)) => "Ok(Ok(value))",
                                            Ok(Err(e)) => {
                                                eprintln!("VM Await: Promise rejected: {}", e);
                                                "Ok(Err(...))"
                                            }
                                            Err(_) => "Err (channel closed)",
                                        });
                                }
                                
                                result
                            };

                            // Update cache
                            let mut polled = is_polled.lock().unwrap();
                            let mut cached = cached_result.lock().unwrap();

                            match result {
                                Ok(Ok(value)) => {
                                    *cached = Some(Ok(value.clone()));
                                    *polled = true;
                                    self.stack.push(value);
                                }
                                Ok(Err(error)) => {
                                    *cached = Some(Err(error.clone()));
                                    *polled = true;
                                    return Err(format!("Promise rejected: {}", error));
                                }
                                Err(_) => {
                                    *cached = Some(Err("Promise never resolved".to_string()));
                                    *polled = true;
                                    return Err(
                                        "Promise never resolved (channel closed)".to_string()
                                    );
                                }
                            }
                        }
                        _ => {
                            // Not a promise - just push it back (treat as already resolved)
                            self.stack.push(promise);
                        }
                    }
                }

                OpCode::MakePromise => {
                    // Pop value from stack and wrap it in a resolved promise
                    let value = self.stack.pop().ok_or("Stack underflow in MakePromise")?;

                    // Create a tokio oneshot channel that immediately sends the value
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    tx.send(Ok(value.clone())).map_err(|_| "Failed to send to promise channel")?;

                    // Create promise with the result already available
                    let promise = Value::Promise {
                        receiver: Arc::new(Mutex::new(rx)),
                        is_polled: Arc::new(Mutex::new(false)),
                        cached_result: Arc::new(Mutex::new(None)),
                task_handle: None,
                    };

                    self.stack.push(promise);
                }

                OpCode::MarkAsync => {
                    // This is a no-op marker used during compilation
                    // It marks that the current context is async but doesn't generate runtime code
                }

                // Exception handling
                OpCode::BeginTry(catch_ip) => {
                    // Push exception handler onto stack
                    self.exception_handlers.push(ExceptionHandlerFrame {
                        catch_ip,
                        stack_offset: self.stack.len(),
                        frame_offset: self.call_frames.len(),
                    });
                }

                OpCode::EndTry => {
                    // Pop exception handler (normal exit from try block)
                    if self.exception_handlers.is_empty() {
                        return Err("EndTry without matching BeginTry".to_string());
                    }
                    self.exception_handlers.pop();
                }

                OpCode::Throw => {
                    // Pop error value from stack
                    let error_value = self.stack.pop().ok_or("Stack underflow in Throw")?;

                    if std::env::var("DEBUG_VM").is_ok() {
                        eprintln!("THROW: error_value={:?}", error_value);
                        eprintln!("  Exception handlers: {}", self.exception_handlers.len());
                        eprintln!("  Call frames: {}", self.call_frames.len());
                    }

                    // Find nearest exception handler
                    if let Some(handler) = self.exception_handlers.pop() {
                        if std::env::var("DEBUG_VM").is_ok() {
                            eprintln!(
                                "  Handler found: catch_ip={}, frame_offset={}, stack_offset={}",
                                handler.catch_ip, handler.frame_offset, handler.stack_offset
                            );
                        }

                        // Unwind call frames to handler's frame offset
                        // We need to restore the chunk from the target frame (or top-level)
                        while self.call_frames.len() > handler.frame_offset {
                            if let Some(frame) = self.call_frames.pop() {
                                if std::env::var("DEBUG_VM").is_ok() {
                                    eprintln!("  Unwinding frame: return_ip={}", frame.return_ip);
                                }
                                // Restore chunk if this was the last frame to unwind
                                if self.call_frames.len() == handler.frame_offset {
                                    if let Some(prev_chunk) = frame.prev_chunk {
                                        self.chunk = prev_chunk;
                                    }
                                }
                            }
                        }

                        // Unwind stack to handler's stack offset
                        self.stack.truncate(handler.stack_offset);

                        // Push error value back onto stack for BeginCatch
                        self.stack.push(error_value);

                        // Jump to catch block
                        self.ip = handler.catch_ip;

                        if std::env::var("DEBUG_VM").is_ok() {
                            eprintln!(
                                "  After unwind: ip={}, frames={}, stack={}",
                                self.ip,
                                self.call_frames.len(),
                                self.stack.len()
                            );
                        }
                    } else {
                        // No exception handler found - uncaught exception
                        let error_msg = match error_value {
                            Value::Str(s) => s.as_ref().clone(),
                            Value::Error(e) => e,
                            Value::ErrorObject { message, .. } => message,
                            other => format!("Uncaught exception: {:?}", other),
                        };
                        return Err(error_msg);
                    }
                }

                OpCode::BeginCatch(var_name) => {
                    // Pop error from stack and bind to local variable
                    let error_value = self.stack.pop().ok_or("Stack underflow in BeginCatch")?;

                    // Convert error to structured error object if needed
                    let error_obj = match error_value {
                        Value::Str(msg) => {
                            // Simple string error - wrap in error struct
                            let mut fields = HashMap::new();
                            fields.insert("message".to_string(), Value::Str(msg));
                            fields.insert("stack".to_string(), Value::Array(Arc::new(Vec::new())));
                            fields.insert("line".to_string(), Value::Int(0));
                            Value::Struct { name: "Error".to_string(), fields }
                        }
                        Value::Error(msg) => {
                            // Legacy Error type - wrap in struct
                            let mut fields = HashMap::new();
                            fields.insert("message".to_string(), Value::Str(Arc::new(msg)));
                            fields.insert("stack".to_string(), Value::Array(Arc::new(Vec::new())));
                            fields.insert("line".to_string(), Value::Int(0));
                            Value::Struct { name: "Error".to_string(), fields }
                        }
                        Value::ErrorObject { message, stack, line, cause } => {
                            // Full error object - convert to struct
                            let mut fields = HashMap::new();
                            fields.insert("message".to_string(), Value::Str(Arc::new(message)));
                            fields.insert(
                                "stack".to_string(),
                                Value::Array(Arc::new(stack.iter().map(|s| Value::Str(Arc::new(s.clone()))).collect())),
                            );
                            fields.insert("line".to_string(), Value::Int(line.unwrap_or(0) as i64));
                            if let Some(cause_val) = cause {
                                fields.insert("cause".to_string(), *cause_val);
                            }
                            Value::Struct { name: "Error".to_string(), fields }
                        }
                        other => {
                            // Any other value - wrap as message
                            let mut fields = HashMap::new();
                            fields
                                .insert("message".to_string(), Value::Str(Arc::new(format!("{:?}", other))));
                            fields.insert("stack".to_string(), Value::Array(Arc::new(Vec::new())));
                            fields.insert("line".to_string(), Value::Int(0));
                            Value::Struct { name: "Error".to_string(), fields }
                        }
                    };

                    // Bind error to variable in current frame
                    if let Some(frame) = self.call_frames.last_mut() {
                        frame.locals.insert(var_name, error_obj);
                    } else {
                        // No call frame - store in globals
                        self.globals.lock().unwrap().set(var_name, error_obj);
                    }
                }

                OpCode::EndCatch => {
                    // Nothing to do - handler already removed by Throw
                    // This opcode marks the end of the catch block for debugging/profiling
                }

                // Native function calls
                OpCode::CallNative(name, arg_count) => {
                    // Collect arguments from stack
                    let mut args = Vec::new();
                    for _ in 0..arg_count {
                        args.push(self.stack.pop().ok_or("Stack underflow in CallNative")?);
                    }
                    args.reverse();

                    // Call the native function through the interpreter
                    let result = self.interpreter.call_native_function_impl(&name, &args);

                    // Check if result is an error
                    match &result {
                        Value::Error(msg) => return Err(msg.clone()),
                        Value::ErrorObject { .. } => {
                            return Err(format!("Error in native function {}", name))
                        }
                        _ => self.stack.push(result),
                    }
                }

                // Closure & upvalue operations
                OpCode::CaptureUpvalue(name) => {
                    // Find the variable in the current scope (locals or globals)
                    let value = if let Some(frame) = self.call_frames.last() {
                        frame.locals.get(&name).cloned()
                    } else {
                        None
                    }
                    .or_else(|| {
                        // Try globals
                        self.globals.lock().unwrap().get(&name)
                    })
                    .ok_or_else(|| format!("Variable '{}' not found for capture", name))?;

                    // Create a new upvalue with the captured value
                    let upvalue = Upvalue {
                        value: Arc::new(Mutex::new(value)),
                        is_closed: true, // Immediately close it (move to heap)
                        stack_index: None,
                    };

                    let upvalue_index = self.upvalues.len();
                    self.upvalues.push(upvalue);

                    // Push the upvalue index onto the stack (for MakeClosure to use)
                    self.stack.push(Value::Int(upvalue_index as i64));
                }

                OpCode::LoadUpvalue(index) => {
                    // Load the value from the upvalue
                    if index >= self.upvalues.len() {
                        return Err(format!("Invalid upvalue index: {}", index));
                    }

                    let upvalue = &self.upvalues[index];
                    let value = upvalue.value.lock().unwrap().clone();
                    self.stack.push(value);
                }

                OpCode::StoreUpvalue(index) => {
                    // Store the top of stack to the upvalue
                    if index >= self.upvalues.len() {
                        return Err(format!("Invalid upvalue index: {}", index));
                    }

                    let value = self.stack.pop().ok_or("Stack underflow in StoreUpvalue")?;
                    let upvalue = &self.upvalues[index];
                    *upvalue.value.lock().unwrap() = value;
                }

                OpCode::CloseUpvalues(_slot) => {
                    // In our simplified implementation, upvalues are immediately closed
                    // (moved to heap) when captured. This operation is a no-op.
                    // A more sophisticated implementation would keep upvalues on the stack
                    // until they go out of scope, then move them to the heap.
                }

                // Channel operations
                OpCode::MakeChannel | OpCode::ChannelSend | OpCode::ChannelRecv => {
                    // Channels require concurrent runtime support
                    // For now, return an error - will implement in Week 5-6
                    return Err("Channel operations not yet implemented in VM".to_string());
                }

                // Debug operations
                OpCode::DebugPrint(msg) => {
                    eprintln!("DEBUG: {}", msg);
                }

                OpCode::Nop => {
                    // Do nothing
                }

                OpCode::DebugStack => {
                    eprintln!("Stack: {:?}", self.stack);
                }
            }
        }
    }

    /// Convert a constant to a runtime value
    fn constant_to_value(&self, constant: &Constant) -> Result<Value, String> {
        match constant {
            Constant::Int(n) => Ok(Value::Int(*n)),
            Constant::Float(f) => Ok(Value::Float(*f)),
            Constant::String(s) => Ok(Value::Str(Arc::new(s.clone()))),
            Constant::Bool(b) => Ok(Value::Bool(*b)),
            Constant::None => Ok(Value::Null),
            Constant::Function(chunk) => {
                Ok(Value::BytecodeFunction { chunk: (**chunk).clone(), captured: HashMap::new() })
            }
            Constant::Pattern(_) => Err("Cannot convert pattern to value".to_string()),
            Constant::Type(_) => Err("Cannot convert type annotation to value".to_string()),
            Constant::Array(elements) => {
                let mut array = Vec::new();
                for elem in elements {
                    array.push(self.constant_to_value(elem)?);
                }
                Ok(Value::Array(Arc::new(array)))
            }
            Constant::Dict(pairs) => {
                let mut dict = HashMap::new();
                for (key_const, value_const) in pairs {
                    let key = self.constant_to_value(key_const)?;
                    let value = self.constant_to_value(value_const)?;

                    // Key must be a string
                    if let Value::Str(key_str) = key {
                        dict.insert(key_str.as_ref().clone(), value);
                    } else {
                        return Err("Dict constant keys must be strings".to_string());
                    }
                }
                Ok(Value::Dict(Arc::new(dict)))
            }
        }
    }


    /// Call a function
    /// Set up a call frame for a bytecode function (doesn't return - Return opcode will handle that)
    fn call_bytecode_function(&mut self, function: Value, args: Vec<Value>) -> Result<(), String> {
        if let Value::BytecodeFunction { chunk, captured } = function {
            // Create new call frame with parameters bound
            let mut locals = HashMap::new();

            // Bind arguments to parameter names
            let param_names = &chunk.params;

            // Check argument count
            if args.len() != param_names.len() {
                return Err(format!(
                    "Function {} expects {} arguments, got {}",
                    chunk.name.as_deref().unwrap_or("<lambda>"),
                    param_names.len(),
                    args.len()
                ));
            }

            // Bind each argument to its corresponding parameter name
            for (param_name, arg_value) in param_names.iter().zip(args.iter()) {
                locals.insert(param_name.clone(), arg_value.clone());
            }

            // Prepare captured variables HashMap for mutable access
            let mut captured_map = HashMap::new();
            for (name, value_ref) in &captured {
                captured_map.insert(name.clone(), value_ref.clone());
            }

            if std::env::var("DEBUG_VM").is_ok() {
                eprintln!(
                    "CallFrame has {} captured variables: {:?}",
                    captured_map.len(),
                    captured_map.keys().collect::<Vec<_>>()
                );
            }

            let frame = CallFrame {
                return_ip: self.ip,
                stack_offset: self.stack.len(),
                locals,
                captured: captured_map,
                prev_chunk: Some(self.chunk.clone()),
                is_async: chunk.is_async,
            };

            self.call_frames.push(frame);

            // Track recursion depth for optimization and profiling
            self.recursion_depth += 1;
            if self.recursion_depth > self.max_recursion_depth {
                self.max_recursion_depth = self.recursion_depth;
            }
            
            // Track function call for error reporting
            let func_name = chunk.name.as_deref().unwrap_or("<anonymous>").to_string();
            self.function_call_stack.push(func_name);

            // Switch to function's chunk and reset IP
            self.chunk = chunk;
            self.ip = 0;

            Ok(())
        } else {
            Err("Expected BytecodeFunction".to_string())
        }
    }

    /// Call a native function (returns synchronously)
    fn call_native_function_vm(
        &mut self,
        function: Value,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        if let Value::NativeFunction(name) = function {
            // Use the interpreter's native function implementation
            // This gives us access to ALL 100+ built-in functions automatically
            let result = self.interpreter.call_native_function_impl(&name, &args);

            // Check if the result is an error
            match result {
                Value::Error(msg) => Err(msg),
                Value::ErrorObject { .. } => Err("Error occurred in native function".to_string()),
                other => Ok(other),
            }
        } else {
            Err("Expected NativeFunction".to_string())
        }
    }

    /// Call a function from JIT-compiled code
    /// This is invoked by the jit_call_function runtime helper
    pub fn call_function_from_jit(
        &mut self,
        function: Value,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        match &function {
            Value::BytecodeFunction { chunk, captured: _ } => {
                // OPTIMIZATION: Check if target function is JIT-compiled
                // If so, make direct JIT → JIT call for maximum performance
                if self.jit_enabled {
                    let func_name = chunk.name.as_deref().unwrap_or("<anonymous>");
                    
                    // PHASE 7 STEP 12: Check for direct-arg optimized variant first
                    // This enables direct JIT recursion without FFI boundary crossing
                    if args.len() == 1 {
                        if let Value::Int(arg_val) = args[0] {
                            // Copy the function info to avoid borrow checker issues
                            let fn_info_opt = self.compiled_fn_info.get(func_name).copied();
                            
                            if let Some(fn_info) = fn_info_opt {
                                if let Some(direct_fn) = fn_info.fn_with_arg {
                                    // ULTRA-FAST PATH: Call the direct-arg variant!
                                    // This is the key optimization for recursive functions
                                    
                                    // Get VM pointer for VMContext
                                    let vm_ptr: *mut std::ffi::c_void = self as *mut _ as *mut std::ffi::c_void;
                                    let stack_ptr: *mut Vec<Value> = &mut self.stack;
                                    
                                    // Get globals pointer
                                    let globals_ptr: *mut HashMap<String, Value> = {
                                        let mut globals_guard = self.globals.lock().unwrap();
                                        let ptr = &mut globals_guard.scopes[0] as *mut HashMap<String, Value>;
                                        drop(globals_guard);
                                        ptr
                                    };
                                    
                                    // Create minimal VMContext - direct-arg functions don't need HashMap
                                    let mut func_locals = HashMap::new();
                                    let locals_ptr: *mut HashMap<String, Value> = &mut func_locals;
                                    
                                    let mut vm_context = crate::jit::VMContext {
                                        stack_ptr,
                                        locals_ptr,
                                        globals_ptr,
                                        var_names_ptr: std::ptr::null_mut(),
                                        vm_ptr,
                                        return_value: 0,
                                        has_return_value: false,
                                        arg0: arg_val,
                                        arg1: 0,
                                        arg2: 0,
                                        arg3: 0,
                                        arg_count: 1,
                                    };
                                    
                                    // Execute the direct-arg variant!
                                    // The function returns the actual result (not a status code)
                                    let result = unsafe {
                                        direct_fn(&mut vm_context, arg_val)
                                    };
                                    
                                    if std::env::var("DEBUG_JIT").is_ok() {
                                        eprintln!("JIT: Direct-arg call to '{}' with arg {} returned {}", 
                                            func_name, arg_val, result);
                                    }
                                    
                                    return Ok(Value::Int(result));
                                }
                            }
                        }
                    }
                    
                    // Copy the function pointer to avoid borrow checker issues
                    let compiled_fn_opt = self.compiled_functions.get(func_name).copied();
                    
                    if let Some(compiled_fn) = compiled_fn_opt {
                        // Fast path: Direct JIT → JIT call!
                        
                        // OPTIMIZATION: For simple integer-only functions with ≤4 args,
                        // pass arguments directly via VMContext fields instead of HashMap
                        let use_fast_args = args.len() <= 4 && args.iter().all(|a| matches!(a, Value::Int(_)));
                        
                        // ULTRA-FAST PATH: Skip HashMap entirely for simple integer functions
                        // The JIT uses jit_get_arg to read parameters directly from VMContext
                        // We only need the HashMap for functions with non-integer args or >4 args
                        let mut func_locals: HashMap<String, Value>;
                        let use_empty_locals = use_fast_args && chunk.params.len() == args.len();
                        
                        if use_empty_locals {
                            // Ultra-fast: Use empty HashMap - JIT will use VMContext.argN
                            func_locals = HashMap::new();
                        } else {
                            // Normal path: Create locals HashMap for the function parameters
                            func_locals = HashMap::new();
                            
                            // Bind arguments to parameter names
                            for (i, param_name) in chunk.params.iter().enumerate() {
                                if let Some(arg) = args.get(i) {
                                    func_locals.insert(param_name.clone(), arg.clone());
                                }
                            }
                        }
                        
                        // Get or create var_names from cache (avoids re-hashing on every call)
                        let func_name_owned = func_name.to_string();
                        
                        // OPTIMIZATION: Get reference to cached var_names instead of cloning
                        // This avoids HashMap clone on every recursive call
                        let cached_var_names_exists = self.jit_var_names_cache.contains_key(&func_name_owned);
                        if !cached_var_names_exists {
                            // Build var_names once and cache it
                            let mut cached_var_names = HashMap::new();
                            
                            // Register parameter names
                            for param_name in &chunk.params {
                                use std::collections::hash_map::DefaultHasher;
                                use std::hash::{Hash, Hasher};
                                let mut hasher = DefaultHasher::new();
                                param_name.hash(&mut hasher);
                                let hash = hasher.finish();
                                cached_var_names.insert(hash, param_name.clone());
                            }
                            
                            // Register all LoadVar names
                            for instr in &chunk.instructions {
                                if let OpCode::LoadVar(name) = instr {
                                    use std::collections::hash_map::DefaultHasher;
                                    use std::hash::{Hash, Hasher};
                                    let mut hasher = DefaultHasher::new();
                                    name.hash(&mut hasher);
                                    let hash = hasher.finish();
                                    cached_var_names.insert(hash, name.clone());
                                }
                            }
                            
                            self.jit_var_names_cache.insert(func_name_owned.clone(), cached_var_names);
                        }
                        
                        // Get pointer to cached var_names (no clone!)
                        let var_names_ptr: *mut HashMap<u64, String> = self.jit_var_names_cache
                            .get_mut(&func_name_owned)
                            .map(|v| v as *mut HashMap<u64, String>)
                            .unwrap_or(std::ptr::null_mut());
                        
                        let vm_ptr: *mut std::ffi::c_void = self as *mut _ as *mut std::ffi::c_void;
                        
                        // Save stack size to detect return value
                        let stack_size_before = self.stack.len();
                        
                        // Get mutable pointers to VM state for VMContext
                        let stack_ptr: *mut Vec<Value> = &mut self.stack;
                        
                        // Get globals - we need to drop the lock before executing
                        // to avoid deadlock on recursive calls. Since JIT execution
                        // is single-threaded, we can safely use a raw pointer.
                        let globals_ptr: *mut HashMap<String, Value> = {
                            let mut globals_guard = self.globals.lock().unwrap();
                            let ptr = &mut globals_guard.scopes[0] as *mut HashMap<String, Value>;
                            // Explicitly drop to release lock before JIT execution
                            drop(globals_guard);
                            ptr
                        };
                        
                        let locals_ptr: *mut HashMap<String, Value> = &mut func_locals;
                        
                        // Create VMContext with fast argument fields
                        let mut vm_context = crate::jit::VMContext {
                            stack_ptr,
                            locals_ptr,
                            globals_ptr,
                            var_names_ptr,
                            vm_ptr,
                            return_value: 0,
                            has_return_value: false,
                            arg0: if use_fast_args && args.len() > 0 { 
                                if let Value::Int(n) = args[0] { n } else { 0 }
                            } else { 0 },
                            arg1: if use_fast_args && args.len() > 1 { 
                                if let Value::Int(n) = args[1] { n } else { 0 }
                            } else { 0 },
                            arg2: if use_fast_args && args.len() > 2 { 
                                if let Value::Int(n) = args[2] { n } else { 0 }
                            } else { 0 },
                            arg3: if use_fast_args && args.len() > 3 { 
                                if let Value::Int(n) = args[3] { n } else { 0 }
                            } else { 0 },
                            arg_count: args.len() as i64,
                        };
                        
                        // Execute the compiled function!
                        // Lock is NOT held during execution to allow recursive calls
                        let result_code = unsafe {
                            compiled_fn(&mut vm_context)
                        };
                        
                        if result_code != 0 {
                            return Err(format!("JIT execution failed with code: {}", result_code));
                        }
                        
                        // Check for return value - prefer optimized VMContext.return_value
                        // This is the FAST PATH from Phase 7 Step 8 optimization
                        if vm_context.has_return_value {
                            // Use the optimized return value directly
                            return Ok(Value::Int(vm_context.return_value));
                        } else if self.stack.len() > stack_size_before {
                            // Fallback: return value was pushed to stack (old path)
                            let result = self.stack.pop().unwrap();
                            return Ok(result);
                        } else {
                            return Err("JIT-compiled function did not return a value".to_string());
                        }
                    }
                }
                
                // Slow path: Execute through interpreter
                // Save current execution state
                let saved_ip = self.ip;
                let saved_chunk = self.chunk.clone();
                let call_frame_depth = self.call_frames.len();
                
                // Set up the call (creates call frame, switches chunk, resets IP)
                self.call_bytecode_function(function, args)?;
                
                // Execute until this function returns
                // (call_frames will pop back to call_frame_depth)
                while self.call_frames.len() > call_frame_depth {
                    // Check bounds
                    if self.ip >= self.chunk.instructions.len() {
                        return Err("Function execution reached end without return".to_string());
                    }
                    
                    // Get instruction (clone to avoid borrow checker issues)
                    let instruction = self.chunk.instructions[self.ip].clone();
                    self.ip += 1;
                    
                    // Execute the instruction
                    // We need to handle the most common opcodes inline
                    // For complex ones, we could call back to the main run loop
                    match instruction {
                        OpCode::LoadConst(idx) => {
                            let constant = &self.chunk.constants[idx];
                            let value = match constant {
                                Constant::Int(n) => Value::Int(*n),
                                Constant::Float(f) => Value::Float(*f),
                                Constant::String(s) => Value::Str(Arc::new(s.clone())),
                                Constant::Bool(b) => Value::Bool(*b),
                                Constant::None => Value::Null,
                                Constant::Function(chunk) => {
                                    Value::BytecodeFunction {
                                        chunk: (**chunk).clone(),
                                        captured: HashMap::new(),
                                    }
                                }
                                _ => {
                                    return Err("Unsupported constant type in JIT function call".to_string());
                                }
                            };
                            self.stack.push(value);
                        }
                        
                        OpCode::LoadVar(name) => {
                            if let Some(frame) = self.call_frames.last() {
                                if let Some(value) = frame.locals.get(&name) {
                                    self.stack.push(value.clone());
                                } else if let Some(value_ref) = frame.captured.get(&name) {
                                    let value = value_ref.lock().unwrap().clone();
                                    self.stack.push(value);
                                } else {
                                    return Err(format!("Undefined local variable: {}", name));
                                }
                            } else {
                                return Err("No call frame for LoadVar".to_string());
                            }
                        }
                        
                        OpCode::StoreVar(name) => {
                            let value = self.stack.pop().ok_or("Stack underflow")?;
                            if let Some(frame) = self.call_frames.last_mut() {
                                frame.locals.insert(name, value);
                            }
                        }
                        
                        OpCode::Add => {
                            let right = self.stack.pop().ok_or("Stack underflow")?;
                            let left = self.stack.pop().ok_or("Stack underflow")?;
                            let result = self.binary_op(&left, "+", &right)?;
                            self.stack.push(result);
                        }
                        
                        OpCode::Sub => {
                            let right = self.stack.pop().ok_or("Stack underflow")?;
                            let left = self.stack.pop().ok_or("Stack underflow")?;
                            let result = self.binary_op(&left, "-", &right)?;
                            self.stack.push(result);
                        }
                        
                        OpCode::Mul => {
                            let right = self.stack.pop().ok_or("Stack underflow")?;
                            let left = self.stack.pop().ok_or("Stack underflow")?;
                            let result = self.binary_op(&left, "*", &right)?;
                            self.stack.push(result);
                        }
                        
                        OpCode::Div => {
                            let right = self.stack.pop().ok_or("Stack underflow")?;
                            let left = self.stack.pop().ok_or("Stack underflow")?;
                            let result = self.binary_op(&left, "/", &right)?;
                            self.stack.push(result);
                        }
                        
                        OpCode::LessThan => {
                            let right = self.stack.pop().ok_or("Stack underflow")?;
                            let left = self.stack.pop().ok_or("Stack underflow")?;
                            let result = self.compare_op(&left, "<", &right)?;
                            self.stack.push(result);
                        }
                        
                        OpCode::GreaterThan => {
                            let right = self.stack.pop().ok_or("Stack underflow")?;
                            let left = self.stack.pop().ok_or("Stack underflow")?;
                            let result = self.compare_op(&left, ">", &right)?;
                            self.stack.push(result);
                        }
                        
                        OpCode::Return => {
                            let return_value = self.stack.pop().ok_or("Stack underflow in return")?;
                            
                            if let Some(frame) = self.call_frames.pop() {
                                // Pop from function call stack
                                self.function_call_stack.pop();
                                
                                // Restore saved state
                                self.ip = saved_ip;
                                self.chunk = saved_chunk;
                                
                                // Clear stack to frame offset
                                self.stack.truncate(frame.stack_offset);
                                
                                // Return the value
                                return Ok(return_value);
                            } else {
                                return Ok(return_value);
                            }
                        }
                        
                        OpCode::ReturnNone => {
                            if let Some(frame) = self.call_frames.pop() {
                                self.function_call_stack.pop();
                                self.ip = saved_ip;
                                self.chunk = saved_chunk;
                                self.stack.truncate(frame.stack_offset);
                                return Ok(Value::Null);
                            } else {
                                return Ok(Value::Null);
                            }
                        }
                        
                        OpCode::Call(arg_count) => {
                            // Nested function call from within JIT-called function
                            // We can handle this recursively
                            let func = self.stack.pop().ok_or("Stack underflow")?;
                            let mut args = Vec::new();
                            for _ in 0..arg_count {
                                args.push(self.stack.pop().ok_or("Stack underflow")?);
                            }
                            args.reverse();
                            
                            // Recursive call
                            let result = self.call_function_from_jit(func, args)?;
                            self.stack.push(result);
                        }
                        
                        _ => {
                            // For now, unsupported opcodes in nested calls
                            return Err(format!("Unsupported opcode in JIT function call: {:?}", instruction));
                        }
                    }
                }
                
                // Should not reach here - function should have returned
                Err("Function execution completed without explicit return".to_string())
            }
            Value::NativeFunction(_) => {
                // Native function - call it directly
                self.call_native_function_vm(function, args)
            }
            _ => Err("Cannot call non-function".to_string()),
        }
    }

    /// Convert a value to string representation for printing
    fn value_to_string(value: &Value) -> String {
        match value {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Str(s) => s.as_ref().clone(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(Self::value_to_string).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Dict(dict) => {
                let mut keys: Vec<&String> = dict.keys().collect();
                keys.sort();
                let items: Vec<String> = keys
                    .iter()
                    .map(|k| format!("{}: {}", k, Self::value_to_string(dict.get(*k).unwrap())))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            _ => format!("{:?}", value),
        }
    }

    /// Binary operation
    fn binary_op(&self, left: &Value, op: &str, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => match op {
                "+" => Ok(Value::Int(a + b)),
                "-" => Ok(Value::Int(a - b)),
                "*" => Ok(Value::Int(a * b)),
                "/" => {
                    if *b == 0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Value::Int(a / b))
                    }
                }
                "%" => Ok(Value::Int(a % b)),
                _ => Err(format!("Unknown operator: {}", op)),
            },
            (Value::Float(a), Value::Float(b)) => match op {
                "+" => Ok(Value::Float(a + b)),
                "-" => Ok(Value::Float(a - b)),
                "*" => Ok(Value::Float(a * b)),
                "/" => Ok(Value::Float(a / b)),
                "%" => Ok(Value::Float(a % b)),
                _ => Err(format!("Unknown operator: {}", op)),
            },
            (Value::Str(a), Value::Str(b)) if op == "+" => Ok(Value::Str(Arc::new(format!("{}{}", a.as_ref(), b.as_ref())))),
            _ => Err("Type mismatch in binary operation".to_string()),
        }
    }

    /// Unary operation
    fn unary_op(&self, op: &str, value: &Value) -> Result<Value, String> {
        match (op, value) {
            ("-", Value::Int(n)) => Ok(Value::Int(-n)),
            ("-", Value::Float(f)) => Ok(Value::Float(-f)),
            _ => Err(format!("Invalid unary operation: {} {:?}", op, value)),
        }
    }

    /// Comparison operation
    fn compare_op(&self, left: &Value, op: &str, right: &Value) -> Result<Value, String> {
        let result = match (left, right) {
            (Value::Int(a), Value::Int(b)) => match op {
                "<" => a < b,
                ">" => a > b,
                "<=" => a <= b,
                ">=" => a >= b,
                _ => return Err(format!("Unknown comparison: {}", op)),
            },
            (Value::Float(a), Value::Float(b)) => match op {
                "<" => a < b,
                ">" => a > b,
                "<=" => a <= b,
                ">=" => a >= b,
                _ => return Err(format!("Unknown comparison: {}", op)),
            },
            _ => return Err("Type mismatch in comparison".to_string()),
        };

        Ok(Value::Bool(result))
    }

    /// Check if value is truthy
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Int(0) => false,
            Value::Float(f) if *f == 0.0 => false,
            Value::Str(s) if s.is_empty() => false,
            Value::Array(arr) if arr.is_empty() => false,
            Value::Dict(dict) if dict.is_empty() => false,
            _ => true,
        }
    }

    /// Check if two values are equal
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            // TODO: Add array, dict, struct comparison
            _ => false,
        }
    }

    /// Match a pattern against a value
    fn match_pattern(&mut self, pattern: &Pattern, value: &Value) -> Result<bool, String> {
        match pattern {
            Pattern::Identifier(name) => {
                // Bind the value to the name
                if let Some(frame) = self.call_frames.last_mut() {
                    frame.locals.insert(name.clone(), value.clone());
                }
                Ok(true)
            }

            Pattern::Ignore => Ok(true),

            Pattern::Array { elements: _, rest: _ } => {
                if let Value::Array(_arr) = value {
                    // TODO: Implement full pattern matching
                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            Pattern::Dict { keys: _, rest: _ } => {
                if let Value::Dict(_dict) = value {
                    // TODO: Implement full pattern matching
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Execute generator until next yield or completion
    /// Returns Some(value) if yielded, None if exhausted
    pub fn generator_next(&mut self, generator: Value) -> Result<Value, String> {
        if let Value::BytecodeGenerator { state } = generator {
            let gen_state = state.lock().unwrap();

            // Check if generator is exhausted
            if gen_state.is_exhausted {
                return Ok(Value::Option { is_some: false, value: Box::new(Value::Null) });
            }

            // Save current VM state
            let saved_ip = self.ip;
            let saved_chunk = self.chunk.clone();
            let saved_stack = self.stack.clone();
            let saved_frames = self.call_frames.clone();

            // Restore generator state
            self.ip = gen_state.ip;
            self.chunk = gen_state.chunk.clone();
            self.stack = gen_state.stack.clone();

            // Restore call frames
            self.call_frames.clear();
            for frame_data in &gen_state.call_frames_data {
                self.call_frames.push(CallFrame {
                    return_ip: frame_data.return_ip,
                    stack_offset: frame_data.stack_offset,
                    locals: frame_data.locals.clone(),
                    captured: frame_data.captured.clone(),
                    prev_chunk: None,
                    is_async: false, // Generators are not async
                });
            }

            // Drop the lock before executing
            drop(gen_state);

            // Execute until yield or completion
            let result = loop {
                if self.ip >= self.chunk.instructions.len() {
                    // Generator completed without explicit return
                    break Ok(Value::Option { is_some: false, value: Box::new(Value::Null) });
                }

                let instruction = self.chunk.instructions[self.ip].clone();
                self.ip += 1;

                // Check for Yield opcode
                if matches!(instruction, OpCode::Yield) {
                    // Get the yielded value
                    let yielded_value = self.stack.pop().ok_or("Stack underflow in Yield")?;

                    // Save current state back to generator
                    let mut gen_state = state.lock().unwrap();
                    gen_state.ip = self.ip;
                    gen_state.stack = self.stack.clone();

                    // Save call frames
                    gen_state.call_frames_data.clear();
                    for frame in &self.call_frames {
                        gen_state.call_frames_data.push(CallFrameData {
                            return_ip: frame.return_ip,
                            stack_offset: frame.stack_offset,
                            locals: frame.locals.clone(),
                            captured: frame.captured.clone(),
                        });
                    }

                    drop(gen_state);

                    // Restore original VM state
                    self.ip = saved_ip;
                    self.chunk = saved_chunk;
                    self.stack = saved_stack;
                    self.call_frames = saved_frames;

                    // Return the yielded value
                    break Ok(Value::Option { is_some: true, value: Box::new(yielded_value) });
                }

                // Check for Return opcodes (generator completed)
                if matches!(instruction, OpCode::Return | OpCode::ReturnNone) {
                    let mut gen_state = state.lock().unwrap();
                    gen_state.is_exhausted = true;
                    drop(gen_state);

                    // Restore original VM state
                    self.ip = saved_ip;
                    self.chunk = saved_chunk;
                    self.stack = saved_stack;
                    self.call_frames = saved_frames;

                    break Ok(Value::Option { is_some: false, value: Box::new(Value::Null) });
                }

                // Execute the instruction normally (by backing up IP and calling execute on single instruction)
                // This is inefficient but simple - a better approach would be to extract instruction execution
                // For now, we'll manually handle key instructions
                match instruction {
                    OpCode::LoadConst(index) => {
                        let constant = &self.chunk.constants[index];
                        let value = self.constant_to_value(constant)?;
                        self.stack.push(value);
                    }
                    OpCode::LoadVar(name) => {
                        let value = if let Some(frame) = self.call_frames.last() {
                            frame
                                .captured
                                .get(&name)
                                .map(|r| r.lock().unwrap().clone())
                                .or_else(|| frame.locals.get(&name).cloned())
                        } else {
                            None
                        };

                        let value = value
                            .or_else(|| self.globals.lock().unwrap().get(&name))
                            .ok_or_else(|| format!("Undefined variable: {}", name))?;
                        self.stack.push(value);
                    }
                    OpCode::StoreVar(name) => {
                        let value = self.stack.last().ok_or("Stack underflow")?.clone();
                        if let Some(frame) = self.call_frames.last_mut() {
                            if let Some(captured_ref) = frame.captured.get(&name) {
                                *captured_ref.lock().unwrap() = value;
                            } else {
                                frame.locals.insert(name, value);
                            }
                        } else {
                            self.globals.lock().unwrap().define(name, value);
                        }
                    }
                    OpCode::Pop => {
                        self.stack.pop().ok_or("Stack underflow")?;
                    }

                    // Arithmetic operations
                    OpCode::Add => {
                        let right = self.stack.pop().ok_or("Stack underflow")?;
                        let left = self.stack.pop().ok_or("Stack underflow")?;
                        let result = self.binary_op(&left, "+", &right)?;
                        self.stack.push(result);
                    }
                    OpCode::Sub => {
                        let right = self.stack.pop().ok_or("Stack underflow")?;
                        let left = self.stack.pop().ok_or("Stack underflow")?;
                        let result = self.binary_op(&left, "-", &right)?;
                        self.stack.push(result);
                    }
                    OpCode::Mul => {
                        let right = self.stack.pop().ok_or("Stack underflow")?;
                        let left = self.stack.pop().ok_or("Stack underflow")?;
                        let result = self.binary_op(&left, "*", &right)?;
                        self.stack.push(result);
                    }
                    OpCode::Div => {
                        let right = self.stack.pop().ok_or("Stack underflow")?;
                        let left = self.stack.pop().ok_or("Stack underflow")?;
                        let result = self.binary_op(&left, "/", &right)?;
                        self.stack.push(result);
                    }
                    OpCode::Mod => {
                        let right = self.stack.pop().ok_or("Stack underflow")?;
                        let left = self.stack.pop().ok_or("Stack underflow")?;
                        let result = self.binary_op(&left, "%", &right)?;
                        self.stack.push(result);
                    }

                    // Comparison operations
                    OpCode::Equal => {
                        let right = self.stack.pop().ok_or("Stack underflow")?;
                        let left = self.stack.pop().ok_or("Stack underflow")?;
                        self.stack.push(Value::Bool(self.values_equal(&left, &right)));
                    }
                    OpCode::NotEqual => {
                        let right = self.stack.pop().ok_or("Stack underflow")?;
                        let left = self.stack.pop().ok_or("Stack underflow")?;
                        self.stack.push(Value::Bool(!self.values_equal(&left, &right)));
                    }
                    OpCode::LessThan => {
                        let right = self.stack.pop().ok_or("Stack underflow")?;
                        let left = self.stack.pop().ok_or("Stack underflow")?;
                        let result = self.compare_op(&left, "<", &right)?;
                        self.stack.push(result);
                    }
                    OpCode::GreaterThan => {
                        let right = self.stack.pop().ok_or("Stack underflow")?;
                        let left = self.stack.pop().ok_or("Stack underflow")?;
                        let result = self.compare_op(&left, ">", &right)?;
                        self.stack.push(result);
                    }
                    OpCode::LessEqual => {
                        let right = self.stack.pop().ok_or("Stack underflow")?;
                        let left = self.stack.pop().ok_or("Stack underflow")?;
                        let result = self.compare_op(&left, "<=", &right)?;
                        self.stack.push(result);
                    }
                    OpCode::GreaterEqual => {
                        let right = self.stack.pop().ok_or("Stack underflow")?;
                        let left = self.stack.pop().ok_or("Stack underflow")?;
                        let result = self.compare_op(&left, ">=", &right)?;
                        self.stack.push(result);
                    }

                    // Control flow
                    OpCode::Jump(target) => {
                        self.ip = target;
                    }
                    OpCode::JumpIfFalse(target) => {
                        let condition = self.stack.last().ok_or("Stack underflow")?;
                        if !self.is_truthy(condition) {
                            self.ip = target;
                        }
                    }
                    OpCode::JumpIfTrue(target) => {
                        let condition = self.stack.last().ok_or("Stack underflow")?;
                        if self.is_truthy(condition) {
                            self.ip = target;
                        }
                    }
                    OpCode::JumpBack(target) => {
                        self.ip = target;
                    }

                    // For now, return error for other unhandled instructions
                    // Full implementation would need to handle all opcodes
                    _ => {
                        return Err(format!(
                            "Instruction {:?} not yet handled in generator execution",
                            instruction
                        ));
                    }
                }
            };

            result
        } else {
            Err("generator_next() requires a BytecodeGenerator".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::lexer;
    use crate::parser::Parser;

    /// Helper to compile and run Ruff code through the VM
    fn run_vm_code(code: &str) -> Result<Value, String> {
        let tokens = lexer::tokenize(code);
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        let mut compiler = Compiler::new();
        let chunk = compiler.compile(&ast)?;

        let mut vm = VM::new();
        vm.execute(chunk)
    }

    #[test]
    fn test_async_function_definition() {
        let code = r#"
            async func fetch_data(id) {
                return "Data for ID";
            }
            
            let promise = fetch_data(42);
            return await promise;
        "#;

        match run_vm_code(code) {
            Ok(Value::Str(s)) => assert_eq!(s.as_ref(), "Data for ID"),
            Ok(other) => panic!("Expected string, got: {:?}", other),
            Err(e) => panic!("VM error: {}", e),
        }
    }

    #[test]
    fn test_simple_return() {
        let code = r#"
            func get_number() {
                return 42;
            }
            
            return get_number();
        "#;

        let result = run_vm_code(code);
        eprintln!("Simple return test: {:?}", result);

        match result {
            Ok(Value::Int(n)) => assert_eq!(n, 42),
            Ok(other) => panic!("Expected int 42, got: {:?}", other),
            Err(e) => panic!("VM error: {}", e),
        }
    }

    #[test]
    fn test_async_await_basic() {
        let code = r#"
            async func get_number() {
                return 42;
            }
            
            let p = get_number();
            return await p;
        "#;

        let result = run_vm_code(code);
        eprintln!("Test result: {:?}", result);

        match result {
            Ok(Value::Int(n)) => assert_eq!(n, 42),
            Ok(other) => panic!("Expected int 42, got: {:?}", other),
            Err(e) => panic!("VM error: {}", e),
        }
    }

    #[test]
    fn test_async_multiple_await() {
        let code = r#"
            async func double(x) {
                return x * 2;
            }
            
            let p1 = double(5);
            let p2 = double(10);
            let p3 = double(15);
            
            let r1 = await p1;
            let r2 = await p2;
            let r3 = await p3;
            
            return r1 + r2 + r3;
        "#;

        match run_vm_code(code) {
            Ok(Value::Int(n)) => assert_eq!(n, 10 + 20 + 30),
            Ok(other) => panic!("Expected int 60, got: {:?}", other),
            Err(e) => panic!("VM error: {}", e),
        }
    }

    #[test]
    fn test_async_nested_calls() {
        let code = r#"
            async func inner(x) {
                return x + 10;
            }
            
            async func outer(x) {
                let p = inner(x);
                let result = await p;
                return result * 2;
            }
            
            let p = outer(5);
            return await p;
        "#;

        match run_vm_code(code) {
            Ok(Value::Int(n)) => assert_eq!(n, (5 + 10) * 2),
            Ok(other) => panic!("Expected int 30, got: {:?}", other),
            Err(e) => panic!("VM error: {}", e),
        }
    }

    #[test]
    fn test_async_with_computation() {
        let code = r#"
            async func calculate_sum(a, b, c) {
                let sum = a + b + c;
                return sum;
            }
            
            let promise = calculate_sum(10, 20, 30);
            return await promise;
        "#;

        match run_vm_code(code) {
            Ok(Value::Int(n)) => assert_eq!(n, 60),
            Ok(other) => panic!("Expected int 60, got: {:?}", other),
            Err(e) => panic!("VM error: {}", e),
        }
    }

    #[test]
    fn test_make_promise_opcode() {
        let code = r#"
            # Test MakePromise opcode (though not directly accessible in syntax)
            # We can test it indirectly through async functions
            async func simple() {
                return 123;
            }
            
            return await simple();
        "#;

        match run_vm_code(code) {
            Ok(Value::Int(n)) => assert_eq!(n, 123),
            Ok(other) => panic!("Expected int 123, got: {:?}", other),
            Err(e) => panic!("VM error: {}", e),
        }
    }

    #[test]
    fn test_async_promise_reuse() {
        let code = r#"
            async func get_value() {
                return 999;
            }
            
            let promise = get_value();
            
            # Await the same promise multiple times
            let first = await promise;
            let second = await promise;
            
            return first == second;
        "#;

        match run_vm_code(code) {
            Ok(Value::Bool(b)) => assert!(b, "Promise should return same value on multiple awaits"),
            Ok(other) => panic!("Expected bool true, got: {:?}", other),
            Err(e) => panic!("VM error: {}", e),
        }
    }

    #[test]
    fn test_await_non_promise() {
        let code = r#"
            # Awaiting a non-promise should just return the value
            let x = 42;
            return await x;
        "#;

        match run_vm_code(code) {
            Ok(Value::Int(n)) => assert_eq!(n, 42),
            Ok(other) => panic!("Expected int 42, got: {:?}", other),
            Err(e) => panic!("VM error: {}", e),
        }
    }
}
