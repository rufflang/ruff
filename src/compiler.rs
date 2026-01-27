// File: src/compiler.rs
//
// Bytecode compiler for the Ruff programming language.
// Compiles AST nodes into bytecode instructions for the VM.

use crate::ast::{ArrayElement, DictElement, Expr, Pattern, Stmt};
use crate::bytecode::{BytecodeChunk, Constant, OpCode};
use crate::optimizer::Optimizer;

/// Compiler state for generating bytecode from AST
#[allow(dead_code)] // Compiler not yet integrated into execution path
pub struct Compiler {
    /// Current bytecode chunk being compiled
    chunk: BytecodeChunk,

    /// Stack of loop start positions for break/continue
    loop_starts: Vec<usize>,

    /// Stack of loop end jump indices to patch for break statements
    loop_ends: Vec<Vec<usize>>,

    /// Current scope depth (0 = global)
    scope_depth: usize,

    /// Local variables in current scope
    locals: Vec<Local>,

    /// Parent compiler (for nested functions/closures)
    parent: Option<Box<Compiler>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Helper struct for incomplete feature
struct Local {
    name: String,
    depth: usize,
}

#[allow(dead_code)] // Compiler not yet integrated into execution path
impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: BytecodeChunk::new(),
            loop_starts: Vec::new(),
            loop_ends: Vec::new(),
            scope_depth: 0,
            locals: Vec::new(),
            parent: None,
        }
    }

    /// Compile a list of statements into bytecode
    pub fn compile(&mut self, statements: &[Stmt]) -> Result<BytecodeChunk, String> {
        self.compile_with_optimization(statements, true)
    }

    /// Compile with optional optimization
    pub fn compile_with_optimization(
        &mut self,
        statements: &[Stmt],
        optimize: bool,
    ) -> Result<BytecodeChunk, String> {
        for stmt in statements {
            self.compile_stmt(stmt)?;
        }

        // Ensure we return None at the end if no explicit return
        self.chunk.emit(OpCode::ReturnNone);

        // Apply optimizations if enabled
        let mut chunk = self.chunk.clone();
        if optimize {
            let mut optimizer = Optimizer::new();
            optimizer.optimize(&mut chunk);

            // Log optimization stats in debug mode
            if cfg!(debug_assertions) {
                if optimizer.stats.constants_folded > 0
                    || optimizer.stats.dead_instructions_removed > 0
                    || optimizer.stats.peephole_optimizations > 0
                {
                    eprintln!("Compiler optimization: {} constants folded, {} dead instructions removed, {} peephole optimizations",
                        optimizer.stats.constants_folded,
                        optimizer.stats.dead_instructions_removed,
                        optimizer.stats.peephole_optimizations);
                }
            }
        }

        Ok(chunk)
    }

    /// Compile a single statement
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::ExprStmt(expr) => {
                self.compile_expr(expr)?;
                self.chunk.emit(OpCode::Pop); // Discard expression result
                Ok(())
            }

            Stmt::Let { pattern, value, .. } => {
                // Compile the value
                self.compile_expr(value)?;

                // Bind the pattern
                self.compile_pattern_binding(pattern)?;

                Ok(())
            }

            Stmt::Assign { target, value } => {
                // Compile the value
                self.compile_expr(value)?;

                // Compile the assignment target
                self.compile_assignment(target)?;

                Ok(())
            }

            Stmt::If { condition, then_branch, else_branch } => {
                // Compile condition
                self.compile_expr(condition)?;

                // Jump to else block if condition is false
                let else_jump = self.chunk.emit(OpCode::JumpIfFalse(0));
                self.chunk.emit(OpCode::Pop); // Pop condition

                // Compile then block
                for stmt in then_branch {
                    self.compile_stmt(stmt)?;
                }

                // Jump over else block
                let end_jump = self.chunk.emit(OpCode::Jump(0));

                // Patch else jump
                self.chunk.patch_jump(else_jump);
                self.chunk.emit(OpCode::Pop); // Pop condition

                // Compile else block if present
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.compile_stmt(stmt)?;
                    }
                }

                // Patch end jump
                self.chunk.patch_jump(end_jump);

                Ok(())
            }

            Stmt::While { condition, body, .. } => {
                let loop_start = self.chunk.instructions.len();
                self.loop_starts.push(loop_start);
                self.loop_ends.push(Vec::new());

                // Compile condition
                self.compile_expr(condition)?;

                // Jump to end if condition is false
                let end_jump = self.chunk.emit(OpCode::JumpIfFalse(0));
                self.chunk.emit(OpCode::Pop); // Pop condition

                // Compile body
                for stmt in body {
                    self.compile_stmt(stmt)?;
                }

                // Jump back to condition
                self.chunk.emit(OpCode::JumpBack(loop_start));

                // Patch end jump
                self.chunk.patch_jump(end_jump);
                self.chunk.emit(OpCode::Pop); // Pop condition

                // Patch all break statements
                if let Some(breaks) = self.loop_ends.pop() {
                    for break_jump in breaks {
                        self.chunk.patch_jump(break_jump);
                    }
                }
                self.loop_starts.pop();

                Ok(())
            }

            Stmt::For { var, iterable, body } => {
                // For now, compile as a while loop with an iterator
                // This is a simplified implementation

                // Evaluate the iterable
                self.compile_expr(iterable)?;

                // Store in a temporary variable for iteration
                let iter_var = format!("__iter_{}", self.scope_depth);
                self.chunk.emit(OpCode::StoreVar(iter_var.clone()));

                // Create index variable
                let index_var = format!("__index_{}", self.scope_depth);
                let zero_index = self.chunk.add_constant(Constant::Int(0));
                self.chunk.emit(OpCode::LoadConst(zero_index));
                self.chunk.emit(OpCode::StoreVar(index_var.clone()));

                let loop_start = self.chunk.instructions.len();
                self.loop_starts.push(loop_start);
                self.loop_ends.push(Vec::new());

                // Load iterator and index
                self.chunk.emit(OpCode::LoadVar(iter_var.clone()));
                self.chunk.emit(OpCode::LoadVar(index_var.clone()));

                // Check if index < len(iterable) - using built-in len
                self.chunk.emit(OpCode::LoadVar(iter_var.clone()));
                self.chunk.emit(OpCode::LoadGlobal("len".to_string()));
                self.chunk.emit(OpCode::Call(1));
                self.chunk.emit(OpCode::LessThan);

                // Jump to end if done
                let end_jump = self.chunk.emit(OpCode::JumpIfFalse(0));
                self.chunk.emit(OpCode::Pop);

                // Get current element: iterable[index]
                self.chunk.emit(OpCode::LoadVar(iter_var.clone()));
                self.chunk.emit(OpCode::LoadVar(index_var.clone()));
                self.chunk.emit(OpCode::IndexGet);

                // Store in loop variable
                self.chunk.emit(OpCode::StoreVar(var.clone()));

                // Compile body
                for stmt in body {
                    self.compile_stmt(stmt)?;
                }

                // Increment index
                self.chunk.emit(OpCode::LoadVar(index_var.clone()));
                let one_index = self.chunk.add_constant(Constant::Int(1));
                self.chunk.emit(OpCode::LoadConst(one_index));
                self.chunk.emit(OpCode::Add);
                self.chunk.emit(OpCode::StoreVar(index_var.clone()));

                // Jump back to start
                self.chunk.emit(OpCode::JumpBack(loop_start));

                // Patch end jump
                self.chunk.patch_jump(end_jump);
                self.chunk.emit(OpCode::Pop);

                // Patch all break statements
                if let Some(breaks) = self.loop_ends.pop() {
                    for break_jump in breaks {
                        self.chunk.patch_jump(break_jump);
                    }
                }
                self.loop_starts.pop();

                Ok(())
            }

            Stmt::Return(value) => {
                if let Some(expr) = value {
                    self.compile_expr(expr)?;
                    self.chunk.emit(OpCode::Return);
                } else {
                    self.chunk.emit(OpCode::ReturnNone);
                }
                Ok(())
            }

            Stmt::Break => {
                // Add a jump that will be patched later
                let jump_index = self.chunk.emit(OpCode::Jump(0));
                if let Some(breaks) = self.loop_ends.last_mut() {
                    breaks.push(jump_index);
                }
                Ok(())
            }

            Stmt::Continue => {
                // Jump back to loop start
                if let Some(&loop_start) = self.loop_starts.last() {
                    self.chunk.emit(OpCode::JumpBack(loop_start));
                }
                Ok(())
            }

            Stmt::FuncDef { name, params, body, is_async, is_generator, .. } => {
                // Create a new compiler for the function body
                let mut func_compiler = Compiler::new();
                func_compiler.chunk.name = Some(name.clone());
                func_compiler.chunk.params = params.clone();
                func_compiler.chunk.is_async = *is_async;
                func_compiler.chunk.is_generator = *is_generator;
                func_compiler.scope_depth = 1; // Functions create a new scope (not global)

                // Add parameters as locals
                for param in params {
                    func_compiler.locals.push(Local { name: param.clone(), depth: 1 });
                }

                // Analyze the function body to find free variables (captures)
                let free_vars = Self::find_free_variables(body, params, &self.locals);
                func_compiler.chunk.upvalues = free_vars.clone();

                // Add captured variables as locals so the compiler knows they exist
                for upvalue_name in &free_vars {
                    func_compiler.locals.push(Local { name: upvalue_name.clone(), depth: 1 });
                }

                // Compile function body
                for stmt in body {
                    func_compiler.compile_stmt(stmt)?;
                }

                // Ensure function returns None if no explicit return
                func_compiler.chunk.emit(OpCode::ReturnNone);

                // Add function as constant
                let func_index =
                    self.chunk.add_constant(Constant::Function(Box::new(func_compiler.chunk)));

                // Create closure and store in variable
                self.chunk.emit(OpCode::MakeClosure(func_index));
                self.chunk.emit(OpCode::StoreGlobal(name.clone()));

                Ok(())
            }

            Stmt::StructDef { .. } => {
                // Struct definitions are handled at runtime for now
                // TODO: Optimize struct construction in bytecode
                Ok(())
            }

            Stmt::Match { value, cases, default } => {
                // Compile the value to match
                self.compile_expr(value)?;

                let mut case_jumps = Vec::new();
                let mut end_jumps = Vec::new();

                for (pattern_name, body) in cases {
                    self.chunk.emit(OpCode::BeginCase);

                    // Duplicate the value for matching
                    self.chunk.emit(OpCode::Dup);

                    // For now, just match against string patterns
                    // TODO: Implement full pattern matching with Pattern enum
                    let pattern_str_index =
                        self.chunk.add_constant(Constant::String(pattern_name.clone()));
                    self.chunk.emit(OpCode::LoadConst(pattern_str_index));
                    self.chunk.emit(OpCode::Equal);

                    // If match fails, jump to next case
                    let next_case_jump = self.chunk.emit(OpCode::JumpIfFalse(0));
                    self.chunk.emit(OpCode::Pop); // Pop match result

                    case_jumps.push(next_case_jump);

                    // Pop the original value since we matched
                    self.chunk.emit(OpCode::Pop);

                    // Compile case body
                    for stmt in body {
                        self.compile_stmt(stmt)?;
                    }

                    // Jump to end of match
                    let end_jump = self.chunk.emit(OpCode::Jump(0));
                    end_jumps.push(end_jump);

                    // Patch the jump to next case
                    self.chunk.patch_jump(next_case_jump);
                    self.chunk.emit(OpCode::Pop); // Pop match result

                    self.chunk.emit(OpCode::EndCase);
                }

                // Compile default case if present
                if let Some(default_body) = default {
                    self.chunk.emit(OpCode::Pop); // Pop the value
                    for stmt in default_body {
                        self.compile_stmt(stmt)?;
                    }
                } else {
                    // Pop the original value if no case matched
                    self.chunk.emit(OpCode::Pop);
                }

                // Patch all end jumps
                for end_jump in end_jumps {
                    self.chunk.patch_jump(end_jump);
                }

                Ok(())
            }

            Stmt::Loop { condition, body } => {
                let loop_start = self.chunk.instructions.len();
                self.loop_starts.push(loop_start);
                self.loop_ends.push(Vec::new());

                // If there's a condition, check it
                if let Some(cond_expr) = condition {
                    self.compile_expr(cond_expr)?;

                    // Jump to end if condition is false
                    let end_jump = self.chunk.emit(OpCode::JumpIfFalse(0));
                    self.chunk.emit(OpCode::Pop); // Pop condition

                    // Compile body
                    for stmt in body {
                        self.compile_stmt(stmt)?;
                    }

                    // Jump back to start
                    self.chunk.emit(OpCode::JumpBack(loop_start));

                    // Patch end jump
                    self.chunk.patch_jump(end_jump);
                    self.chunk.emit(OpCode::Pop); // Pop condition
                } else {
                    // Unconditional loop
                    for stmt in body {
                        self.compile_stmt(stmt)?;
                    }

                    // Jump back to start
                    self.chunk.emit(OpCode::JumpBack(loop_start));
                }

                // Patch all break statements
                if let Some(breaks) = self.loop_ends.pop() {
                    for break_jump in breaks {
                        self.chunk.patch_jump(break_jump);
                    }
                }
                self.loop_starts.pop();

                Ok(())
            }

            Stmt::TryExcept { try_block, except_var, except_block } => {
                // Set up exception handler
                let try_start = self.chunk.instructions.len();

                // Emit BeginTry with placeholder catch address
                let begin_try_index = self.chunk.emit(OpCode::BeginTry(0));

                // Compile try block
                for stmt in try_block {
                    self.compile_stmt(stmt)?;
                }

                // End try block
                self.chunk.emit(OpCode::EndTry);

                // Jump over catch block if no exception
                let end_jump = self.chunk.emit(OpCode::Jump(0));

                // Catch block starts here
                let catch_start = self.chunk.instructions.len();

                // Patch BeginTry with actual catch address
                self.chunk.set_jump_target(begin_try_index, catch_start);

                // Begin catch and bind exception to variable
                self.chunk.emit(OpCode::BeginCatch(except_var.clone()));

                // Compile catch block
                for stmt in except_block {
                    self.compile_stmt(stmt)?;
                }

                // End catch block
                self.chunk.emit(OpCode::EndCatch);

                // Patch the jump over catch block
                self.chunk.patch_jump(end_jump);

                // Record exception handler in metadata
                let try_end = catch_start - 1;
                self.chunk.exception_handlers.push(crate::bytecode::ExceptionHandler {
                    try_start,
                    try_end,
                    catch_start,
                    exception_var: except_var.clone(),
                });

                Ok(())
            }

            Stmt::Block(statements) => {
                // Enter new scope
                self.chunk.emit(OpCode::PushScope);
                self.scope_depth += 1;

                // Compile block statements
                for stmt in statements {
                    self.compile_stmt(stmt)?;
                }

                // Exit scope
                self.scope_depth -= 1;
                self.chunk.emit(OpCode::PopScope);

                Ok(())
            }

            Stmt::Const { name, value, .. } => {
                // Constants are like immutable variables at compile time
                // Evaluate the value
                self.compile_expr(value)?;

                // Store as global (constants are always global)
                self.chunk.emit(OpCode::StoreGlobal(name.clone()));

                Ok(())
            }

            Stmt::EnumDef { name: _, variants: _ } => {
                // Enum definitions are handled at runtime for now
                // They don't generate bytecode, just metadata
                Ok(())
            }

            Stmt::Export { stmt } => {
                // Export is just a marker - compile the inner statement
                self.compile_stmt(stmt)?;
                Ok(())
            }

            Stmt::Spawn { body } => {
                // Spawn creates a background thread
                // For now, compile as a lambda that gets executed asynchronously
                // This is simplified - full implementation needs runtime support

                // Create function for spawn body
                let mut spawn_compiler = Compiler::new();
                spawn_compiler.chunk.name = Some("<spawn>".to_string());
                spawn_compiler.scope_depth = 0;

                for stmt in body {
                    spawn_compiler.compile_stmt(stmt)?;
                }

                spawn_compiler.chunk.emit(OpCode::ReturnNone);

                let func_index =
                    self.chunk.add_constant(Constant::Function(Box::new(spawn_compiler.chunk)));

                // Load function and call it (runtime will handle thread spawning)
                self.chunk.emit(OpCode::MakeClosure(func_index));
                // TODO: Need a SpawnThread opcode for proper thread spawning
                // For now this will just create a closure
                self.chunk.emit(OpCode::Pop); // Pop the closure for now

                Ok(())
            }

            Stmt::Import { .. }
            | Stmt::Test { .. }
            | Stmt::TestSetup { .. }
            | Stmt::TestTeardown { .. }
            | Stmt::TestGroup { .. } => {
                // These are handled at parse/runtime for now
                // Import requires module system
                // Test statements are executed by test runner
                Ok(())
            }
        }
    }

    /// Compile an expression
    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Int(n) => {
                let index = self.chunk.add_constant(Constant::Int(*n));
                self.chunk.emit(OpCode::LoadConst(index));
                Ok(())
            }

            Expr::Float(f) => {
                let index = self.chunk.add_constant(Constant::Float(*f));
                self.chunk.emit(OpCode::LoadConst(index));
                Ok(())
            }

            Expr::String(s) => {
                let index = self.chunk.add_constant(Constant::String(s.clone()));
                self.chunk.emit(OpCode::LoadConst(index));
                Ok(())
            }

            Expr::Bool(b) => {
                let index = self.chunk.add_constant(Constant::Bool(*b));
                self.chunk.emit(OpCode::LoadConst(index));
                Ok(())
            }

            Expr::Identifier(name) => {
                // Check if it's a local variable
                if self.is_local(name) {
                    self.chunk.emit(OpCode::LoadVar(name.clone()));
                } else {
                    self.chunk.emit(OpCode::LoadGlobal(name.clone()));
                }
                Ok(())
            }

            Expr::BinaryOp { left, op, right } => {
                // Compile operands
                self.compile_expr(left)?;
                self.compile_expr(right)?;

                // Emit operation
                match op.as_str() {
                    "+" => self.chunk.emit(OpCode::Add),
                    "-" => self.chunk.emit(OpCode::Sub),
                    "*" => self.chunk.emit(OpCode::Mul),
                    "/" => self.chunk.emit(OpCode::Div),
                    "%" => self.chunk.emit(OpCode::Mod),
                    "==" => self.chunk.emit(OpCode::Equal),
                    "!=" => self.chunk.emit(OpCode::NotEqual),
                    "<" => self.chunk.emit(OpCode::LessThan),
                    ">" => self.chunk.emit(OpCode::GreaterThan),
                    "<=" => self.chunk.emit(OpCode::LessEqual),
                    ">=" => self.chunk.emit(OpCode::GreaterEqual),
                    "&&" => self.chunk.emit(OpCode::And),
                    "||" => self.chunk.emit(OpCode::Or),
                    _ => return Err(format!("Unknown binary operator: {}", op)),
                };

                Ok(())
            }

            Expr::UnaryOp { op, operand } => {
                self.compile_expr(operand)?;

                match op.as_str() {
                    "-" => self.chunk.emit(OpCode::Negate),
                    "!" => self.chunk.emit(OpCode::Not),
                    _ => return Err(format!("Unknown unary operator: {}", op)),
                };

                Ok(())
            }

            Expr::Call { function, args } => {
                // Compile arguments (bottom to top on stack)
                for arg in args {
                    self.compile_expr(arg)?;
                }

                // Compile function expression
                self.compile_expr(function)?;

                // Emit call with argument count
                self.chunk.emit(OpCode::Call(args.len()));

                Ok(())
            }

            Expr::ArrayLiteral(elements) => {
                // Check if we have any spread operations
                let has_spread = elements.iter().any(|e| matches!(e, ArrayElement::Spread(_)));

                if has_spread {
                    // With spreads, push a marker first, then all elements
                    self.chunk.emit(OpCode::PushArrayMarker);
                }

                // Compile all elements and spreads
                for element in elements {
                    match element {
                        ArrayElement::Single(expr) => {
                            self.compile_expr(expr)?;
                        }
                        ArrayElement::Spread(expr) => {
                            self.compile_expr(expr)?;
                            self.chunk.emit(OpCode::SpreadArray);
                        }
                    }
                }

                // MakeArray will collect everything
                // If there was a marker, it collects until marker
                // Otherwise, it collects exactly 'count' elements
                self.chunk.emit(OpCode::MakeArray(elements.len()));

                Ok(())
            }

            Expr::DictLiteral(elements) => {
                let mut pair_count = 0;

                for element in elements {
                    match element {
                        DictElement::Pair(key, value) => {
                            self.compile_expr(key)?;
                            self.compile_expr(value)?;
                            pair_count += 1;
                        }
                        DictElement::Spread(expr) => {
                            self.compile_expr(expr)?;
                            self.chunk.emit(OpCode::SpreadDict);
                        }
                    }
                }

                self.chunk.emit(OpCode::MakeDict(pair_count));

                Ok(())
            }

            Expr::IndexAccess { object, index } => {
                self.compile_expr(object)?;
                self.compile_expr(index)?;
                self.chunk.emit(OpCode::IndexGet);
                Ok(())
            }

            Expr::FieldAccess { object, field } => {
                self.compile_expr(object)?;
                self.chunk.emit(OpCode::FieldGet(field.clone()));
                Ok(())
            }

            Expr::Function { params, body, .. } => {
                // Create anonymous function
                let mut func_compiler = Compiler::new();
                func_compiler.chunk.name = Some("<lambda>".to_string());
                func_compiler.chunk.params = params.clone();
                func_compiler.scope_depth = 1; // Functions create a new scope (not global)

                // Add parameters as locals
                for param in params {
                    func_compiler.locals.push(Local { name: param.clone(), depth: 1 });
                }

                // Analyze the function body to find free variables (captures)
                let free_vars = Self::find_free_variables(body, params, &self.locals);
                func_compiler.chunk.upvalues = free_vars.clone();

                // Add captured variables as locals so the compiler knows they exist
                // They will be resolved from the closure's captured map at runtime
                for upvalue_name in &free_vars {
                    func_compiler.locals.push(Local { name: upvalue_name.clone(), depth: 1 });
                }

                // Compile function body
                for stmt in body {
                    func_compiler.compile_stmt(stmt)?;
                }

                func_compiler.chunk.emit(OpCode::ReturnNone);

                let func_index =
                    self.chunk.add_constant(Constant::Function(Box::new(func_compiler.chunk)));
                self.chunk.emit(OpCode::MakeClosure(func_index));

                Ok(())
            }

            Expr::Ok(value) => {
                self.compile_expr(value)?;
                self.chunk.emit(OpCode::MakeOk);
                Ok(())
            }

            Expr::Err(value) => {
                self.compile_expr(value)?;
                self.chunk.emit(OpCode::MakeErr);
                Ok(())
            }

            Expr::Some(value) => {
                self.compile_expr(value)?;
                self.chunk.emit(OpCode::MakeSome);
                Ok(())
            }

            Expr::None => {
                self.chunk.emit(OpCode::MakeNone);
                Ok(())
            }

            Expr::Try(expr) => {
                self.compile_expr(expr)?;
                self.chunk.emit(OpCode::TryUnwrap);
                Ok(())
            }

            Expr::StructInstance { name, fields } => {
                // Compile field values
                let mut field_names = Vec::new();
                for (field_name, field_value) in fields {
                    field_names.push(field_name.clone());
                    self.compile_expr(field_value)?;
                }

                self.chunk.emit(OpCode::MakeStruct(name.clone(), field_names));
                Ok(())
            }

            Expr::Tag(tag, values) => {
                // Special handling for throw - it's a control flow primitive
                if tag == "throw" {
                    // Compile error value
                    if values.len() != 1 {
                        return Err("throw() requires exactly one argument".to_string());
                    }
                    self.compile_expr(&values[0])?;

                    // Emit throw instruction
                    self.chunk.emit(OpCode::Throw);

                    return Ok(());
                }

                // Compile tag values for other tags
                for value in values {
                    self.compile_expr(value)?;
                }

                // For now, treat as array with tag name
                // TODO: Optimize enum handling
                let tag_index = self.chunk.add_constant(Constant::String(tag.clone()));
                self.chunk.emit(OpCode::LoadConst(tag_index));
                self.chunk.emit(OpCode::MakeArray(values.len() + 1));

                Ok(())
            }

            Expr::InterpolatedString(parts) => {
                // Compile each part and concatenate
                for part in parts {
                    match part {
                        crate::ast::InterpolatedStringPart::Text(s) => {
                            let index = self.chunk.add_constant(Constant::String(s.clone()));
                            self.chunk.emit(OpCode::LoadConst(index));
                        }
                        crate::ast::InterpolatedStringPart::Expr(e) => {
                            self.compile_expr(e)?;
                            // Convert to string using to_string builtin
                            self.chunk.emit(OpCode::LoadGlobal("to_string".to_string()));
                            self.chunk.emit(OpCode::Call(1));
                        }
                    }
                }

                // Concatenate all parts
                // TODO: Optimize with a dedicated string builder
                for _ in 1..parts.len() {
                    self.chunk.emit(OpCode::Add); // String concatenation
                }

                Ok(())
            }

            Expr::Yield(value_expr) => {
                // Compile the value to yield (or None if no value)
                if let Some(expr) = value_expr {
                    self.compile_expr(expr)?;
                } else {
                    let none_index = self.chunk.add_constant(Constant::None);
                    self.chunk.emit(OpCode::LoadConst(none_index));
                }

                // Emit yield instruction
                // This saves the current state and returns to caller
                self.chunk.emit(OpCode::Yield);

                // Mark the chunk as a generator
                self.chunk.is_generator = true;

                Ok(())
            }

            Expr::Await(promise_expr) => {
                // Compile the promise expression
                self.compile_expr(promise_expr)?;

                // Emit await instruction
                // This suspends execution until the promise resolves
                self.chunk.emit(OpCode::Await);

                // Mark the chunk as async
                self.chunk.is_async = true;

                Ok(())
            }

            Expr::MethodCall { object, method, args } => {
                // Method calls are sugar for calling a method on an object
                // Translate: obj.method(a, b) -> method(obj, a, b)

                // Compile the object (becomes first argument)
                self.compile_expr(object)?;

                // Compile other arguments
                for arg in args {
                    self.compile_expr(arg)?;
                }

                // Load the method (it's either a field or a built-in method)
                // For built-in iterator methods (map, filter, etc.), use native calls
                match method.as_str() {
                    "map" | "filter" | "reduce" | "collect" | "take" | "skip" | "zip"
                    | "enumerate" | "chain" | "flatten" | "chunk" => {
                        // These are native iterator functions
                        self.chunk.emit(OpCode::CallNative(method.clone(), args.len() + 1));
                    }
                    _ => {
                        // General method call: load field then call
                        self.compile_expr(object)?;
                        self.chunk.emit(OpCode::FieldGet(method.clone()));

                        // Move function to top of stack (after arguments)
                        // Stack: [obj, arg1, arg2, ..., method]
                        // Need: [obj, arg1, arg2, ..., method] for Call

                        self.chunk.emit(OpCode::Call(args.len() + 1)); // +1 for object as first arg
                    }
                }

                Ok(())
            }

            Expr::Spread(_) => {
                Err("Spread operator cannot be compiled as standalone expression".to_string())
            }
        }
    }

    /// Compile pattern binding (for let statements)
    fn compile_pattern_binding(&mut self, pattern: &Pattern) -> Result<(), String> {
        match pattern {
            Pattern::Identifier(name) => {
                if self.scope_depth == 0 {
                    self.chunk.emit(OpCode::StoreGlobal(name.clone()));
                } else {
                    self.chunk.emit(OpCode::StoreVar(name.clone()));
                    self.locals.push(Local { name: name.clone(), depth: self.scope_depth });
                }
                Ok(())
            }

            Pattern::Ignore => {
                // Just pop the value
                self.chunk.emit(OpCode::Pop);
                Ok(())
            }

            Pattern::Array { elements: _, rest: _ } => {
                // Use MatchPattern for complex binding
                let pattern_index = self.chunk.add_constant(Constant::Pattern(pattern.clone()));
                self.chunk.emit(OpCode::MatchPattern(pattern_index));
                self.chunk.emit(OpCode::Pop); // Pop the success bool
                Ok(())
            }

            Pattern::Dict { keys: _, rest: _ } => {
                // Use MatchPattern for complex binding
                let pattern_index = self.chunk.add_constant(Constant::Pattern(pattern.clone()));
                self.chunk.emit(OpCode::MatchPattern(pattern_index));
                self.chunk.emit(OpCode::Pop); // Pop the success bool
                Ok(())
            }
        }
    }

    /// Compile assignment target
    fn compile_assignment(&mut self, target: &Expr) -> Result<(), String> {
        match target {
            Expr::Identifier(name) => {
                if self.is_local(name) {
                    self.chunk.emit(OpCode::StoreVar(name.clone()));
                } else {
                    self.chunk.emit(OpCode::StoreGlobal(name.clone()));
                }
                Ok(())
            }

            Expr::IndexAccess { object, index } => {
                // Value is already on stack
                // Need: [value, object, index]
                self.compile_expr(object)?;
                self.compile_expr(index)?;
                self.chunk.emit(OpCode::IndexSet);
                Ok(())
            }

            Expr::FieldAccess { object, field } => {
                // Value is already on stack
                self.compile_expr(object)?;
                self.chunk.emit(OpCode::FieldSet(field.clone()));
                Ok(())
            }

            _ => Err("Invalid assignment target".to_string()),
        }
    }

    /// Check if a variable is a local
    fn is_local(&self, name: &str) -> bool {
        self.locals.iter().any(|local| local.name == name)
    }

    /// Find free variables in a function body
    /// Free variables are variables that are used but not defined locally (not params or let bindings)
    fn find_free_variables(
        body: &[Stmt],
        params: &[String],
        _parent_locals: &[Local],
    ) -> Vec<String> {
        use std::collections::HashSet;

        let mut used_vars = HashSet::new();
        let mut defined_vars: HashSet<String> = params.iter().cloned().collect();

        // Helper function to collect variable usage from expressions
        fn collect_expr_vars(expr: &Expr, used: &mut HashSet<String>) {
            match expr {
                Expr::Identifier(name) => {
                    used.insert(name.clone());
                }
                Expr::BinaryOp { left, right, .. } => {
                    collect_expr_vars(left, used);
                    collect_expr_vars(right, used);
                }
                Expr::UnaryOp { operand, .. } => {
                    collect_expr_vars(operand, used);
                }
                Expr::Call { function, args } => {
                    collect_expr_vars(function, used);
                    for arg in args {
                        collect_expr_vars(arg, used);
                    }
                }
                Expr::MethodCall { object, args, .. } => {
                    collect_expr_vars(object, used);
                    for arg in args {
                        collect_expr_vars(arg, used);
                    }
                }
                Expr::ArrayLiteral(elements) => {
                    for elem in elements {
                        match elem {
                            ArrayElement::Single(e) | ArrayElement::Spread(e) => {
                                collect_expr_vars(e, used);
                            }
                        }
                    }
                }
                Expr::DictLiteral(entries) => {
                    for entry in entries {
                        match entry {
                            DictElement::Pair(k, v) => {
                                collect_expr_vars(k, used);
                                collect_expr_vars(v, used);
                            }
                            DictElement::Spread(e) => {
                                collect_expr_vars(e, used);
                            }
                        }
                    }
                }
                Expr::IndexAccess { object, index } => {
                    collect_expr_vars(object, used);
                    collect_expr_vars(index, used);
                }
                Expr::FieldAccess { object, .. } => {
                    collect_expr_vars(object, used);
                }
                Expr::Function { body, .. } => {
                    // Don't descend into nested functions - they have their own scope
                    for stmt in body {
                        collect_stmt_vars(stmt, used, &mut HashSet::new());
                    }
                }
                Expr::Ok(e) | Expr::Err(e) | Expr::Some(e) | Expr::Await(e) => {
                    collect_expr_vars(e, used);
                }
                Expr::Yield(Some(e)) => {
                    collect_expr_vars(e, used);
                }
                Expr::Try(e) => {
                    collect_expr_vars(e, used);
                }
                Expr::StructInstance { fields, .. } => {
                    for (_, expr) in fields {
                        collect_expr_vars(expr, used);
                    }
                }
                Expr::InterpolatedString(parts) => {
                    for part in parts {
                        if let crate::ast::InterpolatedStringPart::Expr(e) = part {
                            collect_expr_vars(e, used);
                        }
                    }
                }
                _ => {}
            }
        }

        // Helper function to collect variable definitions and usage from statements
        fn collect_stmt_vars(
            stmt: &Stmt,
            used: &mut HashSet<String>,
            defined: &mut HashSet<String>,
        ) {
            match stmt {
                Stmt::Let { pattern, value, .. } => {
                    collect_expr_vars(value, used);
                    // Add defined variables from pattern
                    if let Pattern::Identifier(name) = pattern {
                        defined.insert(name.clone());
                    }
                }
                Stmt::Assign { target, value } => {
                    collect_expr_vars(value, used);
                    collect_expr_vars(target, used);
                }
                Stmt::ExprStmt(expr) => {
                    collect_expr_vars(expr, used);
                }
                Stmt::If { condition, then_branch, else_branch } => {
                    collect_expr_vars(condition, used);
                    for s in then_branch {
                        collect_stmt_vars(s, used, defined);
                    }
                    if let Some(else_stmts) = else_branch {
                        for s in else_stmts {
                            collect_stmt_vars(s, used, defined);
                        }
                    }
                }
                Stmt::While { condition, body } => {
                    collect_expr_vars(condition, used);
                    for s in body {
                        collect_stmt_vars(s, used, defined);
                    }
                }
                Stmt::For { var, iterable, body } => {
                    collect_expr_vars(iterable, used);
                    defined.insert(var.clone());
                    for s in body {
                        collect_stmt_vars(s, used, defined);
                    }
                }
                Stmt::Return(expr) => {
                    if let Some(e) = expr {
                        collect_expr_vars(e, used);
                    }
                }
                Stmt::Break | Stmt::Continue => {}
                Stmt::Match { value, cases, default } => {
                    collect_expr_vars(value, used);
                    for (_pattern, stmts) in cases {
                        for s in stmts {
                            collect_stmt_vars(s, used, defined);
                        }
                    }
                    if let Some(default_stmts) = default {
                        for s in default_stmts {
                            collect_stmt_vars(s, used, defined);
                        }
                    }
                }
                Stmt::FuncDef { name, body, .. } => {
                    defined.insert(name.clone());
                    // Don't descend into nested function bodies
                    for s in body {
                        collect_stmt_vars(s, used, &mut HashSet::new());
                    }
                }
                Stmt::TryExcept { try_block, except_block, .. } => {
                    for s in try_block {
                        collect_stmt_vars(s, used, defined);
                    }
                    for s in except_block {
                        collect_stmt_vars(s, used, defined);
                    }
                }
                Stmt::Block(stmts) => {
                    for s in stmts {
                        collect_stmt_vars(s, used, defined);
                    }
                }
                Stmt::Const { name, value, .. } => {
                    defined.insert(name.clone());
                    collect_expr_vars(value, used);
                }
                Stmt::Export { stmt } => {
                    collect_stmt_vars(stmt, used, defined);
                }
                Stmt::Spawn { body } => {
                    for s in body {
                        collect_stmt_vars(s, used, defined);
                    }
                }
                Stmt::StructDef { name, .. } => {
                    defined.insert(name.clone());
                }
                Stmt::EnumDef { name, .. } => {
                    defined.insert(name.clone());
                }
                Stmt::Import { module, symbols } => {
                    // Module itself becomes a variable
                    defined.insert(module.clone());
                    // Imported symbols also become variables
                    if let Some(syms) = symbols {
                        for sym in syms {
                            defined.insert(sym.clone());
                        }
                    }
                }
                Stmt::Loop { condition, body } => {
                    if let Some(cond) = condition {
                        collect_expr_vars(cond, used);
                    }
                    for s in body {
                        collect_stmt_vars(s, used, defined);
                    }
                }
                Stmt::Test { body, .. }
                | Stmt::TestSetup { body }
                | Stmt::TestTeardown { body } => {
                    for s in body {
                        collect_stmt_vars(s, used, defined);
                    }
                }
                Stmt::TestGroup { tests, .. } => {
                    for s in tests {
                        collect_stmt_vars(s, used, defined);
                    }
                }
            }
        }

        // Collect all variable usage and definitions
        for stmt in body {
            collect_stmt_vars(stmt, &mut used_vars, &mut defined_vars);
        }

        // Free variables are those used but not defined locally
        let mut free_vars: Vec<String> = used_vars.difference(&defined_vars).cloned().collect();

        // Sort for deterministic output
        free_vars.sort();
        free_vars
    }
}
