// File: src/interpreter/value.rs
//
// Runtime value types for the Ruff programming language.
// Defines all value types that can be represented and manipulated at runtime.

use crate::ast::Stmt;
use image::DynamicImage;
use mysql_async::Conn as MysqlConn;
use postgres::Client as PostgresClient;
use rusqlite::Connection as SqliteConnection;
use std::collections::HashMap;
use std::fs::File;
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use zip::ZipWriter;

// Forward declaration - Environment is in a sibling module
use super::environment::Environment;

/// Wrapper type for function bodies that prevents deep recursion during drop.
///
/// The issue: Function bodies are Vec<Stmt>, and Stmt contains nested Vec<Stmt>
/// (in For, If, While, etc.). When Rust's automatic drop runs during program cleanup,
/// it recurses deeply through these structures, causing stack overflow.
///
/// Solution: This wrapper uses ManuallyDrop to prevent automatic dropping of the Arc.
/// The memory will be leaked, but since this only happens during program shutdown,
/// the OS will reclaim all memory anyway.
///
/// TODO (Roadmap Task #29): Replace with iterative drop or arena allocation
#[derive(Clone)]
pub struct LeakyFunctionBody(ManuallyDrop<Arc<Vec<Stmt>>>);

impl LeakyFunctionBody {
    pub fn new(body: Vec<Stmt>) -> Self {
        LeakyFunctionBody(ManuallyDrop::new(Arc::new(body)))
    }

    pub fn get(&self) -> &Vec<Stmt> {
        &self.0
    }
}

/// Database connection types
/// Infrastructure for database.rs stub module
#[derive(Clone)]
pub enum DatabaseConnection {
    #[allow(dead_code)]
    Sqlite(Arc<Mutex<SqliteConnection>>),
    #[allow(dead_code)]
    Postgres(Arc<Mutex<PostgresClient>>),
    #[allow(dead_code)]
    Mysql(Arc<Mutex<MysqlConn>>),
}

/// Connection pool for database connections
/// Infrastructure for database.rs stub module
#[derive(Clone)]
pub struct ConnectionPool {
    #[allow(dead_code)]
    pub(crate) db_type: String,
    #[allow(dead_code)]
    pub(crate) connection_string: String,
    #[allow(dead_code)] // Reserved for future use
    pub(crate) min_connections: usize,
    #[allow(dead_code)]
    pub(crate) max_connections: usize,
    #[allow(dead_code)]
    pub(crate) connection_timeout: u64, // seconds
    #[allow(dead_code)]
    pub(crate) available: Arc<Mutex<std::collections::VecDeque<DatabaseConnection>>>,
    #[allow(dead_code)]
    pub(crate) in_use: Arc<Mutex<usize>>,
    #[allow(dead_code)]
    pub(crate) total_created: Arc<Mutex<usize>>,
}

impl ConnectionPool {
    #[allow(dead_code)]
    pub fn new(
        db_type: String,
        connection_string: String,
        config: HashMap<String, Value>,
    ) -> Result<Self, String> {
        // Parse configuration
        let min_connections = config
            .get("min_connections")
            .and_then(|v| match v {
                Value::Int(n) => Some(*n as usize),
                Value::Float(n) => Some(*n as usize),
                _ => None,
            })
            .unwrap_or(5);

        let max_connections = config
            .get("max_connections")
            .and_then(|v| match v {
                Value::Int(n) => Some(*n as usize),
                Value::Float(n) => Some(*n as usize),
                _ => None,
            })
            .unwrap_or(20);

        let connection_timeout = config
            .get("connection_timeout")
            .and_then(|v| match v {
                Value::Int(n) => Some(*n as u64),
                Value::Float(n) => Some(*n as u64),
                _ => None,
            })
            .unwrap_or(30);

        if min_connections > max_connections {
            return Err("min_connections cannot be greater than max_connections".to_string());
        }

        Ok(ConnectionPool {
            db_type,
            connection_string,
            min_connections,
            max_connections,
            connection_timeout,
            available: Arc::new(Mutex::new(std::collections::VecDeque::new())),
            in_use: Arc::new(Mutex::new(0)),
            total_created: Arc::new(Mutex::new(0)),
        })
    }

