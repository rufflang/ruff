// File: src/type_checker.rs
//
// Type checker for the Ruff programming language.
// Performs type inference and type checking on the AST before interpretation.
//
// Features:
// - Type inference for expressions and variables
// - Type checking for assignments, function calls, and return statements
// - Symbol table for tracking variable and function types
// - Support for gradual typing (mixed typed/untyped code)
//
// The type checker uses a two-pass approach:
// 1. First pass: Collect function signatures
// 2. Second pass: Check statements and infer types

use crate::ast::{Expr, Stmt, TypeAnnotation};
use crate::errors::{RuffError, ErrorKind, SourceLocation};
use std::collections::HashMap;

/// Represents a function signature with parameter and return types
#[derive(Debug, Clone)]
struct FunctionSignature {
	param_types: Vec<Option<TypeAnnotation>>,
	return_type: Option<TypeAnnotation>,
}

/// Type checker maintains symbol tables for variables and functions
pub struct TypeChecker {
	/// Symbol table mapping variable names to their types
	variables: HashMap<String, Option<TypeAnnotation>>,
	/// Function signatures mapping function names to their types
	functions: HashMap<String, FunctionSignature>,
	/// Stack of scopes for nested blocks
	scope_stack: Vec<HashMap<String, Option<TypeAnnotation>>>,
	/// Current function return type (for checking return statements)
	current_function_return: Option<TypeAnnotation>,
	/// Collect errors instead of failing immediately
	errors: Vec<RuffError>,
}

impl TypeChecker {
	/// Creates a new type checker with empty symbol tables
	pub fn new() -> Self {
		let mut checker = TypeChecker {
			variables: HashMap::new(),
			functions: HashMap::new(),
			scope_stack: Vec::new(),
			current_function_return: None,
			errors: Vec::new(),
		};
		
		// Register built-in functions
		checker.register_builtins();
		
		checker
	}
	
