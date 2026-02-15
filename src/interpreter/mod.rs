// File: src/interpreter/mod.rs
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

// Module structure
mod async_runtime;
mod control_flow;
mod environment;
mod native_functions;
mod test_runner;
mod value;

// Re-exports for backward compatibility
pub use async_runtime::AsyncRuntime;
pub use environment::Environment;
// Test framework exports - used by CLI test command
#[allow(unused_imports)]
pub use test_runner::{TestCase, TestReport, TestResult, TestRunner};
// Database infrastructure - used by stub database.rs module
#[allow(unused_imports)]
pub use value::{
    ConnectionPool, DatabaseConnection, DenseIntDict, DenseIntDictInt, DenseIntDictIntFull,
    DictMap, IntDictMap, LeakyFunctionBody, Value,
};

// Internal-only imports
use control_flow::ControlFlow;

use crate::ast::{Expr, Stmt};
use crate::builtins;
use crate::errors::RuffError;
use crate::module::ModuleLoader;

// Infrastructure imports for stub modules (crypto.rs, database.rs, network.rs)
// These will be used when stub modules are fully implemented
#[allow(unused_imports)]
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
#[allow(unused_imports)]
use md5::Md5;
#[allow(unused_imports)]
use mysql_async::{prelude::*, Conn as MysqlConn, Opts as MysqlOpts};
#[allow(unused_imports)]
use postgres::{Client as PostgresClient, NoTls};
#[allow(unused_imports)]
use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    Oaep, RsaPrivateKey, RsaPublicKey,
};
#[allow(unused_imports)]
use rusqlite::Connection as SqliteConnection;
#[allow(unused_imports)]
use sha2::Sha256 as RsaSha256;
#[allow(unused_imports)]
use sha2::{Digest as Sha2Digest, Sha256};
#[allow(unused_imports)]
use std::collections::{HashMap, VecDeque};
#[allow(unused_imports)]
use std::fs::File;
#[allow(unused_imports)]
use std::io::Read;
use std::io::Write;
#[allow(unused_imports)]
use std::path::Path;
use std::sync::{Arc, Mutex};
#[allow(unused_imports)]
use zip::{write::FileOptions, ZipArchive, ZipWriter};

pub const DEFAULT_ASYNC_TASK_POOL_SIZE: usize = 256;

#[derive(Clone, Debug)]
enum SpawnCapturedValue {
    Tagged { tag: String, fields: Vec<(String, SpawnCapturedValue)> },
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
    Bytes(Vec<u8>),
    NativeFunction(String),
    Struct { name: String, fields: Vec<(String, SpawnCapturedValue)> },
    Array(Vec<SpawnCapturedValue>),
    Dict(Vec<(String, SpawnCapturedValue)>),
    FixedDict(Vec<(String, SpawnCapturedValue)>),
    IntDict(Vec<(i64, SpawnCapturedValue)>),
    DenseIntDict(Vec<SpawnCapturedValue>),
    DenseIntDictInt(Vec<Option<i64>>),
    DenseIntDictIntFull(Vec<i64>),
    Result { is_ok: bool, value: Box<SpawnCapturedValue> },
    Option { is_some: bool, value: Box<SpawnCapturedValue> },
}

impl SpawnCapturedValue {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Tagged { tag, fields } => {
                let mut captured_fields = Vec::with_capacity(fields.len());
                for (field_name, field_value) in fields {
                    let captured_value = Self::from_value(field_value)?;
                    captured_fields.push((field_name.clone(), captured_value));
                }
                Some(SpawnCapturedValue::Tagged { tag: tag.clone(), fields: captured_fields })
            }
            Value::Int(number) => Some(SpawnCapturedValue::Int(*number)),
            Value::Float(number) => Some(SpawnCapturedValue::Float(*number)),
            Value::Str(text) => Some(SpawnCapturedValue::Str(text.as_ref().clone())),
            Value::Bool(boolean) => Some(SpawnCapturedValue::Bool(*boolean)),
            Value::Null => Some(SpawnCapturedValue::Null),
            Value::Bytes(bytes) => Some(SpawnCapturedValue::Bytes(bytes.clone())),
            Value::NativeFunction(name) => Some(SpawnCapturedValue::NativeFunction(name.clone())),
            Value::Struct { name, fields } => {
                let mut captured_fields = Vec::with_capacity(fields.len());
                for (field_name, field_value) in fields {
                    let captured_value = Self::from_value(field_value)?;
                    captured_fields.push((field_name.clone(), captured_value));
                }
                Some(SpawnCapturedValue::Struct { name: name.clone(), fields: captured_fields })
            }
            Value::Array(elements) => {
                let mut captured_elements = Vec::with_capacity(elements.len());
                for element in elements.iter() {
                    captured_elements.push(Self::from_value(element)?);
                }
                Some(SpawnCapturedValue::Array(captured_elements))
            }
            Value::Dict(entries) => {
                let mut captured_entries = Vec::with_capacity(entries.len());
                for (key, dict_value) in entries.iter() {
                    captured_entries.push((key.to_string(), Self::from_value(dict_value)?));
                }
                Some(SpawnCapturedValue::Dict(captured_entries))
            }
            Value::FixedDict { keys, values } => {
                if keys.len() != values.len() {
                    return None;
                }

                let mut captured_entries = Vec::with_capacity(keys.len());
                for (key, dict_value) in keys.iter().zip(values.iter()) {
                    captured_entries.push((key.to_string(), Self::from_value(dict_value)?));
                }
                Some(SpawnCapturedValue::FixedDict(captured_entries))
            }
            Value::IntDict(entries) => {
                let mut captured_entries = Vec::with_capacity(entries.len());
                for (key, dict_value) in entries.iter() {
                    captured_entries.push((*key, Self::from_value(dict_value)?));
                }
                Some(SpawnCapturedValue::IntDict(captured_entries))
            }
            Value::DenseIntDict(values) => {
                let mut captured_values = Vec::with_capacity(values.len());
                for dict_value in values.iter() {
                    captured_values.push(Self::from_value(dict_value)?);
                }
                Some(SpawnCapturedValue::DenseIntDict(captured_values))
            }
            Value::DenseIntDictInt(values) => {
                Some(SpawnCapturedValue::DenseIntDictInt(values.as_ref().clone()))
            }
            Value::DenseIntDictIntFull(values) => {
                Some(SpawnCapturedValue::DenseIntDictIntFull(values.as_ref().clone()))
            }
            Value::Result { is_ok, value } => Some(SpawnCapturedValue::Result {
                is_ok: *is_ok,
                value: Box::new(Self::from_value(value)?),
            }),
            Value::Option { is_some, value } => Some(SpawnCapturedValue::Option {
                is_some: *is_some,
                value: Box::new(Self::from_value(value)?),
            }),
            _ => None,
        }
    }

    fn into_value(self) -> Value {
        match self {
            SpawnCapturedValue::Tagged { tag, fields } => {
                let mut value_fields = HashMap::with_capacity(fields.len());
                for (field_name, field_value) in fields {
                    value_fields.insert(field_name, field_value.into_value());
                }
                Value::Tagged { tag, fields: value_fields }
            }
            SpawnCapturedValue::Int(number) => Value::Int(number),
            SpawnCapturedValue::Float(number) => Value::Float(number),
            SpawnCapturedValue::Str(text) => Value::Str(Arc::new(text)),
            SpawnCapturedValue::Bool(boolean) => Value::Bool(boolean),
            SpawnCapturedValue::Null => Value::Null,
            SpawnCapturedValue::Bytes(bytes) => Value::Bytes(bytes),
            SpawnCapturedValue::NativeFunction(name) => Value::NativeFunction(name),
            SpawnCapturedValue::Struct { name, fields } => {
                let mut value_fields = HashMap::with_capacity(fields.len());
                for (field_name, field_value) in fields {
                    value_fields.insert(field_name, field_value.into_value());
                }
                Value::Struct { name, fields: value_fields }
            }
            SpawnCapturedValue::Array(elements) => {
                Value::Array(Arc::new(elements.into_iter().map(|v| v.into_value()).collect()))
            }
            SpawnCapturedValue::Dict(entries) => {
                let mut map = DictMap::default();
                for (key, dict_value) in entries {
                    map.insert(Arc::from(key), dict_value.into_value());
                }
                Value::Dict(Arc::new(map))
            }
            SpawnCapturedValue::FixedDict(entries) => {
                let mut keys = Vec::with_capacity(entries.len());
                let mut values = Vec::with_capacity(entries.len());
                for (key, dict_value) in entries {
                    keys.push(Arc::<str>::from(key));
                    values.push(dict_value.into_value());
                }
                Value::FixedDict { keys: Arc::new(keys), values }
            }
            SpawnCapturedValue::IntDict(entries) => {
                let mut map = IntDictMap::default();
                for (key, dict_value) in entries {
                    map.insert(key, dict_value.into_value());
                }
                Value::IntDict(Arc::new(map))
            }
            SpawnCapturedValue::DenseIntDict(values) => Value::DenseIntDict(Arc::new(
                values.into_iter().map(|dict_value| dict_value.into_value()).collect(),
            )),
            SpawnCapturedValue::DenseIntDictInt(values) => Value::DenseIntDictInt(Arc::new(values)),
            SpawnCapturedValue::DenseIntDictIntFull(values) => {
                Value::DenseIntDictIntFull(Arc::new(values))
            }
            SpawnCapturedValue::Result { is_ok, value } => {
                Value::Result { is_ok, value: Box::new(value.into_value()) }
            }
            SpawnCapturedValue::Option { is_some, value } => {
                Value::Option { is_some, value: Box::new(value.into_value()) }
            }
        }
    }
}

/// Main interpreter that executes Ruff programs
pub struct Interpreter {
    pub env: Environment,
    pub return_value: Option<Value>,
    control_flow: ControlFlow,
    output: Option<Arc<Mutex<Vec<u8>>>>,
    pub source_file: Option<String>,
    pub source_lines: Vec<String>,
    pub module_loader: ModuleLoader,
    call_stack: Vec<String>, // Track function calls for stack traces
    async_task_pool_size: usize,
}

impl Interpreter {
    /// Creates a new interpreter with an empty environment
    pub fn new() -> Self {
        let mut interpreter = Interpreter {
            env: Environment::default(),
            return_value: None,
            control_flow: ControlFlow::None,
            output: None,
            source_file: None,
            source_lines: Vec::new(),
            module_loader: ModuleLoader::new(),
            call_stack: Vec::new(),
            async_task_pool_size: DEFAULT_ASYNC_TASK_POOL_SIZE,
        };

        // Register built-in functions and constants
        interpreter.register_builtins();

        interpreter
    }

    /// Set the environment (used by VM to share environment)
    pub fn set_env(&mut self, env: Arc<Mutex<Environment>>) {
        // We need to extract the environment from the Mutex
        let locked_env = env.lock().unwrap();
        self.env = locked_env.clone();
    }

    /// Get the current call stack for error reporting
    pub fn get_call_stack(&self) -> Vec<String> {
        self.call_stack.clone()
    }

    pub fn get_async_task_pool_size(&self) -> usize {
        self.async_task_pool_size
    }

    pub fn set_async_task_pool_size(&mut self, size: usize) -> usize {
        let previous_size = self.async_task_pool_size;
        self.async_task_pool_size = size;
        previous_size
    }

    fn capture_spawn_bindings(&self) -> Vec<(String, SpawnCapturedValue)> {
        let mut merged_bindings: HashMap<String, SpawnCapturedValue> = HashMap::new();

        for scope in &self.env.scopes {
            for (name, value) in scope {
                if let Some(captured_value) = SpawnCapturedValue::from_value(value) {
                    merged_bindings.insert(name.clone(), captured_value);
                }
            }
        }

        merged_bindings.into_iter().collect()
    }

