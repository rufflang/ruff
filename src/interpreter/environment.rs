// File: src/interpreter/environment.rs
//
// Lexical scoping environment for variable management in the Ruff interpreter.
// Implements a stack of scopes where inner scopes shadow outer scopes.

use super::value::Value;
use std::collections::HashMap;

/// Variable storage using lexical scoping
///
/// The Environment maintains a stack of scopes (Vec<HashMap>). When looking up
/// a variable, we search from the innermost scope (end of Vec) outward. This
/// implements proper lexical scoping with shadowing.
///
/// # Examples
///
/// ```ignore
/// let mut env = Environment::new();
/// env.define("x".to_string(), Value::Int(10));  // Global scope
/// 
/// env.push_scope();                             // Enter function scope
/// env.define("x".to_string(), Value::Int(20));  // Shadows outer x
/// assert_eq!(env.get("x"), Some(&Value::Int(20)));
/// 
/// env.pop_scope();                              // Exit function scope
/// assert_eq!(env.get("x"), Some(&Value::Int(10)));  // Original x visible again
/// ```
#[derive(Clone, Debug)]
pub struct Environment {
    pub scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    /// Create a new environment with a single global scope
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    /// Push a new scope onto the stack (e.g., entering a function)
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the innermost scope from the stack (e.g., exiting a function)
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Get a variable from the environment, searching from inner to outer scopes
    /// Returns a cloned value if found
    pub fn get(&self, name: &str) -> Option<Value> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    /// Define a new variable in the current (innermost) scope
    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    /// Set an existing variable, searching from inner to outer scopes
    /// If not found, creates it in the current scope
    pub fn set(&mut self, name: String, value: Value) {
        // Try to find and update existing variable
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name) {
                scope.insert(name, value);
                return;
            }
        }
        // If not found, create in current scope
        self.define(name, value);
    }

    /// Mutate an existing variable using a closure
    ///
    /// This is useful for in-place modifications like `x += 1` where we want to
    /// read the current value, modify it, and write it back atomically.
    pub fn mutate<F>(&mut self, name: &str, f: F) -> bool
    where
        F: FnOnce(&mut Value),
    {
        // Find the scope containing this variable
        for scope in self.scopes.iter_mut().rev() {
            if let Some(value) = scope.get_mut(name) {
                f(value);
                return true;
            }
        }
        false
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