	/// Registers all built-in function signatures
	fn register_builtins(&mut self) {
		// Math constants
		self.variables.insert("PI".to_string(), Some(TypeAnnotation::Float));
		self.variables.insert("E".to_string(), Some(TypeAnnotation::Float));
		
		// Math functions - single arg
		for name in &["abs", "sqrt", "floor", "ceil", "round", "sin", "cos", "tan"] {
			self.functions.insert(
				name.to_string(),
				FunctionSignature {
					param_types: vec![Some(TypeAnnotation::Float)],
					return_type: Some(TypeAnnotation::Float),
				}
			);
		}
		
		// Math functions - two args
		for name in &["pow", "min", "max"] {
			self.functions.insert(
				name.to_string(),
				FunctionSignature {
					param_types: vec![Some(TypeAnnotation::Float), Some(TypeAnnotation::Float)],
					return_type: Some(TypeAnnotation::Float),
				}
			);
		}
		
		// String functions
		self.functions.insert("len".to_string(), FunctionSignature {
			param_types: vec![Some(TypeAnnotation::String)],
			return_type: Some(TypeAnnotation::Int),
		});
		
		self.functions.insert("to_upper".to_string(), FunctionSignature {
			param_types: vec![Some(TypeAnnotation::String)],
			return_type: Some(TypeAnnotation::String),
		});
		
		self.functions.insert("to_lower".to_string(), FunctionSignature {
			param_types: vec![Some(TypeAnnotation::String)],
			return_type: Some(TypeAnnotation::String),
		});
		
		self.functions.insert("trim".to_string(), FunctionSignature {
			param_types: vec![Some(TypeAnnotation::String)],
			return_type: Some(TypeAnnotation::String),
		});
		
		self.functions.insert("contains".to_string(), FunctionSignature {
			param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
			return_type: Some(TypeAnnotation::Bool),
		});
		
		self.functions.insert("substring".to_string(), FunctionSignature {
			param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::Int), Some(TypeAnnotation::Int)],
			return_type: Some(TypeAnnotation::String),
		});
		
		self.functions.insert("replace_str".to_string(), FunctionSignature {
			param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
			return_type: Some(TypeAnnotation::String),
		});
	}

	/// Type check a list of statements
	///
	/// Returns Ok(()) if type checking succeeds, or Err with collected errors
	pub fn check(&mut self, stmts: &[Stmt]) -> Result<(), Vec<RuffError>> {
		// First pass: collect function signatures
		for stmt in stmts {
			if let Stmt::FuncDef { name, param_types, return_type, .. } = stmt {
				self.functions.insert(
					name.clone(),
					FunctionSignature {
						param_types: param_types.clone(),
						return_type: return_type.clone(),
					},
				);
			}
		}

		// Second pass: check statements
		for stmt in stmts {
			self.check_stmt(stmt);
		}

		if self.errors.is_empty() {
			Ok(())
		} else {
			Err(self.errors.clone())
		}
	}

	/// Check a single statement
	fn check_stmt(&mut self, stmt: &Stmt) {
		match stmt {
			Stmt::Let { name, value, type_annotation, .. } => {
				let inferred_type = self.infer_expr(value);
				
				// If type annotation is provided, check compatibility
				if let Some(annotated_type) = type_annotation {
					if let Some(inferred) = &inferred_type {
						if !annotated_type.matches(inferred) {
							self.errors.push(RuffError::new(
								ErrorKind::TypeError,
								format!(
									"Type mismatch: variable '{}' declared as {:?} but assigned {:?}",
									name, annotated_type, inferred
								),
								SourceLocation::unknown(),
							));
						}
					}
					// Store the annotated type
					self.variables.insert(name.clone(), Some(annotated_type.clone()));
				} else {
					// Store the inferred type
					self.variables.insert(name.clone(), inferred_type);
				}
			}

			Stmt::Const { name, value, type_annotation } => {
				let inferred_type = self.infer_expr(value);
				
				// If type annotation is provided, check compatibility
				if let Some(annotated_type) = type_annotation {
					if let Some(inferred) = &inferred_type {
						if !annotated_type.matches(inferred) {
							self.errors.push(RuffError::new(
								ErrorKind::TypeError,
								format!(
									"Type mismatch: constant '{}' declared as {:?} but assigned {:?}",
									name, annotated_type, inferred
								),
								SourceLocation::unknown(),
							));
						}
					}
					// Store the annotated type
					self.variables.insert(name.clone(), Some(annotated_type.clone()));
				} else {
					// Store the inferred type
					self.variables.insert(name.clone(), inferred_type);
				}
			}

			Stmt::FuncDef { name: _, params, param_types, return_type, body } => {
				// Enter function scope
				let saved_return_type = self.current_function_return.clone();
				self.current_function_return = return_type.clone();
				self.push_scope();

				// Add parameters to scope
				for (i, param) in params.iter().enumerate() {
					let param_type = param_types.get(i).and_then(|t| t.clone());
					self.variables.insert(param.clone(), param_type);
				}

				// Check function body
				for stmt in body {
					self.check_stmt(stmt);
				}

				// Exit function scope
				self.pop_scope();
				self.current_function_return = saved_return_type;
			}

			Stmt::Return(expr) => {
				let return_type = expr.as_ref().map(|e| self.infer_expr(e)).flatten();
				
				// Check if return type matches function signature
				if let Some(expected) = &self.current_function_return {
					if let Some(actual) = &return_type {
						if !expected.matches(actual) {
							self.errors.push(RuffError::new(
								ErrorKind::TypeError,
								format!(
									"Return type mismatch: expected {:?} but got {:?}",
									expected, actual
								),
								SourceLocation::unknown(),
							));
						}
					}
				}
			}

			Stmt::If { condition, then_branch, else_branch } => {
				self.infer_expr(condition);
				for s in then_branch {
					self.check_stmt(s);
				}
				if let Some(else_stmts) = else_branch {
					for s in else_stmts {
						self.check_stmt(s);
					}
				}
			}

			Stmt::Loop { condition: _, body } => {
				for s in body {
					self.check_stmt(s);
				}
			}

			Stmt::While { condition, body } => {
				self.infer_expr(condition);
				for s in body {
					self.check_stmt(s);
				}
			}

			Stmt::Break => {
				// No type checking needed for break
			}

			Stmt::Continue => {
				// No type checking needed for continue
			}

			Stmt::For { var, iterable, body } => {
				self.infer_expr(iterable);
				self.push_scope();
				self.variables.insert(var.clone(), None); // Iterator type unknown
				for s in body {
					self.check_stmt(s);
				}
				self.pop_scope();
			}

			Stmt::Match { value, cases, default } => {
				self.infer_expr(value);
				for (_, case_body) in cases {
					for s in case_body {
						self.check_stmt(s);
					}
				}
				if let Some(default_body) = default {
					for s in default_body {
						self.check_stmt(s);
					}
				}
			}

			Stmt::TryExcept { try_block, except_var: _, except_block } => {
				for s in try_block {
					self.check_stmt(s);
				}
				for s in except_block {
					self.check_stmt(s);
				}
			}

			Stmt::ExprStmt(expr) => {
				self.infer_expr(expr);
			}

			Stmt::Assign { target, value } => {
				let inferred_type = self.infer_expr(value);
				
				// Check based on assignment target
				match target {
					Expr::Identifier(name) => {
						// Check if variable exists and types are compatible
						if let Some(var_type) = self.variables.get(name) {
							if let Some(expected) = var_type {
								if let Some(actual) = &inferred_type {
									if !expected.matches(actual) {
										self.errors.push(RuffError::new(
											ErrorKind::TypeError,
											format!(
												"Type mismatch: cannot assign {:?} to variable '{}' of type {:?}",
												actual, name, expected
											),
											SourceLocation::unknown(),
										));
									}
								}
							}
						}
					}
					Expr::IndexAccess { .. } => {
						// Type checking for index assignment would need more sophisticated analysis
						// For now, just type-check the value expression
					}
					_ => {
						// Invalid assignment target - parser should have caught this
					}
				}
			}

			Stmt::Block(stmts) => {
				for s in stmts {
					self.check_stmt(s);
				}
			}

			Stmt::EnumDef { .. } => {
				// Enums don't require type checking
			}

			Stmt::Import { module: _, symbols: _ } => {
				// Module imports don't require type checking
				// TODO: When module system is implemented, verify module exists
			}

			Stmt::Export { stmt } => {
				// Type check the exported statement
				self.check_stmt(stmt);
			}

			Stmt::StructDef { name: _, fields: _, methods } => {
				// Type check methods
				for method in methods {
					self.check_stmt(method);
				}
			}
		}
	}

	/// Infer the type of an expression
	fn infer_expr(&mut self, expr: &Expr) -> Option<TypeAnnotation> {
		match expr {
			Expr::Number(n) => {
				// Check if it's an integer or float
				if n.fract() == 0.0 {
					Some(TypeAnnotation::Int)
				} else {
					Some(TypeAnnotation::Float)
				}
			}

			Expr::String(_) => Some(TypeAnnotation::String),
			
			Expr::Bool(_) => Some(TypeAnnotation::Bool),

			Expr::Identifier(name) => {
				// Look up variable type in symbol table
				self.variables.get(name).cloned().flatten()
			}

			Expr::BinaryOp { op, left, right } => {
				let left_type = self.infer_expr(left);
				let right_type = self.infer_expr(right);

				match op.as_str() {
					"==" | "!=" | "<" | ">" | "<=" | ">=" => {
						// Comparison operations always return bool
						// Check that operands are comparable
						if let (Some(l), Some(r)) = (&left_type, &right_type) {
							if !l.matches(r) && !r.matches(l) {
								self.errors.push(RuffError::new(
									ErrorKind::TypeError,
									format!(
										"Comparison '{}' between incompatible types: {:?} and {:?}",
										op, l, r
									),
									SourceLocation::unknown(),
								));
							}
						}
						Some(TypeAnnotation::Bool)
					}
					"+" | "-" | "*" | "/" => {
						// Arithmetic operations
						match (&left_type, &right_type) {
							(Some(TypeAnnotation::Int), Some(TypeAnnotation::Int)) => {
								Some(TypeAnnotation::Int)
							}
							(Some(TypeAnnotation::Float), _) | (_, Some(TypeAnnotation::Float)) => {
								Some(TypeAnnotation::Float)
							}
							(Some(TypeAnnotation::String), Some(TypeAnnotation::String)) if op == "+" => {
								Some(TypeAnnotation::String)
							}
							(Some(l), Some(r)) if l != r => {
								self.errors.push(RuffError::new(
									ErrorKind::TypeError,
									format!(
										"Binary operation '{}' with incompatible types: {:?} and {:?}",
										op, l, r
									),
									SourceLocation::unknown(),
								));
								None
							}
							_ => None, // Unknown or incompatible types
						}
					}
					_ => None,
				}
			}

			Expr::Call { function, args } => {
				// Look up function signature
				if let Expr::Identifier(func_name) = &**function {
					// Clone the signature to avoid borrow conflicts
					let sig = self.functions.get(func_name).cloned();
					
					if let Some(sig) = sig {
						// Check argument count
						if args.len() != sig.param_types.len() {
							self.errors.push(RuffError::new(
								ErrorKind::TypeError,
								format!(
									"Function '{}' expects {} arguments but got {}",
									func_name, sig.param_types.len(), args.len()
								),
								SourceLocation::unknown(),
							));
						}
						
						// Check argument types
						for (i, arg) in args.iter().enumerate() {
							let arg_type = self.infer_expr(arg);
							if let Some(expected) = sig.param_types.get(i).and_then(|t| t.as_ref()) {
								if let Some(actual) = &arg_type {
									if !expected.matches(actual) {
										self.errors.push(RuffError::new(
											ErrorKind::TypeError,
											format!(
												"Function '{}' parameter {} expects {:?} but got {:?}",
												func_name, i + 1, expected, actual
											),
											SourceLocation::unknown(),
										));
									}
								}
							}
						}
						
						// Return the function's return type
						return sig.return_type.clone();
					} else {
						// Function not found
						self.errors.push(RuffError::new(
							ErrorKind::UndefinedFunction,
							format!("Undefined function '{}'", func_name),
							SourceLocation::unknown(),
						));
					}
				}
				None
			}

			Expr::Tag(_, _) => None, // Enum types not yet supported

			Expr::StructInstance { name: _, fields } => {
				// Type check struct field initializers
				for (_field_name, field_expr) in fields {
					self.infer_expr(field_expr);
				}
				None // TODO: Return struct type when struct types are implemented
			}

			Expr::FieldAccess { object, field: _ } => {
				// Type check the object expression
				self.infer_expr(object);
				None // TODO: Look up field type from struct definition
			}

			Expr::ArrayLiteral(elements) => {
				// Type check all elements
				for elem in elements {
					self.infer_expr(elem);
				}
				None // TODO: Return Array<T> type when generic types are implemented
			}

			Expr::DictLiteral(pairs) => {
				// Type check all keys and values
				for (key, value) in pairs {
					self.infer_expr(key);
					self.infer_expr(value);
				}
				None // TODO: Return Dict<K, V> type when generic types are implemented
			}

			Expr::IndexAccess { object, index } => {
				// Type check object and index
				self.infer_expr(object);
				self.infer_expr(index);
				None // TODO: Return element type based on container type
			}
		}
	}

	/// Push a new scope onto the scope stack
	fn push_scope(&mut self) {
		self.scope_stack.push(self.variables.clone());
	}

	/// Pop a scope from the scope stack
	fn pop_scope(&mut self) {
		if let Some(prev_scope) = self.scope_stack.pop() {
			self.variables = prev_scope;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_simple_type_inference() {
		let mut checker = TypeChecker::new();
		let stmts = vec![
			Stmt::Let {
				name: "x".to_string(),
				value: Expr::Number(42.0),
				mutable: false,
				type_annotation: Some(TypeAnnotation::Int),
			},
		];
		
		assert!(checker.check(&stmts).is_ok());
		assert_eq!(
			checker.variables.get("x"),
			Some(&Some(TypeAnnotation::Int))
		);
	}

	#[test]
	fn test_type_mismatch() {
		let mut checker = TypeChecker::new();
		let stmts = vec![
			Stmt::Let {
				name: "x".to_string(),
				value: Expr::String("hello".to_string()),
				mutable: false,
				type_annotation: Some(TypeAnnotation::Int),
			},
		];
		
		assert!(checker.check(&stmts).is_err());
	}
}
