// File: src/vm.rs
//
// Virtual Machine for executing Ruff bytecode.
// Stack-based VM with support for function calls, closures, and all Ruff features.

use crate::ast::Pattern;
use crate::bytecode::{BytecodeChunk, Constant, OpCode};
use crate::interpreter::{Environment, Interpreter, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Upvalue: heap-allocated captured variable for closures
#[derive(Debug, Clone)]
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
        }
    }

    /// Set the global environment (for accessing built-in functions)
    pub fn set_globals(&mut self, env: Arc<Mutex<Environment>>) {
        self.globals = env.clone();
        // Also set the interpreter's environment so it can resolve native functions
        self.interpreter.set_env(env);
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
                                eprintln!("LoadVar('{}'): checking globals -> {:?}", name, global_val.is_some());
                            }
                            global_val
                        })
                        .ok_or_else(|| {
                            if std::env::var("DEBUG_VM").is_ok() {
                                eprintln!("LoadVar('{}'): FAILED - not in captured, locals or globals", name);
                                eprintln!("  Current frame captured: {:?}", 
                                    self.call_frames.last().map(|f| f.captured.keys().collect::<Vec<_>>()));
                                eprintln!("  Current frame locals: {:?}", 
                                    self.call_frames.last().map(|f| f.locals.keys().collect::<Vec<_>>()));
                            }
                            format!("Undefined variable: {}", name)
                        })?;

                    self.stack.push(value);
                }

                OpCode::LoadGlobal(name) => {
                    let value = self
                        .globals
                        .lock().unwrap()
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
                        Value::BytecodeFunction { .. } => {
                            // Set up call frame and switch context
                            // Return value will be pushed by Return opcode
                            self.call_bytecode_function(function, args)?;
                            // Don't push anything - Return will do it
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
                        // Restore previous state
                        self.ip = frame.return_ip;
                        if let Some(prev_chunk) = frame.prev_chunk {
                            self.chunk = prev_chunk;
                        }

                        // Clear stack to frame offset
                        self.stack.truncate(frame.stack_offset);

                        // Push return value
                        self.stack.push(return_value);
                    } else {
                        // Top-level return
                        return Ok(return_value);
                    }
                }

                OpCode::ReturnNone => {
                    if let Some(frame) = self.call_frames.pop() {
                        self.ip = frame.return_ip;
                        if let Some(prev_chunk) = frame.prev_chunk {
                            self.chunk = prev_chunk;
                        }
                        self.stack.truncate(frame.stack_offset);
                        self.stack.push(Value::Null);
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
                            eprintln!("MakeClosure: function has {} upvalues: {:?}", 
                                chunk.upvalues.len(), chunk.upvalues);
                            eprintln!("  Call stack depth: {}", self.call_frames.len());
                            if let Some(frame) = self.call_frames.last() {
                                eprintln!("  Current frame has {} locals: {:?}, {} captured: {:?}", 
                                    frame.locals.len(), frame.locals.keys().collect::<Vec<_>>(),
                                    frame.captured.len(), frame.captured.keys().collect::<Vec<_>>());
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
                                    eprintln!("  Captured '{}' from locals = {:?}", upvalue_name, val);
                                }
                                // Wrap in Arc<Mutex<>> for shared mutable state
                                captured.insert(upvalue_name.clone(), Arc::new(Mutex::new(val)));
                            } else {
                                if std::env::var("DEBUG_VM").is_ok() {
                                    eprintln!("  Skipped '{}' (not in locals, will resolve at runtime)", upvalue_name);
                                }
                                // Variable not in locals - it's either a global or undefined
                                // Don't capture it - let it be resolved at runtime
                            }
                        }
                        
                        // Create a closure value with captured variables
                        let value = Value::BytecodeFunction {
                            chunk: (**chunk).clone(),
                            captured,
                        };
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
                    self.stack.push(Value::Array(elements));
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
                            Value::Str(s) => s,
                            _ => return Err("Dict keys must be strings".to_string()),
                        };

                        dict.insert(key_str, value);
                    }
                    self.stack.push(Value::Dict(dict));
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
                            dict.get(key).cloned().unwrap_or(Value::Null)
                        }
                        (Value::Str(s), Value::Int(i)) => {
                            let idx =
                                if *i < 0 { (s.len() as i64 + i) as usize } else { *i as usize };
                            s.chars()
                                .nth(idx)
                                .map(|c| Value::Str(c.to_string()))
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
                        (Value::Array(mut arr), Value::Int(i)) => {
                            let idx =
                                if i < 0 { (arr.len() as i64 + i) as usize } else { i as usize };

                            if idx < arr.len() {
                                arr[idx] = value;
                                self.stack.push(Value::Array(arr));
                            } else {
                                return Err(format!("Index out of bounds: {}", i));
                            }
                        }
                        (Value::Dict(mut dict), Value::Str(key)) => {
                            dict.insert(key, value);
                            self.stack.push(Value::Dict(dict));
                        }
                        _ => return Err("Invalid index assignment".to_string()),
                    }
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
                        Value::Dict(mut dict) => {
                            dict.insert(field, value);
                            self.stack.push(Value::Dict(dict));
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
                                self.stack.push(Value::Str(key.clone()));
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
                    let result = self.interpreter.call_native_function_impl("iterator_next", &[iterator.clone()]);
                    self.stack.push(iterator); // Keep iterator on stack
                    self.stack.push(result); // Push result (Some/None)
                }

                OpCode::IteratorHasNext => {
                    let iterator = self.stack.last().ok_or("Stack underflow")?.clone();
                    // Check if iterator has more values
                    let has_next = match &iterator {
                        Value::Iterator { index, source, .. } => {
                            match source.as_ref() {
                                Value::Array(arr) => *index < arr.len(),
                                _ => false,
                            }
                        }
                        _ => false,
                    };
                    self.stack.push(Value::Bool(has_next));
                }

                // Generator operations
                OpCode::Yield | OpCode::ResumeGenerator | OpCode::MakeGenerator => {
                    // Generators require more complex state management
                    // For now, return an error - will implement in Week 5-6
                    return Err("Generator operations not yet implemented in VM".to_string());
                }

                // Async/await operations
                OpCode::Await | OpCode::MakePromise | OpCode::MarkAsync => {
                    // Async operations require integration with the runtime
                    // For now, return an error - will implement in Week 5-6
                    return Err("Async/await operations not yet implemented in VM".to_string());
                }

                // Exception handling
                OpCode::BeginTry(_) | OpCode::EndTry | OpCode::Throw |
                OpCode::BeginCatch(_) | OpCode::EndCatch => {
                    // Exception handling requires exception table lookup
                    // For now, return an error - will implement in Week 5-6
                    return Err("Exception handling not yet implemented in VM".to_string());
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
                        Value::ErrorObject { .. } => return Err(format!("Error in native function {}", name)),
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
                    }.or_else(|| {
                        // Try globals
                        self.globals.lock().unwrap().get(&name)
                    }).ok_or_else(|| format!("Variable '{}' not found for capture", name))?;
                    
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
            Constant::String(s) => Ok(Value::Str(s.clone())),
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
                Ok(Value::Array(array))
            }
            Constant::Dict(pairs) => {
                let mut dict = HashMap::new();
                for (key_const, value_const) in pairs {
                    let key = self.constant_to_value(key_const)?;
                    let value = self.constant_to_value(value_const)?;
                    
                    // Key must be a string
                    if let Value::Str(key_str) = key {
                        dict.insert(key_str, value);
                    } else {
                        return Err("Dict constant keys must be strings".to_string());
                    }
                }
                Ok(Value::Dict(dict))
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
                eprintln!("CallFrame has {} captured variables: {:?}", captured_map.len(), captured_map.keys().collect::<Vec<_>>());
            }

            let frame = CallFrame {
                return_ip: self.ip,
                stack_offset: self.stack.len(),
                locals,
                captured: captured_map,
                prev_chunk: Some(self.chunk.clone()),
            };

            self.call_frames.push(frame);

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

    /// Convert a value to string representation for printing
    fn value_to_string(value: &Value) -> String {
        match value {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Str(s) => s.clone(),
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
            (Value::Str(a), Value::Str(b)) if op == "+" => Ok(Value::Str(format!("{}{}", a, b))),
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
}
