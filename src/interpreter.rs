// File: src/interpreter.rs
//
// Tree-walking interpreter for the Ruff programming language.
// Executes Ruff programs by traversing the Abstract Syntax Tree (AST).
//
// The interpreter maintains an environment (symbol table) for variables and
// functions, evaluates expressions to produce values, and executes statements
// to perform actions. It supports:
// - Variable binding and mutation
// - Function calls with lexical scoping
// - Enum variants and pattern matching
// - Error handling with try/except/throw
// - Control flow (if/else, loops, match)
// - Binary operations on numbers and strings
//
// Values in Ruff can be numbers, strings, tagged enum variants, functions,
// or error values for exception handling.

use crate::ast::{Expr, Stmt};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// Runtime values in the Ruff interpreter
#[derive(Clone)]
pub enum Value {
    Tagged { tag: String, fields: HashMap<String, Value> },
    Number(f64),
    Str(String),
    Function(Vec<String>, Vec<Stmt>),
    Return(Box<Value>),
    Error(String),
    #[allow(dead_code)]
    Enum(String),
}

/// Environment holds variable and function bindings
#[derive(Default)]
pub struct Environment {
    pub vars: HashMap<String, Value>,
}

/// Main interpreter that executes Ruff programs
pub struct Interpreter {
    pub env: Environment,
    pub return_value: Option<Value>,
    output: Option<Arc<Mutex<Vec<u8>>>>,
}

impl Interpreter {
    /// Creates a new interpreter with an empty environment
    pub fn new() -> Self {
        Interpreter {
            env: Environment::default(),
            return_value: None,
            output: None,
        }
    }

    /// Sets the output sink for print statements (used for testing)
    pub fn set_output(&mut self, output: Arc<Mutex<Vec<u8>>>) {
        self.output = Some(output);
    }