    #[allow(dead_code)]
    pub fn acquire(&self) -> Result<DatabaseConnection, String> {
        let start_time = std::time::Instant::now();

        loop {
            // Try to get an available connection
            {
                let mut available = self.available.lock().unwrap();
                if let Some(conn) = available.pop_front() {
                    let mut in_use = self.in_use.lock().unwrap();
                    *in_use += 1;
                    return Ok(conn);
                }
            }

            // No available connections - try to create a new one
            {
                let total = self.total_created.lock().unwrap();
                if *total < self.max_connections {
                    drop(total); // Release lock before creating connection

                    // Create new connection
                    let conn = self.create_connection()?;

                    let mut total = self.total_created.lock().unwrap();
                    *total += 1;
                    let mut in_use = self.in_use.lock().unwrap();
                    *in_use += 1;

                    return Ok(conn);
                }
            }

            // All connections in use and at max - check timeout
            if start_time.elapsed().as_secs() >= self.connection_timeout {
                return Err("Connection pool timeout: all connections are in use".to_string());
            }

            // Wait a bit before retrying
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    #[allow(dead_code)]
    pub fn release(&self, conn: DatabaseConnection) {
        let mut available = self.available.lock().unwrap();
        available.push_back(conn);
        let mut in_use = self.in_use.lock().unwrap();
        if *in_use > 0 {
            *in_use -= 1;
        }
    }

    #[allow(dead_code)]
    pub fn stats(&self) -> HashMap<String, usize> {
        let available = self.available.lock().unwrap();
        let in_use = self.in_use.lock().unwrap();
        let total = self.total_created.lock().unwrap();

        let mut stats = HashMap::new();
        stats.insert("available".to_string(), available.len());
        stats.insert("in_use".to_string(), *in_use);
        stats.insert("total".to_string(), *total);
        stats.insert("max".to_string(), self.max_connections);
        stats
    }

    #[allow(dead_code)]
    pub fn close(&self) {
        let mut available = self.available.lock().unwrap();
        available.clear();
        let mut in_use = self.in_use.lock().unwrap();
        *in_use = 0;
        let mut total = self.total_created.lock().unwrap();
        *total = 0;
    }

    #[allow(dead_code)]
    fn create_connection(&self) -> Result<DatabaseConnection, String> {
        use postgres::NoTls;

        match self.db_type.as_str() {
            "sqlite" => SqliteConnection::open(&self.connection_string)
                .map(|conn| DatabaseConnection::Sqlite(Arc::new(Mutex::new(conn))))
                .map_err(|e| format!("Failed to create SQLite connection: {}", e)),
            "postgres" | "postgresql" => PostgresClient::connect(&self.connection_string, NoTls)
                .map(|client| DatabaseConnection::Postgres(Arc::new(Mutex::new(client))))
                .map_err(|e| format!("Failed to create PostgreSQL connection: {}", e)),
            "mysql" => {
                let opts = mysql_async::OptsBuilder::from_opts(
                    mysql_async::Opts::from_url(&self.connection_string)
                        .map_err(|e| format!("Invalid MySQL connection string: {}", e))?,
                );

                let runtime = tokio::runtime::Runtime::new()
                    .map_err(|e| format!("Failed to create runtime: {}", e))?;

                runtime.block_on(async {
                    mysql_async::Conn::new(opts)
                        .await
                        .map(|conn| DatabaseConnection::Mysql(Arc::new(Mutex::new(conn))))
                        .map_err(|e| format!("Failed to create MySQL connection: {}", e))
                })
            }
            _ => Err(format!("Unsupported database type: {}", self.db_type)),
        }
    }
}

/// Runtime values in the Ruff interpreter
///
/// This enum represents all possible runtime values in Ruff. It's a large enum
/// with 30+ variants covering primitives, collections, functions, I/O types, and more.
#[derive(Clone)]
pub enum Value {
    /// Tagged enum variant with named fields
    Tagged { tag: String, fields: HashMap<String, Value> },
    /// 64-bit signed integer
    Int(i64),
    /// 64-bit floating point number
    Float(f64),
    /// String value (reference-counted for cheap cloning)
    Str(Arc<String>),
    /// Boolean value
    Bool(bool),
    /// Null value for optional chaining and null coalescing
    Null,
    /// Binary data for files, HTTP downloads, etc.
    Bytes(Vec<u8>),
    /// Function: parameters, body, optional captured environment
    Function(Vec<String>, LeakyFunctionBody, Option<Arc<Mutex<Environment>>>),
    /// Async function: parameters, body, optional captured environment
    AsyncFunction(Vec<String>, LeakyFunctionBody, Option<Arc<Mutex<Environment>>>),
    /// Native (built-in) function by name
    NativeFunction(String),
    /// Bytecode function (experimental - VM not yet default)
    #[allow(dead_code)]
    BytecodeFunction {
        chunk: crate::bytecode::BytecodeChunk,
        /// Captured variables with shared mutable state
        captured: HashMap<String, Arc<Mutex<Value>>>,
    },
    /// Bytecode generator instance with execution state (VM-based generators)
    #[allow(dead_code)]
    BytecodeGenerator { state: Arc<Mutex<crate::vm::GeneratorState>> },
    /// Internal marker for dynamic array construction in VM
    ArrayMarker,
    /// Return value wrapper
    Return(Box<Value>),
    /// Legacy simple error string
    Error(String),
    /// Rich error object with stack trace and chaining
    ErrorObject {
        message: String,
        stack: Vec<String>,
        line: Option<usize>,
        cause: Option<Box<Value>>,
    },
    /// Enum type (currently unused)
    #[allow(dead_code)]
    Enum(String),
    /// Struct instance with fields
    Struct { name: String, fields: HashMap<String, Value> },
    /// Struct definition with methods
    StructDef { name: String, field_names: Vec<String>, methods: HashMap<String, Value> },
    /// Array of values (reference-counted for cheap cloning)
    Array(Arc<Vec<Value>>),
    /// Dictionary (hash map) of string keys to values (reference-counted for cheap cloning)
    Dict(Arc<HashMap<String, Value>>),
    /// Set of unique values
    Set(Vec<Value>),
    /// FIFO queue
    Queue(std::collections::VecDeque<Value>),
    /// LIFO stack
    Stack(Vec<Value>),
    /// Thread-safe channel for message passing
    Channel(Arc<Mutex<(std::sync::mpsc::Sender<Value>, std::sync::mpsc::Receiver<Value>)>>),
    /// HTTP server with routes
    HttpServer {
        port: u16,
        routes: Vec<(String, String, Value)>, // (method, path, handler)
    },
    /// HTTP response
    HttpResponse { status: u16, body: String, headers: HashMap<String, String> },
    /// Database connection
    /// Infrastructure for database.rs stub module
    #[allow(dead_code)]
    Database {
        connection: DatabaseConnection,
        db_type: String,
        connection_string: String,
        in_transaction: Arc<Mutex<bool>>,
    },
    /// Database connection pool
    /// Infrastructure for database.rs stub module
    #[allow(dead_code)]
    DatabasePool { pool: Arc<Mutex<ConnectionPool>> },
    /// Image data
    Image { data: Arc<Mutex<DynamicImage>>, format: String },
    /// Zip archive writer
    /// Infrastructure for zip.rs stub module
    #[allow(dead_code)]
    ZipArchive { writer: Arc<Mutex<Option<ZipWriter<File>>>>, path: String },
    /// TCP listener for accepting connections
    /// Infrastructure for network.rs stub module
    #[allow(dead_code)]
    TcpListener { listener: Arc<Mutex<std::net::TcpListener>>, addr: String },
    /// TCP stream for bidirectional communication
    /// Infrastructure for network.rs stub module
    #[allow(dead_code)]
    TcpStream { stream: Arc<Mutex<std::net::TcpStream>>, peer_addr: String },
    /// UDP socket for datagram communication
    /// Infrastructure for network.rs stub module
    #[allow(dead_code)]
    UdpSocket { socket: Arc<Mutex<std::net::UdpSocket>>, addr: String },
    /// Result type: Ok(value) or Err(error)
    Result { is_ok: bool, value: Box<Value> },
    /// Option type: Some(value) or None
    Option { is_some: bool, value: Box<Value> },
    /// Generator definition (before being called)
    GeneratorDef(Vec<String>, LeakyFunctionBody),
    /// Generator instance with execution state
    Generator {
        params: Vec<String>,
        body: LeakyFunctionBody,
        env: Arc<Mutex<Environment>>,
        pc: usize, // Program counter
        is_exhausted: bool,
    },
    /// Iterator instance wrapping a collection or generator
    Iterator {
        source: Box<Value>,
        index: usize,
        transformer: Option<Box<Value>>,
        filter_fn: Option<Box<Value>>,
        take_count: Option<usize>,
    },
    /// Promise for async computation results
    Promise {
        receiver: Arc<Mutex<tokio::sync::oneshot::Receiver<Result<Value, String>>>>,
        is_polled: Arc<Mutex<bool>>,
        cached_result: Arc<Mutex<Option<Result<Value, String>>>>,
        /// Optional join handle for spawned async task (None for already-resolved promises)
        task_handle: Option<Arc<Mutex<Option<tokio::task::JoinHandle<Result<Value, String>>>>>>,
    },
    /// Task handle for spawned async tasks
    TaskHandle {
        handle: Arc<Mutex<Option<tokio::task::JoinHandle<Value>>>>,
        is_cancelled: Arc<Mutex<bool>>,
    },
}

// Manual Debug implementation for Value
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Tagged { tag, fields } => {
                f.debug_struct("Tagged").field("tag", tag).field("fields", fields).finish()
            }
            Value::Int(n) => write!(f, "Int({})", n),
            Value::Float(n) => write!(f, "Float({})", n),
            Value::Str(s) => write!(f, "Str({:?})", s.as_ref()),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Null => write!(f, "Null"),
            Value::Bytes(bytes) => write!(f, "Bytes({} bytes)", bytes.len()),
            Value::Function(params, body, captured_env) => {
                let env_info = if captured_env.is_some() { " +closure" } else { "" };
                write!(f, "Function({:?}, {} stmts{})", params, body.get().len(), env_info)
            }
            Value::AsyncFunction(params, body, captured_env) => {
                let env_info = if captured_env.is_some() { " +closure" } else { "" };
                write!(f, "AsyncFunction({:?}, {} stmts{})", params, body.get().len(), env_info)
            }
            Value::NativeFunction(name) => write!(f, "NativeFunction({})", name),
            Value::BytecodeFunction { chunk, captured } => {
                let name = chunk.name.as_deref().unwrap_or("<lambda>");
                write!(
                    f,
                    "BytecodeFunction({}, {} instructions, {} captured)",
                    name,
                    chunk.instructions.len(),
                    captured.len()
                )
            }
            Value::BytecodeGenerator { state } => {
                let state_lock = state.lock().unwrap();
                write!(
                    f,
                    "BytecodeGenerator(ip={}, exhausted={})",
                    state_lock.ip, state_lock.is_exhausted
                )
            }
            Value::ArrayMarker => write!(f, "ArrayMarker"),
            Value::Return(v) => write!(f, "Return({:?})", v),
            Value::Error(e) => write!(f, "Error({})", e),
            Value::ErrorObject { message, stack, line, cause } => f
                .debug_struct("ErrorObject")
                .field("message", message)
                .field("stack", stack)
                .field("line", line)
                .field("cause", &cause.as_ref().map(|_| "..."))
                .finish(),
            Value::Enum(e) => write!(f, "Enum({})", e),
            Value::Struct { name, fields } => {
                f.debug_struct("Struct").field("name", name).field("fields", fields).finish()
            }
            Value::StructDef { name, field_names, methods } => f
                .debug_struct("StructDef")
                .field("name", name)
                .field("field_names", field_names)
                .field("methods", &format!("{} methods", methods.len()))
                .finish(),
            Value::Array(elements) => write!(f, "Array[{}]", elements.len()),
            Value::Dict(map) => write!(f, "Dict{{{} keys}}", map.len()),
            Value::Set(elements) => write!(f, "Set{{{} items}}", elements.len()),
            Value::Queue(queue) => write!(f, "Queue({} items)", queue.len()),
            Value::Stack(stack) => write!(f, "Stack({} items)", stack.len()),
            Value::Channel(_) => write!(f, "Channel"),
            Value::HttpServer { port, routes } => {
                write!(f, "HttpServer(port={}, {} routes)", port, routes.len())
            }
            Value::HttpResponse { status, body, .. } => {
                write!(f, "HttpResponse(status={}, body_len={})", status, body.len())
            }
            Value::Database { db_type, connection_string, .. } => {
                write!(f, "Database(type={}, connection={})", db_type, connection_string)
            }
            Value::DatabasePool { pool } => {
                let p = pool.lock().unwrap();
                write!(f, "DatabasePool(type={}, max={})", p.db_type, p.max_connections)
            }
            Value::Image { format, data } => {
                let img = data.lock().unwrap();
                write!(f, "Image({}x{}, format={})", img.width(), img.height(), format)
            }
            Value::ZipArchive { path, .. } => {
                write!(f, "ZipArchive(path={})", path)
            }
            Value::TcpListener { addr, .. } => {
                write!(f, "TcpListener(addr={})", addr)
            }
            Value::TcpStream { peer_addr, .. } => {
                write!(f, "TcpStream(peer={})", peer_addr)
            }
            Value::UdpSocket { addr, .. } => {
                write!(f, "UdpSocket(addr={})", addr)
            }
            Value::Result { is_ok, value } => {
                if *is_ok {
                    write!(f, "Ok({:?})", value)
                } else {
                    write!(f, "Err({:?})", value)
                }
            }
            Value::Option { is_some, value } => {
                if *is_some {
                    write!(f, "Some({:?})", value)
                } else {
                    write!(f, "None")
                }
            }
            Value::GeneratorDef(params, body) => {
                write!(f, "GeneratorDef({:?}, {} stmts)", params, body.get().len())
            }
            Value::Generator { params, is_exhausted, pc, .. } => {
                write!(f, "Generator({:?}, pc={}, exhausted={})", params, pc, is_exhausted)
            }
            Value::Iterator { source, index, .. } => {
                write!(f, "Iterator(source={:?}, index={})", source, index)
            }
            Value::Promise { cached_result, .. } => {
                let result = cached_result.lock().unwrap();
                match &*result {
                    None => write!(f, "Promise(Pending)"),
                    Some(Ok(_)) => write!(f, "Promise(Resolved)"),
                    Some(Err(err)) => write!(f, "Promise(Rejected: {})", err),
                }
            }
            Value::TaskHandle { is_cancelled, .. } => {
                let cancelled = is_cancelled.lock().unwrap();
                if *cancelled {
                    write!(f, "TaskHandle(Cancelled)")
                } else {
                    write!(f, "TaskHandle(Running)")
                }
            }
        }
    }
}

impl Value {
    /// Helper to create a Str value from a String
    pub fn str(s: String) -> Self {
        Value::Str(Arc::new(s))
    }

    /// Helper to create a Str value from a &str
    pub fn str_ref(s: &str) -> Self {
        Value::Str(Arc::new(s.to_string()))
    }

    /// Helper to create an Array value from a Vec<Value>
    pub fn array(vec: Vec<Value>) -> Self {
        Value::Array(Arc::new(vec))
    }

    /// Helper to create a Dict value from a HashMap<String, Value>
    pub fn dict(map: HashMap<String, Value>) -> Self {
        Value::Dict(Arc::new(map))
    }
}
