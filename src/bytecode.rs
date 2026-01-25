// File: src/bytecode.rs
//
// Bytecode instruction definitions and structures for the Ruff VM.
// Defines OpCode enum representing all bytecode instructions and supporting types.

use std::collections::HashMap;

/// Bytecode instruction opcodes for the Ruff VM
/// Stack-based virtual machine with separate value and call stacks
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Many opcodes not yet used - VM is work in progress
pub enum OpCode {
    // === Stack Operations ===
    /// Load a constant from the constant pool onto the stack
    /// Operand: constant pool index
    LoadConst(usize),
    
    /// Load a variable value onto the stack
    /// Operand: variable name
    LoadVar(String),
    
    /// Load a global variable value onto the stack
    /// Operand: variable name
    LoadGlobal(String),
    
    /// Store top of stack to a variable (creates or updates)
    /// Operand: variable name
    StoreVar(String),
    
    /// Store top of stack to a global variable
    /// Operand: variable name
    StoreGlobal(String),
    
    /// Pop top value from stack (discard result)
    Pop,
    
    /// Duplicate top value on stack
    Dup,
    
    // === Arithmetic Operations ===
    /// Pop two values, add them, push result
    Add,
    
    /// Pop two values, subtract (top from second), push result
    Sub,
    
    /// Pop two values, multiply them, push result
    Mul,
    
    /// Pop two values, divide (second by top), push result
    Div,
    
    /// Pop two values, modulo (second % top), push result
    Mod,
    
    /// Pop one value, negate it, push result
    Negate,
    
    // === Comparison Operations ===
    /// Pop two values, compare equal, push bool result
    Equal,
    
    /// Pop two values, compare not equal, push bool result
    NotEqual,
    
    /// Pop two values, compare less than, push bool result
    LessThan,
    
    /// Pop two values, compare greater than, push bool result
    GreaterThan,
    
    /// Pop two values, compare less than or equal, push bool result
    LessEqual,
    
    /// Pop two values, compare greater than or equal, push bool result
    GreaterEqual,
    
    // === Logical Operations ===
    /// Pop one value, logical NOT, push result
    Not,
    
    /// Pop two values, logical AND (short-circuit handled by jumps)
    And,
    
    /// Pop two values, logical OR (short-circuit handled by jumps)
    Or,
    
    // === Control Flow ===
    /// Unconditional jump to instruction
    /// Operand: target instruction index
    Jump(usize),
    
    /// Pop value, jump if false
    /// Operand: target instruction index
    JumpIfFalse(usize),
    
    /// Pop value, jump if true
    /// Operand: target instruction index
    JumpIfTrue(usize),
    
    /// Jump backwards (for loops)
    /// Operand: target instruction index
    JumpBack(usize),
    
    // === Function Operations ===
    /// Call a function with N arguments
    /// Arguments are already on stack (bottom to top)
    /// Operand: number of arguments
    Call(usize),
    
    /// Return from function with value on stack
    Return,
    
    /// Return None/null from function (for void functions)
    ReturnNone,
    
    /// Create a closure and push onto stack
    /// Operand: constant pool index (contains function code)
    MakeClosure(usize),
    
    // === Collection Operations ===
    /// Create an array from N values on stack
    /// Operand: number of elements
    MakeArray(usize),
    
    /// Create a dict from 2N values on stack (key1, val1, key2, val2, ...)
    /// Operand: number of key-value pairs
    MakeDict(usize),
    
    /// Pop index and object, push object[index]
    IndexGet,
    
    /// Pop value, index, and object, set object[index] = value
    IndexSet,
    
    /// Pop object, push object.field
    /// Operand: field name
    FieldGet(String),
    
    /// Pop value and object, set object.field = value
    /// Operand: field name
    FieldSet(String),
    
    // === Spread Operations ===
    /// Spread an array/dict onto stack for collection construction
    /// Pops one collection, pushes all its elements
    SpreadArray,
    
    /// Spread an array as function arguments
    SpreadArgs,
    
    /// Spread a dict into another dict (merge operation)
    SpreadDict,
    
    // === Pattern Matching ===
    /// Match a value against a pattern and bind variables
    /// Stack: [value] -> [success: bool]
    /// If match succeeds, variables are bound in current scope
    /// Operand: pattern index in constant pool
    MatchPattern(usize),
    