    /// Evaluates a list of statements sequentially, stopping on return/error
    pub fn eval_stmts(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.eval_stmt(stmt);
            if self.return_value.is_some() {
                break;
            }
        }
    }

    /// Helper to write output to either the output buffer or stdout
    fn write_output(&self, msg: &str) {
        if let Some(out) = &self.output {
            let mut buffer = out.lock().unwrap();
            let _ = writeln!(buffer, "{}", msg); // already includes newline
        } else {
            println!("{}", msg);
        }
    }

    /// Evaluates a single statement
    fn eval_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::If { condition, then_branch, else_branch } => {
                let cond_val = self.eval_expr(condition);
                if let Value::Number(n) = cond_val {
                    if n != 0.0 {
                        self.eval_stmts(then_branch);
                    } else if let Some(else_branch) = else_branch {
                        self.eval_stmts(else_branch);
                    }
                }
            }
            Stmt::Block(stmts) => {
                self.eval_stmts(&stmts);
            }
            Stmt::Let { name, value, mutable: _ } => {
                let val = self.eval_expr(&value);
                // If expression evaluation resulted in an error, propagate it
                if matches!(val, Value::Error(_)) {
                    self.return_value = Some(val.clone());
                }
                self.env.vars.insert(name.clone(), val);
            }
            Stmt::Const { name, value } => {
                let val = self.eval_expr(&value);
                // If expression evaluation resulted in an error, propagate it
                if matches!(val, Value::Error(_)) {
                    self.return_value = Some(val.clone());
                }
                self.env.vars.insert(name.clone(), val);
            }
            Stmt::Assign { name, value } => {
                let val = self.eval_expr(&value);
                // If expression evaluation resulted in an error, propagate it
                if matches!(val, Value::Error(_)) {
                    self.return_value = Some(val.clone());
                }
                if self.env.vars.contains_key(name.as_str()) {
                    self.env.vars.insert(name.clone(), val);
                }
            }
            Stmt::FuncDef { name, params, body } => {
                let func = Value::Function(params.clone(), body.clone());
                self.env.vars.insert(name.clone(), func);
            }
            Stmt::EnumDef { name, variants } => {
                for variant in variants {
                    let tag = format!("{}::{}", name, variant);
                    // Store constructor function in env
                    let func = Value::Function(
                        vec!["$0".to_string()],
                        vec![Stmt::Return(Some(Expr::Tag(
                            tag.clone(),
                            vec![Expr::Identifier("$0".to_string())],
                        )))],
                    );
                    self.env.vars.insert(tag.clone(), func);
                }
            }
            Stmt::Match { value, cases, default } => {
                let val = self.eval_expr(&value);

                let (tag, fields): (String, &HashMap<String, Value>) = match &val {
                    Value::Tagged { tag, fields } => (tag.clone(), fields),
                    Value::Enum(e) => {
                        static EMPTY: once_cell::sync::Lazy<HashMap<String, Value>> =
                            once_cell::sync::Lazy::new(HashMap::new);
                        (e.clone(), &EMPTY)
                    }
                    Value::Str(s) => {
                        static EMPTY: once_cell::sync::Lazy<HashMap<String, Value>> =
                            once_cell::sync::Lazy::new(HashMap::new);
                        (s.clone(), &EMPTY)
                    }
                    Value::Number(n) => {
                        static EMPTY: once_cell::sync::Lazy<HashMap<String, Value>> =
                            once_cell::sync::Lazy::new(HashMap::new);
                        (n.to_string(), &EMPTY)
                    }
                    _ => {
                        if let Some(default_body) = default {
                            self.eval_stmts(&default_body);
                        }
                        return;
                    }
                };

                for (pattern, body) in cases {
                    if let Some(open_paren) = pattern.find('(') {
                        let (enum_tag, param_var) = pattern.split_at(open_paren);
                        let param_var = param_var.trim_matches(&['(', ')'][..]);
                        if tag == enum_tag.trim() {
                            let mut scoped = self.env.vars.clone();
                            for i in 0.. {
                                let key = format!("${}", i);
                                if let Some(val) = fields.get(&key) {
                                    let param_name = if i == 0 {
                                        param_var.to_string()
                                    } else {
                                        format!("{}_{}", param_var, i)
                                    };
                                    scoped.insert(param_name, val.clone());
                                } else {
                                    break;
                                }
                            }
                            let old_env = std::mem::replace(&mut self.env, Environment { vars: scoped });
                            self.eval_stmts(body);
                            self.env = old_env;
                            return;
                        }
                    } else if pattern.as_str() == tag {
                        self.eval_stmts(body);
                        return;
                    }
                }

                if let Some(default_body) = default {
                    self.eval_stmts(&default_body);
                }
            }
            Stmt::Loop { condition, body } => {
                while condition
                    .as_ref()
                    .map(|c| matches!(self.eval_expr(&c), Value::Number(n) if n != 0.0))
                    .unwrap_or(true)
                {
                    self.eval_stmts(&body);
                    if self.return_value.is_some() {
                        break;
                    }
                }
            }
            Stmt::For { var, iterable, body } => {
                if let Value::Number(n) = self.eval_expr(&iterable) {
                    for i in 0..n as i64 {
                        let mut local_env = self.env.vars.clone();
                        local_env.insert(var.clone(), Value::Number(i as f64));
                        let mut inner = Interpreter {
                            env: Environment { vars: local_env },
                            return_value: None,
                            output: self.output.clone(),
                        };
                        inner.eval_stmts(&body);
                        if let Some(rv) = inner.return_value {
                            self.return_value = Some(rv);
                            break;
                        }
                    }
                }
            }
            Stmt::Return(expr) => {
                let value = expr.as_ref().map(|e| self.eval_expr(&e)).unwrap_or(Value::Number(0.0));
                self.return_value = Some(Value::Return(Box::new(value)));
            }
            Stmt::TryExcept { try_block, except_var, except_block } => {
                let backup_env = self.env.vars.clone();
                let mut inner = Interpreter {
                    env: Environment { vars: backup_env.clone() },
                    return_value: None,
                    output: self.output.clone(),
                };
                inner.eval_stmts(&try_block);
                if let Some(Value::Error(msg)) = inner.return_value {
                    let mut except_env = backup_env;
                    except_env.insert(except_var.clone(), Value::Str(msg));
                    let mut handler = Interpreter {
                        env: Environment { vars: except_env },
                        return_value: None,
                        output: self.output.clone(),
                    };
                    handler.eval_stmts(&except_block);
                    self.return_value = handler.return_value;
                } else {
                    self.return_value = inner.return_value;
                }
            }
            Stmt::ExprStmt(expr) => {
                match expr {
                    // built-in print
                    Expr::Tag(name, args) if name == "print" => {
                        let output_parts: Vec<String> = args.iter()
                            .map(|arg| {
                                let v = self.eval_expr(arg);
                                Interpreter::stringify_value(&v)
                            })
                            .collect();
                        self.write_output(&output_parts.join(" "));
                    }

                    // built-in throw
                    Expr::Tag(name, args) if name == "throw" => {
                        if let Some(arg) = args.get(0) {
                            match self.eval_expr(arg) {
                                Value::Str(s) => self.return_value = Some(Value::Error(s)),
                                _            => self.return_value = Some(Value::Error("error".into())),
                            }
                        }
                    }

                    // enum constructors or user functions (tags)
                    Expr::Tag(_, _) => {
                        let _ = self.eval_expr(expr);
                    }

                    // user-defined Function calls, if you ever use Expr::Call
                    Expr::Call { function, args } => {
                        let func_val = self.eval_expr(&function);
                        if let Value::Function(params, body) = func_val {
                            let mut local_env = self.env.vars.clone();
                            for (i, param) in params.iter().enumerate() {
                                if let Some(arg) = args.get(i) {
                                    let val = self.eval_expr(arg);
                                    local_env.insert(param.clone(), val);
                                }
                            }
                            let mut inner = Interpreter {
                                env: Environment { vars: local_env },
                                return_value: None,
                                output: self.output.clone(),
                            };
                            inner.eval_stmts(&body);
                            if let Some(Value::Return(val)) = inner.return_value {
                                self.return_value = Some(*val);
                            } else if let Some(Value::Error(msg)) = inner.return_value {
                                self.return_value = Some(Value::Error(msg));
                            }
                        }
                    }

                    // everything else
                    _ => {
                        let _ = self.eval_expr(expr);
                    }
                }
            }
        }
    }

    /// Evaluates an expression to produce a value
    fn eval_expr(&self, expr: &Expr) -> Value {
        match expr {
            Expr::Number(n) => Value::Number(*n),
            Expr::String(s) => Value::Str(s.clone()),
            Expr::Identifier(name) => self.env.vars.get(name).cloned().unwrap_or(Value::Str(name.clone())),
            Expr::BinaryOp { left, op, right } => {
                let l = self.eval_expr(&left);
                let r = self.eval_expr(&right);
                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => match op.as_str() {
                        "+" => Value::Number(a + b),
                        "-" => Value::Number(a - b),
                        "*" => Value::Number(a * b),
                        "/" => Value::Number(a / b),
                        "==" => Value::Number(if (a - b).abs() < f64::EPSILON { 1.0 } else { 0.0 }),
                        ">" => Value::Number(if a > b { 1.0 } else { 0.0 }),
                        "<" => Value::Number(if a < b { 1.0 } else { 0.0 }),
                        ">=" => Value::Number(if a >= b { 1.0 } else { 0.0 }),
                        "<=" => Value::Number(if a <= b { 1.0 } else { 0.0 }),
                        _ => Value::Number(0.0),
                    },
                    (Value::Str(a), Value::Str(b)) => match op.as_str() {
                        "+" => Value::Str(a + &b),
                        "==" => Value::Number(if a == b { 1.0 } else { 0.0 }),
                        _ => Value::Number(0.0),
                    },
                    _ => Value::Number(0.0),
                }
            }
            Expr::Call { function, args } => {
                let func_val = self.eval_expr(&function);
                if let Value::Function(params, body) = func_val {
                    let mut local_env = self.env.vars.clone();
                    for (i, param) in params.iter().enumerate() {
                        if let Some(arg) = args.get(i) {
                            let val = self.eval_expr(arg);
                            local_env.insert(param.clone(), val);
                        }
                    }
                    let mut inner = Interpreter {
                        env: Environment { vars: local_env },
                        return_value: None,
                        output: self.output.clone(),
                    };
                    inner.eval_stmts(&body);
                    if let Some(Value::Return(val)) = inner.return_value {
                        *val
                    } else if let Some(Value::Error(msg)) = inner.return_value {
                        Value::Error(msg) // Propagate error instead of returning 0
                    } else {
                        Value::Number(0.0)
                    }
                } else {
                    Value::Number(0.0)
                }
            }
            Expr::Tag(name, args) => {
                let mut fields = HashMap::new();
                for (i, arg) in args.iter().enumerate() {
                    fields.insert(format!("${}", i), self.eval_expr(&arg));
                }
                Value::Tagged { tag: name.clone(), fields }
            }
        }
    }
    
    /// Converts a runtime value to a string for display
    fn stringify_value(value: &Value) -> String {
        match value {
            Value::Str(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Tagged { tag, fields } => {
                if fields.is_empty() {
                    tag.clone()
                } else {
                    let args: Vec<String> = fields.values()
                        .map(|v| Interpreter::stringify_value(v))
                        .collect();
                    format!("{}({})", tag, args.join(","))
                }
            }
            Value::Return(inner) => Interpreter::stringify_value(inner),
            Value::Error(msg) => format!("Error: {}", msg),
            _ => "<unknown>".into(),
        }
    }
}