    /// Get all built-in function names (for VM initialization)
    /// This returns a list of all native functions that the interpreter supports
    pub fn get_builtin_names() -> Vec<&'static str> {
        vec![
            // I/O functions
            "print",
            // Math functions
            "abs",
            "sqrt",
            "pow",
            "floor",
            "ceil",
            "round",
            "min",
            "max",
            "sin",
            "cos",
            "tan",
            "log",
            "exp",
            // String functions
            "len",
            "substring",
            "to_upper",
            "upper",
            "to_lower",
            "lower",
            "capitalize",
            "trim",
            "trim_start",
            "trim_end",
            "contains",
            "replace_str",
            "replace",
            "split",
            "join",
            "ssg_render_pages",
            "starts_with",
            "ends_with",
            "pad_left",
            "pad_right",
            "lines",
            "words",
            "str_reverse",
            "slugify",
            "truncate",
            "to_camel_case",
            "to_snake_case",
            "to_kebab_case",
            // Array functions
            "push",
            "append",
            "pop",
            "insert",
            "remove",
            "remove_at",
            "clear",
            "slice",
            "concat",
            // Array higher-order functions
            "map",
            "filter",
            "reduce",
            "find",
            // Array utility functions
            "sort",
            "reverse",
            "unique",
            "sum",
            "any",
            "all",
            // Advanced array methods
            "chunk",
            "flatten",
            "zip",
            "enumerate",
            "take",
            "skip",
            "windows",
            // Array generation functions
            "range",
            // String formatting functions
            "format",
            // Dict functions
            "keys",
            "values",
            "items",
            "has_key",
            "get",
            "remove",
            "clear",
            "merge",
            // Advanced dict methods
            "invert",
            "update",
            "get_default",
            // I/O functions
            "input",
            // Type conversion functions
            "parse_int",
            "parse_float",
            "to_int",
            "to_float",
            "to_string",
            "to_bool",
            // Type checking functions
            "type",
            "is_int",
            "is_float",
            "is_string",
            "is_bool",
            "is_array",
            "is_dict",
            "is_null",
            "is_function",
            // Assert & Debug functions
            "assert",
            "debug",
            // File I/O functions
            "read_file",
            "write_file",
            "append_file",
            "file_exists",
            "read_lines",
            "list_dir",
            "create_dir",
            "file_size",
            "delete_file",
            "rename_file",
            "copy_file",
            // Binary file I/O functions
            "read_binary_file",
            "write_binary_file",
            // IO module functions (advanced binary operations)
            "io_read_bytes",
            "io_write_bytes",
            "io_append_bytes",
            "io_read_at",
            "io_write_at",
            "io_seek_read",
            "io_file_metadata",
            "io_truncate",
            "io_copy_range",
            // JSON functions
            "parse_json",
            "to_json",
            // TOML functions
            "parse_toml",
            "to_toml",
            // YAML functions
            "parse_yaml",
            "to_yaml",
            // CSV functions
            "parse_csv",
            "to_csv",
            // Base64 encoding/decoding functions
            "encode_base64",
            "decode_base64",
            // Random functions
            "random",
            "random_int",
            "random_choice",
            "set_random_seed",
            "clear_random_seed",
            // Date/Time functions
            "now",
            "current_timestamp",
            "performance_now",
            "time_us",
            "time_ns",
            "format_duration",
            "elapsed",
            "format_date",
            "parse_date",
            // System operation functions
            "env",
            "env_or",
            "env_int",
            "env_float",
            "env_bool",
            "env_required",
            "env_set",
            "env_list",
            "args",
            "arg_parser",
            "exit",
            "sleep",
            "execute",
            // OS module functions
            "os_getcwd",
            "os_chdir",
            "os_rmdir",
            "os_environ",
            // Path operation functions
            "join_path",
            "dirname",
            "basename",
            "path_exists",
            "path_join",
            "path_absolute",
            "path_is_dir",
            "path_is_file",
            "path_extension",
            // Regular expression functions
            "regex_match",
            "regex_find_all",
            "regex_replace",
            "regex_split",
            // HTTP client functions
            "http_get",
            "http_post",
            "http_put",
            "http_delete",
            "http_get_binary",
            // Concurrent HTTP functions
            "parallel_http",
            // JWT authentication functions
            "jwt_encode",
            "jwt_decode",
            // OAuth2 helper functions
            "oauth2_auth_url",
            "oauth2_get_token",
            // HTTP streaming functions
            "http_get_stream",
            // HTTP server functions
            "http_server",
            "http_response",
            "json_response",
            "html_response",
            "redirect_response",
            "set_header",
            "set_headers",
            // Database functions
            "db_connect",
            "db_execute",
            "db_query",
            "db_close",
            "db_begin",
            "db_commit",
            "db_rollback",
            "db_last_insert_id",
            // Collection constructors and methods
            // Set
            "Set",
            "set_add",
            "set_has",
            "set_remove",
            "set_union",
            "set_intersect",
            "set_difference",
            "set_to_array",
            // Queue
            "Queue",
            "queue_enqueue",
            "queue_dequeue",
            "queue_peek",
            "queue_size",
            "queue_is_empty",
            "queue_to_array",
            // Stack
            "Stack",
            "stack_push",
            "stack_pop",
            "stack_peek",
            "stack_size",
            "stack_is_empty",
            "stack_to_array",
            // Concurrency functions
            "channel",
            "shared_set",
            "shared_get",
            "shared_has",
            "shared_delete",
            "shared_add_int",
            // Async operations
            "async_sleep",
            "async_timeout",
            "async_http_get",
            "async_http_post",
            "async_read_file",
            "async_read_files",
            "async_write_file",
            "async_write_files",
            "spawn_task",
            "await_task",
            "cancel_task",
            "Promise.all",
            "promise_all",
            "await_all",
            "parallel_map",
            "par_map",
            "par_each",
            "set_task_pool_size",
            "get_task_pool_size",
            // Testing assertion functions
            "assert_equal",
            "assert_true",
            "assert_false",
            "assert_contains",
            // Image processing functions
            "load_image",
            // Compression & Archive functions
            "zip_create",
            "zip_add_file",
            "zip_add_dir",
            "zip_close",
            "unzip",
            // Hashing & Cryptography functions
            "sha256",
            "md5",
            "md5_file",
            "hash_password",
            "verify_password",
            // Crypto functions
            "aes_encrypt",
            "aes_decrypt",
            "aes_encrypt_bytes",
            "aes_decrypt_bytes",
            "rsa_generate_keypair",
            "rsa_encrypt",
            "rsa_decrypt",
            "rsa_sign",
            "rsa_verify",
            // Process management functions
            "spawn_process",
            "pipe_commands",
        ]
    }

    /// Registers all built-in functions and constants
    fn register_builtins(&mut self) {
        // I/O functions
        self.env.define("print".to_string(), Value::NativeFunction("print".to_string()));

        // Math constants
        self.env.define("PI".to_string(), Value::Float(std::f64::consts::PI));
        self.env.define("E".to_string(), Value::Float(std::f64::consts::E));

        // Math functions
        self.env.define("abs".to_string(), Value::NativeFunction("abs".to_string()));
        self.env.define("sqrt".to_string(), Value::NativeFunction("sqrt".to_string()));
        self.env.define("pow".to_string(), Value::NativeFunction("pow".to_string()));
        self.env.define("floor".to_string(), Value::NativeFunction("floor".to_string()));
        self.env.define("ceil".to_string(), Value::NativeFunction("ceil".to_string()));
        self.env.define("round".to_string(), Value::NativeFunction("round".to_string()));
        self.env.define("min".to_string(), Value::NativeFunction("min".to_string()));
        self.env.define("max".to_string(), Value::NativeFunction("max".to_string()));
        self.env.define("sin".to_string(), Value::NativeFunction("sin".to_string()));
        self.env.define("cos".to_string(), Value::NativeFunction("cos".to_string()));
        self.env.define("tan".to_string(), Value::NativeFunction("tan".to_string()));
        self.env.define("log".to_string(), Value::NativeFunction("log".to_string()));
        self.env.define("exp".to_string(), Value::NativeFunction("exp".to_string()));

        // String functions
        self.env.define("len".to_string(), Value::NativeFunction("len".to_string()));
        self.env.define("substring".to_string(), Value::NativeFunction("substring".to_string()));
        self.env.define("to_upper".to_string(), Value::NativeFunction("to_upper".to_string()));
        self.env.define("upper".to_string(), Value::NativeFunction("upper".to_string())); // Alias
        self.env.define("to_lower".to_string(), Value::NativeFunction("to_lower".to_string()));
        self.env.define("lower".to_string(), Value::NativeFunction("lower".to_string())); // Alias
        self.env.define("capitalize".to_string(), Value::NativeFunction("capitalize".to_string()));
        self.env.define("trim".to_string(), Value::NativeFunction("trim".to_string()));
        self.env.define("trim_start".to_string(), Value::NativeFunction("trim_start".to_string()));
        self.env.define("trim_end".to_string(), Value::NativeFunction("trim_end".to_string()));
        self.env.define("contains".to_string(), Value::NativeFunction("contains".to_string()));
        self.env
            .define("replace_str".to_string(), Value::NativeFunction("replace_str".to_string()));
        self.env.define("replace".to_string(), Value::NativeFunction("replace".to_string())); // Alias
        self.env.define("split".to_string(), Value::NativeFunction("split".to_string()));
        self.env.define("join".to_string(), Value::NativeFunction("join".to_string()));
        self.env.define(
            "ssg_render_pages".to_string(),
            Value::NativeFunction("ssg_render_pages".to_string()),
        );
        self.env
            .define("starts_with".to_string(), Value::NativeFunction("starts_with".to_string()));
        self.env.define("ends_with".to_string(), Value::NativeFunction("ends_with".to_string()));
        self.env.define("index_of".to_string(), Value::NativeFunction("index_of".to_string()));
        self.env.define("repeat".to_string(), Value::NativeFunction("repeat".to_string()));
        self.env.define("char_at".to_string(), Value::NativeFunction("char_at".to_string()));
        self.env.define("is_empty".to_string(), Value::NativeFunction("is_empty".to_string()));
        self.env
            .define("count_chars".to_string(), Value::NativeFunction("count_chars".to_string()));

        // Advanced string methods
        self.env.define("pad_left".to_string(), Value::NativeFunction("pad_left".to_string()));
        self.env.define("pad_right".to_string(), Value::NativeFunction("pad_right".to_string()));
        self.env.define("lines".to_string(), Value::NativeFunction("lines".to_string()));
        self.env.define("words".to_string(), Value::NativeFunction("words".to_string()));
        self.env
            .define("str_reverse".to_string(), Value::NativeFunction("str_reverse".to_string()));
        self.env.define("slugify".to_string(), Value::NativeFunction("slugify".to_string()));
        self.env.define("truncate".to_string(), Value::NativeFunction("truncate".to_string()));
        self.env.define(
            "to_camel_case".to_string(),
            Value::NativeFunction("to_camel_case".to_string()),
        );
        self.env.define(
            "to_snake_case".to_string(),
            Value::NativeFunction("to_snake_case".to_string()),
        );
        self.env.define(
            "to_kebab_case".to_string(),
            Value::NativeFunction("to_kebab_case".to_string()),
        );

        // Array functions
        self.env.define("push".to_string(), Value::NativeFunction("push".to_string()));
        self.env.define("append".to_string(), Value::NativeFunction("append".to_string())); // Alias
        self.env.define("pop".to_string(), Value::NativeFunction("pop".to_string()));
        self.env.define("insert".to_string(), Value::NativeFunction("insert".to_string()));
        self.env.define("remove".to_string(), Value::NativeFunction("remove".to_string()));
        self.env.define("remove_at".to_string(), Value::NativeFunction("remove_at".to_string()));
        self.env.define("clear".to_string(), Value::NativeFunction("clear".to_string()));
        self.env.define("slice".to_string(), Value::NativeFunction("slice".to_string()));
        self.env.define("concat".to_string(), Value::NativeFunction("concat".to_string()));

        // Array higher-order functions
        self.env.define("map".to_string(), Value::NativeFunction("map".to_string()));
        self.env.define("filter".to_string(), Value::NativeFunction("filter".to_string()));
        self.env.define("reduce".to_string(), Value::NativeFunction("reduce".to_string()));
        self.env.define("find".to_string(), Value::NativeFunction("find".to_string()));

        // Array utility functions
        self.env.define("sort".to_string(), Value::NativeFunction("sort".to_string()));
        self.env.define("reverse".to_string(), Value::NativeFunction("reverse".to_string()));
        self.env.define("unique".to_string(), Value::NativeFunction("unique".to_string()));
        self.env.define("sum".to_string(), Value::NativeFunction("sum".to_string()));
        self.env.define("any".to_string(), Value::NativeFunction("any".to_string()));
        self.env.define("all".to_string(), Value::NativeFunction("all".to_string()));

        // Advanced array methods
        self.env.define("chunk".to_string(), Value::NativeFunction("chunk".to_string()));
        self.env.define("flatten".to_string(), Value::NativeFunction("flatten".to_string()));
        self.env.define("zip".to_string(), Value::NativeFunction("zip".to_string()));
        self.env.define("enumerate".to_string(), Value::NativeFunction("enumerate".to_string()));
        self.env.define("take".to_string(), Value::NativeFunction("take".to_string()));
        self.env.define("skip".to_string(), Value::NativeFunction("skip".to_string()));
        self.env.define("windows".to_string(), Value::NativeFunction("windows".to_string()));

        // Array generation functions
        self.env.define("range".to_string(), Value::NativeFunction("range".to_string()));

        // String formatting functions
        self.env.define("format".to_string(), Value::NativeFunction("format".to_string()));

        // Dict functions
        self.env.define("keys".to_string(), Value::NativeFunction("keys".to_string()));
        self.env.define("values".to_string(), Value::NativeFunction("values".to_string()));
        self.env.define("items".to_string(), Value::NativeFunction("items".to_string()));
        self.env.define("has_key".to_string(), Value::NativeFunction("has_key".to_string()));
        self.env.define("get".to_string(), Value::NativeFunction("get".to_string()));
        self.env.define("remove".to_string(), Value::NativeFunction("remove".to_string()));
        self.env.define("clear".to_string(), Value::NativeFunction("clear".to_string()));
        self.env.define("merge".to_string(), Value::NativeFunction("merge".to_string()));

        // Advanced dict methods
        self.env.define("invert".to_string(), Value::NativeFunction("invert".to_string()));
        self.env.define("update".to_string(), Value::NativeFunction("update".to_string()));
        self.env
            .define("get_default".to_string(), Value::NativeFunction("get_default".to_string()));

        // I/O functions
        self.env.define("input".to_string(), Value::NativeFunction("input".to_string()));

        // Type conversion functions
        self.env.define("parse_int".to_string(), Value::NativeFunction("parse_int".to_string()));
        self.env
            .define("parse_float".to_string(), Value::NativeFunction("parse_float".to_string()));
        self.env.define("to_int".to_string(), Value::NativeFunction("to_int".to_string()));
        self.env.define("to_float".to_string(), Value::NativeFunction("to_float".to_string()));
        self.env.define("to_string".to_string(), Value::NativeFunction("to_string".to_string()));
        self.env.define("to_bool".to_string(), Value::NativeFunction("to_bool".to_string()));
        self.env.define("bytes".to_string(), Value::NativeFunction("bytes".to_string()));

        // Type introspection functions
        self.env.define("type".to_string(), Value::NativeFunction("type".to_string()));
        self.env.define("is_int".to_string(), Value::NativeFunction("is_int".to_string()));
        self.env.define("is_float".to_string(), Value::NativeFunction("is_float".to_string()));
        self.env.define("is_string".to_string(), Value::NativeFunction("is_string".to_string()));
        self.env.define("is_array".to_string(), Value::NativeFunction("is_array".to_string()));
        self.env.define("is_dict".to_string(), Value::NativeFunction("is_dict".to_string()));
        self.env.define("is_bool".to_string(), Value::NativeFunction("is_bool".to_string()));
        self.env.define("is_null".to_string(), Value::NativeFunction("is_null".to_string()));
        self.env
            .define("is_function".to_string(), Value::NativeFunction("is_function".to_string()));

        // Assert & Debug functions
        self.env.define("assert".to_string(), Value::NativeFunction("assert".to_string()));
        self.env.define("debug".to_string(), Value::NativeFunction("debug".to_string()));

        // File I/O functions
        self.env.define("read_file".to_string(), Value::NativeFunction("read_file".to_string()));
        self.env.define("write_file".to_string(), Value::NativeFunction("write_file".to_string()));
        self.env
            .define("append_file".to_string(), Value::NativeFunction("append_file".to_string()));
        self.env
            .define("file_exists".to_string(), Value::NativeFunction("file_exists".to_string()));
        self.env.define("read_lines".to_string(), Value::NativeFunction("read_lines".to_string()));
        self.env.define("list_dir".to_string(), Value::NativeFunction("list_dir".to_string()));
        self.env.define("create_dir".to_string(), Value::NativeFunction("create_dir".to_string()));
        self.env.define("file_size".to_string(), Value::NativeFunction("file_size".to_string()));
        self.env
            .define("delete_file".to_string(), Value::NativeFunction("delete_file".to_string()));
        self.env
            .define("rename_file".to_string(), Value::NativeFunction("rename_file".to_string()));
        self.env.define("copy_file".to_string(), Value::NativeFunction("copy_file".to_string()));

        // Binary file I/O functions
        self.env.define(
            "read_binary_file".to_string(),
            Value::NativeFunction("read_binary_file".to_string()),
        );
        self.env.define(
            "write_binary_file".to_string(),
            Value::NativeFunction("write_binary_file".to_string()),
        );

        // IO module functions (advanced binary operations)
        self.env.define(
            "io_read_bytes".to_string(),
            Value::NativeFunction("io_read_bytes".to_string()),
        );
        self.env.define(
            "io_write_bytes".to_string(),
            Value::NativeFunction("io_write_bytes".to_string()),
        );
        self.env.define(
            "io_append_bytes".to_string(),
            Value::NativeFunction("io_append_bytes".to_string()),
        );
        self.env.define("io_read_at".to_string(), Value::NativeFunction("io_read_at".to_string()));
        self.env
            .define("io_write_at".to_string(), Value::NativeFunction("io_write_at".to_string()));
        self.env
            .define("io_seek_read".to_string(), Value::NativeFunction("io_seek_read".to_string()));
        self.env.define(
            "io_file_metadata".to_string(),
            Value::NativeFunction("io_file_metadata".to_string()),
        );
        self.env
            .define("io_truncate".to_string(), Value::NativeFunction("io_truncate".to_string()));
        self.env.define(
            "io_copy_range".to_string(),
            Value::NativeFunction("io_copy_range".to_string()),
        );

        // JSON functions
        self.env.define("parse_json".to_string(), Value::NativeFunction("parse_json".to_string()));
        self.env.define("to_json".to_string(), Value::NativeFunction("to_json".to_string()));

        // TOML functions
        self.env.define("parse_toml".to_string(), Value::NativeFunction("parse_toml".to_string()));
        self.env.define("to_toml".to_string(), Value::NativeFunction("to_toml".to_string()));

        // YAML functions
        self.env.define("parse_yaml".to_string(), Value::NativeFunction("parse_yaml".to_string()));
        self.env.define("to_yaml".to_string(), Value::NativeFunction("to_yaml".to_string()));

        // CSV functions
        self.env.define("parse_csv".to_string(), Value::NativeFunction("parse_csv".to_string()));
        self.env.define("to_csv".to_string(), Value::NativeFunction("to_csv".to_string()));

        // Base64 encoding/decoding functions
        self.env.define(
            "encode_base64".to_string(),
            Value::NativeFunction("encode_base64".to_string()),
        );
        self.env.define(
            "decode_base64".to_string(),
            Value::NativeFunction("decode_base64".to_string()),
        );

        // Random functions
        self.env.define("random".to_string(), Value::NativeFunction("random".to_string()));
        self.env.define("random_int".to_string(), Value::NativeFunction("random_int".to_string()));
        self.env.define(
            "random_choice".to_string(),
            Value::NativeFunction("random_choice".to_string()),
        );
        self.env.define(
            "set_random_seed".to_string(),
            Value::NativeFunction("set_random_seed".to_string()),
        );
        self.env.define(
            "clear_random_seed".to_string(),
            Value::NativeFunction("clear_random_seed".to_string()),
        );

        // Date/Time functions
        self.env.define("now".to_string(), Value::NativeFunction("now".to_string()));
        self.env.define(
            "current_timestamp".to_string(),
            Value::NativeFunction("current_timestamp".to_string()),
        );
        self.env.define(
            "performance_now".to_string(),
            Value::NativeFunction("performance_now".to_string()),
        );
        self.env.define("time_us".to_string(), Value::NativeFunction("time_us".to_string()));
        self.env.define("time_ns".to_string(), Value::NativeFunction("time_ns".to_string()));
        self.env.define(
            "format_duration".to_string(),
            Value::NativeFunction("format_duration".to_string()),
        );
        self.env.define("elapsed".to_string(), Value::NativeFunction("elapsed".to_string()));
        self.env
            .define("format_date".to_string(), Value::NativeFunction("format_date".to_string()));
        self.env.define("parse_date".to_string(), Value::NativeFunction("parse_date".to_string()));

        // System operation functions
        self.env.define("env".to_string(), Value::NativeFunction("env".to_string()));
        self.env.define("env_or".to_string(), Value::NativeFunction("env_or".to_string()));
        self.env.define("env_int".to_string(), Value::NativeFunction("env_int".to_string()));
        self.env.define("env_float".to_string(), Value::NativeFunction("env_float".to_string()));
        self.env.define("env_bool".to_string(), Value::NativeFunction("env_bool".to_string()));
        self.env
            .define("env_required".to_string(), Value::NativeFunction("env_required".to_string()));
        self.env.define("env_set".to_string(), Value::NativeFunction("env_set".to_string()));
        self.env.define("env_list".to_string(), Value::NativeFunction("env_list".to_string()));
        self.env.define("args".to_string(), Value::NativeFunction("args".to_string()));
        self.env.define("arg_parser".to_string(), Value::NativeFunction("arg_parser".to_string()));
        self.env.define("exit".to_string(), Value::NativeFunction("exit".to_string()));
        self.env.define("sleep".to_string(), Value::NativeFunction("sleep".to_string()));
        self.env.define("execute".to_string(), Value::NativeFunction("execute".to_string()));

        // OS module functions
        self.env.define("os_getcwd".to_string(), Value::NativeFunction("os_getcwd".to_string()));
        self.env.define("os_chdir".to_string(), Value::NativeFunction("os_chdir".to_string()));
        self.env.define("os_rmdir".to_string(), Value::NativeFunction("os_rmdir".to_string()));
        self.env.define("os_environ".to_string(), Value::NativeFunction("os_environ".to_string()));

        // Path operation functions
        self.env.define("join_path".to_string(), Value::NativeFunction("join_path".to_string()));
        self.env.define("dirname".to_string(), Value::NativeFunction("dirname".to_string()));
        self.env.define("basename".to_string(), Value::NativeFunction("basename".to_string()));
        self.env
            .define("path_exists".to_string(), Value::NativeFunction("path_exists".to_string()));
        self.env.define("path_join".to_string(), Value::NativeFunction("path_join".to_string()));
        self.env.define(
            "path_absolute".to_string(),
            Value::NativeFunction("path_absolute".to_string()),
        );
        self.env
            .define("path_is_dir".to_string(), Value::NativeFunction("path_is_dir".to_string()));
        self.env
            .define("path_is_file".to_string(), Value::NativeFunction("path_is_file".to_string()));
        self.env.define(
            "path_extension".to_string(),
            Value::NativeFunction("path_extension".to_string()),
        );

        // Regular expression functions
        self.env
            .define("regex_match".to_string(), Value::NativeFunction("regex_match".to_string()));
        self.env.define(
            "regex_find_all".to_string(),
            Value::NativeFunction("regex_find_all".to_string()),
        );
        self.env.define(
            "regex_replace".to_string(),
            Value::NativeFunction("regex_replace".to_string()),
        );
        self.env
            .define("regex_split".to_string(), Value::NativeFunction("regex_split".to_string()));

        // HTTP client functions
        self.env.define("http_get".to_string(), Value::NativeFunction("http_get".to_string()));
        self.env.define("http_post".to_string(), Value::NativeFunction("http_post".to_string()));
        self.env.define("http_put".to_string(), Value::NativeFunction("http_put".to_string()));
        self.env
            .define("http_delete".to_string(), Value::NativeFunction("http_delete".to_string()));
        self.env.define(
            "http_get_binary".to_string(),
            Value::NativeFunction("http_get_binary".to_string()),
        );

        // Concurrent HTTP functions
        self.env.define(
            "parallel_http".to_string(),
            Value::NativeFunction("parallel_http".to_string()),
        );

        // JWT authentication functions
        self.env.define("jwt_encode".to_string(), Value::NativeFunction("jwt_encode".to_string()));
        self.env.define("jwt_decode".to_string(), Value::NativeFunction("jwt_decode".to_string()));

        // OAuth2 helper functions
        self.env.define(
            "oauth2_auth_url".to_string(),
            Value::NativeFunction("oauth2_auth_url".to_string()),
        );
        self.env.define(
            "oauth2_get_token".to_string(),
            Value::NativeFunction("oauth2_get_token".to_string()),
        );

        // HTTP streaming functions
        self.env.define(
            "http_get_stream".to_string(),
            Value::NativeFunction("http_get_stream".to_string()),
        );

        // HTTP server functions
        self.env
            .define("http_server".to_string(), Value::NativeFunction("http_server".to_string()));
        self.env.define(
            "http_response".to_string(),
            Value::NativeFunction("http_response".to_string()),
        );
        self.env.define(
            "json_response".to_string(),
            Value::NativeFunction("json_response".to_string()),
        );
        self.env.define(
            "html_response".to_string(),
            Value::NativeFunction("html_response".to_string()),
        );
        self.env.define(
            "redirect_response".to_string(),
            Value::NativeFunction("redirect_response".to_string()),
        );
        self.env.define("set_header".to_string(), Value::NativeFunction("set_header".to_string()));
        self.env
            .define("set_headers".to_string(), Value::NativeFunction("set_headers".to_string()));

        // Database functions
        self.env.define("db_connect".to_string(), Value::NativeFunction("db_connect".to_string()));
        self.env.define("db_execute".to_string(), Value::NativeFunction("db_execute".to_string()));
        self.env.define("db_query".to_string(), Value::NativeFunction("db_query".to_string()));
        self.env.define("db_close".to_string(), Value::NativeFunction("db_close".to_string()));
        self.env.define("db_pool".to_string(), Value::NativeFunction("db_pool".to_string()));
        self.env.define(
            "db_pool_acquire".to_string(),
            Value::NativeFunction("db_pool_acquire".to_string()),
        );
        self.env.define(
            "db_pool_release".to_string(),
            Value::NativeFunction("db_pool_release".to_string()),
        );
        self.env.define(
            "db_pool_stats".to_string(),
            Value::NativeFunction("db_pool_stats".to_string()),
        );
        self.env.define(
            "db_pool_close".to_string(),
            Value::NativeFunction("db_pool_close".to_string()),
        );
        self.env.define("db_begin".to_string(), Value::NativeFunction("db_begin".to_string()));
        self.env.define("db_commit".to_string(), Value::NativeFunction("db_commit".to_string()));
        self.env
            .define("db_rollback".to_string(), Value::NativeFunction("db_rollback".to_string()));
        self.env.define(
            "db_last_insert_id".to_string(),
            Value::NativeFunction("db_last_insert_id".to_string()),
        );

        // Collection constructors and methods
        // Set
        self.env.define("Set".to_string(), Value::NativeFunction("Set".to_string()));
        self.env.define("set_add".to_string(), Value::NativeFunction("set_add".to_string()));
        self.env.define("set_has".to_string(), Value::NativeFunction("set_has".to_string()));
        self.env.define("set_remove".to_string(), Value::NativeFunction("set_remove".to_string()));
        self.env.define("set_union".to_string(), Value::NativeFunction("set_union".to_string()));
        self.env.define(
            "set_intersect".to_string(),
            Value::NativeFunction("set_intersect".to_string()),
        );
        self.env.define(
            "set_difference".to_string(),
            Value::NativeFunction("set_difference".to_string()),
        );
        self.env
            .define("set_to_array".to_string(), Value::NativeFunction("set_to_array".to_string()));

        // Queue
        self.env.define("Queue".to_string(), Value::NativeFunction("Queue".to_string()));
        self.env.define(
            "queue_enqueue".to_string(),
            Value::NativeFunction("queue_enqueue".to_string()),
        );
        self.env.define(
            "queue_dequeue".to_string(),
            Value::NativeFunction("queue_dequeue".to_string()),
        );
        self.env.define("queue_peek".to_string(), Value::NativeFunction("queue_peek".to_string()));
        self.env.define(
            "queue_is_empty".to_string(),
            Value::NativeFunction("queue_is_empty".to_string()),
        );
        self.env.define(
            "queue_to_array".to_string(),
            Value::NativeFunction("queue_to_array".to_string()),
        );

        // Stack
        self.env.define("Stack".to_string(), Value::NativeFunction("Stack".to_string()));
        self.env.define("stack_push".to_string(), Value::NativeFunction("stack_push".to_string()));
        self.env.define("stack_pop".to_string(), Value::NativeFunction("stack_pop".to_string()));
        self.env.define("stack_peek".to_string(), Value::NativeFunction("stack_peek".to_string()));
        self.env.define(
            "stack_is_empty".to_string(),
            Value::NativeFunction("stack_is_empty".to_string()),
        );
        self.env.define(
            "stack_to_array".to_string(),
            Value::NativeFunction("stack_to_array".to_string()),
        );

        // Concurrency functions
        self.env.define("channel".to_string(), Value::NativeFunction("channel".to_string()));
        self.env.define("shared_set".to_string(), Value::NativeFunction("shared_set".to_string()));
        self.env.define("shared_get".to_string(), Value::NativeFunction("shared_get".to_string()));
        self.env.define("shared_has".to_string(), Value::NativeFunction("shared_has".to_string()));
        self.env.define(
            "shared_delete".to_string(),
            Value::NativeFunction("shared_delete".to_string()),
        );
        self.env.define(
            "shared_add_int".to_string(),
            Value::NativeFunction("shared_add_int".to_string()),
        );

        // Async operations
        self.env
            .define("async_sleep".to_string(), Value::NativeFunction("async_sleep".to_string()));
        self.env.define(
            "async_timeout".to_string(),
            Value::NativeFunction("async_timeout".to_string()),
        );
        self.env.define(
            "async_http_get".to_string(),
            Value::NativeFunction("async_http_get".to_string()),
        );
        self.env.define(
            "async_http_post".to_string(),
            Value::NativeFunction("async_http_post".to_string()),
        );
        self.env.define(
            "async_read_file".to_string(),
            Value::NativeFunction("async_read_file".to_string()),
        );
        self.env.define(
            "async_read_files".to_string(),
            Value::NativeFunction("async_read_files".to_string()),
        );
        self.env.define(
            "async_write_file".to_string(),
            Value::NativeFunction("async_write_file".to_string()),
        );
        self.env.define(
            "async_write_files".to_string(),
            Value::NativeFunction("async_write_files".to_string()),
        );
        self.env.define("spawn_task".to_string(), Value::NativeFunction("spawn_task".to_string()));
        self.env.define("await_task".to_string(), Value::NativeFunction("await_task".to_string()));
        self.env
            .define("cancel_task".to_string(), Value::NativeFunction("cancel_task".to_string()));
        self.env
            .define("Promise.all".to_string(), Value::NativeFunction("Promise.all".to_string()));
        self.env
            .define("promise_all".to_string(), Value::NativeFunction("promise_all".to_string())); // Alias
        self.env.define("await_all".to_string(), Value::NativeFunction("await_all".to_string())); // Alias
        self.env
            .define("parallel_map".to_string(), Value::NativeFunction("parallel_map".to_string()));
        self.env.define("par_map".to_string(), Value::NativeFunction("par_map".to_string()));
        self.env.define("par_each".to_string(), Value::NativeFunction("par_each".to_string()));
        self.env.define(
            "set_task_pool_size".to_string(),
            Value::NativeFunction("set_task_pool_size".to_string()),
        );
        self.env.define(
            "get_task_pool_size".to_string(),
            Value::NativeFunction("get_task_pool_size".to_string()),
        );

        // Testing assertion functions
        self.env
            .define("assert_equal".to_string(), Value::NativeFunction("assert_equal".to_string()));
        self.env
            .define("assert_true".to_string(), Value::NativeFunction("assert_true".to_string()));
        self.env
            .define("assert_false".to_string(), Value::NativeFunction("assert_false".to_string()));
        self.env.define(
            "assert_contains".to_string(),
            Value::NativeFunction("assert_contains".to_string()),
        );

        // Image processing functions
        self.env.define("load_image".to_string(), Value::NativeFunction("load_image".to_string()));

        // Compression & Archive functions
        self.env.define("zip_create".to_string(), Value::NativeFunction("zip_create".to_string()));
        self.env
            .define("zip_add_file".to_string(), Value::NativeFunction("zip_add_file".to_string()));
        self.env
            .define("zip_add_dir".to_string(), Value::NativeFunction("zip_add_dir".to_string()));
        self.env.define("zip_close".to_string(), Value::NativeFunction("zip_close".to_string()));
        self.env.define("unzip".to_string(), Value::NativeFunction("unzip".to_string()));

        // Hashing & Crypto functions
        self.env.define("sha256".to_string(), Value::NativeFunction("sha256".to_string()));
        self.env.define("md5".to_string(), Value::NativeFunction("md5".to_string()));
        self.env.define("md5_file".to_string(), Value::NativeFunction("md5_file".to_string()));
        self.env.define(
            "hash_password".to_string(),
            Value::NativeFunction("hash_password".to_string()),
        );
        self.env.define(
            "verify_password".to_string(),
            Value::NativeFunction("verify_password".to_string()),
        );

        // Crypto functions (AES/RSA encryption)
        self.env
            .define("aes_encrypt".to_string(), Value::NativeFunction("aes_encrypt".to_string()));
        self.env
            .define("aes_decrypt".to_string(), Value::NativeFunction("aes_decrypt".to_string()));
        self.env.define(
            "aes_encrypt_bytes".to_string(),
            Value::NativeFunction("aes_encrypt_bytes".to_string()),
        );
        self.env.define(
            "aes_decrypt_bytes".to_string(),
            Value::NativeFunction("aes_decrypt_bytes".to_string()),
        );
        self.env.define(
            "rsa_generate_keypair".to_string(),
            Value::NativeFunction("rsa_generate_keypair".to_string()),
        );
        self.env
            .define("rsa_encrypt".to_string(), Value::NativeFunction("rsa_encrypt".to_string()));
        self.env
            .define("rsa_decrypt".to_string(), Value::NativeFunction("rsa_decrypt".to_string()));
        self.env.define("rsa_sign".to_string(), Value::NativeFunction("rsa_sign".to_string()));
        self.env.define("rsa_verify".to_string(), Value::NativeFunction("rsa_verify".to_string()));

        // Process management functions
        self.env.define(
            "spawn_process".to_string(),
            Value::NativeFunction("spawn_process".to_string()),
        );
        self.env.define(
            "pipe_commands".to_string(),
            Value::NativeFunction("pipe_commands".to_string()),
        );

        // Network functions (TCP/UDP)
        self.env.define("tcp_listen".to_string(), Value::NativeFunction("tcp_listen".to_string()));
        self.env.define("tcp_accept".to_string(), Value::NativeFunction("tcp_accept".to_string()));
        self.env
            .define("tcp_connect".to_string(), Value::NativeFunction("tcp_connect".to_string()));
        self.env.define("tcp_send".to_string(), Value::NativeFunction("tcp_send".to_string()));
        self.env
            .define("tcp_receive".to_string(), Value::NativeFunction("tcp_receive".to_string()));
        self.env.define("tcp_close".to_string(), Value::NativeFunction("tcp_close".to_string()));
        self.env.define(
            "tcp_set_nonblocking".to_string(),
            Value::NativeFunction("tcp_set_nonblocking".to_string()),
        );
        self.env.define("udp_bind".to_string(), Value::NativeFunction("udp_bind".to_string()));
        self.env
            .define("udp_send_to".to_string(), Value::NativeFunction("udp_send_to".to_string()));
        self.env.define(
            "udp_receive_from".to_string(),
            Value::NativeFunction("udp_receive_from".to_string()),
        );
        self.env.define("udp_close".to_string(), Value::NativeFunction("udp_close".to_string()));
    }

    /// Sets the source file and content for error reporting
    pub fn set_source(&mut self, file: String, content: &str) {
        self.source_file = Some(file);
        self.source_lines = content.lines().map(|s| s.to_string()).collect();
    }

    /// Reports a runtime error with source location
    #[allow(dead_code)]
    fn report_error(&self, error: RuffError) {
        eprintln!("{}", error);
    }

    /// Gets the source line at a given line number (1-indexed)
    #[allow(dead_code)]
    fn get_source_line(&self, line: usize) -> Option<String> {
        if line > 0 && line <= self.source_lines.len() {
            Some(self.source_lines[line - 1].clone())
        } else {
            None
        }
    }

    /// Sets the output sink for print statements (used for testing)
    pub fn set_output(&mut self, output: Arc<Mutex<Vec<u8>>>) {
        self.output = Some(output);
    }

    /// Helper function to call a user-defined function with given arguments
    /// Used by higher-order functions like map, filter, reduce
    fn call_user_function(&mut self, func: &Value, args: &[Value]) -> Value {
        match func {
            Value::GeneratorDef(params, body) => {
                // Calling a generator function returns a Generator instance
                // Create a new environment for the generator
                let mut gen_env = self.env.clone();
                gen_env.push_scope();

                // Bind parameters to arguments
                for (i, param) in params.iter().enumerate() {
                    if let Some(arg) = args.get(i) {
                        gen_env.define(param.clone(), arg.clone());
                    }
                }

                // Return a Generator instance
                Value::Generator {
                    params: params.clone(),
                    body: body.clone(),
                    env: Arc::new(Mutex::new(gen_env)),
                    pc: 0,
                    is_exhausted: false,
                }
            }
            Value::Function(params, body, captured_env) => {
                // Push function name to call stack
                let func_name = format!("<function with {} params>", params.len());
                self.call_stack.push(func_name);

                // If this is a closure with captured environment, use it
                // Otherwise just create a new scope on top of current
                if let Some(closure_env_ref) = captured_env {
                    // Save current environment
                    let saved_env = self.env.clone();

                    // Use the captured environment (which is shared via Arc<Mutex<>>)
                    self.env = closure_env_ref.lock().unwrap().clone();
                    self.env.push_scope();

                    // Bind parameters to arguments
                    for (i, param) in params.iter().enumerate() {
                        if let Some(arg) = args.get(i) {
                            self.env.define(param.clone(), arg.clone());
                        }
                    }

                    // Execute function body
                    self.eval_stmts(body.get());

                    // Get return value
                    let result = if let Some(Value::Return(val)) = self.return_value.clone() {
                        self.return_value = None; // Clear return value
                        *val
                    } else if let Some(Value::Error(msg)) = self.return_value.clone() {
                        // Propagate error - don't clear
                        Value::Error(msg)
                    } else if let Some(Value::ErrorObject { .. }) = self.return_value.clone() {
                        // Propagate error object - don't clear
                        self.return_value.clone().unwrap()
                    } else {
                        // No explicit return - function returns 0
                        self.return_value = None;
                        Value::Int(0)
                    };

                    // Pop the parameter scope
                    self.env.pop_scope();

                    // Update the captured environment with the modified state
                    *closure_env_ref.lock().unwrap() = self.env.clone();

                    // Restore the saved environment
                    self.env = saved_env;

                    // Pop from call stack
                    self.call_stack.pop();

                    result
                } else {
                    // Non-closure: just create new scope on current environment
                    self.env.push_scope();

                    // Bind parameters to arguments
                    for (i, param) in params.iter().enumerate() {
                        if let Some(arg) = args.get(i) {
                            self.env.define(param.clone(), arg.clone());
                        }
                    }

                    // Execute function body
                    self.eval_stmts(body.get());

                    // Get return value
                    let result = if let Some(Value::Return(val)) = self.return_value.clone() {
                        self.return_value = None; // Clear return value
                        *val
                    } else if let Some(Value::Error(msg)) = self.return_value.clone() {
                        // Propagate error - don't clear
                        Value::Error(msg)
                    } else if let Some(Value::ErrorObject { .. }) = self.return_value.clone() {
                        // Propagate error object - don't clear
                        self.return_value.clone().unwrap()
                    } else {
                        // No explicit return - function returns 0
                        self.return_value = None;
                        Value::Int(0)
                    };

                    // Restore parent environment
                    self.env.pop_scope();

                    // Pop from call stack
                    self.call_stack.pop();

                    result
                }
            }
            _ => Value::Int(0),
        }
    }

    /// Attempts to call an operator method on a struct value
    /// Returns Some(result) if the struct has the operator method, None otherwise
    fn try_call_operator_method(
        &mut self,
        struct_val: &Value,
        method_name: &str,
        other: &Value,
    ) -> Option<Value> {
        if let Value::Struct { name, fields } = struct_val {
            // Look up the struct definition to find the operator method
            if let Some(Value::StructDef { name: _, field_names: _, methods }) = self.env.get(name)
            {
                if let Some(Value::Function(params, body, _captured_env)) = methods.get(method_name)
                {
                    // Create new scope for operator method call
                    self.env.push_scope();

                    // Check if method has 'self' as first parameter
                    let has_self_param = params.first().map(|p| p == "self").unwrap_or(false);

                    if has_self_param {
                        // Bind self to the struct instance
                        self.env.define("self".to_string(), struct_val.clone());

                        // Bind the other operand as the second parameter (after self)
                        if let Some(param_name) = params.get(1) {
                            self.env.define(param_name.clone(), other.clone());
                        }
                    } else {
                        // Backward compatibility: bind fields directly into scope
                        for (field_name, field_value) in fields {
                            self.env.define(field_name.clone(), field_value.clone());
                        }

                        // Bind the other operand as the first parameter
                        if let Some(param_name) = params.first() {
                            self.env.define(param_name.clone(), other.clone());
                        }
                    }

                    // Execute method body
                    self.eval_stmts(body.get());

                    let result = if let Some(Value::Return(val)) = self.return_value.clone() {
                        self.return_value = None;
                        *val
                    } else {
                        self.return_value = None;
                        Value::Int(0)
                    };

                    // Restore parent environment
                    self.env.pop_scope();

                    return Some(result);
                }
            }
        }
        None
    }

    /// Attempts to call a unary operator method on a struct value
    /// Returns Some(result) if the struct has the operator method, None otherwise
    fn try_call_unary_operator_method(
        &mut self,
        struct_val: &Value,
        method_name: &str,
    ) -> Option<Value> {
        if let Value::Struct { name, fields } = struct_val {
            // Look up the struct definition to find the operator method
            if let Some(Value::StructDef { name: _, field_names: _, methods }) = self.env.get(name)
            {
                if let Some(Value::Function(params, body, _captured_env)) = methods.get(method_name)
                {
                    // Create new scope for operator method call
                    self.env.push_scope();

                    // Check if method has 'self' as first (and only) parameter
                    let has_self_param = params.first().map(|p| p == "self").unwrap_or(false);

                    if has_self_param {
                        // Bind self to the struct instance
                        self.env.define("self".to_string(), struct_val.clone());
                    } else {
                        // Backward compatibility: bind fields directly into scope
                        for (field_name, field_value) in fields {
                            self.env.define(field_name.clone(), field_value.clone());
                        }
                    }

                    // Execute method body
                    self.eval_stmts(body.get());

                    let result = if let Some(Value::Return(val)) = self.return_value.clone() {
                        self.return_value = None;
                        *val
                    } else {
                        self.return_value = None;
                        Value::Int(0)
                    };

                    // Restore parent environment
                    self.env.pop_scope();

                    return Some(result);
                }
            }
        }
        None
    }

    /// Matches a route pattern against a URL path, extracting path parameters
    /// Returns Some(HashMap) with extracted params if matched, None if no match
    fn match_route_pattern(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        // Must have same number of segments
        if pattern_parts.len() != path_parts.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if let Some(param_name) = pattern_part.strip_prefix(':') {
                // This is a path parameter - extract it
                params.insert(param_name.to_string(), path_part.to_string());
            } else if *pattern_part != *path_part {
                // Static segment doesn't match
                return None;
            }
        }

        Some(params)
    }

    /// Starts an HTTP server with registered routes
    fn start_http_server(&mut self, port: u16, routes: Vec<(String, String, Value)>) -> Value {
        use tiny_http::{Response, Server};

        println!("Starting HTTP server on port {}...", port);

        let server = match Server::http(format!("0.0.0.0:{}", port)) {
            Ok(s) => s,
            Err(e) => return Value::Error(format!("Failed to start server: {}", e)),
        };

        println!("Server listening on http://localhost:{}", port);
        println!("Press Ctrl+C to stop");

        // Main server loop
        for mut request in server.incoming_requests() {
            let method = request.method().to_string();
            let url_path = request.url().to_string();

            // Read body first (before any response handling)
            let body_content = {
                let mut reader = request.as_reader();
                let mut buffer = Vec::new();
                std::io::Read::read_to_end(&mut reader, &mut buffer).ok();
                String::from_utf8_lossy(&buffer).to_string()
            };

            // Find matching route (supports path parameters like /:code)
            // Exact matches take priority over parameterized routes
            let mut response_to_send: Option<Response<std::io::Cursor<Vec<u8>>>> = None;
            let mut matched_handler: Option<(&Value, HashMap<String, String>)> = None;

            // First pass: look for exact matches
            for (route_method, route_path, handler) in &routes {
                if method == *route_method && url_path == *route_path {
                    matched_handler = Some((handler, HashMap::new()));
                    break;
                }
            }

            // Second pass: if no exact match, try parameterized routes
            if matched_handler.is_none() {
                for (route_method, route_path, handler) in &routes {
                    if method != *route_method {
                        continue;
                    }
                    // Only try parameterized matching for routes with ':'
                    if route_path.contains(':') {
                        if let Some(path_params) = Self::match_route_pattern(route_path, &url_path)
                        {
                            matched_handler = Some((handler, path_params));
                            break;
                        }
                    }
                }
            }

            if let Some((handler, path_params)) = matched_handler {
                // Create params dict for request object
                let mut params_dict = DictMap::default();
                for (key, value) in &path_params {
                    params_dict
                        .insert(Arc::from(key.as_str()), Value::Str(Arc::new(value.clone())));
                }

                // Create request object as a Dict (not Struct) so has_key() and bracket access work
                let mut req_fields = DictMap::default();
                req_fields.insert("method".into(), Value::Str(Arc::new(method.clone())));
                req_fields.insert("path".into(), Value::Str(Arc::new(url_path.clone())));
                req_fields.insert("body".into(), Value::Str(Arc::new(body_content.clone())));
                req_fields.insert("params".into(), Value::Dict(Arc::new(params_dict)));

                // Extract headers from request
                let mut headers_dict = DictMap::default();
                for header in request.headers() {
                    let header_name = header.field.as_str().to_string();
                    let header_value = header.value.as_str().to_string();
                    headers_dict.insert(header_name.into(), Value::Str(Arc::new(header_value)));
                }
                req_fields.insert("headers".into(), Value::Dict(Arc::new(headers_dict)));

                let req_obj = Value::Dict(Arc::new(req_fields));

                // Call handler function
                if let Value::Function(params, body, _captured_env) = handler {
                    self.env.push_scope();

                    // Bind request parameter
                    if let Some(param) = params.first() {
                        self.env.define(param.clone(), req_obj);
                    }

                    self.eval_stmts(body.get());

                    // Get result
                    let result = if let Some(Value::Return(val)) = self.return_value.clone() {
                        self.return_value = None;
                        *val
                    } else {
                        self.return_value = None;
                        Value::HttpResponse {
                            status: 200,
                            body: "OK".to_string(),
                            headers: HashMap::new(),
                        }
                    };

                    self.env.pop_scope();

                    // Build response
                    if let Value::HttpResponse { status, body, headers } = result {
                        let mut response = Response::from_string(body);
                        response = response.with_status_code(status);

                        for (key, value) in headers {
                            if let Ok(header) =
                                tiny_http::Header::from_bytes(key.as_bytes(), value.as_bytes())
                            {
                                response = response.with_header(header);
                            }
                        }

                        response_to_send = Some(response);
                    } else {
                        // Handler didn't return HttpResponse
                        response_to_send = Some(
                            Response::from_string("Internal Server Error").with_status_code(500),
                        );
                    }
                }
            }

            // Send response
            if let Some(response) = response_to_send {
                let _ = request.respond(response);
            } else {
                // 404 Not Found
                let _ = request.respond(Response::from_string("Not Found").with_status_code(404));
            }
        }

        Value::Int(0)
    }
    /// Calls a native built-in function
    fn call_native_function(&mut self, name: &str, args: &[Expr]) -> Value {
        // Evaluate all arguments
        let arg_values: Vec<Value> = args.iter().map(|arg| self.eval_expr(arg)).collect();

        // Delegate to the implementation that works with Values
        self.call_native_function_impl(name, &arg_values)
    }

    /// Implementation of native function calling with pre-evaluated Value arguments
    /// This is used both by call_native_function (after evaluating Expr args)
    /// and by the VM (which already has Value args)
    pub fn call_native_function_impl(&mut self, name: &str, arg_values: &[Value]) -> Value {
        // Delegate to the native_functions module dispatcher
        native_functions::call_native_function(self, name, arg_values)
    }

    /// Helper method to check if two values are equal (for Set operations)
    fn values_equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => (x - y).abs() < f64::EPSILON,
            (Value::Str(x), Value::Str(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Null, Value::Null) => true,
            (Value::Array(x), Value::Array(y)) => {
                x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| Self::values_equal(a, b))
            }
            (Value::Dict(x), Value::Dict(y)) => {
                // Dictionaries are equal if they have the same keys with equal values
                if x.len() != y.len() {
                    return false;
                }
                x.iter().all(|(key, val)| y.get(key).map_or(false, |v| Self::values_equal(val, v)))
            }
            _ => false, // Different types or complex types not supported for equality
        }
    }

    /// Binds a pattern to a value, defining variables as needed
    fn bind_pattern(&mut self, pattern: &crate::ast::Pattern, value: Value) {
        use crate::ast::Pattern;

        match pattern {
            Pattern::Identifier(name) => {
                self.env.define(name.clone(), value);
            }
            Pattern::Ignore => {
                // Do nothing - value is discarded
            }
            Pattern::Array { elements, rest } => {
                // Extract array elements
                if let Value::Array(arr) = value {
                    let mut i = 0;
                    let arr_len = arr.len();

                    // Bind each pattern element
                    for pattern_elem in elements {
                        if i < arr_len {
                            self.bind_pattern(pattern_elem, arr[i].clone());
                            i += 1;
                        } else {
                            // Not enough elements - bind to null
                            self.bind_pattern(pattern_elem, Value::Null);
                        }
                    }

                    // Bind rest elements if present
                    if let Some(rest_name) = rest {
                        let rest_values: Vec<Value> =
                            if i < arr_len { arr[i..].to_vec() } else { vec![] };
                        self.env.define(rest_name.clone(), Value::Array(Arc::new(rest_values)));
                    }
                } else {
                    // Not an array - bind all patterns to null
                    for pattern_elem in elements {
                        self.bind_pattern(pattern_elem, Value::Null);
                    }
                    if let Some(rest_name) = rest {
                        self.env.define(rest_name.clone(), Value::Array(Arc::new(vec![])));
                    }
                }
            }
            Pattern::Dict { keys, rest } => {
                // Extract dict values
                if let Value::Dict(dict) = value {
                    // Bind each key
                    for key in keys {
                        let val = dict.get(key.as_str()).cloned().unwrap_or(Value::Null);
                        self.env.define(key.clone(), val);
                    }

                    // Bind rest elements if present
                    if let Some(rest_name) = rest {
                        let mut rest_dict = DictMap::default();
                        for (k, v) in dict.iter() {
                            if !keys.iter().any(|key| key.as_str() == k.as_ref()) {
                                rest_dict.insert(k.clone(), v.clone());
                            }
                        }
                        self.env.define(rest_name.clone(), Value::Dict(Arc::new(rest_dict)));
                    }
                } else if let Value::FixedDict { keys: dict_keys, values } = value {
                    for key in keys {
                        let idx = dict_keys.iter().position(|k| k.as_ref() == key.as_str());
                        let val = idx.and_then(|i| values.get(i).cloned()).unwrap_or(Value::Null);
                        self.env.define(key.clone(), val);
                    }

                    if let Some(rest_name) = rest {
                        let mut rest_dict = DictMap::default();
                        for (k, v) in dict_keys.iter().cloned().zip(values.iter().cloned()) {
                            if !keys.iter().any(|key| key.as_str() == k.as_ref()) {
                                rest_dict.insert(k, v);
                            }
                        }
                        self.env.define(rest_name.clone(), Value::Dict(Arc::new(rest_dict)));
                    }
                } else if let Value::DenseIntDict(values) = value {
                    for key in keys {
                        let val = match key.parse::<i64>() {
                            Ok(int_key) => {
                                if int_key < 0 {
                                    Value::Null
                                } else {
                                    values.get(int_key as usize).cloned().unwrap_or(Value::Null)
                                }
                            }
                            Err(_) => Value::Null,
                        };
                        self.env.define(key.clone(), val);
                    }

                    if let Some(rest_name) = rest {
                        let mut rest_dict = DictMap::default();
                        for (index, value) in values.iter().enumerate() {
                            let key = index.to_string();
                            if !keys.iter().any(|existing| existing.as_str() == key.as_str()) {
                                rest_dict.insert(Arc::from(key.as_str()), value.clone());
                            }
                        }
                        self.env.define(rest_name.clone(), Value::Dict(Arc::new(rest_dict)));
                    }
                } else if let Value::DenseIntDictInt(values) = value {
                    for key in keys {
                        let val = match key.parse::<i64>() {
                            Ok(int_key) => {
                                if int_key < 0 {
                                    Value::Null
                                } else {
                                    match values.get(int_key as usize) {
                                        Some(value) => {
                                            (*value).map(Value::Int).unwrap_or(Value::Null)
                                        }
                                        None => Value::Null,
                                    }
                                }
                            }
                            Err(_) => Value::Null,
                        };
                        self.env.define(key.clone(), val);
                    }

                    if let Some(rest_name) = rest {
                        let mut rest_dict = DictMap::default();
                        for (index, value) in values.iter().enumerate() {
                            let key = index.to_string();
                            if !keys.iter().any(|existing| existing.as_str() == key.as_str()) {
                                rest_dict.insert(
                                    Arc::from(key.as_str()),
                                    (*value).map(Value::Int).unwrap_or(Value::Null),
                                );
                            }
                        }
                        self.env.define(rest_name.clone(), Value::Dict(Arc::new(rest_dict)));
                    }
                } else if let Value::DenseIntDictIntFull(values) = value {
                    for key in keys {
                        let val = match key.parse::<i64>() {
                            Ok(int_key) => {
                                if int_key < 0 {
                                    Value::Null
                                } else {
                                    values
                                        .get(int_key as usize)
                                        .map(|value| Value::Int(*value))
                                        .unwrap_or(Value::Null)
                                }
                            }
                            Err(_) => Value::Null,
                        };
                        self.env.define(key.clone(), val);
                    }

                    if let Some(rest_name) = rest {
                        let mut rest_dict = DictMap::default();
                        for (index, value) in values.iter().enumerate() {
                            let key = index.to_string();
                            if !keys.iter().any(|existing| existing.as_str() == key.as_str()) {
                                rest_dict.insert(Arc::from(key.as_str()), Value::Int(*value));
                            }
                        }
                        self.env.define(rest_name.clone(), Value::Dict(Arc::new(rest_dict)));
                    }
                } else {
                    // Not a dict - bind all to null
                    for key in keys {
                        self.env.define(key.clone(), Value::Null);
                    }
                    if let Some(rest_name) = rest {
                        self.env
                            .define(rest_name.clone(), Value::Dict(Arc::new(DictMap::default())));
                    }
                }
            }
        }
    }

    /// Evaluates a list of statements sequentially, stopping on return/error
    pub fn eval_stmts(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.eval_stmt(stmt);
            if self.return_value.is_some() || self.control_flow != ControlFlow::None {
                break;
            }
        }
    }

    /// Public wrapper for evaluating a single statement (for REPL use)
    /// Returns an error if execution fails
    pub fn eval_stmt_repl(&mut self, stmt: &Stmt) -> Result<(), Box<RuffError>> {
        self.eval_stmt(stmt);

        // Check if an error occurred during evaluation
        if let Some(ref val) = self.return_value {
            match val {
                Value::Error(msg) => {
                    let err = RuffError::runtime_error(
                        msg.clone(),
                        crate::errors::SourceLocation::unknown(),
                    )
                    .with_call_stack(self.call_stack.clone());
                    self.return_value = None; // Clear error for next input
                    return Err(Box::new(err));
                }
                Value::ErrorObject { message, .. } => {
                    let err = RuffError::runtime_error(
                        message.clone(),
                        crate::errors::SourceLocation::unknown(),
                    )
                    .with_call_stack(self.call_stack.clone());
                    self.return_value = None; // Clear error for next input
                    return Err(Box::new(err));
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Public wrapper for evaluating an expression (for REPL use)
    /// Returns the evaluated value or an error
    pub fn eval_expr_repl(&mut self, expr: &Expr) -> Result<Value, Box<RuffError>> {
        let value = self.eval_expr(expr);

        // Check if the value is an error
        match value {
            Value::Error(msg) => Err(Box::new(
                RuffError::runtime_error(msg, crate::errors::SourceLocation::unknown())
                    .with_call_stack(self.call_stack.clone()),
            )),
            Value::ErrorObject { message, .. } => Err(Box::new(
                RuffError::runtime_error(message, crate::errors::SourceLocation::unknown())
                    .with_call_stack(self.call_stack.clone()),
            )),
            _ => Ok(value),
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
                let is_truthy = match cond_val {
                    Value::Bool(b) => b,
                    Value::Float(n) => n != 0.0,
                    Value::Str(s) => {
                        // Handle string representations of booleans for backward compatibility
                        if s.as_ref() == "true" {
                            true
                        } else if s.as_ref() == "false" {
                            false
                        } else {
                            !s.is_empty()
                        }
                    }
                    Value::Array(ref arr) => !arr.is_empty(),
                    Value::Dict(ref dict) => !dict.is_empty(),
                    _ => true, // Other values are truthy
                };

                if is_truthy {
                    self.eval_stmts(then_branch);
                } else if let Some(else_branch) = else_branch {
                    self.eval_stmts(else_branch);
                }
            }
            Stmt::Block(stmts) => {
                // Create new scope for block
                // Push new scope
                self.env.push_scope();

                self.eval_stmts(stmts);

                // Restore parent environment
                self.env.pop_scope();
            }
            Stmt::Let { pattern, value, mutable: _, type_annotation: _ } => {
                let val = self.eval_expr(value);
                // If expression evaluation resulted in an error, propagate it
                if matches!(val, Value::Error(_)) {
                    self.return_value = Some(val.clone());
                }
                self.bind_pattern(pattern, val);
            }
            Stmt::Const { name, value, type_annotation: _ } => {
                let val = self.eval_expr(value);
                // If expression evaluation resulted in an error, propagate it
                if matches!(val, Value::Error(_)) {
                    self.return_value = Some(val.clone());
                }
                self.env.define(name.clone(), val);
            }
            Stmt::Assign { target, value } => {
                let val = self.eval_expr(value);

                // Always perform the assignment, even for errors
                match target {
                    Expr::Identifier(name) => {
                        // Simple variable assignment - use set to update in correct scope
                        self.env.set(name.clone(), val.clone());
                    }
                    Expr::IndexAccess { object, index } => {
                        // Array or dict element assignment
                        let index_val = self.eval_expr(index);

                        // Get the container (array or dict) from the object expression
                        // For now, only support direct identifiers as the object
                        if let Expr::Identifier(container_name) = object.as_ref() {
                            let val_clone = val.clone();
                            let idx_clone = index_val.clone();
                            self.env.mutate(container_name.as_str(), |container| match container {
                                Value::Array(arr) => {
                                    let i = match &idx_clone {
                                        Value::Int(n) => *n as usize,
                                        Value::Float(n) => *n as usize,
                                        _ => return,
                                    };
                                    let arr_mut = Arc::make_mut(arr);
                                    if i < arr_mut.len() {
                                        arr_mut[i] = val_clone.clone();
                                    } else {
                                        eprintln!("Array index out of bounds: {}", i);
                                    }
                                }
                                Value::Dict(dict) => {
                                    let key = Self::stringify_value(&idx_clone);
                                    Arc::make_mut(dict).insert(key.into(), val_clone.clone());
                                }
                                _ => {
                                    eprintln!("Cannot index non-collection type");
                                }
                            });
                        } else {
                            eprintln!("Complex index assignment not yet supported");
                        }
                    }
                    Expr::FieldAccess { object, field } => {
                        // Field assignment like obj.field or arr[0].field
                        // We need to evaluate the object, update the field, then assign it back

                        // Handle different types of object expressions
                        match object.as_ref() {
                            Expr::Identifier(name) => {
                                // Direct field assignment: obj.field := value
                                let field_clone = field.clone();
                                let val_clone = val.clone();
                                self.env.mutate(name.as_str(), |obj_val| {
                                    if let Value::Struct { name: _, fields } = obj_val {
                                        fields.insert(field_clone, val_clone);
                                    } else {
                                        eprintln!("Cannot access field on non-struct type");
                                    }
                                });
                            }
                            Expr::IndexAccess { object: index_obj, index } => {
                                // Array/dict element field assignment: arr[0].field := value
                                let index_val = self.eval_expr(index);

                                if let Expr::Identifier(container_name) = index_obj.as_ref() {
                                    let field_clone = field.clone();
                                    let val_clone = val.clone();
                                    let idx_clone = index_val.clone();

                                    self.env.mutate(container_name.as_str(), |container| {
                                        match container {
                                            Value::Array(arr) => {
                                                let i = match &idx_clone {
                                                    Value::Int(n) => *n as usize,
                                                    Value::Float(n) => *n as usize,
                                                    _ => return,
                                                };
                                                let arr_mut = Arc::make_mut(arr);
                                                if i < arr_mut.len() {
                                                    if let Value::Struct { name: _, fields } =
                                                        &mut arr_mut[i]
                                                    {
                                                        fields.insert(field_clone, val_clone);
                                                    } else {
                                                        eprintln!("Array element is not a struct");
                                                    }
                                                } else {
                                                    eprintln!("Array index out of bounds: {}", i);
                                                }
                                            }
                                            Value::Dict(dict) => {
                                                let key = Self::stringify_value(&idx_clone);
                                                if let Some(Value::Struct { name: _, fields }) =
                                                    Arc::make_mut(dict).get_mut(key.as_str())
                                                {
                                                    fields.insert(field_clone, val_clone);
                                                } else {
                                                    eprintln!("Dict value is not a struct");
                                                }
                                            }
                                            _ => {
                                                eprintln!("Cannot index non-collection type");
                                            }
                                        }
                                    });
                                }
                            }
                            _ => {
                                eprintln!("Complex field assignment not yet supported");
                            }
                        }
                    }
                    _ => {
                        eprintln!("Invalid assignment target");
                    }
                }

                // If expression evaluation resulted in an error, propagate it
                if matches!(val, Value::Error(_)) {
                    self.return_value = Some(val);
                }
            }
            Stmt::FuncDef {
                name,
                params,
                param_types: _,
                return_type: _,
                body,
                is_generator,
                is_async,
            } => {
                // Regular functions don't capture environment - they use the environment at call time
                // Only lambda expressions (closures) should capture environment

                // If it's a generator, create a generator value instead
                if *is_generator {
                    let gen =
                        Value::GeneratorDef(params.clone(), LeakyFunctionBody::new(body.clone()));
                    self.env.define(name.clone(), gen);
                } else if *is_async {
                    // Async functions are marked with a flag
                    // When called, they return a Promise and execute in background
                    let func = Value::AsyncFunction(
                        params.clone(),
                        LeakyFunctionBody::new(body.clone()),
                        None, // No captured environment for function definitions
                    );
                    self.env.define(name.clone(), func);
                } else {
                    let func = Value::Function(
                        params.clone(),
                        LeakyFunctionBody::new(body.clone()),
                        None, // No captured environment for regular function definitions
                    );
                    self.env.define(name.clone(), func);
                }
            }
            Stmt::EnumDef { name, variants } => {
                for variant in variants {
                    let tag = format!("{}::{}", name, variant);
                    // Store constructor function in env
                    let func = Value::Function(
                        vec!["$0".to_string()],
                        LeakyFunctionBody::new(vec![Stmt::Return(Some(Expr::Tag(
                            tag.clone(),
                            vec![Expr::Identifier("$0".to_string())],
                        )))]),
                        None, // Enum constructors don't need closure
                    );
                    self.env.define(tag.clone(), func);
                }
            }
            Stmt::Import { module, symbols } => {
                // Load the module
                match symbols {
                    None => {
                        // Import entire module: import math
                        // Load all exports into the current namespace
                        match self.module_loader.get_all_exports(module) {
                            Ok(exports) => {
                                for (name, value) in exports {
                                    self.env.define(name, value);
                                }
                            }
                            Err(_) => {
                                // Module not found or error loading - silently continue for now
                                // In production, should report error
                            }
                        }
                    }
                    Some(symbol_list) => {
                        // Selective import: from math import add, sub
                        for symbol_name in symbol_list {
                            match self.module_loader.get_symbol(module, symbol_name) {
                                Ok(value) => {
                                    self.env.define(symbol_name.clone(), value);
                                }
                                Err(_) => {
                                    // Symbol not found - silently continue for now
                                }
                            }
                        }
                    }
                }
            }
            Stmt::Export { stmt } => {
                // Export is metadata for module system - execute the inner statement
                self.eval_stmt(stmt);
            }
            Stmt::Match { value, cases, default } => {
                let val = self.eval_expr(value);

                // Clone cases and default to avoid borrow issues during evaluation
                let cases_clone = cases.clone();
                let default_clone = default.clone();

                // Handle Result and Option types specially by extracting data first
                let (is_result_or_option, tag_str, extracted_value) = match &val {
                    Value::Result { is_ok, value } => {
                        let tag = if *is_ok { "Ok" } else { "Err" };
                        (true, tag.to_string(), Some((**value).clone()))
                    }
                    Value::Option { is_some, value } => {
                        let tag = if *is_some { "Some" } else { "None" };
                        let val = if *is_some { Some((**value).clone()) } else { None };
                        (true, tag.to_string(), val)
                    }
                    _ => (false, String::new(), None),
                };

                if is_result_or_option {
                    // Match against Result or Option
                    for (pattern, body) in cases_clone.iter() {
                        if let Some(open_paren) = pattern.find('(') {
                            let (enum_tag, param_var) = pattern.split_at(open_paren);
                            let param_var = param_var.trim_matches(&['(', ')'][..]);
                            if tag_str == enum_tag.trim() {
                                // Create new scope for pattern match body
                                self.env.push_scope();
                                if let Some(val) = extracted_value {
                                    self.env.define(param_var.to_string(), val);
                                }
                                self.eval_stmts(body);
                                self.env.pop_scope();
                                return;
                            }
                        } else if pattern.as_str() == tag_str {
                            self.eval_stmts(body);
                            return;
                        }
                    }

                    if let Some(default_body) = default_clone {
                        self.eval_stmts(&default_body);
                    }
                    return;
                }

                // Handle other value types (existing code)
                let empty_map = HashMap::new();
                let (tag, fields): (String, &HashMap<String, Value>) = match &val {
                    Value::Tagged { tag, fields } => (tag.clone(), fields),
                    Value::Enum(e) => (e.clone(), &empty_map),
                    Value::Str(s) => (s.as_ref().clone(), &empty_map),
                    Value::Float(n) => (n.to_string(), &empty_map),
                    _ => {
                        if let Some(default_body) = default_clone {
                            self.eval_stmts(&default_body);
                        }
                        return;
                    }
                };

                for (pattern, body) in &cases_clone {
                    if let Some(open_paren) = pattern.find('(') {
                        let (enum_tag, param_var) = pattern.split_at(open_paren);
                        let param_var = param_var.trim_matches(&['(', ')'][..]);
                        if tag == enum_tag.trim() {
                            // Create new scope for pattern match body
                            // Push new scope
                            self.env.push_scope();

                            for i in 0.. {
                                let key = format!("${}", i);
                                if let Some(val) = fields.get(&key) {
                                    let param_name = if i == 0 {
                                        param_var.to_string()
                                    } else {
                                        format!("{}_{}", param_var, i)
                                    };
                                    self.env.define(param_name, val.clone());
                                } else {
                                    break;
                                }
                            }

                            self.eval_stmts(body);

                            // Restore parent environment
                            self.env.pop_scope();
                            return;
                        }
                    } else if pattern.as_str() == tag {
                        self.eval_stmts(body);
                        return;
                    }
                }

                if let Some(default_body) = default_clone {
                    self.eval_stmts(&default_body);
                }
            }
            Stmt::Loop { condition, body } => {
                while condition
                    .as_ref()
                    .map(|c| matches!(self.eval_expr(c), Value::Float(n) if n != 0.0))
                    .unwrap_or(true)
                {
                    self.eval_stmts(body);

                    // Handle control flow
                    if self.control_flow == ControlFlow::Break {
                        self.control_flow = ControlFlow::None;
                        break;
                    } else if self.control_flow == ControlFlow::Continue {
                        self.control_flow = ControlFlow::None;
                        continue;
                    }

                    if self.return_value.is_some() {
                        break;
                    }
                }
            }
            Stmt::For { var, iterable, body } => {
                let mut iterable_value = self.eval_expr(iterable);

                // If we got a GeneratorDef, call it to get a Generator instance
                // This handles cases like: for x in generator_func() { ... }
                if let Value::GeneratorDef(_, _) = &iterable_value {
                    iterable_value = self.call_user_function(&iterable_value, &[]);
                }

                // Check if this is a generator and handle it separately (needs to be mut)
                if matches!(&iterable_value, Value::Generator { .. }) {
                    let mut gen_value = iterable_value;
                    loop {
                        let next_option = self.generator_next(&mut gen_value);
                        match next_option {
                            Value::Option { is_some: true, value } => {
                                // Got a value from generator
                                self.env.push_scope();
                                self.env.define(var.clone(), *value);

                                self.eval_stmts(body);

                                self.env.pop_scope();

                                // Handle control flow
                                if self.control_flow == ControlFlow::Break {
                                    self.control_flow = ControlFlow::None;
                                    break;
                                } else if self.control_flow == ControlFlow::Continue {
                                    self.control_flow = ControlFlow::None;
                                    continue;
                                }

                                if self.return_value.is_some() {
                                    break;
                                }
                            }
                            Value::Option { is_some: false, .. } => {
                                // Generator exhausted
                                break;
                            }
                            Value::Error(msg) => {
                                eprintln!("Error iterating generator: {}", msg);
                                break;
                            }
                            _ => {
                                eprintln!("Unexpected value from generator iteration");
                                break;
                            }
                        }
                    }
                    return;
                }

                match &iterable_value {
                    Value::Int(n) => {
                        // Numeric range: for i in 5 { ... } iterates 0..5
                        for i in 0..*n {
                            // Create new scope for loop iteration
                            // Push new scope
                            self.env.push_scope();
                            self.env.define(var.clone(), Value::Int(i));

                            self.eval_stmts(body);

                            // Restore parent environment
                            self.env.pop_scope();

                            // Handle control flow
                            if self.control_flow == ControlFlow::Break {
                                self.control_flow = ControlFlow::None;
                                break;
                            } else if self.control_flow == ControlFlow::Continue {
                                self.control_flow = ControlFlow::None;
                                continue;
                            }

                            if self.return_value.is_some() {
                                break;
                            }
                        }
                    }
                    Value::Float(n) => {
                        // Numeric range: for i in 5.0 { ... } iterates 0..5
                        for i in 0..*n as i64 {
                            // Create new scope for loop iteration
                            // Push new scope
                            self.env.push_scope();
                            self.env.define(var.clone(), Value::Int(i));

                            self.eval_stmts(body);

                            // Restore parent environment
                            self.env.pop_scope();

                            // Handle control flow
                            if self.control_flow == ControlFlow::Break {
                                self.control_flow = ControlFlow::None;
                                break;
                            } else if self.control_flow == ControlFlow::Continue {
                                self.control_flow = ControlFlow::None;
                                continue;
                            }

                            if self.return_value.is_some() {
                                break;
                            }
                        }
                    }
                    Value::Array(arr) => {
                        // Array iteration: for item in [1, 2, 3] { ... }
                        let arr_clone = arr.as_ref().clone();
                        for item in arr_clone {
                            // Create new scope for loop iteration
                            // Push new scope
                            self.env.push_scope();
                            self.env.define(var.clone(), item);

                            self.eval_stmts(body);

                            // Restore parent environment
                            self.env.pop_scope();

                            // Handle control flow
                            if self.control_flow == ControlFlow::Break {
                                self.control_flow = ControlFlow::None;
                                break;
                            } else if self.control_flow == ControlFlow::Continue {
                                self.control_flow = ControlFlow::None;
                                continue;
                            }

                            if self.return_value.is_some() {
                                break;
                            }
                        }
                    }
                    Value::Dict(dict) => {
                        // Dictionary iteration: for key in {"a": 1, "b": 2} { ... }
                        // Iterate over keys
                        let keys: Vec<String> = dict.keys().map(|key| key.to_string()).collect();
                        for key in keys {
                            // Create new scope for loop iteration
                            // Push new scope
                            self.env.push_scope();
                            self.env.define(var.clone(), Value::Str(Arc::new(key)));

                            self.eval_stmts(body);

                            // Restore parent environment
                            self.env.pop_scope();

                            // Handle control flow
                            if self.control_flow == ControlFlow::Break {
                                self.control_flow = ControlFlow::None;
                                break;
                            } else if self.control_flow == ControlFlow::Continue {
                                self.control_flow = ControlFlow::None;
                                continue;
                            }

                            if self.return_value.is_some() {
                                break;
                            }
                        }
                    }
                    Value::Str(s) => {
                        // String iteration: for char in "hello" { ... }
                        let chars: Vec<char> = s.chars().collect();
                        for ch in chars {
                            // Create new scope for loop iteration
                            // Push new scope
                            self.env.push_scope();
                            self.env.define(var.clone(), Value::Str(Arc::new(ch.to_string())));

                            self.eval_stmts(body);

                            // Restore parent environment
                            self.env.pop_scope();

                            // Handle control flow
                            if self.control_flow == ControlFlow::Break {
                                self.control_flow = ControlFlow::None;
                                break;
                            } else if self.control_flow == ControlFlow::Continue {
                                self.control_flow = ControlFlow::None;
                                continue;
                            }

                            if self.return_value.is_some() {
                                break;
                            }
                        }
                    }
                    _ => {
                        eprintln!("Cannot iterate over non-iterable type");
                    }
                }
            }
            Stmt::While { condition, body } => {
                // While loop: execute body while condition is truthy
                loop {
                    let cond_val = self.eval_expr(condition);
                    let is_truthy = match cond_val {
                        Value::Bool(b) => b,
                        Value::Float(n) => n != 0.0,
                        Value::Str(s) => {
                            if s.as_ref() == "true" {
                                true
                            } else if s.as_ref() == "false" {
                                false
                            } else {
                                !s.is_empty()
                            }
                        }
                        Value::Array(ref arr) => !arr.is_empty(),
                        Value::Dict(ref dict) => !dict.is_empty(),
                        _ => true,
                    };

                    if !is_truthy {
                        break;
                    }

                    self.eval_stmts(body);

                    // Handle control flow
                    if self.control_flow == ControlFlow::Break {
                        self.control_flow = ControlFlow::None;
                        break;
                    } else if self.control_flow == ControlFlow::Continue {
                        self.control_flow = ControlFlow::None;
                        continue;
                    }

                    if self.return_value.is_some() {
                        break;
                    }
                }
            }
            Stmt::Break => {
                self.control_flow = ControlFlow::Break;
            }
            Stmt::Continue => {
                self.control_flow = ControlFlow::Continue;
            }
            Stmt::Return(expr) => {
                let value = expr.as_ref().map(|e| self.eval_expr(e)).unwrap_or(Value::Int(0));
                self.return_value = Some(Value::Return(Box::new(value)));
            }
            Stmt::TryExcept { try_block, except_var, except_block } => {
                // Save current environment and create child scope for try block
                // Push new scope
                self.env.push_scope();

                self.eval_stmts(try_block);

                // Check if an error occurred (support both old Error and new ErrorObject)
                let error_occurred = matches!(
                    self.return_value,
                    Some(Value::Error(_)) | Some(Value::ErrorObject { .. })
                );

                if error_occurred {
                    let error_value = self.return_value.clone().unwrap();

                    // Pop try scope and create new scope for except block
                    self.env.pop_scope();
                    self.env.push_scope();

                    // Create error object with properties accessible via field access
                    match error_value {
                        Value::Error(msg) => {
                            // Legacy simple error - convert to struct-like object
                            let mut fields = HashMap::new();
                            fields.insert("message".to_string(), Value::Str(Arc::new(msg)));
                            fields.insert("stack".to_string(), Value::Array(Arc::new(Vec::new())));
                            fields.insert("line".to_string(), Value::Int(0));

                            self.env.define(
                                except_var.clone(),
                                Value::Struct { name: "Error".to_string(), fields },
                            );
                        }
                        Value::ErrorObject { message, stack, line, cause } => {
                            // New error object with full info
                            let mut fields = HashMap::new();
                            fields.insert("message".to_string(), Value::Str(Arc::new(message)));
                            fields.insert(
                                "stack".to_string(),
                                Value::Array(Arc::new(
                                    stack.iter().map(|s| Value::Str(Arc::new(s.clone()))).collect(),
                                )),
                            );
                            fields.insert("line".to_string(), Value::Int(line.unwrap_or(0) as i64));
                            if let Some(cause_val) = cause {
                                fields.insert("cause".to_string(), *cause_val);
                            }

                            self.env.define(
                                except_var.clone(),
                                Value::Struct { name: "Error".to_string(), fields },
                            );
                        }
                        _ => unreachable!(),
                    }

                    // Clear error and execute except block
                    self.return_value = None;
                    self.eval_stmts(except_block);
                }

                // Restore parent environment
                self.env.pop_scope();
            }
            Stmt::ExprStmt(expr) => {
                match expr {
                    // built-in print
                    Expr::Tag(name, args) if name == "print" => {
                        let output_parts: Vec<String> = args
                            .iter()
                            .map(|arg| {
                                let v = self.eval_expr(arg);
                                Interpreter::stringify_value(&v)
                            })
                            .collect();
                        self.write_output(&output_parts.join(" "));
                    }

                    // built-in throw
                    Expr::Tag(name, args) if name == "throw" => {
                        if let Some(arg) = args.first() {
                            let val = self.eval_expr(arg);
                            match val {
                                Value::Str(s) => {
                                    // Simple string error - create ErrorObject
                                    self.return_value = Some(Value::ErrorObject {
                                        message: s.as_ref().clone(),
                                        stack: self.call_stack.clone(),
                                        line: None,
                                        cause: None,
                                    });
                                }
                                Value::Struct { name, fields } => {
                                    // Custom error struct - wrap it in ErrorObject
                                    let message =
                                        if let Some(Value::Str(msg)) = fields.get("message") {
                                            msg.as_ref().clone()
                                        } else {
                                            format!("{} error", name)
                                        };

                                    let cause = fields.get("cause").cloned();

                                    self.return_value = Some(Value::ErrorObject {
                                        message,
                                        stack: self.call_stack.clone(),
                                        line: None,
                                        cause: cause.map(Box::new),
                                    });
                                }
                                Value::ErrorObject { .. } => {
                                    // Already an error object, propagate it
                                    self.return_value = Some(val);
                                }
                                _ => {
                                    self.return_value = Some(Value::ErrorObject {
                                        message: "error".to_string(),
                                        stack: self.call_stack.clone(),
                                        line: None,
                                        cause: None,
                                    });
                                }
                            }
                        }
                    }

                    // enum constructors or user functions (tags)
                    Expr::Tag(_, _) => {
                        let _ = self.eval_expr(expr);
                    }

                    // everything else (including Call expressions and yield)
                    _ => {
                        let result = self.eval_expr(expr);
                        // Check if this is a yield (signaled by Return value)
                        if matches!(result, Value::Return(_)) {
                            self.return_value = Some(result);
                        }
                    }
                }
            }
            Stmt::Spawn { body } => {
                // Clone the body for the spawned thread
                let body_clone = body.clone();
                let captured_bindings = self.capture_spawn_bindings();

                // Spawn a new thread to execute the body with a transferable snapshot
                // of parent bindings. Unsupported non-transferable values remain isolated.
                std::thread::spawn(move || {
                    let mut thread_interp = Interpreter::new();

                    for (name, captured_value) in captured_bindings {
                        thread_interp.env.define(name, captured_value.into_value());
                    }

                    thread_interp.eval_stmts(&body_clone);
                });
                // Don't wait for the thread to finish - it runs in the background
            }
            Stmt::Test { .. }
            | Stmt::TestSetup { .. }
            | Stmt::TestTeardown { .. }
            | Stmt::TestGroup { .. } => {
                // Test statements are collected and executed by the test runner
                // When running normally (not in test mode), they are no-ops
                // This allows test files to be syntax-checked without running tests
            }
            Stmt::StructDef { name, fields, methods } => {
                // Extract field names
                let field_names: Vec<String> =
                    fields.iter().map(|(name, _type)| name.clone()).collect();

                // Store methods
                let mut method_map = HashMap::new();
                for method_stmt in methods {
                    if let Stmt::FuncDef {
                        name: method_name,
                        params,
                        param_types: _,
                        return_type: _,
                        body,
                        is_generator,
                        is_async: _,
                    } = method_stmt
                    {
                        if *is_generator {
                            // Generators not supported as methods yet
                            let gen = Value::GeneratorDef(
                                params.clone(),
                                LeakyFunctionBody::new(body.clone()),
                            );
                            method_map.insert(method_name.clone(), gen);
                        } else {
                            let func = Value::Function(
                                params.clone(),
                                LeakyFunctionBody::new(body.clone()),
                                Some(Arc::new(Mutex::new(self.env.clone()))),
                            );
                            method_map.insert(method_name.clone(), func);
                        }
                    }
                }

                // Store struct definition
                let struct_def =
                    Value::StructDef { name: name.clone(), field_names, methods: method_map };
                self.env.define(name.clone(), struct_def);
            }
        }
    }

    /// Evaluates an expression to produce a value
    fn eval_expr(&mut self, expr: &Expr) -> Value {
        let result = match expr {
            Expr::Int(n) => Value::Int(*n),
            Expr::Float(n) => Value::Float(*n),
            Expr::String(s) => Value::Str(Arc::new(s.clone())),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::InterpolatedString(parts) => {
                use crate::ast::InterpolatedStringPart;
                let mut result = String::new();
                for part in parts {
                    match part {
                        InterpolatedStringPart::Text(text) => {
                            result.push_str(text);
                        }
                        InterpolatedStringPart::Expr(expr) => {
                            let val = self.eval_expr(expr);
                            result.push_str(&Self::stringify_value(&val));
                        }
                    }
                }
                Value::Str(Arc::new(result))
            }
            Expr::Identifier(name) => {
                if name == "null" {
                    Value::Null
                } else {
                    self.env.get(name).unwrap_or(Value::Str(Arc::new(name.clone())))
                }
            }
            Expr::Function {
                params,
                param_types: _,
                return_type: _,
                body,
                is_generator,
                is_async,
            } => {
                // Anonymous function expression - return as a value with captured environment
                if *is_generator {
                    Value::GeneratorDef(params.clone(), LeakyFunctionBody::new(body.clone()))
                } else if *is_async {
                    Value::AsyncFunction(
                        params.clone(),
                        LeakyFunctionBody::new(body.clone()),
                        Some(Arc::new(Mutex::new(self.env.clone()))),
                    )
                } else {
                    Value::Function(
                        params.clone(),
                        LeakyFunctionBody::new(body.clone()),
                        Some(Arc::new(Mutex::new(self.env.clone()))),
                    )
                }
            }
            Expr::UnaryOp { op, operand } => {
                let val = self.eval_expr(operand);

                // Check if the operand is a struct with an unary operator method
                if let Some(method_name) = crate::ast::operator_methods::unary_op_method(op) {
                    if let Some(result) = self.try_call_unary_operator_method(&val, method_name) {
                        return result;
                    }
                }

                // Default behavior for built-in types
                match (op.as_str(), val) {
                    ("-", Value::Float(n)) => Value::Float(-n),
                    ("!", Value::Bool(b)) => Value::Bool(!b),
                    _ => Value::Int(0), // Default for unsupported operations
                }
            }
            Expr::BinaryOp { left, op, right } => {
                // Handle special operators that need custom evaluation
                match op.as_str() {
                    // Null coalescing: return left if not null, otherwise right
                    "??" => {
                        let l = self.eval_expr(left);
                        if matches!(l, Value::Null) {
                            return self.eval_expr(right);
                        }
                        return l;
                    }
                    // Optional chaining: return null if left is null, otherwise access field
                    "?." => {
                        let l = self.eval_expr(left);
                        if matches!(l, Value::Null) {
                            return Value::Null;
                        }
                        // Right side is a String containing the field name
                        if let Expr::String(field_name) = right.as_ref() {
                            // Handle different value types
                            match l {
                                Value::Struct { name: _, fields } => {
                                    return fields.get(field_name).cloned().unwrap_or(Value::Null);
                                }
                                Value::Dict(map) => {
                                    return map
                                        .get(field_name.as_str())
                                        .cloned()
                                        .unwrap_or(Value::Null);
                                }
                                _ => return Value::Null,
                            }
                        }
                        return Value::Null;
                    }
                    // Pipe operator: pass left value as first argument to right function
                    "|>" => {
                        let value = self.eval_expr(left);
                        let func = self.eval_expr(right);

                        // Call the function with the value as the first argument
                        if let Value::Function(params, body, captured_env) = func {
                            // Push new scope
                            self.env.push_scope();

                            // Restore captured environment if this is a closure
                            let restore_env = if let Some(ref closure_env) = captured_env {
                                // Store current environment
                                let current = self.env.clone();
                                // Set interpreter's environment to the closure's captured environment
                                self.env = closure_env.lock().unwrap().clone();
                                Some(current)
                            } else {
                                None
                            };

                            // Bind the piped value as the first parameter
                            if let Some(param) = params.first() {
                                self.env.define(param.clone(), value);
                            }

                            // Execute function body
                            self.eval_stmts(body.get());
                            let mut result = Value::Int(0);
                            if let Some(Value::Return(val)) = self.return_value.clone() {
                                self.return_value = None;
                                result = *val;
                            }

                            // Restore environment if we changed it
                            if let Some(env) = restore_env {
                                self.env = env;
                            }

                            // Pop scope
                            self.env.pop_scope();

                            return result;
                        } else if let Value::NativeFunction(ref name) = func {
                            // Handle built-in functions
                            // Create a simple expression for the value and call the native function
                            let arg_expr = match value {
                                Value::Int(n) => Expr::Int(n),
                                Value::Float(n) => Expr::Float(n),
                                Value::Str(s) => Expr::String(s.as_ref().clone()),
                                Value::Bool(b) => Expr::Bool(b),
                                _ => {
                                    return Value::Error(
                                        "Cannot pipe this value type to native function"
                                            .to_string(),
                                    )
                                }
                            };
                            return self.call_native_function(name, &[arg_expr]);
                        }

                        return Value::Error(
                            "Pipe operator requires a function on the right side".to_string(),
                        );
                    }
                    _ => {}
                }

                let l = self.eval_expr(left);
                let r = self.eval_expr(right);

                // Check if left operand is a struct with an operator method
                if let Some(method_name) = crate::ast::operator_methods::binary_op_method(op) {
                    if let Some(result) = self.try_call_operator_method(&l, method_name, &r) {
                        return result;
                    }
                }

                // Default behavior for built-in types
                match (l, r) {
                    // Int + Int = Int (preserve integer type)
                    (Value::Int(a), Value::Int(b)) => match op.as_str() {
                        "+" => Value::Int(a.wrapping_add(b)),
                        "-" => Value::Int(a.wrapping_sub(b)),
                        "*" => Value::Int(a.wrapping_mul(b)),
                        "/" => {
                            if b == 0 {
                                Value::Error("Division by zero".to_string())
                            } else {
                                Value::Int(a / b) // Integer division: 5/2 = 2
                            }
                        }
                        "%" => {
                            if b == 0 {
                                Value::Error("Modulo by zero".to_string())
                            } else {
                                Value::Int(a % b)
                            }
                        }
                        "==" => Value::Bool(a == b),
                        "!=" => Value::Bool(a != b),
                        ">" => Value::Bool(a > b),
                        "<" => Value::Bool(a < b),
                        ">=" => Value::Bool(a >= b),
                        "<=" => Value::Bool(a <= b),
                        _ => Value::Int(0),
                    },
                    // Float + Float = Float
                    (Value::Float(a), Value::Float(b)) => match op.as_str() {
                        "+" => Value::Float(a + b),
                        "-" => Value::Float(a - b),
                        "*" => Value::Float(a * b),
                        "/" => Value::Float(a / b),
                        "%" => Value::Float(a % b),
                        "==" => Value::Bool((a - b).abs() < f64::EPSILON),
                        "!=" => Value::Bool((a - b).abs() >= f64::EPSILON),
                        ">" => Value::Bool(a > b),
                        "<" => Value::Bool(a < b),
                        ">=" => Value::Bool(a >= b),
                        "<=" => Value::Bool(a <= b),
                        _ => Value::Int(0),
                    },
                    // Int + Float = Float (promote to float)
                    (Value::Int(a), Value::Float(b)) => {
                        let a_float = a as f64;
                        match op.as_str() {
                            "+" => Value::Float(a_float + b),
                            "-" => Value::Float(a_float - b),
                            "*" => Value::Float(a_float * b),
                            "/" => Value::Float(a_float / b),
                            "%" => Value::Float(a_float % b),
                            "==" => Value::Bool((a_float - b).abs() < f64::EPSILON),
                            "!=" => Value::Bool((a_float - b).abs() >= f64::EPSILON),
                            ">" => Value::Bool(a_float > b),
                            "<" => Value::Bool(a_float < b),
                            ">=" => Value::Bool(a_float >= b),
                            "<=" => Value::Bool(a_float <= b),
                            _ => Value::Int(0),
                        }
                    }
                    // Float + Int = Float (promote to float)
                    (Value::Float(a), Value::Int(b)) => {
                        let b_float = b as f64;
                        match op.as_str() {
                            "+" => Value::Float(a + b_float),
                            "-" => Value::Float(a - b_float),
                            "*" => Value::Float(a * b_float),
                            "/" => Value::Float(a / b_float),
                            "%" => Value::Float(a % b_float),
                            "==" => Value::Bool((a - b_float).abs() < f64::EPSILON),
                            "!=" => Value::Bool((a - b_float).abs() >= f64::EPSILON),
                            ">" => Value::Bool(a > b_float),
                            "<" => Value::Bool(a < b_float),
                            ">=" => Value::Bool(a >= b_float),
                            "<=" => Value::Bool(a <= b_float),
                            _ => Value::Int(0),
                        }
                    }
                    (Value::Str(a), Value::Str(b)) => match op.as_str() {
                        "+" => {
                            let mut result = a.clone();
                            let result_str = Arc::make_mut(&mut result);
                            result_str.push_str(b.as_ref());
                            Value::Str(result)
                        }
                        "==" => Value::Bool(a.as_ref() == b.as_ref()),
                        "!=" => Value::Bool(a.as_ref() != b.as_ref()),
                        _ => Value::Int(0),
                    },
                    (Value::Bool(a), Value::Bool(b)) => match op.as_str() {
                        "==" => Value::Bool(a == b),
                        "!=" => Value::Bool(a != b),
                        "&&" => Value::Bool(a && b),
                        "||" => Value::Bool(a || b),
                        _ => Value::Int(0),
                    },
                    _ => Value::Int(0),
                }
            }
            Expr::Call { function, args } => {
                // Special handling for method calls: obj.method(args)
                if let Expr::FieldAccess { object, field } = function.as_ref() {
                    let obj_val = self.eval_expr(object);

                    // Handle HttpServer methods
                    if let Value::HttpServer { port, routes } = &obj_val {
                        match field.as_str() {
                            "route" => {
                                // server.route(method, path, handler)
                                if args.len() >= 3 {
                                    let method_val = self.eval_expr(&args[0]);
                                    let path_val = self.eval_expr(&args[1]);
                                    let handler_val = self.eval_expr(&args[2]);

                                    if let (
                                        Value::Str(method),
                                        Value::Str(path),
                                        Value::Function(_, _, _),
                                    ) = (&method_val, &path_val, &handler_val)
                                    {
                                        let mut new_routes = routes.clone();
                                        new_routes.push((
                                            method.as_ref().clone(),
                                            path.as_ref().clone(),
                                            handler_val,
                                        ));
                                        return Value::HttpServer {
                                            port: *port,
                                            routes: new_routes,
                                        };
                                    }
                                }
                                return Value::Error(
                                    "route() requires (method, path, handler_function)".to_string(),
                                );
                            }
                            "listen" => {
                                // server.listen() - start the HTTP server
                                return self.start_http_server(*port, routes.clone());
                            }
                            _ => {}
                        }
                    }

                    // Handle Image methods
                    if let Value::Image { data, format } = &obj_val {
                        match field.as_str() {
                            "resize" => {
                                // img.resize(width, height) or img.resize(width, height, "fit")
                                if args.len() < 2 {
                                    return Value::Error(
                                        "resize requires at least width and height arguments"
                                            .to_string(),
                                    );
                                }

                                let width_val = self.eval_expr(&args[0]);
                                let height_val = self.eval_expr(&args[1]);

                                if let (Value::Float(w), Value::Float(h)) = (width_val, height_val)
                                {
                                    let width = w as u32;
                                    let height = h as u32;

                                    let img = data.lock().unwrap();
                                    let resized = if args.len() >= 3 {
                                        let mode_val = self.eval_expr(&args[2]);
                                        if let Value::Str(mode) = mode_val {
                                            if mode.as_ref() == "fit" {
                                                // Maintain aspect ratio
                                                img.resize(
                                                    width,
                                                    height,
                                                    image::imageops::FilterType::Lanczos3,
                                                )
                                            } else {
                                                // Exact dimensions
                                                img.resize_exact(
                                                    width,
                                                    height,
                                                    image::imageops::FilterType::Lanczos3,
                                                )
                                            }
                                        } else {
                                            img.resize_exact(
                                                width,
                                                height,
                                                image::imageops::FilterType::Lanczos3,
                                            )
                                        }
                                    } else {
                                        // Exact dimensions by default
                                        img.resize_exact(
                                            width,
                                            height,
                                            image::imageops::FilterType::Lanczos3,
                                        )
                                    };

                                    return Value::Image {
                                        data: Arc::new(Mutex::new(resized)),
                                        format: format.clone(),
                                    };
                                } else {
                                    return Value::Error(
                                        "resize requires numeric width and height".to_string(),
                                    );
                                }
                            }
                            "crop" => {
                                // img.crop(x, y, width, height)
                                if args.len() < 4 {
                                    return Value::Error(
                                        "crop requires x, y, width, and height arguments"
                                            .to_string(),
                                    );
                                }

                                let x_val = self.eval_expr(&args[0]);
                                let y_val = self.eval_expr(&args[1]);
                                let w_val = self.eval_expr(&args[2]);
                                let h_val = self.eval_expr(&args[3]);

                                if let (
                                    Value::Float(x),
                                    Value::Float(y),
                                    Value::Float(w),
                                    Value::Float(h),
                                ) = (x_val, y_val, w_val, h_val)
                                {
                                    let mut img = data.lock().unwrap().clone();
                                    let cropped = img.crop(x as u32, y as u32, w as u32, h as u32);

                                    return Value::Image {
                                        data: Arc::new(Mutex::new(cropped)),
                                        format: format.clone(),
                                    };
                                } else {
                                    return Value::Error(
                                        "crop requires numeric x, y, width, and height".to_string(),
                                    );
                                }
                            }
                            "rotate" => {
                                // img.rotate(degrees)
                                if args.is_empty() {
                                    return Value::Error(
                                        "rotate requires a degrees argument".to_string(),
                                    );
                                }

                                let degrees_val = self.eval_expr(&args[0]);
                                let degrees = match degrees_val {
                                    Value::Int(n) => n as f32,
                                    Value::Float(n) => n as f32,
                                    _ => {
                                        return Value::Error(
                                            "rotate requires a numeric degrees argument"
                                                .to_string(),
                                        )
                                    }
                                };
                                let img = data.lock().unwrap();
                                let rotated = match degrees as i32 {
                                    90 => img.rotate90(),
                                    180 => img.rotate180(),
                                    270 => img.rotate270(),
                                    _ => {
                                        return Value::Error(
                                            "rotate only supports 90, 180, or 270 degrees"
                                                .to_string(),
                                        )
                                    }
                                };

                                return Value::Image {
                                    data: Arc::new(Mutex::new(rotated)),
                                    format: format.clone(),
                                };
                            }
                            "flip" => {
                                // img.flip("horizontal") or img.flip("vertical")
                                if args.is_empty() {
                                    return Value::Error("flip requires a direction argument ('horizontal' or 'vertical')".to_string());
                                }

                                let direction_val = self.eval_expr(&args[0]);
                                if let Value::Str(direction) = direction_val {
                                    let img = data.lock().unwrap();
                                    let flipped =
                                        match direction.as_str() {
                                            "horizontal" => img.fliph(),
                                            "vertical" => img.flipv(),
                                            _ => return Value::Error(
                                                "flip direction must be 'horizontal' or 'vertical'"
                                                    .to_string(),
                                            ),
                                        };

                                    return Value::Image {
                                        data: Arc::new(Mutex::new(flipped)),
                                        format: format.clone(),
                                    };
                                } else {
                                    return Value::Error(
                                        "flip requires a string direction argument".to_string(),
                                    );
                                }
                            }
                            "save" => {
                                // img.save(path) or img.save(path, options)
                                if args.is_empty() {
                                    return Value::Error(
                                        "save requires a path argument".to_string(),
                                    );
                                }

                                let path_val = self.eval_expr(&args[0]);
                                if let Value::Str(path) = path_val {
                                    let img = data.lock().unwrap();

                                    // The image crate will auto-detect format from extension
                                    // No need to manually specify the format
                                    match img.save(path.as_ref()) {
                                        Ok(_) => return Value::Bool(true),
                                        Err(e) => {
                                            return Value::Error(format!(
                                                "Failed to save image: {}",
                                                e
                                            ))
                                        }
                                    }
                                } else {
                                    return Value::Error(
                                        "save requires a string path argument".to_string(),
                                    );
                                }
                            }
                            "to_grayscale" => {
                                // img.to_grayscale()
                                let img = data.lock().unwrap();
                                let gray = img.grayscale();

                                return Value::Image {
                                    data: Arc::new(Mutex::new(gray)),
                                    format: format.clone(),
                                };
                            }
                            "blur" => {
                                // img.blur(sigma)
                                if args.is_empty() {
                                    return Value::Error(
                                        "blur requires a sigma argument".to_string(),
                                    );
                                }

                                let sigma_val = self.eval_expr(&args[0]);
                                let sigma = match sigma_val {
                                    Value::Int(n) => n as f32,
                                    Value::Float(n) => n as f32,
                                    _ => {
                                        return Value::Error(
                                            "blur requires a numeric sigma argument".to_string(),
                                        )
                                    }
                                };
                                let img = data.lock().unwrap();
                                let blurred = img.blur(sigma);

                                return Value::Image {
                                    data: Arc::new(Mutex::new(blurred)),
                                    format: format.clone(),
                                };
                            }
                            "adjust_brightness" => {
                                // img.adjust_brightness(factor)
                                if args.is_empty() {
                                    return Value::Error(
                                        "adjust_brightness requires a factor argument".to_string(),
                                    );
                                }

                                let factor_val = self.eval_expr(&args[0]);
                                let factor =
                                    match factor_val {
                                        Value::Int(n) => n as f64,
                                        Value::Float(n) => n,
                                        _ => return Value::Error(
                                            "adjust_brightness requires a numeric factor argument"
                                                .to_string(),
                                        ),
                                    };
                                let img = data.lock().unwrap();
                                let adjusted = img.brighten((factor * 50.0) as i32);

                                return Value::Image {
                                    data: Arc::new(Mutex::new(adjusted)),
                                    format: format.clone(),
                                };
                            }
                            "adjust_contrast" => {
                                // img.adjust_contrast(factor)
                                if args.is_empty() {
                                    return Value::Error(
                                        "adjust_contrast requires a factor argument".to_string(),
                                    );
                                }

                                let factor_val = self.eval_expr(&args[0]);
                                let factor =
                                    match factor_val {
                                        Value::Int(n) => n as f32,
                                        Value::Float(n) => n as f32,
                                        _ => return Value::Error(
                                            "adjust_contrast requires a numeric factor argument"
                                                .to_string(),
                                        ),
                                    };
                                let img = data.lock().unwrap();
                                let adjusted = img.adjust_contrast(factor);

                                return Value::Image {
                                    data: Arc::new(Mutex::new(adjusted)),
                                    format: format.clone(),
                                };
                            }
                            _ => return Value::Error(format!("Image has no method '{}'", field)),
                        }
                    }

                    // Handle Channel methods
                    if let Value::Channel(chan) = &obj_val {
                        match field.as_str() {
                            "send" => {
                                // chan.send(value) - send value to channel
                                if args.is_empty() {
                                    return Value::Error(
                                        "send requires a value argument".to_string(),
                                    );
                                }

                                let value = self.eval_expr(&args[0]);
                                let chan_lock = chan.lock().unwrap();
                                let (sender, _) = &*chan_lock;

                                match sender.send(value) {
                                    Ok(_) => return Value::Bool(true),
                                    Err(_) => {
                                        return Value::Error(
                                            "Failed to send to channel".to_string(),
                                        )
                                    }
                                }
                            }
                            "receive" => {
                                // chan.receive() - receive value from channel (non-blocking for now)
                                // TODO: Implement proper blocking receive
                                let chan_lock = chan.lock().unwrap();
                                let (_, receiver) = &*chan_lock;

                                match receiver.try_recv() {
                                    Ok(value) => return value,
                                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                                        // Channel is empty - return null
                                        return Value::Null;
                                    }
                                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                        return Value::Error("Channel disconnected".to_string());
                                    }
                                }
                            }
                            _ => return Value::Error(format!("Channel has no method '{}'", field)),
                        }
                    }

                    // Handle ArgParser methods
                    if let Value::Struct { name, fields } = &obj_val {
                        if name == "ArgParser" {
                            match field.as_str() {
                                "add_argument" => {
                                    // parser.add_argument(long, short, type, required, help, default)
                                    // Extract arguments
                                    let mut long_name = String::new();
                                    let mut short_name: Option<String> = None;
                                    let mut arg_type = String::from("string");
                                    let mut required = false;
                                    let mut help = String::new();
                                    let mut default: Option<String> = None;

                                    // First argument is always the long name
                                    if !args.is_empty() {
                                        if let Value::Str(s) = self.eval_expr(&args[0]) {
                                            long_name = s.as_ref().clone();
                                        }
                                    }

                                    // Process remaining keyword-style arguments
                                    // In Ruff, these come as alternating key-value pairs
                                    let mut i = 1;
                                    while i < args.len() {
                                        if let Value::Str(key) = self.eval_expr(&args[i]) {
                                            if i + 1 < args.len() {
                                                let value = self.eval_expr(&args[i + 1]);
                                                match key.as_str() {
                                                    "short" => {
                                                        if let Value::Str(s) = value {
                                                            short_name = Some(s.as_ref().clone());
                                                        }
                                                    }
                                                    "type" => {
                                                        if let Value::Str(s) = value {
                                                            arg_type = s.as_ref().clone();
                                                        }
                                                    }
                                                    "required" => {
                                                        if let Value::Bool(b) = value {
                                                            required = b;
                                                        }
                                                    }
                                                    "help" => {
                                                        if let Value::Str(s) = value {
                                                            help = s.as_ref().clone();
                                                        }
                                                    }
                                                    "default" => {
                                                        if let Value::Str(s) = value {
                                                            default = Some(s.as_ref().clone());
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                                i += 2;
                                            } else {
                                                i += 1;
                                            }
                                        } else {
                                            i += 1;
                                        }
                                    }

                                    // Create argument definition
                                    let mut arg_def = DictMap::default();
                                    arg_def.insert("long".into(), Value::Str(Arc::new(long_name)));
                                    if let Some(short) = short_name {
                                        arg_def.insert("short".into(), Value::Str(Arc::new(short)));
                                    }
                                    arg_def.insert("type".into(), Value::Str(Arc::new(arg_type)));
                                    arg_def.insert("required".into(), Value::Bool(required));
                                    arg_def.insert("help".into(), Value::Str(Arc::new(help)));
                                    if let Some(def) = default {
                                        arg_def.insert("default".into(), Value::Str(Arc::new(def)));
                                    }

                                    // Add to the parser's argument list
                                    let mut new_fields = fields.clone();
                                    if let Some(Value::Array(arg_list)) =
                                        new_fields.get("_args").cloned()
                                    {
                                        let mut arg_list_vec = Arc::try_unwrap(arg_list)
                                            .unwrap_or_else(|arc| (*arc).clone());
                                        arg_list_vec.push(Value::Dict(Arc::new(arg_def)));
                                        new_fields.insert(
                                            "_args".to_string(),
                                            Value::Array(Arc::new(arg_list_vec)),
                                        );
                                    }

                                    return Value::Struct {
                                        name: "ArgParser".to_string(),
                                        fields: new_fields,
                                    };
                                }
                                "parse" => {
                                    // parser.parse() - parse command-line arguments
                                    // Convert stored argument definitions to ArgumentDef structs
                                    let mut arg_defs = Vec::new();

                                    if let Some(Value::Array(arg_list)) = fields.get("_args") {
                                        for arg_val in arg_list.iter() {
                                            if let Value::Dict(arg_dict) = arg_val {
                                                let long_name = match arg_dict.get("long") {
                                                    Some(Value::Str(s)) => s.as_ref().clone(),
                                                    _ => continue,
                                                };

                                                let short_name = match arg_dict.get("short") {
                                                    Some(Value::Str(s)) => Some(s.as_ref().clone()),
                                                    _ => None,
                                                };

                                                let arg_type = match arg_dict.get("type") {
                                                    Some(Value::Str(s)) => s.as_ref().clone(),
                                                    _ => "string".to_string(),
                                                };

                                                let required = match arg_dict.get("required") {
                                                    Some(Value::Bool(b)) => *b,
                                                    _ => false,
                                                };

                                                let help = match arg_dict.get("help") {
                                                    Some(Value::Str(s)) => s.as_ref().clone(),
                                                    _ => String::new(),
                                                };

                                                let default = match arg_dict.get("default") {
                                                    Some(Value::Str(s)) => Some(s.as_ref().clone()),
                                                    _ => None,
                                                };

                                                arg_defs.push(builtins::ArgumentDef {
                                                    long_name,
                                                    short_name,
                                                    arg_type,
                                                    required,
                                                    help,
                                                    default,
                                                });
                                            }
                                        }
                                    }

                                    // Get command-line arguments
                                    let cli_args = builtins::get_args();

                                    // Parse arguments
                                    match builtins::parse_arguments(&arg_defs, &cli_args) {
                                        Ok(parsed) => return Value::Dict(Arc::new(parsed)),
                                        Err(msg) => {
                                            return Value::ErrorObject {
                                                message: msg,
                                                stack: Vec::new(),
                                                line: None,
                                                cause: None,
                                            }
                                        }
                                    }
                                }
                                "help" => {
                                    // parser.help() - generate help text
                                    let mut arg_defs = Vec::new();

                                    if let Some(Value::Array(arg_list)) = fields.get("_args") {
                                        for arg_val in arg_list.iter() {
                                            if let Value::Dict(arg_dict) = arg_val {
                                                let long_name = match arg_dict.get("long") {
                                                    Some(Value::Str(s)) => s.as_ref().clone(),
                                                    _ => continue,
                                                };

                                                let short_name = match arg_dict.get("short") {
                                                    Some(Value::Str(s)) => Some(s.as_ref().clone()),
                                                    _ => None,
                                                };

                                                let arg_type = match arg_dict.get("type") {
                                                    Some(Value::Str(s)) => s.as_ref().clone(),
                                                    _ => "string".to_string(),
                                                };

                                                let required = match arg_dict.get("required") {
                                                    Some(Value::Bool(b)) => *b,
                                                    _ => false,
                                                };

                                                let help = match arg_dict.get("help") {
                                                    Some(Value::Str(s)) => s.as_ref().clone(),
                                                    _ => String::new(),
                                                };

                                                let default = match arg_dict.get("default") {
                                                    Some(Value::Str(s)) => Some(s.as_ref().clone()),
                                                    _ => None,
                                                };

                                                arg_defs.push(builtins::ArgumentDef {
                                                    long_name,
                                                    short_name,
                                                    arg_type,
                                                    required,
                                                    help,
                                                    default,
                                                });
                                            }
                                        }
                                    }

                                    let app_name = match fields.get("_app_name") {
                                        Some(Value::Str(s)) => s.as_ref().clone(),
                                        _ => "program".to_string(),
                                    };

                                    let description = match fields.get("_description") {
                                        Some(Value::Str(s)) => s.as_ref().clone(),
                                        _ => String::new(),
                                    };

                                    let help_text =
                                        builtins::generate_help(&arg_defs, &app_name, &description);
                                    return Value::Str(Arc::new(help_text));
                                }
                                _ => {
                                    return Value::Error(format!(
                                        "ArgParser has no method '{}'",
                                        field
                                    ))
                                }
                            }
                        }
                    }

                    if let Value::Struct { name, fields } = &obj_val {
                        // Look up the struct definition to find the method
                        if let Some(Value::StructDef { name: _, field_names: _, methods }) =
                            self.env.get(name)
                        {
                            if let Some(Value::Function(params, body, _captured_env)) =
                                methods.get(field)
                            {
                                // Create new scope for method call
                                // Push new scope
                                self.env.push_scope();

                                // Check if method has 'self' as first parameter
                                let has_self_param =
                                    params.first().map(|p| p == "self").unwrap_or(false);

                                if has_self_param {
                                    // Bind self to the struct instance
                                    self.env.define("self".to_string(), obj_val.clone());

                                    // Bind remaining method parameters (skip first 'self' param)
                                    for (i, param) in params.iter().skip(1).enumerate() {
                                        if let Some(arg) = args.get(i) {
                                            let val = self.eval_expr(arg);
                                            self.env.define(param.clone(), val);
                                        }
                                    }
                                } else {
                                    // Backward compatibility: bind fields directly into scope
                                    for (field_name, field_value) in fields {
                                        self.env.define(field_name.clone(), field_value.clone());
                                    }

                                    // Bind method parameters
                                    for (i, param) in params.iter().enumerate() {
                                        if let Some(arg) = args.get(i) {
                                            let val = self.eval_expr(arg);
                                            self.env.define(param.clone(), val);
                                        }
                                    }
                                }

                                // Execute method body
                                self.eval_stmts(body.get());
                                let result = if let Some(Value::Return(val)) =
                                    self.return_value.clone()
                                {
                                    self.return_value = None; // Clear return value
                                    *val
                                } else if let Some(Value::Error(msg)) = self.return_value.clone() {
                                    // Propagate error - don't clear
                                    Value::Error(msg)
                                } else {
                                    // No explicit return - clear any lingering state
                                    self.return_value = None;
                                    Value::Int(0)
                                };

                                // Restore parent environment
                                self.env.pop_scope();

                                return result;
                            }
                        }
                    }
                }

                // Regular function call
                let func_val = self.eval_expr(function);
                let call_result = match func_val {
                    Value::NativeFunction(name) => {
                        // Handle native function calls
                        let res = self.call_native_function(&name, args);
                        // Check if result is an error and set return_value to trigger try/except handling
                        match res {
                            Value::ErrorObject { .. } => {
                                self.return_value = Some(res.clone());
                                res
                            }
                            Value::Error(_) => {
                                self.return_value = Some(res.clone());
                                res
                            }
                            _ => res,
                        }
                    }
                    Value::Function(params, body, captured_env) => {
                        // Push to call stack
                        self.call_stack.push("<anonymous function>".to_string());

                        // Handle closure with captured environment
                        if let Some(closure_env_ref) = captured_env {
                            // Save current environment
                            let saved_env = self.env.clone();

                            // Use the captured environment
                            self.env = closure_env_ref.lock().unwrap().clone();
                            self.env.push_scope();

                            for (i, param) in params.iter().enumerate() {
                                if let Some(arg) = args.get(i) {
                                    let val = self.eval_expr(arg);
                                    self.env.define(param.clone(), val);
                                }
                            }

                            self.eval_stmts(body.get());

                            let result = if let Some(Value::Return(val)) = self.return_value.clone()
                            {
                                self.return_value = None;
                                *val
                            } else if let Some(Value::Error(msg)) = self.return_value.clone() {
                                Value::Error(msg)
                            } else if let Some(Value::ErrorObject { .. }) =
                                self.return_value.clone()
                            {
                                self.return_value.clone().unwrap()
                            } else {
                                self.return_value = None;
                                Value::Int(0)
                            };

                            self.env.pop_scope();
                            // Update the captured environment
                            *closure_env_ref.lock().unwrap() = self.env.clone();
                            self.env = saved_env;
                            self.call_stack.pop();

                            result
                        } else {
                            // Non-closure: just create new scope
                            self.env.push_scope();

                            for (i, param) in params.iter().enumerate() {
                                if let Some(arg) = args.get(i) {
                                    let val = self.eval_expr(arg);
                                    self.env.define(param.clone(), val);
                                }
                            }

                            self.eval_stmts(body.get());

                            let result = if let Some(Value::Return(val)) = self.return_value.clone()
                            {
                                self.return_value = None;
                                *val
                            } else if let Some(Value::Error(msg)) = self.return_value.clone() {
                                Value::Error(msg)
                            } else if let Some(Value::ErrorObject { .. }) =
                                self.return_value.clone()
                            {
                                self.return_value.clone().unwrap()
                            } else {
                                self.return_value = None;
                                Value::Int(0)
                            };

                            self.env.pop_scope();
                            self.call_stack.pop();

                            result
                        }
                    }
                    Value::AsyncFunction(params, body, captured_env) => {
                        // Evaluate arguments
                        let args_vec: Vec<Value> =
                            args.iter().map(|arg| self.eval_expr(arg)).collect();

                        // Clone what we need for the thread
                        let params = params.clone();
                        let body = body.clone();
                        let base_env = if let Some(ref env_ref) = captured_env {
                            env_ref.lock().unwrap().clone()
                        } else {
                            self.env.clone()
                        };

                        // Create a tokio oneshot channel for the result
                        let (tx, rx) = tokio::sync::oneshot::channel();

                        // Spawn a tokio task to execute the async function
                        AsyncRuntime::spawn_task(async move {
                            let mut async_interpreter = Interpreter::new();
                            async_interpreter.register_builtins(); // Register built-in functions
                            async_interpreter.env = base_env;
                            async_interpreter.env.push_scope();

                            // Bind parameters
                            for (i, param) in params.iter().enumerate() {
                                if let Some(arg) = args_vec.get(i) {
                                    async_interpreter.env.define(param.clone(), arg.clone());
                                }
                            }

                            // Execute the async function body
                            async_interpreter.eval_stmts(body.get());

                            // Get the return value
                            let result =
                                if let Some(Value::Return(val)) = async_interpreter.return_value {
                                    *val
                                } else {
                                    Value::Int(0)
                                };

                            // Send the result back
                            let _ = tx.send(Ok(result));

                            // Return a dummy value (task result not used, only channel matters)
                            Value::Null
                        });

                        // Return a Promise containing the receiver
                        Value::Promise {
                            receiver: Arc::new(Mutex::new(rx)),
                            is_polled: Arc::new(Mutex::new(false)),
                            cached_result: Arc::new(Mutex::new(None)),
                            task_handle: None,
                        }
                    }
                    Value::GeneratorDef(ref params, ref body) => {
                        // Calling a generator function creates a Generator instance
                        let args_vec: Vec<Value> =
                            args.iter().map(|arg| self.eval_expr(arg)).collect();

                        // Create a new environment for the generator
                        let mut gen_env = self.env.clone();
                        gen_env.push_scope();

                        // Bind parameters to arguments
                        for (i, param) in params.iter().enumerate() {
                            if let Some(arg) = args_vec.get(i) {
                                gen_env.define(param.clone(), arg.clone());
                            }
                        }

                        // Return a Generator instance
                        Value::Generator {
                            params: params.clone(),
                            body: body.clone(),
                            env: Arc::new(Mutex::new(gen_env)),
                            pc: 0,
                            is_exhausted: false,
                        }
                    }
                    _ => Value::Int(0),
                };
                call_result
            }
            Expr::Tag(name, args) => {
                // First check if this is a native or user function
                if let Some(func_val) = self.env.get(name) {
                    match func_val {
                        Value::NativeFunction(_) => {
                            // Call native function
                            return self.call_native_function(name, args);
                        }
                        Value::Function(params, body, captured_env) => {
                            // Push function name to call stack
                            self.call_stack.push(name.clone());

                            // Handle closure with captured environment
                            if let Some(closure_env_ref) = captured_env {
                                // Save current environment
                                let saved_env = self.env.clone();

                                // Use the captured environment
                                self.env = closure_env_ref.lock().unwrap().clone();
                                self.env.push_scope();

                                for (i, param) in params.iter().enumerate() {
                                    if let Some(arg) = args.get(i) {
                                        let val = self.eval_expr(arg);
                                        self.env.define(param.clone(), val);
                                    }
                                }

                                self.eval_stmts(body.get());

                                let result = if let Some(Value::Return(val)) =
                                    self.return_value.clone()
                                {
                                    self.return_value = None;
                                    *val
                                } else if let Some(Value::Error(msg)) = self.return_value.clone() {
                                    Value::Error(msg)
                                } else if let Some(Value::ErrorObject { .. }) =
                                    self.return_value.clone()
                                {
                                    self.return_value.clone().unwrap()
                                } else {
                                    self.return_value = None;
                                    Value::Int(0)
                                };

                                self.env.pop_scope();
                                // Update the captured environment
                                *closure_env_ref.lock().unwrap() = self.env.clone();
                                self.env = saved_env;
                                self.call_stack.pop();

                                return result;
                            } else {
                                // Non-closure: just create new scope
                                self.env.push_scope();

                                for (i, param) in params.iter().enumerate() {
                                    if let Some(arg) = args.get(i) {
                                        let val = self.eval_expr(arg);
                                        self.env.define(param.clone(), val);
                                    }
                                }

                                self.eval_stmts(body.get());

                                let result = if let Some(Value::Return(val)) =
                                    self.return_value.clone()
                                {
                                    self.return_value = None;
                                    *val
                                } else if let Some(Value::Error(msg)) = self.return_value.clone() {
                                    Value::Error(msg)
                                } else if let Some(Value::ErrorObject { .. }) =
                                    self.return_value.clone()
                                {
                                    self.return_value.clone().unwrap()
                                } else {
                                    self.return_value = None;
                                    Value::Int(0)
                                };

                                self.env.pop_scope();
                                self.call_stack.pop();

                                return result;
                            }
                        }
                        Value::GeneratorDef(ref params, ref body) => {
                            // Calling a generator function creates a Generator instance
                            let args_vec: Vec<Value> =
                                args.iter().map(|arg| self.eval_expr(arg)).collect();

                            // Create a new environment for the generator
                            let mut gen_env = self.env.clone();
                            gen_env.push_scope();

                            // Bind parameters to arguments
                            for (i, param) in params.iter().enumerate() {
                                if let Some(arg) = args_vec.get(i) {
                                    gen_env.define(param.clone(), arg.clone());
                                }
                            }

                            // Return a Generator instance
                            return Value::Generator {
                                params: params.clone(),
                                body: body.clone(),
                                env: Arc::new(Mutex::new(gen_env)),
                                pc: 0,
                                is_exhausted: false,
                            };
                        }
                        _ => {}
                    }
                }

                // Otherwise, treat as enum constructor
                let mut fields = HashMap::new();
                for (i, arg) in args.iter().enumerate() {
                    fields.insert(format!("${}", i), self.eval_expr(arg));
                }
                Value::Tagged { tag: name.clone(), fields }
            }
            Expr::StructInstance { name, fields } => {
                // Create a struct instance
                let mut field_values = HashMap::new();
                for (field_name, field_expr) in fields {
                    field_values.insert(field_name.clone(), self.eval_expr(field_expr));
                }
                Value::Struct { name: name.clone(), fields: field_values }
            }
            Expr::FieldAccess { object, field } => {
                let obj_val = self.eval_expr(object);
                match obj_val {
                    Value::Struct { name: _, fields } => {
                        // Access field from struct instance
                        fields.get(field).cloned().unwrap_or(Value::Int(0))
                    }
                    Value::Image { data, format } => {
                        // Access image properties
                        match field.as_str() {
                            "width" => {
                                let img = data.lock().unwrap();
                                Value::Float(img.width() as f64)
                            }
                            "height" => {
                                let img = data.lock().unwrap();
                                Value::Float(img.height() as f64)
                            }
                            "format" => Value::Str(Arc::new(format)),
                            _ => Value::Error(format!("Image has no field '{}'", field)),
                        }
                    }
                    _ => Value::Int(0),
                }
            }
            Expr::ArrayLiteral(elements) => {
                use crate::ast::ArrayElement;
                let mut values = Vec::new();

                for elem in elements {
                    match elem {
                        ArrayElement::Single(expr) => {
                            values.push(self.eval_expr(expr));
                        }
                        ArrayElement::Spread(expr) => {
                            // Evaluate spread expression and merge its elements
                            let spread_val = self.eval_expr(expr);
                            if let Value::Array(arr) = spread_val {
                                values.extend(arr.iter().cloned());
                            }
                            // If not an array, ignore the spread
                        }
                    }
                }

                Value::Array(Arc::new(values))
            }
            Expr::DictLiteral(pairs) => {
                use crate::ast::DictElement;
                let mut map = DictMap::default();

                for elem in pairs {
                    match elem {
                        DictElement::Pair(key_expr, val_expr) => {
                            let key = match self.eval_expr(key_expr) {
                                Value::Str(s) => s.as_ref().clone(),
                                Value::Int(n) => n.to_string(),
                                Value::Float(n) => n.to_string(),
                                _ => continue,
                            };
                            let value = self.eval_expr(val_expr);
                            map.insert(Arc::from(key), value);
                        }
                        DictElement::Spread(expr) => {
                            // Evaluate spread expression and merge its entries
                            let spread_val = self.eval_expr(expr);
                            if let Value::Dict(dict) = spread_val {
                                for (k, v) in dict.iter() {
                                    map.insert(k.clone(), v.clone());
                                }
                            }
                            // If not a dict, ignore the spread
                        }
                    }
                }

                Value::Dict(Arc::new(map))
            }
            Expr::IndexAccess { object, index } => {
                let obj_val = self.eval_expr(object);
                let idx_val = self.eval_expr(index);

                match (obj_val, idx_val) {
                    (Value::Array(arr), Value::Int(n)) => {
                        let idx = n as usize;
                        arr.get(idx).cloned().unwrap_or(Value::Int(0))
                    }
                    (Value::Array(arr), Value::Float(n)) => {
                        let idx = n as usize;
                        arr.get(idx).cloned().unwrap_or(Value::Int(0))
                    }
                    (Value::Dict(map), Value::Str(key)) => {
                        map.get(key.as_str()).cloned().unwrap_or(Value::Int(0))
                    }
                    (Value::Str(s), Value::Int(n)) => {
                        let idx = n as usize;
                        s.chars()
                            .nth(idx)
                            .map(|c| Value::Str(Arc::new(c.to_string())))
                            .unwrap_or(Value::Str(Arc::new(String::new())))
                    }
                    (Value::Str(s), Value::Float(n)) => {
                        let idx = n as usize;
                        s.chars()
                            .nth(idx)
                            .map(|c| Value::Str(Arc::new(c.to_string())))
                            .unwrap_or(Value::Str(Arc::new(String::new())))
                    }
                    _ => Value::Int(0),
                }
            }
            Expr::Ok(value_expr) => {
                let value = self.eval_expr(value_expr);
                Value::Result { is_ok: true, value: Box::new(value) }
            }
            Expr::Err(error_expr) => {
                let error = self.eval_expr(error_expr);
                Value::Result { is_ok: false, value: Box::new(error) }
            }
            Expr::Some(value_expr) => {
                let value = self.eval_expr(value_expr);
                Value::Option { is_some: true, value: Box::new(value) }
            }
            Expr::None => Value::Option { is_some: false, value: Box::new(Value::Null) },
            Expr::Try(expr) => {
                let value = self.eval_expr(expr);
                match value {
                    Value::Result { is_ok, value } => {
                        if is_ok {
                            // Return the Ok value
                            *value
                        } else {
                            // Early return with the Err value wrapped in Result
                            self.return_value = Some(Value::Return(Box::new(Value::Result {
                                is_ok: false,
                                value,
                            })));
                            Value::Null // This will be ignored due to early return
                        }
                    }
                    _ => {
                        // ? operator only works on Result values
                        Value::Error(
                            "Try operator (?) can only be used on Result values".to_string(),
                        )
                    }
                }
            }
            Expr::Yield(value_expr) => {
                // Yield expression - should only be used inside generators
                // For now, return the yielded value wrapped in a special marker
                // The generator execution logic will handle this properly
                let yielded =
                    if let Some(expr) = value_expr { self.eval_expr(expr) } else { Value::Null };
                // Use a Return value to signal yield - generators will intercept this
                Value::Return(Box::new(yielded))
            }
            Expr::Await(promise_expr) => {
                // Await expression - wait for a promise to resolve using tokio runtime
                let promise_value = self.eval_expr(promise_expr);

                // If it's a promise, wait for it to resolve
                match promise_value {
                    Value::Promise { receiver, is_polled, cached_result, .. } => {
                        // Check if we've already polled this promise
                        {
                            let polled = is_polled.lock().unwrap();
                            let cached = cached_result.lock().unwrap();

                            if *polled {
                                // Use cached result
                                return match cached.as_ref() {
                                    Some(Ok(val)) => val.clone(),
                                    Some(Err(err)) => {
                                        Value::Error(format!("Promise rejected: {}", err))
                                    }
                                    None => Value::Error(
                                        "Promise polled but no result cached".to_string(),
                                    ),
                                };
                            }
                            // Locks dropped here
                        }

                        // Poll the promise using tokio runtime
                        // We need to take ownership of the receiver to await it
                        let result = {
                            let mut recv_guard = receiver.lock().unwrap();
                            // Take ownership by replacing with a dummy closed channel
                            let (dummy_tx, dummy_rx) = tokio::sync::oneshot::channel();
                            drop(dummy_tx); // Close immediately
                            let actual_rx = std::mem::replace(&mut *recv_guard, dummy_rx);
                            drop(recv_guard); // Release lock before blocking

                            // Block on the receiver using tokio runtime
                            AsyncRuntime::block_on(actual_rx)
                        };

                        // Now update the cache with the result
                        let mut polled = is_polled.lock().unwrap();
                        let mut cached = cached_result.lock().unwrap();

                        match result {
                            Ok(Ok(value)) => {
                                // Cache the successful result
                                *cached = Some(Ok(value.clone()));
                                *polled = true;
                                value
                            }
                            Ok(Err(error)) => {
                                // Cache the error
                                *cached = Some(Err(error.clone()));
                                *polled = true;
                                Value::Error(format!("Promise rejected: {}", error))
                            }
                            Err(_) => {
                                // Channel closed without sending - this shouldn't happen
                                *cached = Some(Err("Promise never resolved".to_string()));
                                *polled = true;
                                Value::Error("Promise never resolved (channel closed)".to_string())
                            }
                        }
                    }
                    _ => {
                        // Not a promise - just return the value
                        promise_value
                    }
                }
            }
            Expr::MethodCall { object, method, args } => {
                // Method call on an expression - used for iterator chaining
                let obj_value = self.eval_expr(object);
                let arg_values: Vec<Value> = args.iter().map(|arg| self.eval_expr(arg)).collect();

                // Call the method on the object
                self.call_method(obj_value, method, arg_values)
            }
            Expr::Spread(_) => {
                // Spread expressions should only appear inside array/dict literals
                // If we reach here, it's a syntax error, but we'll return an error value
                Value::Error(
                    "Spread operator (...) can only be used inside array or dict literals"
                        .to_string(),
                )
            }
        };
        result
    }

    /// Call a method on a value (used for iterator chaining and other method calls)
    fn call_method(&mut self, obj: Value, method: &str, args: Vec<Value>) -> Value {
        match method {
            // Iterator methods
            "filter" if args.len() == 1 => {
                // Create an iterator with a filter function
                match &obj {
                    Value::Iterator {
                        source,
                        index,
                        transformer,
                        filter_fn: existing_filter,
                        take_count,
                    } => {
                        // Chain filters if there's already one
                        let new_filter = if existing_filter.is_some() {
                            // TODO: Combine filters
                            Some(Box::new(args[0].clone()))
                        } else {
                            Some(Box::new(args[0].clone()))
                        };
                        Value::Iterator {
                            source: source.clone(),
                            index: *index,
                            transformer: transformer.clone(),
                            filter_fn: new_filter,
                            take_count: *take_count,
                        }
                    }
                    Value::Array(_) => {
                        // Convert array to iterator with filter
                        Value::Iterator {
                            source: Box::new(obj),
                            index: 0,
                            transformer: None,
                            filter_fn: Some(Box::new(args[0].clone())),
                            take_count: None,
                        }
                    }
                    _ => Value::Error(
                        "filter() can only be called on iterators or arrays".to_string(),
                    ),
                }
            }
            "map" if args.len() == 1 => {
                // Create an iterator with a transformer function
                match &obj {
                    Value::Iterator {
                        source,
                        index,
                        transformer: existing_transformer,
                        filter_fn,
                        take_count,
                    } => {
                        // Chain transformers if there's already one
                        let new_transformer = if existing_transformer.is_some() {
                            // TODO: Combine transformers
                            Some(Box::new(args[0].clone()))
                        } else {
                            Some(Box::new(args[0].clone()))
                        };
                        Value::Iterator {
                            source: source.clone(),
                            index: *index,
                            transformer: new_transformer,
                            filter_fn: filter_fn.clone(),
                            take_count: *take_count,
                        }
                    }
                    Value::Array(_) => {
                        // Convert array to iterator with map
                        Value::Iterator {
                            source: Box::new(obj),
                            index: 0,
                            transformer: Some(Box::new(args[0].clone())),
                            filter_fn: None,
                            take_count: None,
                        }
                    }
                    _ => {
                        Value::Error("map() can only be called on iterators or arrays".to_string())
                    }
                }
            }
            "take" if args.len() == 1 => {
                // Limit the number of items
                if let Value::Int(n) = args[0] {
                    match &obj {
                        Value::Iterator {
                            source,
                            index,
                            transformer,
                            filter_fn,
                            take_count: _,
                        } => Value::Iterator {
                            source: source.clone(),
                            index: *index,
                            transformer: transformer.clone(),
                            filter_fn: filter_fn.clone(),
                            take_count: Some(n as usize),
                        },
                        Value::Array(_) => Value::Iterator {
                            source: Box::new(obj),
                            index: 0,
                            transformer: None,
                            filter_fn: None,
                            take_count: Some(n as usize),
                        },
                        _ => Value::Error(
                            "take() can only be called on iterators or arrays".to_string(),
                        ),
                    }
                } else {
                    Value::Error("take() requires an integer argument".to_string())
                }
            }
            "collect" if args.is_empty() => {
                // Collect iterator into an array
                self.collect_iterator(obj)
            }
            "next" if args.is_empty() => {
                // Get next value from iterator
                self.iterator_next(obj)
            }
            _ => {
                // Check if it's a struct method
                match &obj {
                    Value::Struct { name: _, fields: _ } => {
                        // Look for method in struct definition
                        // For now, return error
                        Value::Error(format!("Unknown method: {}", method))
                    }
                    _ => Value::Error(format!("Unknown method: {}", method)),
                }
            }
        }
    }

    /// Collect all values from an iterator into an array
    fn collect_iterator(&mut self, mut iterator: Value) -> Value {
        let mut result = Vec::new();

        loop {
            match &mut iterator {
                Value::Iterator { source, index, transformer, filter_fn, take_count } => {
                    // Check if we've reached the take limit
                    if let Some(limit) = take_count {
                        if result.len() >= *limit {
                            return Value::Array(Arc::new(result));
                        }
                    }

                    // Get next item from source
                    match source.as_ref() {
                        Value::Array(items) => {
                            // Find next item that passes filter
                            loop {
                                if *index >= items.len() {
                                    // No more items
                                    return Value::Array(Arc::new(result));
                                }

                                let mut item = items[*index].clone();
                                *index += 1;

                                // Apply filter if present
                                if let Some(filter) = filter_fn {
                                    let args_vec = vec![item.clone()];
                                    let filter_result =
                                        self.call_user_function(filter.as_ref(), &args_vec);
                                    match filter_result {
                                        Value::Bool(true) => {
                                            // Item passes filter - continue processing
                                        }
                                        _ => {
                                            // Item filtered out, try next
                                            continue;
                                        }
                                    }
                                }

                                // Apply transformer if present
                                if let Some(trans) = transformer {
                                    let args_vec = vec![item];
                                    item = self.call_user_function(trans.as_ref(), &args_vec);
                                }

                                result.push(item);

                                // Check take limit after adding
                                if let Some(limit) = take_count {
                                    if result.len() >= *limit {
                                        return Value::Array(Arc::new(result));
                                    }
                                }

                                break; // Found one item, continue outer loop
                            }
                        }
                        Value::Generator { .. } => {
                            // Get next value from generator
                            let next_option = self.generator_next(source);
                            match next_option {
                                Value::Option { is_some: true, value } => {
                                    let mut item = *value;

                                    // Apply filter if present
                                    if let Some(filter) = filter_fn {
                                        let args_vec = vec![item.clone()];
                                        let filter_result =
                                            self.call_user_function(filter.as_ref(), &args_vec);
                                        match filter_result {
                                            Value::Bool(false) => {
                                                // Item filtered out, try next iteration of outer loop
                                                continue;
                                            }
                                            _ => {}
                                        }
                                    }

                                    // Apply transformer if present
                                    if let Some(trans) = transformer {
                                        let args_vec = vec![item];
                                        item = self.call_user_function(trans.as_ref(), &args_vec);
                                    }

                                    result.push(item);

                                    // Check take limit after adding
                                    if let Some(limit) = take_count {
                                        if result.len() >= *limit {
                                            return Value::Array(Arc::new(result));
                                        }
                                    }
                                    // Continue to next iteration of outer loop
                                }
                                Value::Option { is_some: false, .. } => {
                                    // Generator exhausted
                                    return Value::Array(Arc::new(result));
                                }
                                Value::Error(msg) => {
                                    return Value::Error(msg);
                                }
                                _ => {
                                    return Value::Error(
                                        "Unexpected value from generator".to_string(),
                                    );
                                }
                            }
                        }
                        _ => {
                            return Value::Error("Invalid iterator source".to_string());
                        }
                    }
                }
                _ => {
                    return Value::Error("collect() can only be called on iterators".to_string());
                }
            }
        }
    }

    /// Execute a generator until it yields a value or completes
    /// Returns Some(value) if yielded, None if exhausted
    fn generator_next(&mut self, generator: &mut Value) -> Value {
        match generator {
            Value::Generator { params: _, body, env, pc, is_exhausted } => {
                if *is_exhausted {
                    return Value::Option { is_some: false, value: Box::new(Value::Null) };
                }

                // Save current interpreter state
                let saved_env = self.env.clone();
                let saved_return_value = self.return_value.take();

                // Use the generator's environment
                self.env = env.lock().unwrap().clone();

                let stmts = body.get();
                let mut yielded_value = None;

                // Execute statements starting from PC until yield or end
                while *pc < stmts.len() {
                    let current_pc = *pc;

                    self.eval_stmt(&stmts[current_pc]);

                    // Check if a yield occurred (signaled by Return value)
                    if let Some(ret_val) = &self.return_value {
                        match ret_val {
                            Value::Return(inner) => {
                                // This is a yield - extract the value and suspend
                                // Advance PC so next call continues from next statement
                                *pc += 1;
                                yielded_value = Some(inner.as_ref().clone());
                                self.return_value = None;
                                break;
                            }
                            _ => {
                                // Regular return - generator is done
                                *is_exhausted = true;
                                break;
                            }
                        }
                    } else {
                        // Statement completed without yield - advance to next statement
                        *pc += 1;
                    }
                }

                // Save the generator's environment state
                *env.lock().unwrap() = self.env.clone();

                // If we finished all statements without explicit return/yield, generator is exhausted
                if *pc >= stmts.len() {
                    *is_exhausted = true;
                }

                // Restore interpreter state
                self.env = saved_env;
                self.return_value = saved_return_value;

                // Return the yielded value or None if exhausted
                if let Some(value) = yielded_value {
                    Value::Option { is_some: true, value: Box::new(value) }
                } else {
                    Value::Option { is_some: false, value: Box::new(Value::Null) }
                }
            }
            _ => Value::Error("generator_next() can only be called on generators".to_string()),
        }
    }

    /// Get the next value from an iterator
    fn iterator_next(&mut self, mut iterator: Value) -> Value {
        match &mut iterator {
            Value::Iterator { source, index, transformer, filter_fn, take_count } => {
                // Check if we've reached the take limit
                if let Some(limit) = take_count {
                    if *index >= *limit {
                        return Value::Option { is_some: false, value: Box::new(Value::Null) };
                    }
                }

                // Get next item from source
                match source.as_mut() {
                    Value::Array(items) => {
                        // Find next item that passes filter
                        while *index < items.len() {
                            let mut item = items[*index].clone();
                            *index += 1;

                            // Apply filter if present
                            if let Some(filter) = filter_fn {
                                let args_vec = vec![item.clone()];
                                let filter_result =
                                    self.call_user_function(filter.as_ref(), &args_vec);
                                match filter_result {
                                    Value::Bool(true) => {
                                        // Item passes filter
                                    }
                                    _ => {
                                        // Item filtered out, continue to next
                                        continue;
                                    }
                                }
                            }

                            // Apply transformer if present
                            if let Some(trans) = transformer {
                                let args_vec = vec![item];
                                item = self.call_user_function(trans.as_ref(), &args_vec);
                            }

                            return Value::Option { is_some: true, value: Box::new(item) };
                        }
                        // No more items
                        Value::Option { is_some: false, value: Box::new(Value::Null) }
                    }
                    Value::Generator { .. } => {
                        // Delegate to generator_next
                        let result = self.generator_next(source);

                        // Apply transformer if present and we got a value
                        match result {
                            Value::Option { is_some: true, value } => {
                                let mut item = *value;

                                // Apply filter if present
                                if let Some(filter) = filter_fn {
                                    let args_vec = vec![item.clone()];
                                    let filter_result =
                                        self.call_user_function(filter.as_ref(), &args_vec);
                                    match filter_result {
                                        Value::Bool(false) => {
                                            // Item filtered out - need to get next one
                                            // For now, just return None (TODO: could recursively call)
                                            return Value::Option {
                                                is_some: false,
                                                value: Box::new(Value::Null),
                                            };
                                        }
                                        _ => {}
                                    }
                                }

                                // Apply transformer if present
                                if let Some(trans) = transformer {
                                    let args_vec = vec![item];
                                    item = self.call_user_function(trans.as_ref(), &args_vec);
                                }

                                Value::Option { is_some: true, value: Box::new(item) }
                            }
                            other => other,
                        }
                    }
                    _ => Value::Error("Invalid iterator source".to_string()),
                }
            }
            _ => Value::Error("next() can only be called on iterators".to_string()),
        }
    }

    /// Converts a runtime value to a string for display
    fn stringify_value(value: &Value) -> String {
        match value {
            Value::Str(s) => s.as_ref().clone(),
            Value::Int(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Tagged { tag, fields } => {
                if fields.is_empty() {
                    tag.clone()
                } else {
                    let args: Vec<String> =
                        fields.values().map(Interpreter::stringify_value).collect();
                    format!("{}({})", tag, args.join(","))
                }
            }
            Value::Struct { name, fields } => {
                let mut keys: Vec<&String> = fields.keys().collect();
                keys.sort();
                let field_strs: Vec<String> = keys
                    .iter()
                    .map(|k| {
                        format!("{}: {}", k, Interpreter::stringify_value(fields.get(*k).unwrap()))
                    })
                    .collect();
                format!("{} {{ {} }}", name, field_strs.join(", "))
            }
            Value::Array(elements) => {
                let elem_strs: Vec<String> =
                    elements.iter().map(Interpreter::stringify_value).collect();
                format!("[{}]", elem_strs.join(", "))
            }
            Value::Dict(map) => {
                let mut keys: Vec<&Arc<str>> = map.keys().collect();
                keys.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
                let pair_strs: Vec<String> = keys
                    .iter()
                    .map(|k| {
                        format!(
                            "\"{}\": {}",
                            k,
                            Interpreter::stringify_value(map.get(k.as_ref()).unwrap())
                        )
                    })
                    .collect();
                format!("{{{}}}", pair_strs.join(", "))
            }
            Value::FixedDict { keys, values } => {
                let mut pairs: Vec<(&Arc<str>, &Value)> = keys.iter().zip(values.iter()).collect();
                pairs.sort_by(|(a, _), (b, _)| a.as_ref().cmp(b.as_ref()));
                let pair_strs: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, Interpreter::stringify_value(v)))
                    .collect();
                format!("{{{}}}", pair_strs.join(", "))
            }
            Value::IntDict(dict) => {
                let mut keys: Vec<i64> = dict.keys().copied().collect();
                keys.sort();
                let pair_strs: Vec<String> = keys
                    .iter()
                    .map(|k| {
                        format!("\"{}\": {}", k, Interpreter::stringify_value(dict.get(k).unwrap()))
                    })
                    .collect();
                format!("{{{}}}", pair_strs.join(", "))
            }
            Value::DenseIntDict(values) => {
                let pair_strs: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        format!("\"{}\": {}", index, Interpreter::stringify_value(value))
                    })
                    .collect();
                format!("{{{}}}", pair_strs.join(", "))
            }
            Value::DenseIntDictInt(values) => {
                let pair_strs: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| match value {
                        Some(value) => format!("\"{}\": {}", index, value),
                        None => format!("\"{}\": null", index),
                    })
                    .collect();
                format!("{{{}}}", pair_strs.join(", "))
            }
            Value::DenseIntDictIntFull(values) => {
                let pair_strs: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| format!("\"{}\": {}", index, value))
                    .collect();
                format!("{{{}}}", pair_strs.join(", "))
            }
            Value::Return(inner) => Interpreter::stringify_value(inner),
            Value::Error(msg) => format!("Error: {}", msg),
            Value::ErrorObject { message, .. } => format!("Error: {}", message),
            Value::NativeFunction(name) => format!("<native function: {}>", name),
            Value::Result { is_ok, value } => {
                if *is_ok {
                    format!("Ok({})", Interpreter::stringify_value(value))
                } else {
                    format!("Err({})", Interpreter::stringify_value(value))
                }
            }
            Value::Option { is_some, value } => {
                if *is_some {
                    format!("Some({})", Interpreter::stringify_value(value))
                } else {
                    "None".to_string()
                }
            }
            Value::GeneratorDef(params, _) => {
                format!("<generator function with {} params>", params.len())
            }
            Value::Generator { params, is_exhausted, .. } => {
                if *is_exhausted {
                    format!("<exhausted generator ({} params)>", params.len())
                } else {
                    format!("<generator ({} params)>", params.len())
                }
            }
            Value::Iterator { .. } => "<iterator>".to_string(),
            _ => "<unknown>".into(),
        }
    }

    /// Cleanup method to rollback any active transactions before interpreter is dropped
    /// This prevents hanging when SQLite connections are dropped while in transaction
    pub fn cleanup(&mut self) {
        // Get all variables from the environment
        let var_names: Vec<String> =
            self.env.scopes.iter().flat_map(|scope| scope.keys().cloned()).collect();

        for var_name in var_names {
            if let Some(Value::Database { connection, db_type, in_transaction, .. }) =
                self.env.get(&var_name)
            {
                // Check if in transaction
                let is_in_trans = {
                    let in_trans = in_transaction.lock().unwrap();
                    *in_trans
                };

                if is_in_trans {
                    // Rollback the transaction
                    match (connection, db_type.as_str()) {
                        (DatabaseConnection::Sqlite(conn_arc), "sqlite") => {
                            let conn = conn_arc.lock().unwrap();
                            let _ = conn.execute("ROLLBACK", []);
                        }
                        (DatabaseConnection::Postgres(client_arc), "postgres") => {
                            let mut client = client_arc.lock().unwrap();
                            let _ = client.execute("ROLLBACK", &[]);
                        }
                        (DatabaseConnection::Mysql(conn_arc), "mysql") => {
                            let mut conn = conn_arc.lock().unwrap();
                            if let Ok(runtime) = tokio::runtime::Runtime::new() {
                                let _ = runtime.block_on(async {
                                    conn.exec_drop("ROLLBACK", mysql_async::Params::Empty).await
                                });
                            }
                        }
                        _ => {}
                    }

                    // Update transaction flag
                    let mut in_trans = in_transaction.lock().unwrap();
                    *in_trans = false;
                }
            }
        }
    }
}