    /// Start a new match case branch
    /// Used for organizing match statement bytecode
    BeginCase,
    
    /// End a match case branch
    EndCase,
    
    // === Result/Option Operations ===
    /// Create an Ok(value) Result
    /// Stack: [value] -> [Result::Ok(value)]
    MakeOk,
    
    /// Create an Err(value) Result  
    /// Stack: [value] -> [Result::Err(value)]
    MakeErr,
    
    /// Create a Some(value) Option
    /// Stack: [value] -> [Option::Some(value)]
    MakeSome,
    
    /// Create a None Option
    /// Stack: [] -> [Option::None]
    MakeNone,
    
    /// Try operator: propagate errors or unwrap success
    /// If Result::Err or Option::None, early return with that value
    /// Otherwise, unwrap the inner value
    /// Stack: [Result/Option] -> [inner_value]
    TryUnwrap,
    
    // === Struct Operations ===
    /// Create a struct instance from N field values on stack
    /// Operand: (struct_name, field_names)
    MakeStruct(String, Vec<String>),
    
    // === Environment Management ===
    /// Push a new scope (for blocks, functions)
    PushScope,
    
    /// Pop a scope
    PopScope,
    
    // === Debugging ===
    /// Print current stack state (for debugging)
    #[allow(dead_code)]
    DebugStack,
    
    /// No operation (placeholder)
    Nop,
}

/// A compiled bytecode chunk containing instructions and constants
#[derive(Debug, Clone, PartialEq)]
pub struct BytecodeChunk {
    /// The bytecode instructions
    pub instructions: Vec<OpCode>,
    
    /// Constant pool containing literals and other constants
    pub constants: Vec<Constant>,
    
    /// Source location mapping for error reporting
    /// Maps instruction index to (line, column)
    pub source_map: HashMap<usize, (usize, usize)>,
    
    /// Optional name (for functions)
    pub name: Option<String>,
    
    /// Parameter names for functions
    pub params: Vec<String>,
}

#[allow(dead_code)] // Methods not yet used - VM integration incomplete
impl BytecodeChunk {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            source_map: HashMap::new(),
            name: None,            params: Vec::new(),        }
    }
    
    /// Add a constant to the pool and return its index
    pub fn add_constant(&mut self, constant: Constant) -> usize {
        // Check if constant already exists (simple optimization)
        if let Some(index) = self.constants.iter().position(|c| c == &constant) {
            return index;
        }
        
        let index = self.constants.len();
        self.constants.push(constant);
        index
    }
    
    /// Emit an instruction and return its index
    pub fn emit(&mut self, instruction: OpCode) -> usize {
        let index = self.instructions.len();
        self.instructions.push(instruction);
        index
    }
    
    /// Patch a jump instruction at the given index with the current position
    pub fn patch_jump(&mut self, jump_index: usize) {
        let target = self.instructions.len();
        match &mut self.instructions[jump_index] {
            OpCode::Jump(ref mut addr) |
            OpCode::JumpIfFalse(ref mut addr) |
            OpCode::JumpIfTrue(ref mut addr) => {
                *addr = target;
            }
            _ => panic!("Attempted to patch non-jump instruction"),
        }
    }
    
    /// Set the jump target for a jump instruction
    pub fn set_jump_target(&mut self, jump_index: usize, target: usize) {
        match &mut self.instructions[jump_index] {
            OpCode::Jump(ref mut addr) |
            OpCode::JumpIfFalse(ref mut addr) |
            OpCode::JumpIfTrue(ref mut addr) |
            OpCode::JumpBack(ref mut addr) => {
                *addr = target;
            }
            _ => panic!("Attempted to set target on non-jump instruction"),
        }
    }
}

/// Constants that can be stored in the constant pool
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Not all variants used yet - VM is work in progress
pub enum Constant {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    None,
    /// A compiled function (stored as bytecode chunk)
    Function(Box<BytecodeChunk>),
    /// Pattern for matching (stored AST pattern)
    Pattern(crate::ast::Pattern),
}

/// Represents a compiled function with its bytecode
#[derive(Debug, Clone)]
#[allow(dead_code)] // Not yet used - function compilation incomplete
pub struct CompiledFunction {
    pub name: String,
    pub arity: usize, // Number of parameters
    pub chunk: BytecodeChunk,
    pub upvalues: Vec<String>, // Captured variables for closures
}
