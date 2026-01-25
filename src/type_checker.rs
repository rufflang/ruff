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
use crate::errors::{ErrorKind, RuffError, SourceLocation};
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
    /// Recursion depth counter to prevent infinite loops
    recursion_depth: usize,
}

/// Maximum recursion depth for type checking to prevent infinite loops
const MAX_RECURSION_DEPTH: usize = 1000;

impl TypeChecker {
    /// Creates a new type checker with empty symbol tables
    pub fn new() -> Self {
        let mut checker = TypeChecker {
            variables: HashMap::new(),
            functions: HashMap::new(),
            scope_stack: Vec::new(),
            current_function_return: None,
            errors: Vec::new(),
            recursion_depth: 0,
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
                },
            );
        }

        // Math functions - two args
        for name in &["pow", "min", "max"] {
            self.functions.insert(
                name.to_string(),
                FunctionSignature {
                    param_types: vec![Some(TypeAnnotation::Float), Some(TypeAnnotation::Float)],
                    return_type: Some(TypeAnnotation::Float),
                },
            );
        }

        // String functions
        self.functions.insert(
            "len".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::Int),
            },
        );

        self.functions.insert(
            "to_upper".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "to_lower".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "trim".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "contains".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::Bool),
            },
        );

        self.functions.insert(
            "substring".to_string(),
            FunctionSignature {
                param_types: vec![
                    Some(TypeAnnotation::String),
                    Some(TypeAnnotation::Int),
                    Some(TypeAnnotation::Int),
                ],
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "replace_str".to_string(),
            FunctionSignature {
                param_types: vec![
                    Some(TypeAnnotation::String),
                    Some(TypeAnnotation::String),
                    Some(TypeAnnotation::String),
                ],
                return_type: Some(TypeAnnotation::String),
            },
        );

        // New string functions
        self.functions.insert(
            "starts_with".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::Bool),
            },
        );

        self.functions.insert(
            "ends_with".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::Bool),
            },
        );

        self.functions.insert(
            "index_of".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::Int),
            },
        );

        self.functions.insert(
            "repeat".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::Float)],
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "split".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
                return_type: None, // Returns array, but we don't have array type annotation yet
            },
        );

        self.functions.insert(
            "join".to_string(),
            FunctionSignature {
                param_types: vec![None, Some(TypeAnnotation::String)], // First param is array
                return_type: Some(TypeAnnotation::String),
            },
        );

        // Array higher-order functions
        self.functions.insert(
            "map".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Array and function
                return_type: None,             // Returns array
            },
        );

        self.functions.insert(
            "filter".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Array and function
                return_type: None,             // Returns array
            },
        );

        self.functions.insert(
            "reduce".to_string(),
            FunctionSignature {
                param_types: vec![None, None, None], // Array, initial value, and function
                return_type: None,                   // Returns value of initial type
            },
        );

        self.functions.insert(
            "find".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Array and function
                return_type: None,             // Returns element or 0
            },
        );

        // JSON functions
        self.functions.insert(
            "parse_json".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: None, // Returns any type (dict, array, etc.)
            },
        );

        self.functions.insert(
            "to_json".to_string(),
            FunctionSignature {
                param_types: vec![None], // Accepts any value
                return_type: Some(TypeAnnotation::String),
            },
        );

        // TOML functions
        self.functions.insert(
            "parse_toml".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: None, // Returns any type (dict, array, etc.)
            },
        );

        self.functions.insert(
            "to_toml".to_string(),
            FunctionSignature {
                param_types: vec![None], // Accepts any value
                return_type: Some(TypeAnnotation::String),
            },
        );

        // YAML functions
        self.functions.insert(
            "parse_yaml".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: None, // Returns any type (dict, array, etc.)
            },
        );

        self.functions.insert(
            "to_yaml".to_string(),
            FunctionSignature {
                param_types: vec![None], // Accepts any value
                return_type: Some(TypeAnnotation::String),
            },
        );

        // CSV functions
        self.functions.insert(
            "parse_csv".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: None, // Returns array of dicts
            },
        );

        self.functions.insert(
            "to_csv".to_string(),
            FunctionSignature {
                param_types: vec![None], // Accepts array of dicts
                return_type: Some(TypeAnnotation::String),
            },
        );

        // Type conversion functions
        self.functions.insert(
            "to_int".to_string(),
            FunctionSignature {
                param_types: vec![None], // Accepts any type
                return_type: Some(TypeAnnotation::Int),
            },
        );

        self.functions.insert(
            "to_float".to_string(),
            FunctionSignature {
                param_types: vec![None], // Accepts any type
                return_type: Some(TypeAnnotation::Float),
            },
        );

        self.functions.insert(
            "to_string".to_string(),
            FunctionSignature {
                param_types: vec![None], // Accepts any type
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "to_bool".to_string(),
            FunctionSignature {
                param_types: vec![None], // Accepts any type
                return_type: Some(TypeAnnotation::Bool),
            },
        );

        // Random functions
        self.functions.insert(
            "random".to_string(),
            FunctionSignature { param_types: vec![], return_type: Some(TypeAnnotation::Float) },
        );

        self.functions.insert(
            "random_int".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::Float), Some(TypeAnnotation::Float)],
                return_type: Some(TypeAnnotation::Float),
            },
        );

        self.functions.insert(
            "random_choice".to_string(),
            FunctionSignature {
                param_types: vec![None], // Array
                return_type: None,       // Returns element from array
            },
        );

        // Date/Time functions
        self.functions.insert(
            "now".to_string(),
            FunctionSignature { param_types: vec![], return_type: Some(TypeAnnotation::Float) },
        );

        self.functions.insert(
            "current_timestamp".to_string(),
            FunctionSignature { param_types: vec![], return_type: Some(TypeAnnotation::Float) },
        );

        self.functions.insert(
            "performance_now".to_string(),
            FunctionSignature { param_types: vec![], return_type: Some(TypeAnnotation::Float) },
        );

        self.functions.insert(
            "time_us".to_string(),
            FunctionSignature { param_types: vec![], return_type: Some(TypeAnnotation::Float) },
        );

        self.functions.insert(
            "time_ns".to_string(),
            FunctionSignature { param_types: vec![], return_type: Some(TypeAnnotation::Float) },
        );

        self.functions.insert(
            "format_duration".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::Float)],
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "elapsed".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::Float), Some(TypeAnnotation::Float)],
                return_type: Some(TypeAnnotation::Float),
            },
        );

        self.functions.insert(
            "format_date".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::Float), Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "parse_date".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::Float),
            },
        );

        // System operation functions
        self.functions.insert(
            "env".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "args".to_string(),
            FunctionSignature {
                param_types: vec![],
                return_type: None, // Returns array of strings
            },
        );

        self.functions.insert(
            "exit".to_string(),
            FunctionSignature { param_types: vec![Some(TypeAnnotation::Float)], return_type: None },
        );

        self.functions.insert(
            "sleep".to_string(),
            FunctionSignature { param_types: vec![Some(TypeAnnotation::Float)], return_type: None },
        );

        self.functions.insert(
            "execute".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::String),
            },
        );

        // Path operation functions
        self.functions.insert(
            "join_path".to_string(),
            FunctionSignature {
                param_types: vec![None], // Variadic string arguments
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "dirname".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "basename".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "path_exists".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::Bool),
            },
        );

        // Regular expression functions
        self.functions.insert(
            "regex_match".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
                return_type: Some(TypeAnnotation::Bool),
            },
        );

        self.functions.insert(
            "regex_find_all".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
                return_type: None, // Returns array of strings
            },
        );

        self.functions.insert(
            "regex_replace".to_string(),
            FunctionSignature {
                param_types: vec![
                    Some(TypeAnnotation::String),
                    Some(TypeAnnotation::String),
                    Some(TypeAnnotation::String),
                ],
                return_type: Some(TypeAnnotation::String),
            },
        );

        self.functions.insert(
            "regex_split".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
                return_type: None, // Returns array of strings
            },
        );

        // HTTP client functions
        self.functions.insert(
            "http_get".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: None, // Returns Result<dict, string>
            },
        );

        self.functions.insert(
            "http_post".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
                return_type: None, // Returns Result<dict, string>
            },
        );

        self.functions.insert(
            "http_put".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)],
                return_type: None, // Returns Result<dict, string>
            },
        );

        self.functions.insert(
            "http_delete".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)],
                return_type: None, // Returns Result<dict, string>
            },
        );

        // HTTP server functions
        self.functions.insert(
            "http_server".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::Int)],
                return_type: None, // Returns HttpServer object
            },
        );

        self.functions.insert(
            "http_response".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::Int), Some(TypeAnnotation::String)],
                return_type: None, // Returns HttpResponse object
            },
        );

        self.functions.insert(
            "json_response".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::Int), None], // Status code and any data
                return_type: None,                                  // Returns HttpResponse object
            },
        );

        self.functions.insert(
            "redirect_response".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), None], // URL to redirect to, optional headers dict
                return_type: None, // Returns HttpResponse object
            },
        );

        self.functions.insert(
            "set_header".to_string(),
            FunctionSignature {
                param_types: vec![None, Some(TypeAnnotation::String), Some(TypeAnnotation::String)], // Response, key, value
                return_type: None, // Returns HttpResponse object
            },
        );

        self.functions.insert(
            "set_headers".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Response, headers dict
                return_type: None,             // Returns HttpResponse object
            },
        );

        // Database functions
        self.functions.insert(
            "db_connect".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)], // db_type, connection_string
                return_type: None, // Returns Database object
            },
        );

        self.functions.insert(
            "db_execute".to_string(),
            FunctionSignature {
                param_types: vec![None, Some(TypeAnnotation::String), None], // db, sql, params (optional array)
                return_type: None, // Returns number (rows affected) or Error
            },
        );

        self.functions.insert(
            "db_query".to_string(),
            FunctionSignature {
                param_types: vec![None, Some(TypeAnnotation::String), None], // db, sql, params (optional array)
                return_type: None, // Returns array of dicts
            },
        );

        self.functions.insert(
            "db_close".to_string(),
            FunctionSignature {
                param_types: vec![None], // Database connection
                return_type: Some(TypeAnnotation::Bool),
            },
        );

        self.functions.insert(
            "db_pool".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String), None], // db_type, connection_string, options
                return_type: None, // Returns DatabasePool object
            },
        );

        self.functions.insert(
            "db_begin".to_string(),
            FunctionSignature {
                param_types: vec![None], // Database connection
                return_type: None,
            },
        );

        self.functions.insert(
            "db_commit".to_string(),
            FunctionSignature {
                param_types: vec![None], // Database connection
                return_type: None,
            },
        );

        self.functions.insert(
            "db_rollback".to_string(),
            FunctionSignature {
                param_types: vec![None], // Database connection
                return_type: None,
            },
        );

        // File I/O functions
        self.functions.insert(
            "create_dir".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)], // Directory path
                return_type: Some(TypeAnnotation::Bool),
            },
        );

        self.functions.insert(
            "write_binary_file".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), None], // Path and bytes
                return_type: Some(TypeAnnotation::Bool),
            },
        );

        // HTTP streaming functions
        self.functions.insert(
            "http_get_stream".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)], // URL
                return_type: None,                               // Returns bytes array
            },
        );

        // v0.6.0 Authentication & Streaming functions
        self.functions.insert(
            "jwt_encode".to_string(),
            FunctionSignature {
                param_types: vec![None, Some(TypeAnnotation::String)], // Payload dict and secret
                return_type: Some(TypeAnnotation::String),             // Returns JWT token string
            },
        );

        self.functions.insert(
            "jwt_decode".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String), Some(TypeAnnotation::String)], // Token and secret
                return_type: None, // Returns dict or error
            },
        );

        self.functions.insert(
            "oauth2_auth_url".to_string(),
            FunctionSignature {
                param_types: vec![
                    Some(TypeAnnotation::String), // client_id
                    Some(TypeAnnotation::String), // redirect_uri
                    Some(TypeAnnotation::String), // auth_url
                    Some(TypeAnnotation::String), // scope
                ],
                return_type: Some(TypeAnnotation::String), // Returns authorization URL
            },
        );

        self.functions.insert(
            "oauth2_get_token".to_string(),
            FunctionSignature {
                param_types: vec![
                    Some(TypeAnnotation::String), // code
                    Some(TypeAnnotation::String), // client_id
                    Some(TypeAnnotation::String), // client_secret
                    Some(TypeAnnotation::String), // token_url
                    Some(TypeAnnotation::String), // redirect_uri
                ],
                return_type: None, // Returns dict with token data
            },
        );

        self.functions.insert(
            "html_response".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::Int), Some(TypeAnnotation::String)], // Status code and HTML
                return_type: None, // Returns HttpResponse object
            },
        );

        // Collection constructors and methods
        // Set operations
        self.functions.insert(
            "Set".to_string(),
            FunctionSignature {
                param_types: vec![None], // Array
                return_type: None,       // Returns Set
            },
        );
        self.functions.insert(
            "set_add".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Set and item
                return_type: None,             // Returns modified Set
            },
        );
        self.functions.insert(
            "set_has".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Set and item
                return_type: Some(TypeAnnotation::Bool),
            },
        );
        self.functions.insert(
            "set_remove".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Set and item
                return_type: None,             // Returns modified Set
            },
        );
        self.functions.insert(
            "set_union".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Two Sets
                return_type: None,             // Returns new Set
            },
        );
        self.functions.insert(
            "set_intersect".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Two Sets
                return_type: None,             // Returns new Set
            },
        );
        self.functions.insert(
            "set_difference".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Two Sets
                return_type: None,             // Returns new Set
            },
        );
        self.functions.insert(
            "set_to_array".to_string(),
            FunctionSignature {
                param_types: vec![None], // Set
                return_type: None,       // Returns Array
            },
        );

        // Queue operations
        self.functions.insert(
            "Queue".to_string(),
            FunctionSignature {
                param_types: vec![None], // Optional array
                return_type: None,       // Returns Queue
            },
        );
        self.functions.insert(
            "queue_enqueue".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Queue and item
                return_type: None,             // Returns modified Queue
            },
        );
        self.functions.insert(
            "queue_dequeue".to_string(),
            FunctionSignature {
                param_types: vec![None], // Queue
                return_type: None,       // Returns [modified Queue, item]
            },
        );
        self.functions.insert(
            "queue_peek".to_string(),
            FunctionSignature {
                param_types: vec![None], // Queue
                return_type: None,       // Returns item or null
            },
        );
        self.functions.insert(
            "queue_is_empty".to_string(),
            FunctionSignature {
                param_types: vec![None], // Queue
                return_type: Some(TypeAnnotation::Bool),
            },
        );
        self.functions.insert(
            "queue_to_array".to_string(),
            FunctionSignature {
                param_types: vec![None], // Queue
                return_type: None,       // Returns Array
            },
        );

        // Stack operations
        self.functions.insert(
            "Stack".to_string(),
            FunctionSignature {
                param_types: vec![None], // Optional array
                return_type: None,       // Returns Stack
            },
        );
        self.functions.insert(
            "stack_push".to_string(),
            FunctionSignature {
                param_types: vec![None, None], // Stack and item
                return_type: None,             // Returns modified Stack
            },
        );
        self.functions.insert(
            "stack_pop".to_string(),
            FunctionSignature {
                param_types: vec![None], // Stack
                return_type: None,       // Returns [modified Stack, item]
            },
        );
        self.functions.insert(
            "stack_peek".to_string(),
            FunctionSignature {
                param_types: vec![None], // Stack
                return_type: None,       // Returns item or null
            },
        );
        self.functions.insert(
            "stack_is_empty".to_string(),
            FunctionSignature {
                param_types: vec![None], // Stack
                return_type: Some(TypeAnnotation::Bool),
            },
        );
        self.functions.insert(
            "stack_to_array".to_string(),
            FunctionSignature {
                param_types: vec![None], // Stack
                return_type: None,       // Returns Array
            },
        );

        // Image processing functions
        self.functions.insert(
            "load_image".to_string(),
            FunctionSignature {
                param_types: vec![Some(TypeAnnotation::String)], // Image path
                return_type: None,                               // Returns Image object
            },
        );
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
        // Check for excessive recursion depth
        if self.recursion_depth >= MAX_RECURSION_DEPTH {
            self.errors.push(RuffError::new(
                ErrorKind::TypeError,
                format!("Type checker recursion depth exceeded (max: {}). Possible infinite loop in type checking.", MAX_RECURSION_DEPTH),
                SourceLocation::unknown(),
            ));
            return;
        }

        self.recursion_depth += 1;

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

            Stmt::Spawn { body } => {
                // Check the spawn body in a new scope
                self.push_scope();
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

        self.recursion_depth -= 1;
    }

    /// Infer the type of an expression
    fn infer_expr(&mut self, expr: &Expr) -> Option<TypeAnnotation> {
        // Check for excessive recursion depth
        if self.recursion_depth >= MAX_RECURSION_DEPTH {
            self.errors.push(RuffError::new(
                ErrorKind::TypeError,
                format!("Type checker recursion depth exceeded (max: {}). Possible infinite loop in type inference.", MAX_RECURSION_DEPTH),
                SourceLocation::unknown(),
            ));
            return None;
        }

        self.recursion_depth += 1;

        let result = match expr {
            Expr::Int(_) => Some(TypeAnnotation::Int),
            Expr::Float(_) => Some(TypeAnnotation::Float),

            Expr::String(_) => Some(TypeAnnotation::String),

            Expr::InterpolatedString(parts) => {
                // Type check all embedded expressions
                use crate::ast::InterpolatedStringPart;
                for part in parts {
                    if let InterpolatedStringPart::Expr(expr) = part {
                        self.infer_expr(expr);
                    }
                }
                // Interpolated strings always produce strings
                Some(TypeAnnotation::String)
            }

            Expr::Bool(_) => Some(TypeAnnotation::Bool),

            Expr::Identifier(name) => {
                // Look up variable type in symbol table
                self.variables.get(name).cloned().flatten()
            }

            Expr::UnaryOp { op, operand } => {
                let operand_type = self.infer_expr(operand);

                match op.as_str() {
                    "-" => {
                        // Unary minus on numbers
                        match operand_type {
                            Some(TypeAnnotation::Int) => Some(TypeAnnotation::Int),
                            Some(TypeAnnotation::Float) => Some(TypeAnnotation::Float),
                            _ => operand_type, // Could be struct with op_neg
                        }
                    }
                    "!" => {
                        // Logical not on booleans
                        match operand_type {
                            Some(TypeAnnotation::Bool) => Some(TypeAnnotation::Bool),
                            _ => operand_type, // Could be struct with op_not
                        }
                    }
                    _ => None,
                }
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
                        // Arithmetic operations with type promotion
                        match (&left_type, &right_type) {
                            // Int op Int => Int
                            (Some(TypeAnnotation::Int), Some(TypeAnnotation::Int)) => {
                                Some(TypeAnnotation::Int)
                            }
                            // Int op Float or Float op Int => Float (type promotion)
                            (Some(TypeAnnotation::Int), Some(TypeAnnotation::Float))
                            | (Some(TypeAnnotation::Float), Some(TypeAnnotation::Int))
                            | (Some(TypeAnnotation::Float), Some(TypeAnnotation::Float)) => {
                                Some(TypeAnnotation::Float)
                            }
                            // String + String => String
                            (Some(TypeAnnotation::String), Some(TypeAnnotation::String))
                                if op == "+" =>
                            {
                                Some(TypeAnnotation::String)
                            }
                            // Incompatible types
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
                            _ => None, // Unknown types
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
                        // Check argument count - allow fewer args than params if trailing params are optional (None)
                        let min_required =
                            sig.param_types.iter().take_while(|p| p.is_some()).count();
                        let max_allowed = sig.param_types.len();

                        if args.len() < min_required || args.len() > max_allowed {
                            self.errors.push(RuffError::new(
                                ErrorKind::TypeError,
                                format!(
                                    "Function '{}' expects {}-{} arguments but got {}",
                                    func_name,
                                    min_required,
                                    max_allowed,
                                    args.len()
                                ),
                                SourceLocation::unknown(),
                            ));
                        }

                        // Check argument types
                        for (i, arg) in args.iter().enumerate() {
                            let arg_type = self.infer_expr(arg);
                            if let Some(expected) = sig.param_types.get(i).and_then(|t| t.as_ref())
                            {
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

            Expr::Function { params: _, param_types, return_type, body } => {
                // Type check function expression (anonymous function)
                // Enter function scope
                self.push_scope();

                // Add parameters to scope
                for param_type in param_types {
                    if let Some(t) = param_type {
                        // We would need the param name here, but it's not available in this context
                        // For now, just validate the function body
                        let _ = t;
                    }
                }

                // Check function body
                for stmt in body {
                    self.check_stmt(stmt);
                }

                // Exit function scope
                self.pop_scope();

                // Return function type annotation if available
                // For now, just return None since we don't have full function types yet
                let _ = return_type;
                None
            }
        };

        self.recursion_depth -= 1;
        result
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
        let stmts = vec![Stmt::Let {
            name: "x".to_string(),
            value: Expr::Int(42),
            mutable: false,
            type_annotation: Some(TypeAnnotation::Int),
        }];

        assert!(checker.check(&stmts).is_ok());
        assert_eq!(checker.variables.get("x"), Some(&Some(TypeAnnotation::Int)));
    }

    #[test]
    fn test_type_mismatch() {
        let mut checker = TypeChecker::new();
        let stmts = vec![Stmt::Let {
            name: "x".to_string(),
            value: Expr::String("hello".to_string()),
            mutable: false,
            type_annotation: Some(TypeAnnotation::Int),
        }];

        assert!(checker.check(&stmts).is_err());
    }

    #[test]
    fn test_int_literal_inference() {
        let mut checker = TypeChecker::new();
        let stmts = vec![Stmt::Let {
            name: "count".to_string(),
            value: Expr::Int(42),
            mutable: false,
            type_annotation: None,
        }];

        assert!(checker.check(&stmts).is_ok());
        // Variable should be inferred as Int
        assert_eq!(checker.variables.get("count"), Some(&Some(TypeAnnotation::Int)));
    }

    #[test]
    fn test_float_literal_inference() {
        let mut checker = TypeChecker::new();
        let stmts = vec![Stmt::Let {
            name: "pi".to_string(),
            value: Expr::Float(3.14),
            mutable: false,
            type_annotation: None,
        }];

        assert!(checker.check(&stmts).is_ok());
        // Variable should be inferred as Float
        assert_eq!(checker.variables.get("pi"), Some(&Some(TypeAnnotation::Float)));
    }

    #[test]
    fn test_int_plus_int_equals_int() {
        let mut checker = TypeChecker::new();
        let result = checker.infer_expr(&Expr::BinaryOp {
            op: "+".to_string(),
            left: Box::new(Expr::Int(5)),
            right: Box::new(Expr::Int(10)),
        });

        assert_eq!(result, Some(TypeAnnotation::Int));
    }

    #[test]
    fn test_int_plus_float_equals_float() {
        let mut checker = TypeChecker::new();
        let result = checker.infer_expr(&Expr::BinaryOp {
            op: "+".to_string(),
            left: Box::new(Expr::Int(5)),
            right: Box::new(Expr::Float(10.5)),
        });

        assert_eq!(result, Some(TypeAnnotation::Float));
    }

    #[test]
    fn test_float_plus_int_equals_float() {
        let mut checker = TypeChecker::new();
        let result = checker.infer_expr(&Expr::BinaryOp {
            op: "+".to_string(),
            left: Box::new(Expr::Float(5.5)),
            right: Box::new(Expr::Int(10)),
        });

        assert_eq!(result, Some(TypeAnnotation::Float));
    }

    #[test]
    fn test_float_plus_float_equals_float() {
        let mut checker = TypeChecker::new();
        let result = checker.infer_expr(&Expr::BinaryOp {
            op: "+".to_string(),
            left: Box::new(Expr::Float(5.5)),
            right: Box::new(Expr::Float(10.5)),
        });

        assert_eq!(result, Some(TypeAnnotation::Float));
    }

    #[test]
    fn test_int_to_float_promotion_in_assignment() {
        let mut checker = TypeChecker::new();
        let stmts = vec![Stmt::Let {
            name: "x".to_string(),
            value: Expr::Int(42),
            mutable: false,
            type_annotation: Some(TypeAnnotation::Float),
        }];

        // Should succeed due to Int → Float promotion
        assert!(checker.check(&stmts).is_ok());
    }

    #[test]
    fn test_float_to_int_no_promotion() {
        let mut checker = TypeChecker::new();
        let stmts = vec![Stmt::Let {
            name: "x".to_string(),
            value: Expr::Float(42.5),
            mutable: false,
            type_annotation: Some(TypeAnnotation::Int),
        }];

        // Should fail - Float cannot be assigned to Int without explicit conversion
        assert!(checker.check(&stmts).is_err());
    }

    #[test]
    fn test_math_function_accepts_int_via_promotion() {
        let mut checker = TypeChecker::new();

        // abs(5) should be accepted (Int promoted to Float)
        let result = checker.infer_expr(&Expr::Call {
            function: Box::new(Expr::Identifier("abs".to_string())),
            args: vec![Expr::Int(5)],
        });

        // Function should accept Int via promotion and return Float
        assert!(checker.errors.is_empty());
        assert_eq!(result, Some(TypeAnnotation::Float));
    }

    #[test]
    fn test_min_with_ints() {
        let mut checker = TypeChecker::new();

        // min(5, 10) should be accepted
        let result = checker.infer_expr(&Expr::Call {
            function: Box::new(Expr::Identifier("min".to_string())),
            args: vec![Expr::Int(5), Expr::Int(10)],
        });

        // Should accept via promotion
        assert!(checker.errors.is_empty());
        assert_eq!(result, Some(TypeAnnotation::Float));
    }

    #[test]
    fn test_min_with_mixed_types() {
        let mut checker = TypeChecker::new();

        // min(5, 10.5) should be accepted
        let result = checker.infer_expr(&Expr::Call {
            function: Box::new(Expr::Identifier("min".to_string())),
            args: vec![Expr::Int(5), Expr::Float(10.5)],
        });

        // Should accept via promotion
        assert!(checker.errors.is_empty());
        assert_eq!(result, Some(TypeAnnotation::Float));
    }

    #[test]
    fn test_arithmetic_type_promotion_all_operators() {
        let mut checker = TypeChecker::new();

        for op in &["+", "-", "*", "/"] {
            let result = checker.infer_expr(&Expr::BinaryOp {
                op: op.to_string(),
                left: Box::new(Expr::Int(10)),
                right: Box::new(Expr::Float(5.0)),
            });

            assert_eq!(
                result,
                Some(TypeAnnotation::Float),
                "Operator {} should promote Int+Float to Float",
                op
            );
        }
    }
}
