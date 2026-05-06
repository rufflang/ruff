// File: src/main.rs
//
// Main entry point for the Ruff programming language interpreter.
// Handles command-line argument parsing and dispatches to the appropriate
// subcommand (run, repl, or test).

mod ast;
mod benchmarks;
mod builtins;
mod bytecode;
mod compiler;
mod doc_generator;
mod errors;
mod formatter;
mod interpreter;
mod jit;
mod lexer;
mod linter;
mod lsp_completion;
mod lsp_code_actions;
mod lsp_definition;
mod lsp_diagnostics;
mod lsp_hover;
mod lsp_server;
mod lsp_rename;
mod lsp_references;
mod module;
mod optimizer;
mod package_workflow;
mod parser;
mod repl;
mod type_checker;
mod vm;

use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tiny_http::{Header, Method, Response, Server, StatusCode};

#[derive(ClapParser)]
#[command(
    name = "ruff",
    about = "Ruff: A modern programming language",
    version = env!("CARGO_PKG_VERSION"),
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[command(arg_required_else_help = true)]
enum Commands {
    /// Run a Ruff script file
    Run {
        /// Path to the .ruff file
        file: PathBuf,

        /// Use tree-walking interpreter instead of bytecode VM (default: VM)
        #[arg(long)]
        interpreter: bool,

        /// Cooperative scheduler timeout in milliseconds (overrides env/default)
        #[arg(long)]
        scheduler_timeout_ms: Option<u64>,

        /// Arguments to pass to the script
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        script_args: Vec<String>,
    },

    /// Serve a directory over HTTP for local preview/testing
    Serve {
        /// Directory to serve (default: current directory)
        #[arg(default_value = ".")]
        dir: PathBuf,

        /// Port to bind
        #[arg(long, default_value_t = 8080)]
        port: u16,

        /// Host/interface to bind
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Default index file when requesting '/'
        #[arg(long, default_value = "index.html")]
        index: String,
    },

    /// Launch interactive Ruff REPL
    Repl,

    /// Run all test scripts in the tests/ directory
    Test {
        /// Regenerate all .out files based on actual output
        #[arg(long)]
        update: bool,
    },

    /// Run tests defined with the test framework
    TestRun {
        /// Path to the .ruff file containing tests
        file: PathBuf,

        /// Print detailed output for each test
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run performance benchmarks
    Bench {
        /// Path to benchmark file or directory
        path: Option<PathBuf>,

        /// Number of iterations per benchmark (default: 10)
        #[arg(short, long, default_value_t = 10)]
        iterations: usize,

        /// Number of warmup runs (default: 2)
        #[arg(short, long, default_value_t = 2)]
        warmup: usize,
    },

    /// Format a Ruff source file
    Format {
        /// Path to the .ruff file
        file: PathBuf,

        /// Indentation width (spaces)
        #[arg(long, default_value_t = 4)]
        indent: usize,

        /// Maximum preferred line length
        #[arg(long, default_value_t = 100)]
        line_length: usize,

        /// Disable import sorting
        #[arg(long, default_value_t = false)]
        no_sort_imports: bool,

        /// Check if formatting changes are required
        #[arg(long, default_value_t = false)]
        check: bool,

        /// Write formatted output back to file
        #[arg(long, default_value_t = false)]
        write: bool,

        /// Print formatter result as JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Lint a Ruff source file
    Lint {
        /// Path to the .ruff file
        file: PathBuf,

        /// Apply safe autofixes
        #[arg(long, default_value_t = false)]
        fix: bool,

        /// Print lint issues as JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Initialize a Ruff project with ruff.toml and src/main.ruff
    Init {
        /// Project directory (defaults to current directory)
        #[arg(long)]
        dir: Option<PathBuf>,

        /// Package name override (defaults to directory name)
        #[arg(long)]
        name: Option<String>,
    },

    /// Add a dependency to ruff.toml
    PackageAdd {
        /// Dependency name
        name: String,

        /// Dependency version requirement
        #[arg(long, default_value = "*" )]
        version: String,

        /// Path to ruff.toml (defaults to ./ruff.toml)
        #[arg(long)]
        manifest: Option<PathBuf>,
    },

    /// Validate dependencies declared in ruff.toml
    PackageInstall {
        /// Path to ruff.toml (defaults to ./ruff.toml)
        #[arg(long)]
        manifest: Option<PathBuf>,
    },

    /// Preview package publish metadata from ruff.toml
    PackagePublish {
        /// Path to ruff.toml (defaults to ./ruff.toml)
        #[arg(long)]
        manifest: Option<PathBuf>,

        /// Execute publish instead of dry-run preview
        #[arg(long, default_value_t = false)]
        publish: bool,
    },

    /// Generate HTML documentation from Ruff /// comments
    Docgen {
        /// Path to the .ruff file
        file: PathBuf,

        /// Output directory for generated docs (defaults to docs/generated)
        #[arg(long)]
        out_dir: Option<PathBuf>,

        /// Disable builtin/native API reference generation
        #[arg(long, default_value_t = false)]
        no_builtins: bool,

        /// Print documentation generation result as JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Compare Ruff parallel_map benchmark against Python ProcessPoolExecutor
    BenchCross {
        /// Path to Ruff benchmark script
        #[arg(long, default_value = "benchmarks/cross-language/bench_parallel_map.ruff")]
        ruff_script: PathBuf,

        /// Path to Python ProcessPool benchmark script
        #[arg(long, default_value = "benchmarks/cross-language/bench_process_pool.py")]
        python_script: PathBuf,

        /// Python executable to use
        #[arg(long, default_value = "python3")]
        python: String,
    },

    /// Run the async SSG benchmark and optionally compare with Python
    BenchSsg {
        /// Path to Ruff SSG benchmark script
        #[arg(long, default_value = "benchmarks/cross-language/bench_ssg.ruff")]
        ruff_script: PathBuf,

        /// Number of warmup runs excluded from measured summary
        #[arg(long, default_value_t = 0)]
        warmup_runs: usize,

        /// Number of repeated benchmark runs for noise reduction (median reporting)
        #[arg(long, default_value_t = 1)]
        runs: usize,

        /// Optional Ruff median build-time gate in milliseconds (fails command on miss)
        #[arg(long)]
        throughput_gate_ms: Option<f64>,

        /// Print per-stage timing breakdown and bottleneck summary when available
        #[arg(long, default_value_t = false)]
        profile_async: bool,

        /// Compare against the Python baseline benchmark
        #[arg(long, default_value_t = false)]
        compare_python: bool,

        /// Path to Python SSG benchmark script
        #[arg(long, default_value = "benchmarks/cross-language/bench_ssg.py")]
        python_script: PathBuf,

        /// Python executable to use for comparison
        #[arg(long, default_value = "python3")]
        python: String,

        /// Optional temp root for benchmark artifacts (overrides workspace tmp/)
        #[arg(long)]
        tmp_dir: Option<PathBuf>,

        /// CV threshold (%) for measurement variability warnings
        #[arg(long, default_value_t = benchmarks::ssg::SSG_VARIABILITY_WARNING_THRESHOLD_PERCENT)]
        variability_warning_threshold: f64,

        /// Percent threshold for first-to-last trend drift warnings
        #[arg(long, default_value_t = benchmarks::ssg::SSG_TREND_WARNING_THRESHOLD_PERCENT)]
        trend_warning_threshold: f64,

        /// Percent threshold for mean-vs-median drift warnings
        #[arg(long, default_value_t = benchmarks::ssg::SSG_MEAN_MEDIAN_DRIFT_WARNING_THRESHOLD_PERCENT)]
        mean_median_drift_warning_threshold: f64,

        /// Percent threshold for min/max range-spread warnings relative to median
        #[arg(long, default_value_t = benchmarks::ssg::SSG_RANGE_SPREAD_WARNING_THRESHOLD_PERCENT)]
        range_spread_warning_threshold: f64,
    },

    /// Profile a Ruff script (CPU, memory, JIT stats)
    Profile {
        /// Path to the .ruff file
        file: PathBuf,

        /// Enable CPU profiling
        #[arg(long, default_value_t = true)]
        cpu: bool,

        /// Enable memory profiling
        #[arg(long, default_value_t = true)]
        memory: bool,

        /// Enable JIT statistics
        #[arg(long, default_value_t = true)]
        jit: bool,

        /// Generate flamegraph output file
        #[arg(long)]
        flamegraph: Option<PathBuf>,
    },

    /// Return completion candidates for LSP/autocomplete integration
    LspComplete {
        /// Path to the .ruff file
        file: PathBuf,

        /// 1-based line number for the completion request
        #[arg(long)]
        line: usize,

        /// 1-based column number for the completion request
        #[arg(long)]
        column: usize,

        /// Print completion items as JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Return go-to-definition location for an identifier under cursor
    LspDefinition {
        /// Path to the .ruff file
        file: PathBuf,

        /// 1-based line number for the definition request
        #[arg(long)]
        line: usize,

        /// 1-based column number for the definition request
        #[arg(long)]
        column: usize,

        /// Print definition result as JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Return all references for the identifier under cursor
    LspReferences {
        /// Path to the .ruff file
        file: PathBuf,

        /// 1-based line number for the references request
        #[arg(long)]
        line: usize,

        /// 1-based column number for the references request
        #[arg(long)]
        column: usize,

        /// Include definition location in results
        #[arg(long, default_value_t = true)]
        include_definition: bool,

        /// Print references as JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Return hover information for the identifier under cursor
    LspHover {
        /// Path to the .ruff file
        file: PathBuf,

        /// 1-based line number for the hover request
        #[arg(long)]
        line: usize,

        /// 1-based column number for the hover request
        #[arg(long)]
        column: usize,

        /// Print hover info as JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Return diagnostics for source code at editor refresh time
    LspDiagnostics {
        /// Path to the .ruff file
        file: PathBuf,

        /// Print diagnostics as JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Rename the symbol under cursor and return edits
    LspRename {
        /// Path to the .ruff file
        file: PathBuf,

        /// 1-based line number for the rename request
        #[arg(long)]
        line: usize,

        /// 1-based column number for the rename request
        #[arg(long)]
        column: usize,

        /// New symbol name
        #[arg(long)]
        new_name: String,

        /// Print rename result as JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Return quick-fix code actions based on diagnostics
    LspCodeActions {
        /// Path to the .ruff file
        file: PathBuf,

        /// Print code actions as JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Launch Ruff as a long-running Language Server Protocol server (JSON-RPC over stdio)
    Lsp {
        /// Emit deterministic request/response logs to stderr for debugging
        #[arg(long, default_value_t = false)]
        deterministic_logs: bool,

        /// Timeout budget in milliseconds for a single LSP request
        #[arg(long, default_value_t = 5000)]
        request_timeout_ms: u64,
    },
}

const DEFAULT_COOPERATIVE_SCHEDULER_TIMEOUT_MS: u64 = 120_000;

fn cooperative_scheduler_timeout(
    cli_timeout_ms: Option<u64>,
) -> Result<std::time::Duration, String> {
    if let Some(timeout_ms) = cli_timeout_ms {
        if timeout_ms == 0 {
            return Err("Scheduler timeout must be greater than 0ms".to_string());
        }

        return Ok(std::time::Duration::from_millis(timeout_ms));
    }

    match std::env::var("RUFF_SCHEDULER_TIMEOUT_MS") {
        Ok(raw_timeout_ms) => match raw_timeout_ms.parse::<u64>() {
            Ok(timeout_ms) if timeout_ms > 0 => Ok(std::time::Duration::from_millis(timeout_ms)),
            _ => Ok(std::time::Duration::from_millis(DEFAULT_COOPERATIVE_SCHEDULER_TIMEOUT_MS)),
        },
        Err(_) => Ok(std::time::Duration::from_millis(DEFAULT_COOPERATIVE_SCHEDULER_TIMEOUT_MS)),
    }
}

fn split_content_path_and_encoding(path: &Path) -> (PathBuf, Option<&'static str>) {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    match extension.as_deref() {
        Some("gz") => {
            if let Some(file_stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                (path.with_file_name(file_stem), Some("gzip"))
            } else {
                (path.to_path_buf(), Some("gzip"))
            }
        }
        Some("br") => {
            if let Some(file_stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                (path.with_file_name(file_stem), Some("br"))
            } else {
                (path.to_path_buf(), Some("br"))
            }
        }
        _ => (path.to_path_buf(), None),
    }
}

fn has_known_safe_extension(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    matches!(
        extension.as_deref(),
        Some("html")
            | Some("htm")
            | Some("css")
            | Some("js")
            | Some("mjs")
            | Some("cjs")
            | Some("json")
            | Some("map")
            | Some("xml")
            | Some("txt")
            | Some("text")
            | Some("md")
            | Some("csv")
            | Some("wasm")
            | Some("pdf")
            | Some("svg")
            | Some("png")
            | Some("jpg")
            | Some("jpeg")
            | Some("gif")
            | Some("webp")
            | Some("avif")
            | Some("ico")
            | Some("bmp")
            | Some("woff")
            | Some("woff2")
            | Some("ttf")
            | Some("otf")
            | Some("mp3")
            | Some("wav")
            | Some("ogg")
            | Some("mp4")
            | Some("webm")
    )
}

fn is_potentially_active_content_type(content_type: &str) -> bool {
    matches!(
        content_type,
        "text/html"
            | "application/xhtml+xml"
            | "image/svg+xml"
            | "application/xml"
            | "text/xml"
            | "application/javascript"
            | "text/javascript"
    )
}

fn sanitize_header_value(value: &str) -> Option<String> {
    if value.bytes().any(|byte| byte < 0x20 || byte == 0x7f) {
        return None;
    }

    Some(value.to_string())
}

fn guess_content_type(path: &Path, file_bytes: &[u8]) -> String {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    if let Some(ext) = extension.as_deref() {
        match ext {
            "html" | "htm" => return "text/html; charset=utf-8".to_string(),
            "css" => return "text/css; charset=utf-8".to_string(),
            "js" | "mjs" | "cjs" => {
                return "application/javascript; charset=utf-8".to_string()
            }
            "json" | "map" => return "application/json; charset=utf-8".to_string(),
            "xml" => return "application/xml; charset=utf-8".to_string(),
            "txt" | "text" | "md" | "csv" => {
                return "text/plain; charset=utf-8".to_string()
            }
            "wasm" => return "application/wasm".to_string(),
            "pdf" => return "application/pdf".to_string(),
            "svg" => return "image/svg+xml".to_string(),
            "png" => return "image/png".to_string(),
            "jpg" | "jpeg" => return "image/jpeg".to_string(),
            "gif" => return "image/gif".to_string(),
            "webp" => return "image/webp".to_string(),
            "avif" => return "image/avif".to_string(),
            "ico" => return "image/x-icon".to_string(),
            "bmp" => return "image/bmp".to_string(),
            "woff" => return "font/woff".to_string(),
            "woff2" => return "font/woff2".to_string(),
            "ttf" => return "font/ttf".to_string(),
            "otf" => return "font/otf".to_string(),
            "mp3" => return "audio/mpeg".to_string(),
            "wav" => return "audio/wav".to_string(),
            "ogg" => return "audio/ogg".to_string(),
            "mp4" => return "video/mp4".to_string(),
            "webm" => return "video/webm".to_string(),
            _ => {}
        }
    }

    let has_known_extension = has_known_safe_extension(path);

    if let Some(guessed) = mime_guess::from_path(path).first_raw() {
        if !has_known_extension && is_potentially_active_content_type(guessed) {
            return "application/octet-stream".to_string();
        }

        if guessed.starts_with("text/") {
            return format!("{}; charset=utf-8", guessed);
        }

        return guessed.to_string();
    }

    if let Some(inferred) = infer::get(file_bytes) {
        let inferred_mime = inferred.mime_type();
        if !has_known_extension && is_potentially_active_content_type(inferred_mime) {
            return "application/octet-stream".to_string();
        }

        return inferred_mime.to_string();
    }

    "application/octet-stream".to_string()
}

fn guess_response_headers(path: &Path, file_bytes: &[u8]) -> (String, Option<&'static str>) {
    let (content_path, content_encoding) = split_content_path_and_encoding(path);
    let content_type = guess_content_type(&content_path, file_bytes);
    (content_type, content_encoding)
}

#[derive(Debug)]
struct ServeResponse {
    status_code: u16,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
}

fn add_serve_header(headers: &mut Vec<(String, String)>, name: &str, value: &str) {
    if let Some(safe_value) = sanitize_header_value(value) {
        headers.push((name.to_string(), safe_value));
    }
}

fn text_response(status_code: u16, body: &str) -> ServeResponse {
    let mut headers = Vec::new();
    add_serve_header(&mut headers, "Content-Type", "text/plain; charset=utf-8");
    add_serve_header(&mut headers, "X-Content-Type-Options", "nosniff");

    ServeResponse {
        status_code,
        body: body.as_bytes().to_vec(),
        headers,
    }
}

fn read_served_file(path: &Path) -> std::io::Result<Vec<u8>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)?;

        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Requested path is not a regular file",
            ));
        }

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    #[cfg(not(unix))]
    {
        let metadata = fs::metadata(path)?;
        if !metadata.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Requested path is not a regular file",
            ));
        }

        fs::read(path)
    }
}

fn build_serve_response(root_dir: &Path, index: &str, method: &Method, url: &str) -> ServeResponse {
    if method != &Method::Get {
        return text_response(405, "Method Not Allowed");
    }

    let raw_path = url.split('?').next().unwrap_or("/");
    let mut relative_path = raw_path.trim_start_matches('/').to_string();
    if relative_path.is_empty() {
        relative_path = index.to_string();
    }

    let mut target_path = root_dir.join(&relative_path);
    if target_path.is_dir() {
        target_path = target_path.join(index);
    }

    let canonical_target = match fs::canonicalize(&target_path) {
        Ok(path) => path,
        Err(_) => {
            return text_response(404, "Not Found");
        }
    };

    if !canonical_target.starts_with(root_dir) {
        return text_response(403, "Forbidden");
    }

    let file_bytes = match read_served_file(&canonical_target) {
        Ok(bytes) => bytes,
        Err(_) => {
            return text_response(404, "Not Found");
        }
    };

    let (content_type, content_encoding) = guess_response_headers(&canonical_target, &file_bytes);
    let mut headers = Vec::new();
    add_serve_header(&mut headers, "Content-Type", &content_type);
    if let Some(content_encoding_value) = content_encoding {
        add_serve_header(&mut headers, "Content-Encoding", content_encoding_value);
    }
    add_serve_header(&mut headers, "X-Content-Type-Options", "nosniff");

    ServeResponse {
        status_code: 200,
        body: file_bytes,
        headers,
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, interpreter, scheduler_timeout_ms, script_args } => {
            let scheduler_timeout = match cooperative_scheduler_timeout(scheduler_timeout_ms) {
                Ok(timeout) => timeout,
                Err(error_message) => {
                    eprintln!("{}", error_message);
                    std::process::exit(1);
                }
            };

            // Store script arguments in environment for args() function to retrieve
            // We need to prepend the script filename so the filtering logic works correctly
            if !script_args.is_empty() {
                // Create a modified args list: [script_name, ...script_args]
                let mut full_args: Vec<String> = std::env::args().take(3).collect(); // ruff, run, script_name
                full_args.extend(script_args);

                // Clear current args and set new ones
                // Note: This is a workaround since we can't directly modify env::args()
                // Instead, we'll pass these to the interpreter through the environment
                std::env::set_var("RUFF_SCRIPT_ARGS", full_args[3..].join("\x1f"));
                // Use unit separator
            }

            let code = fs::read_to_string(&file).expect("Failed to read .ruff file");
            let filename = file.to_string_lossy().to_string();
            let tokens = lexer::tokenize(&code);
            let mut parser = parser::Parser::new(tokens);
            let stmts = parser.parse();

            // Debug: print AST for inspection
            if !interpreter && std::env::var("DEBUG_AST").is_ok() {
                eprintln!("DEBUG AST: {:#?}", stmts);
            }

            if !interpreter {
                // Use bytecode compiler and VM
                use std::sync::{Arc, Mutex};

                let mut compiler = compiler::Compiler::new();
                match compiler.compile(&stmts) {
                    Ok(chunk) => {
                        // Spawn VM execution in a blocking task to avoid runtime conflicts
                        let result = tokio::task::spawn_blocking(move || {
                            let mut vm = vm::VM::new();
                            if std::env::var("DISABLE_JIT").is_ok() {
                                vm.set_jit_enabled(false);
                            }

                            // Set up global environment with built-in functions
                            // We need to populate it with NativeFunction values for all built-ins
                            let env = Arc::new(Mutex::new(interpreter::Environment::new()));

                            // Register all built-in functions as NativeFunction values
                            // Get the complete list from the interpreter
                            let builtins = interpreter::Interpreter::get_builtin_names();

                            for builtin_name in builtins {
                                env.lock().unwrap().set(
                                    builtin_name.to_string(),
                                    interpreter::Value::NativeFunction(builtin_name.to_string()),
                                );
                            }

                            // Register constant globals that are not callable native functions.
                            {
                                let mut env_lock = env.lock().unwrap();
                                env_lock.set(
                                    "PI".to_string(),
                                    interpreter::Value::Float(std::f64::consts::PI),
                                );
                                env_lock
                                    .set("E".to_string(), interpreter::Value::Float(std::f64::consts::E));
                                env_lock.set("null".to_string(), interpreter::Value::Null);
                            }

                            vm.set_globals(env);

                            // Execute using cooperative suspend/resume for true concurrency
                            // Initial execution
                            let exec_result = match vm.execute_until_suspend(chunk.clone()) {
                                Ok(vm::VmExecutionResult::Completed) => Ok(()),
                                Ok(vm::VmExecutionResult::Suspended { .. }) => {
                                    // Run scheduler until all contexts complete.
                                    // Use a timeout budget so long-running async workloads
                                    // can complete without relying on a fixed round count.
                                    vm.run_scheduler_until_complete_with_timeout(scheduler_timeout)
                                }
                                Err(e) => Err(e),
                            };

                            (exec_result, vm.get_call_stack())
                        })
                        .await;

                        match result {
                            Ok((Ok(_result), _)) => {
                                // Success - program executed cooperatively to completion
                            }
                            Ok((Err(e), call_stack)) => {
                                // Create a proper error with call stack
                                use crate::errors::{RuffError, SourceLocation};
                                let error = RuffError::runtime_error(e, SourceLocation::unknown())
                                    .with_call_stack(call_stack);

                                eprintln!("{}", error);
                                std::process::exit(1);
                            }
                            Err(e) => {
                                eprintln!("VM execution panicked: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Compilation error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Use tree-walking interpreter (fallback mode)
                // Type checking phase (optional - won't stop execution even if errors found)
                let mut type_checker = type_checker::TypeChecker::new();
                if let Err(errors) = type_checker.check(&stmts) {
                    eprintln!("Type checking warnings:");
                    for error in &errors {
                        eprintln!("  {}", error);
                    }
                    eprintln!();
                }

                let mut interpreter = interpreter::Interpreter::new();
                interpreter.set_source(filename, &code);

                // Execute statements
                interpreter.eval_stmts(&stmts);

                // Check for errors in return_value and display with call stack
                if let Some(ref val) = interpreter.return_value {
                    use crate::errors::RuffError;
                    match val {
                        interpreter::Value::Error(msg) => {
                            let err = RuffError::runtime_error(
                                msg.clone(),
                                crate::errors::SourceLocation::unknown(),
                            )
                            .with_call_stack(interpreter.get_call_stack());
                            eprintln!("{}", err);
                            std::process::exit(1);
                        }
                        interpreter::Value::ErrorObject { message, .. } => {
                            let err = RuffError::runtime_error(
                                message.clone(),
                                crate::errors::SourceLocation::unknown(),
                            )
                            .with_call_stack(interpreter.get_call_stack());
                            eprintln!("{}", err);
                            std::process::exit(1);
                        }
                        _ => {}
                    }
                }

                interpreter.cleanup();
                drop(interpreter);
            }
        }

        Commands::Serve { dir, port, host, index } => {
            let root_dir = match fs::canonicalize(&dir) {
                Ok(path) => path,
                Err(err) => {
                    eprintln!(
                        "Failed to resolve serve directory '{}': {}",
                        dir.display(),
                        err
                    );
                    std::process::exit(1);
                }
            };

            if !root_dir.is_dir() {
                eprintln!("Serve target is not a directory: {}", root_dir.display());
                std::process::exit(1);
            }

            let bind_addr = format!("{}:{}", host, port);
            let server = match Server::http(&bind_addr) {
                Ok(server) => server,
                Err(err) => {
                    eprintln!("Failed to start server on {}: {}", bind_addr, err);
                    std::process::exit(1);
                }
            };

            println!("Serving {} on http://{}", root_dir.display(), bind_addr);
            println!("Press Ctrl+C to stop");

            for request in server.incoming_requests() {
                let serve_response =
                    build_serve_response(&root_dir, &index, request.method(), request.url());
                let mut response = Response::from_data(serve_response.body)
                    .with_status_code(StatusCode(serve_response.status_code));

                for (name, value) in serve_response.headers.iter() {
                    if let Ok(header) = Header::from_bytes(name.as_bytes(), value.as_bytes()) {
                        response.add_header(header);
                    }
                }

                let _ = request.respond(response);
            }
        }

        Commands::Repl => match repl::Repl::new() {
            Ok(mut repl) => {
                if let Err(e) = repl.run() {
                    eprintln!("REPL error: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Failed to start REPL: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Test { update } => {
            use std::path::Path;
            parser::Parser::run_all_tests(Path::new("tests"), update);
        }

        Commands::TestRun { file, verbose } => {
            let code = fs::read_to_string(&file).expect("Failed to read test file");
            let tokens = lexer::tokenize(&code);
            let mut parser = parser::Parser::new(tokens);
            let stmts = parser.parse();

            // Create base interpreter with standard library loaded
            let base_interp = interpreter::Interpreter::new();

            // Create test runner and collect tests
            let mut runner = interpreter::TestRunner::new();
            runner.collect_tests(&stmts);

            if runner.tests.is_empty() {
                println!("No tests found in {}", file.display());
                std::process::exit(1);
            }

            // Run all tests
            let report = runner.run_all(&base_interp);

            // Print results
            report.print(verbose);

            // Exit with appropriate code
            std::process::exit(report.exit_code());
        }

        Commands::Bench { path, iterations, warmup } => {
            use benchmarks::{BenchmarkRunner, Reporter};

            let runner = BenchmarkRunner::new().with_iterations(iterations).with_warmup(warmup);

            Reporter::print_header("Ruff Performance Benchmarks");

            let results = if let Some(p) = path {
                if p.is_dir() {
                    runner.run_directory(p)
                } else if p.is_file() {
                    vec![(
                        p.file_stem().and_then(|s| s.to_str()).unwrap_or("benchmark").to_string(),
                        runner.run_file(p),
                    )]
                } else {
                    eprintln!("Error: Path does not exist: {}", p.display());
                    std::process::exit(1);
                }
            } else {
                // Default to benchmarks directory
                let default_path = PathBuf::from("examples/benchmarks");
                if default_path.exists() {
                    runner.run_directory(default_path)
                } else {
                    eprintln!("Error: No benchmark directory found. Please specify a path.");
                    std::process::exit(1);
                }
            };

            // Print individual results
            for (name, bench_results) in &results {
                println!("\n{}", name);
                for result in bench_results {
                    Reporter::print_benchmark_result(result);
                }
            }

            // Print comparison table
            Reporter::print_comparison_table(&results);

            // Print summary
            Reporter::print_summary(&results);
        }

        Commands::Format {
            file,
            indent,
            line_length,
            no_sort_imports,
            check,
            write,
            json,
        } => {
            let source = match fs::read_to_string(&file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Failed to read .ruff file '{}': {}", file.display(), err);
                    std::process::exit(1);
                }
            };
            let options = formatter::FormatterOptions {
                indent_width: indent,
                line_length,
                sort_imports: !no_sort_imports,
            };
            let formatted = formatter::format_source(&source, &options);
            let changed = source != formatted;

            if write {
                if let Err(err) = fs::write(&file, &formatted) {
                    eprintln!("Failed to write formatted file '{}': {}", file.display(), err);
                    std::process::exit(1);
                }
            }

            if json {
                let status = if write {
                    "written"
                } else if check {
                    if changed {
                        "needs_formatting"
                    } else {
                        "already_formatted"
                    }
                } else {
                    "preview"
                };

                let output = serde_json::json!({
                    "command": "format",
                    "file": file.display().to_string(),
                    "status": status,
                    "changed": changed,
                    "options": {
                        "indent": indent,
                        "line_length": line_length,
                        "sort_imports": !no_sort_imports,
                        "check": check,
                        "write": write,
                    },
                    "formatted_source": if write { serde_json::Value::Null } else { serde_json::Value::String(formatted.clone()) },
                });

                match serde_json::to_string_pretty(&output) {
                    Ok(serialized) => println!("{}", serialized),
                    Err(e) => {
                        eprintln!("Failed to serialize formatter result: {}", e);
                        std::process::exit(1);
                    }
                }

                if check && changed {
                    std::process::exit(1);
                }

                return;
            }

            if check {
                if !changed {
                    println!("already formatted");
                } else {
                    println!("needs formatting");
                    std::process::exit(1);
                }
            } else if write {
                println!("formatted {}", file.display());
            } else {
                println!("{}", formatted);
            }
        }

        Commands::Lint { file, fix, json } => {
            let source = match fs::read_to_string(&file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Failed to read .ruff file '{}': {}", file.display(), err);
                    std::process::exit(1);
                }
            };
            let issues = linter::lint_source(&source);

            if fix {
                let fixed = linter::apply_safe_fixes(&source, &issues);
                if fixed != source {
                    if let Err(err) = fs::write(&file, fixed) {
                        eprintln!("Failed to write lint fixes to '{}': {}", file.display(), err);
                        std::process::exit(1);
                    }
                    println!("applied safe lint fixes to {}", file.display());
                } else {
                    println!("no safe lint fixes applied");
                }
            }

            if json {
                let json_items: Vec<serde_json::Value> = issues
                    .iter()
                    .map(|issue| {
                        serde_json::json!({
                            "rule_id": issue.rule_id,
                            "line": issue.line,
                            "column": issue.column,
                            "severity": issue.severity.as_str(),
                            "message": issue.message,
                            "fix": issue.fix.as_ref().map(|fix| serde_json::json!({
                                "replacement_line": fix.replacement_line,
                                "description": fix.description,
                            })),
                        })
                    })
                    .collect();

                match serde_json::to_string_pretty(&json_items) {
                    Ok(serialized) => println!("{}", serialized),
                    Err(e) => {
                        eprintln!("Failed to serialize lint results: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                for issue in issues.iter() {
                    println!(
                        "{}\t{}\t{}:{}:{}\t{}",
                        issue.severity.as_str(),
                        issue.rule_id,
                        file.display(),
                        issue.line,
                        issue.column,
                        issue.message
                    );
                }
            }

            if issues.iter().any(|issue| matches!(issue.severity, linter::LintSeverity::Error)) {
                std::process::exit(1);
            }
        }

        Commands::Init { dir, name } => {
            let project_dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let package_name = match name {
                Some(explicit_name) => explicit_name,
                None => project_dir
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("ruff_project")
                    .to_string(),
            };

            let src_dir = project_dir.join("src");
            std::fs::create_dir_all(&src_dir).expect("Failed to create src directory");

            let manifest_path = project_dir.join("ruff.toml");
            if !manifest_path.exists() {
                let manifest = package_workflow::default_manifest(&package_name);
                std::fs::write(&manifest_path, manifest).expect("Failed to write ruff.toml");
            }

            let main_source_path = src_dir.join("main.ruff");
            if !main_source_path.exists() {
                std::fs::write(&main_source_path, "print(\"Hello from Ruff\")\n")
                    .expect("Failed to write src/main.ruff");
            }

            println!("initialized project at {}", project_dir.display());
        }

        Commands::PackageAdd {
            name,
            version,
            manifest,
        } => {
            let manifest_path = manifest.unwrap_or_else(|| PathBuf::from("ruff.toml"));
            let content = fs::read_to_string(&manifest_path).expect("Failed to read ruff.toml");
            let updated = match package_workflow::add_dependency(&content, &name, &version) {
                Ok(manifest_content) => manifest_content,
                Err(message) => {
                    eprintln!("{}", message);
                    std::process::exit(1);
                }
            };

            fs::write(&manifest_path, updated).expect("Failed to write updated ruff.toml");
            println!("added dependency {} {}", name, version);
        }

        Commands::PackageInstall { manifest } => {
            let manifest_path = manifest.unwrap_or_else(|| PathBuf::from("ruff.toml"));
            let content = fs::read_to_string(&manifest_path).expect("Failed to read ruff.toml");
            let parsed = match package_workflow::parse_manifest(&content) {
                Ok(manifest_data) => manifest_data,
                Err(message) => {
                    eprintln!("{}", message);
                    std::process::exit(1);
                }
            };

            if parsed.dependencies.is_empty() {
                println!("no dependencies declared");
            } else {
                for (dependency_name, dependency_version) in parsed.dependencies.iter() {
                    println!("install\t{}\t{}", dependency_name, dependency_version);
                }
            }
        }

        Commands::PackagePublish { manifest, publish } => {
            let manifest_path = manifest.unwrap_or_else(|| PathBuf::from("ruff.toml"));
            let content = fs::read_to_string(&manifest_path).expect("Failed to read ruff.toml");
            let parsed = match package_workflow::parse_manifest(&content) {
                Ok(manifest_data) => manifest_data,
                Err(message) => {
                    eprintln!("{}", message);
                    std::process::exit(1);
                }
            };

            if publish {
                println!(
                    "published\t{}\t{}",
                    parsed.package.name, parsed.package.version
                );
            } else {
                println!(
                    "publish preview\t{}\t{}\tdependencies={}",
                    parsed.package.name,
                    parsed.package.version,
                    parsed.dependencies.len()
                );
            }
        }

        Commands::Docgen {
            file,
            out_dir,
            no_builtins,
            json,
        } => {
            let output_dir = out_dir.unwrap_or_else(|| PathBuf::from("docs/generated"));
            let summary = match doc_generator::generate_docs_for_file(
                &file,
                &output_dir,
                !no_builtins,
            ) {
                Ok(result) => result,
                Err(message) => {
                    eprintln!("{}", message);
                    std::process::exit(1);
                }
            };

            if json {
                let output = serde_json::json!({
                    "command": "docgen",
                    "file": file.display().to_string(),
                    "output_dir": summary.output_dir.display().to_string(),
                    "module_doc_path": summary.module_doc_path.display().to_string(),
                    "builtin_doc_path": summary.builtin_doc_path.as_ref().map(|path| path.display().to_string()),
                    "item_count": summary.item_count,
                });

                match serde_json::to_string_pretty(&output) {
                    Ok(serialized) => println!("{}", serialized),
                    Err(e) => {
                        eprintln!("Failed to serialize docgen result: {}", e);
                        std::process::exit(1);
                    }
                }

                return;
            }

            println!("generated docs in {}", summary.output_dir.display());
            println!("module docs: {}", summary.module_doc_path.display());
            if let Some(builtin_path) = summary.builtin_doc_path {
                println!("builtin docs: {}", builtin_path.display());
            }
            println!("documented items: {}", summary.item_count);
        }

        Commands::BenchCross { ruff_script, python_script, python } => {
            use benchmarks::run_process_pool_comparison;

            let ruff_binary = match std::env::current_exe() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Failed to determine Ruff binary path: {}", e);
                    std::process::exit(1);
                }
            };

            if !ruff_script.exists() {
                eprintln!("Ruff benchmark script not found: {}", ruff_script.display());
                std::process::exit(1);
            }

            if !python_script.exists() {
                eprintln!("Python benchmark script not found: {}", python_script.display());
                std::process::exit(1);
            }

            match run_process_pool_comparison(
                ruff_binary.as_path(),
                ruff_script.as_path(),
                python.as_str(),
                python_script.as_path(),
            ) {
                Ok(comparison) => {
                    println!("Ruff parallel_map vs Python ProcessPoolExecutor");
                    println!("-----------------------------------------------");
                    println!("Ruff parallel_map: {:.3} ms", comparison.ruff_parallel_map_ms);
                    println!(
                        "Python ProcessPoolExecutor: {:.3} ms",
                        comparison.python_process_pool_ms
                    );
                    if let Some(serial_ms) = comparison.python_serial_ms {
                        println!("Python serial baseline: {:.3} ms", serial_ms);
                    }

                    println!(
                        "Ruff speedup vs Python ProcessPool: {:.2}x",
                        comparison.ruff_vs_process_pool_speedup()
                    );
                    if let Some(pool_speedup) = comparison.process_pool_vs_serial_speedup() {
                        println!("Python ProcessPool speedup vs serial: {:.2}x", pool_speedup);
                    }
                }
                Err(e) => {
                    eprintln!("Cross-language benchmark failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::BenchSsg {
            ruff_script,
            warmup_runs,
            runs,
            throughput_gate_ms,
            profile_async,
            compare_python,
            python_script,
            python,
            tmp_dir,
            variability_warning_threshold,
            trend_warning_threshold,
            mean_median_drift_warning_threshold,
            range_spread_warning_threshold,
        } => {
            use benchmarks::ssg::{
                analyze_ssg_benchmark_trends,
                collect_ssg_mean_median_drift_warnings_with_threshold,
                collect_ssg_range_spread_warnings_with_threshold,
                collect_ssg_trend_warnings_with_threshold,
                collect_ssg_variability_warnings_with_threshold,
                collect_ssg_warning_operator_hints, evaluate_ssg_throughput_gate,
                format_ssg_measurement_warning_header, format_ssg_throughput_gate_summary,
                format_ssg_trend_warning_header, SsgStageProfile, SsgTrendMetric,
                SsgWarningThresholds,
            };
            use benchmarks::{aggregate_ssg_results, run_ssg_benchmark_series};

            let ruff_binary = match std::env::current_exe() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Failed to determine Ruff binary path: {}", e);
                    std::process::exit(1);
                }
            };

            if !ruff_script.exists() {
                eprintln!("Ruff SSG benchmark script not found: {}", ruff_script.display());
                std::process::exit(1);
            }

            let python_script_path = if compare_python {
                if !python_script.exists() {
                    eprintln!("Python SSG benchmark script not found: {}", python_script.display());
                    std::process::exit(1);
                }
                Some(python_script.as_path())
            } else {
                None
            };

            let python_binary = if compare_python { Some(python.as_str()) } else { None };

            if runs == 0 {
                eprintln!("SSG benchmark runs must be >= 1");
                std::process::exit(1);
            }

            if variability_warning_threshold < 0.0
                || trend_warning_threshold < 0.0
                || mean_median_drift_warning_threshold < 0.0
                || range_spread_warning_threshold < 0.0
            {
                eprintln!(
                    "SSG warning thresholds must be >= 0.0 (got variability={}, trend={}, mean-median={}, range-spread={})",
                    variability_warning_threshold,
                    trend_warning_threshold,
                    mean_median_drift_warning_threshold,
                    range_spread_warning_threshold
                );
                std::process::exit(1);
            }

            let warning_thresholds = SsgWarningThresholds {
                variability_percent: variability_warning_threshold,
                trend_percent: trend_warning_threshold,
                mean_median_drift_percent: mean_median_drift_warning_threshold,
                range_spread_percent: range_spread_warning_threshold,
            };

            if warmup_runs > 0 {
                println!("Running SSG benchmark warmups ({})...", warmup_runs);
            }
            if runs > 1 {
                println!("Running SSG benchmark measured runs ({})...", runs);
            }

            let run_results = match run_ssg_benchmark_series(
                ruff_binary.as_path(),
                ruff_script.as_path(),
                python_binary,
                python_script_path,
                tmp_dir.as_deref(),
                warmup_runs,
                runs,
                profile_async,
            ) {
                Ok(results) => results,
                Err(e) => {
                    eprintln!("SSG benchmark failed: {}", e);
                    std::process::exit(1);
                }
            };

            let summary = match aggregate_ssg_results(&run_results) {
                Ok(summary) => summary,
                Err(e) => {
                    eprintln!("SSG benchmark aggregation failed: {}", e);
                    std::process::exit(1);
                }
            };

            println!("Ruff async SSG benchmark");
            println!("------------------------");
            println!("Warmup runs: {}", warmup_runs);
            println!("Runs: {}", summary.ruff_build_ms.runs);
            println!("Files rendered: {}", summary.files);
            println!("Ruff checksum: {}", summary.ruff_checksum);
            println!(
                "Ruff build time (median): {:.3} ms [mean {:.3}, p90 {:.3}, p95 {:.3}, min {:.3}, max {:.3}, stddev {:.3}]",
                summary.ruff_build_ms.median,
                summary.ruff_build_ms.mean,
                summary.ruff_build_ms.p90,
                summary.ruff_build_ms.p95,
                summary.ruff_build_ms.min,
                summary.ruff_build_ms.max,
                summary.ruff_build_ms.stddev
            );
            println!(
                "Ruff throughput (median): {:.2} files/sec [mean {:.2}, p90 {:.2}, p95 {:.2}, min {:.2}, max {:.2}, stddev {:.2}]",
                summary.ruff_files_per_sec.median,
                summary.ruff_files_per_sec.mean,
                summary.ruff_files_per_sec.p90,
                summary.ruff_files_per_sec.p95,
                summary.ruff_files_per_sec.min,
                summary.ruff_files_per_sec.max,
                summary.ruff_files_per_sec.stddev
            );

            let throughput_gate = if let Some(gate_threshold_ms) = throughput_gate_ms {
                let gate = match evaluate_ssg_throughput_gate(
                    summary.ruff_build_ms.median,
                    gate_threshold_ms,
                ) {
                    Ok(gate) => gate,
                    Err(e) => {
                        eprintln!("SSG throughput gate validation failed: {}", e);
                        std::process::exit(1);
                    }
                };
                println!("{}", format_ssg_throughput_gate_summary(gate));
                Some(gate)
            } else {
                None
            };

            if profile_async {
                if let Some(ruff_profile) = summary.ruff_stage_profile.as_ref() {
                    println!("Ruff stage breakdown (median):");
                    println!("  read stage: {:.3} ms", ruff_profile.read_ms.median);
                    println!("  render/write stage: {:.3} ms", ruff_profile.render_write_ms.median);
                    let profile = SsgStageProfile {
                        read_ms: ruff_profile.read_ms.median,
                        render_write_ms: ruff_profile.render_write_ms.median,
                    };
                    if let Some((stage_name, stage_ms, stage_percent)) = profile.bottleneck_stage()
                    {
                        println!(
                            "  bottleneck: {} ({:.3} ms, {:.2}% of profiled median)",
                            stage_name, stage_ms, stage_percent
                        );
                    }
                } else {
                    println!("Ruff stage breakdown: unavailable (metrics not emitted by script)");
                }
            }

            if let (Some(python_build_ms), Some(python_files_per_sec)) =
                (summary.python_build_ms.as_ref(), summary.python_files_per_sec.as_ref())
            {
                println!(
                    "Python build time (median): {:.3} ms [mean {:.3}, p90 {:.3}, p95 {:.3}, min {:.3}, max {:.3}, stddev {:.3}]",
                    python_build_ms.median,
                    python_build_ms.mean,
                    python_build_ms.p90,
                    python_build_ms.p95,
                    python_build_ms.min,
                    python_build_ms.max,
                    python_build_ms.stddev
                );
                println!(
                    "Python throughput (median): {:.2} files/sec [mean {:.2}, p90 {:.2}, p95 {:.2}, min {:.2}, max {:.2}, stddev {:.2}]",
                    python_files_per_sec.median,
                    python_files_per_sec.mean,
                    python_files_per_sec.p90,
                    python_files_per_sec.p95,
                    python_files_per_sec.min,
                    python_files_per_sec.max,
                    python_files_per_sec.stddev
                );

                if let Some(speedup) = summary.ruff_vs_python_speedup.as_ref() {
                    println!(
                        "Ruff speedup vs Python (median): {:.2}x [mean {:.2}x, p90 {:.2}x, p95 {:.2}x, min {:.2}x, max {:.2}x, stddev {:.2}]",
                        speedup.median,
                        speedup.mean,
                        speedup.p90,
                        speedup.p95,
                        speedup.min,
                        speedup.max,
                        speedup.stddev
                    );
                }

                if profile_async {
                    if let Some(python_profile) = summary.python_stage_profile.as_ref() {
                        println!("Python stage breakdown (median):");
                        println!("  read stage: {:.3} ms", python_profile.read_ms.median);
                        println!(
                            "  render/write stage: {:.3} ms",
                            python_profile.render_write_ms.median
                        );
                        let profile = SsgStageProfile {
                            read_ms: python_profile.read_ms.median,
                            render_write_ms: python_profile.render_write_ms.median,
                        };
                        if let Some((stage_name, stage_ms, stage_percent)) =
                            profile.bottleneck_stage()
                        {
                            println!(
                                "  bottleneck: {} ({:.3} ms, {:.2}% of profiled median)",
                                stage_name, stage_ms, stage_percent
                            );
                        }
                    } else {
                        println!(
                            "Python stage breakdown: unavailable (metrics not emitted by script)"
                        );
                    }
                }
            }

            let trend_report = match analyze_ssg_benchmark_trends(&run_results) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("SSG benchmark trend analysis failed: {}", e);
                    std::process::exit(1);
                }
            };

            if let Some(trends) = trend_report {
                let format_delta = |metric: &SsgTrendMetric, unit_suffix: &str| {
                    let absolute = if metric.absolute_delta >= 0.0 {
                        format!("+{:.3}{}", metric.absolute_delta, unit_suffix)
                    } else {
                        format!("{:.3}{}", metric.absolute_delta, unit_suffix)
                    };

                    match metric.percent_delta {
                        Some(percent) if percent >= 0.0 => {
                            format!("{} (+{:.2}%)", absolute, percent)
                        }
                        Some(percent) => format!("{} ({:.2}%)", absolute, percent),
                        None => format!("{} (n/a %)", absolute),
                    }
                };

                println!("Measured trend (first→last across {} runs):", trends.measured_runs);
                println!(
                    "  Ruff build time: {:.3} ms → {:.3} ms [{}]",
                    trends.ruff_build_ms.first,
                    trends.ruff_build_ms.last,
                    format_delta(&trends.ruff_build_ms, " ms")
                );
                println!(
                    "  Ruff throughput: {:.2} files/sec → {:.2} files/sec [{}]",
                    trends.ruff_files_per_sec.first,
                    trends.ruff_files_per_sec.last,
                    format_delta(&trends.ruff_files_per_sec, " files/sec")
                );

                if let Some(metric) = trends.python_build_ms.as_ref() {
                    println!(
                        "  Python build time: {:.3} ms → {:.3} ms [{}]",
                        metric.first,
                        metric.last,
                        format_delta(metric, " ms")
                    );
                }

                if let Some(metric) = trends.python_files_per_sec.as_ref() {
                    println!(
                        "  Python throughput: {:.2} files/sec → {:.2} files/sec [{}]",
                        metric.first,
                        metric.last,
                        format_delta(metric, " files/sec")
                    );
                }

                if let Some(metric) = trends.ruff_vs_python_speedup.as_ref() {
                    println!(
                        "  Ruff/Python speedup: {:.2}x → {:.2}x [{}]",
                        metric.first,
                        metric.last,
                        format_delta(metric, "x")
                    );
                }

                let trend_warnings = collect_ssg_trend_warnings_with_threshold(
                    &trends,
                    warning_thresholds.trend_percent,
                );
                if !trend_warnings.is_empty() {
                    println!("{}", format_ssg_trend_warning_header(warning_thresholds));
                    for warning in trend_warnings {
                        println!("  - {}", warning);
                    }
                    for hint in collect_ssg_warning_operator_hints(warning_thresholds) {
                        println!("  - hint: {}", hint);
                    }
                }
            }

            let variability_warnings = collect_ssg_variability_warnings_with_threshold(
                &summary,
                warning_thresholds.variability_percent,
            );
            let mean_median_drift_warnings = collect_ssg_mean_median_drift_warnings_with_threshold(
                &summary,
                warning_thresholds.mean_median_drift_percent,
            );
            let range_spread_warnings = collect_ssg_range_spread_warnings_with_threshold(
                &summary,
                warning_thresholds.range_spread_percent,
            );
            if !variability_warnings.is_empty()
                || !mean_median_drift_warnings.is_empty()
                || !range_spread_warnings.is_empty()
            {
                println!("{}", format_ssg_measurement_warning_header(warning_thresholds));
                for warning in variability_warnings {
                    println!("  - {}", warning);
                }
                for warning in mean_median_drift_warnings {
                    println!("  - {}", warning);
                }
                for warning in range_spread_warnings {
                    println!("  - {}", warning);
                }
                for hint in collect_ssg_warning_operator_hints(warning_thresholds) {
                    println!("  - hint: {}", hint);
                }
            }

            if let Some(gate) = throughput_gate {
                if !gate.passed {
                    eprintln!(
                        "SSG throughput gate failed: Ruff median build time {:.3} ms exceeded target {:.3} ms",
                        gate.observed_median_ms,
                        gate.threshold_ms
                    );
                    std::process::exit(1);
                }
            }
        }

        Commands::Profile { file, cpu, memory, jit, flamegraph } => {
            use benchmarks::{
                print_profile_report, profiler::generate_flamegraph_data, ProfileConfig, Profiler,
            };

            // Read and parse the file
            let code = fs::read_to_string(&file).expect("Failed to read file");
            let filename = file.to_string_lossy().to_string();

            // Create profile configuration
            let config = ProfileConfig {
                cpu_profiling: cpu,
                memory_profiling: memory,
                jit_stats: jit,
                ..Default::default()
            };

            let mut profiler = Profiler::new(config);
            profiler.start();

            // Execute the code
            let tokens = lexer::tokenize(&code);
            let mut parser = parser::Parser::new(tokens);
            let stmts = parser.parse();

            let mut interp = interpreter::Interpreter::new();
            interp.set_source(filename, &code);

            // Run the program
            interp.eval_stmts(&stmts);

            // Stop profiling
            let elapsed = profiler.stop();

            // Get profile data
            let profile_data = profiler.into_data();

            // Print profile report
            print_profile_report(&profile_data);

            println!("\nTotal execution time: {:.3}s", elapsed.as_secs_f64());

            // Generate flamegraph if requested
            if let Some(fg_path) = flamegraph {
                let fg_data = generate_flamegraph_data(&profile_data.cpu);
                fs::write(&fg_path, fg_data).expect("Failed to write flamegraph data");
                println!("\nFlamegraph data written to: {}", fg_path.display());
                println!("Generate SVG with: flamegraph.pl {} > flamegraph.svg", fg_path.display());
            }

            // Cleanup
            interp.cleanup();
        }

        Commands::LspComplete { file, line, column, json } => {
            let code = fs::read_to_string(&file).expect("Failed to read .ruff file");
            let completion_items = lsp_completion::complete(&code, line, column);

            if json {
                let json_items: Vec<serde_json::Value> = completion_items
                    .iter()
                    .map(|item| {
                        serde_json::json!({
                            "label": item.label,
                            "kind": item.kind.as_str(),
                        })
                    })
                    .collect();

                match serde_json::to_string_pretty(&json_items) {
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        eprintln!("Failed to serialize completion results: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                for item in completion_items {
                    println!("{}\t{}", item.label, item.kind.as_str());
                }
            }
        }

        Commands::LspDefinition { file, line, column, json } => {
            let code = fs::read_to_string(&file).expect("Failed to read .ruff file");
            let definition = lsp_definition::find_definition(&code, line, column);

            if json {
                let output = match definition {
                    Some(location) => serde_json::json!({
                        "name": location.name,
                        "line": location.line,
                        "column": location.column,
                        "kind": location.kind.as_str(),
                    }),
                    None => serde_json::Value::Null,
                };

                match serde_json::to_string_pretty(&output) {
                    Ok(serialized) => println!("{}", serialized),
                    Err(e) => {
                        eprintln!("Failed to serialize definition result: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                match definition {
                    Some(location) => {
                        println!(
                            "{}\t{}:{}:{}",
                            location.name,
                            file.display(),
                            location.line,
                            location.column
                        );
                    }
                    None => {
                        println!("not found");
                    }
                }
            }
        }

        Commands::LspReferences {
            file,
            line,
            column,
            include_definition,
            json,
        } => {
            let code = fs::read_to_string(&file).expect("Failed to read .ruff file");
            let references =
                lsp_references::find_references(&code, line, column, include_definition);

            if json {
                let json_items: Vec<serde_json::Value> = references
                    .iter()
                    .map(|reference| {
                        serde_json::json!({
                            "line": reference.line,
                            "column": reference.column,
                            "is_definition": reference.is_definition,
                        })
                    })
                    .collect();

                match serde_json::to_string_pretty(&json_items) {
                    Ok(serialized) => println!("{}", serialized),
                    Err(e) => {
                        eprintln!("Failed to serialize references result: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                for reference in references {
                    let role = if reference.is_definition {
                        "definition"
                    } else {
                        "reference"
                    };

                    println!(
                        "{}\t{}:{}:{}",
                        role,
                        file.display(),
                        reference.line,
                        reference.column
                    );
                }
            }
        }

        Commands::LspHover { file, line, column, json } => {
            let code = fs::read_to_string(&file).expect("Failed to read .ruff file");
            let hover_info = lsp_hover::hover(&code, line, column);

            if json {
                let output = match hover_info {
                    Some(info) => serde_json::json!({
                        "symbol": info.symbol,
                        "kind": info.kind,
                        "detail": info.detail,
                        "line": info.line,
                        "column": info.column,
                    }),
                    None => serde_json::Value::Null,
                };

                match serde_json::to_string_pretty(&output) {
                    Ok(serialized) => println!("{}", serialized),
                    Err(e) => {
                        eprintln!("Failed to serialize hover result: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                match hover_info {
                    Some(info) => {
                        println!(
                            "{}\t{}\t{}:{}:{}",
                            info.symbol,
                            info.detail,
                            file.display(),
                            info.line,
                            info.column
                        );
                    }
                    None => {
                        println!("not found");
                    }
                }
            }
        }

        Commands::LspDiagnostics { file, json } => {
            let code = fs::read_to_string(&file).expect("Failed to read .ruff file");
            let diagnostics = lsp_diagnostics::diagnose(&code);

            if json {
                let json_items: Vec<serde_json::Value> = diagnostics
                    .iter()
                    .map(|diagnostic| {
                        serde_json::json!({
                            "line": diagnostic.line,
                            "column": diagnostic.column,
                            "severity": diagnostic.severity.as_str(),
                            "message": diagnostic.message,
                        })
                    })
                    .collect();

                match serde_json::to_string_pretty(&json_items) {
                    Ok(serialized) => println!("{}", serialized),
                    Err(e) => {
                        eprintln!("Failed to serialize diagnostics result: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                for diagnostic in diagnostics {
                    println!(
                        "{}\t{}:{}:{}\t{}",
                        diagnostic.severity.as_str(),
                        file.display(),
                        diagnostic.line,
                        diagnostic.column,
                        diagnostic.message
                    );
                }
            }
        }

        Commands::LspRename {
            file,
            line,
            column,
            new_name,
            json,
        } => {
            let code = fs::read_to_string(&file).expect("Failed to read .ruff file");
            let rename_result = match lsp_rename::rename_symbol(&code, line, column, &new_name) {
                Ok(result) => result,
                Err(message) => {
                    eprintln!("{}", message);
                    std::process::exit(1);
                }
            };

            if json {
                let json_edits: Vec<serde_json::Value> = rename_result
                    .edits
                    .iter()
                    .map(|edit| {
                        serde_json::json!({
                            "line": edit.line,
                            "column": edit.column,
                            "old_name": edit.old_name,
                            "new_name": edit.new_name,
                        })
                    })
                    .collect();

                let output = serde_json::json!({
                    "edit_count": json_edits.len(),
                    "edits": json_edits,
                    "updated_source": rename_result.updated_source,
                });

                match serde_json::to_string_pretty(&output) {
                    Ok(serialized) => println!("{}", serialized),
                    Err(e) => {
                        eprintln!("Failed to serialize rename result: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("renamed\t{} edits", rename_result.edits.len());
                for edit in rename_result.edits.iter() {
                    println!(
                        "{}:{}:{}\t{} -> {}",
                        file.display(),
                        edit.line,
                        edit.column,
                        edit.old_name,
                        edit.new_name
                    );
                }
            }
        }

        Commands::LspCodeActions { file, json } => {
            let code = fs::read_to_string(&file).expect("Failed to read .ruff file");
            let actions = lsp_code_actions::code_actions(&code);

            if json {
                let json_items: Vec<serde_json::Value> = actions
                    .iter()
                    .map(|action| {
                        serde_json::json!({
                            "title": action.title,
                            "kind": action.kind,
                            "line": action.line,
                            "column": action.column,
                            "replacement": action.replacement,
                            "description": action.description,
                        })
                    })
                    .collect();

                match serde_json::to_string_pretty(&json_items) {
                    Ok(serialized) => println!("{}", serialized),
                    Err(e) => {
                        eprintln!("Failed to serialize code actions result: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                for action in actions.iter() {
                    println!(
                        "{}\t{}:{}:{}\t{}",
                        action.title,
                        file.display(),
                        action.line,
                        action.column,
                        action.replacement
                    );
                }
            }
        }

        Commands::Lsp { deterministic_logs, request_timeout_ms } => {
            let exit_code = lsp_server::run_stdio_server(lsp_server::LspServerConfig {
                deterministic_logging: deterministic_logs,
                request_timeout_ms,
            });
            if 0 != exit_code {
                std::process::exit(exit_code);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_serve_response, cooperative_scheduler_timeout, guess_content_type,
        guess_response_headers,
        sanitize_header_value,
        DEFAULT_COOPERATIVE_SCHEDULER_TIMEOUT_MS,
    };
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tiny_http::Method;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_scheduler_timeout_env<F>(value: Option<&str>, test_fn: F)
    where
        F: FnOnce(),
    {
        let _guard = ENV_LOCK.lock().expect("environment lock poisoned");

        match value {
            Some(raw) => std::env::set_var("RUFF_SCHEDULER_TIMEOUT_MS", raw),
            None => std::env::remove_var("RUFF_SCHEDULER_TIMEOUT_MS"),
        }

        test_fn();
        std::env::remove_var("RUFF_SCHEDULER_TIMEOUT_MS");
    }

    #[test]
    fn cooperative_scheduler_timeout_uses_default_when_unset() {
        with_scheduler_timeout_env(None, || {
            let timeout = cooperative_scheduler_timeout(None)
                .expect("default scheduler timeout should resolve successfully");
            assert_eq!(timeout, Duration::from_millis(DEFAULT_COOPERATIVE_SCHEDULER_TIMEOUT_MS));
        });
    }

    #[test]
    fn cooperative_scheduler_timeout_uses_env_when_cli_missing() {
        with_scheduler_timeout_env(Some("2345"), || {
            let timeout = cooperative_scheduler_timeout(None)
                .expect("env scheduler timeout should resolve successfully");
            assert_eq!(timeout, Duration::from_millis(2345));
        });
    }

    #[test]
    fn cooperative_scheduler_timeout_prefers_cli_over_env() {
        with_scheduler_timeout_env(Some("5000"), || {
            let timeout = cooperative_scheduler_timeout(Some(2500))
                .expect("cli scheduler timeout should resolve successfully");
            assert_eq!(timeout, Duration::from_millis(2500));
        });
    }

    #[test]
    fn cooperative_scheduler_timeout_rejects_cli_zero() {
        with_scheduler_timeout_env(Some("5000"), || {
            let error = cooperative_scheduler_timeout(Some(0))
                .expect_err("zero cli scheduler timeout should be rejected");
            assert_eq!(error, "Scheduler timeout must be greater than 0ms");
        });
    }

    #[test]
    fn cooperative_scheduler_timeout_falls_back_on_invalid_env() {
        with_scheduler_timeout_env(Some("invalid"), || {
            let timeout = cooperative_scheduler_timeout(None)
                .expect("invalid env value should fall back to default");
            assert_eq!(timeout, Duration::from_millis(DEFAULT_COOPERATIVE_SCHEDULER_TIMEOUT_MS));
        });
    }

    #[test]
    fn guess_content_type_handles_uppercase_extensions() {
        let content_type = guess_content_type(Path::new("INDEX.HTML"), b"");
        assert_eq!(content_type, "text/html; charset=utf-8");
    }

    #[test]
    fn guess_content_type_covers_common_web_assets() {
        assert_eq!(
            guess_content_type(Path::new("runtime.WASM"), b""),
            "application/wasm"
        );
        assert_eq!(
            guess_content_type(Path::new("font.woff2"), b""),
            "font/woff2"
        );
        assert_eq!(
            guess_content_type(Path::new("module.mjs"), b""),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(
            guess_content_type(Path::new("site.webm"), b""),
            "video/webm"
        );
    }

    #[test]
    fn guess_response_headers_respects_compressed_suffixes() {
        let (content_type, content_encoding) = guess_response_headers(Path::new("app.js.gz"), b"");
        assert_eq!(content_type, "application/javascript; charset=utf-8");
        assert_eq!(content_encoding, Some("gzip"));

        let (content_type, content_encoding) =
            guess_response_headers(Path::new("styles.css.br"), b"");
        assert_eq!(content_type, "text/css; charset=utf-8");
        assert_eq!(content_encoding, Some("br"));
    }

    #[test]
    fn guess_content_type_uses_magic_bytes_when_extension_unknown() {
        let png_signature: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
        let content_type = guess_content_type(Path::new("asset.unknown"), &png_signature);
        assert_eq!(content_type, "image/png");
    }

    #[test]
    fn guess_content_type_falls_back_to_octet_stream_when_unknown() {
        let content_type = guess_content_type(Path::new("asset.unknown"), b"not-a-known-format");
        assert_eq!(content_type, "application/octet-stream");
    }

    #[test]
    fn guess_content_type_blocks_active_inferred_type_for_unknown_extension() {
        let html_bytes = b"<!DOCTYPE html><html><body>hello</body></html>";
        let content_type = guess_content_type(Path::new("payload.unknown"), html_bytes);
        assert_eq!(content_type, "application/octet-stream");
    }

    #[test]
    fn sanitize_header_value_rejects_control_characters() {
        assert_eq!(sanitize_header_value("text/html"), Some("text/html".to_string()));
        assert_eq!(sanitize_header_value("text/html\r\nX-Test: injected"), None);
    }

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "ruff_{}_{}_{}",
            prefix,
            std::process::id(),
            timestamp
        ));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        fs::canonicalize(dir).expect("temp dir should canonicalize")
    }

    fn get_header_value(response: &super::ServeResponse, name: &str) -> Option<String> {
        response
            .headers
            .iter()
            .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.clone())
    }

    #[test]
    fn build_serve_response_returns_404_for_missing_file() {
        let root_dir = unique_temp_dir("serve_404");

        let response = build_serve_response(&root_dir, "index.html", &Method::Get, "/missing.txt");
        assert_eq!(response.status_code, 404);
        assert_eq!(String::from_utf8_lossy(&response.body), "Not Found");

        fs::remove_dir_all(root_dir).expect("temp dir should be removed");
    }

    #[test]
    fn build_serve_response_returns_403_for_path_escape_attempt() {
        let parent_dir = unique_temp_dir("serve_parent");
        let root_dir = parent_dir.join("root");
        fs::create_dir_all(&root_dir).expect("root dir should be created");

        let escaped_file = parent_dir.join("secret.txt");
        fs::write(&escaped_file, "secret").expect("escaped file should be created");

        let response = build_serve_response(&root_dir, "index.html", &Method::Get, "/../secret.txt");
        assert_eq!(response.status_code, 403);
        assert_eq!(String::from_utf8_lossy(&response.body), "Forbidden");

        fs::remove_dir_all(parent_dir).expect("temp dir should be removed");
    }

    #[test]
    fn build_serve_response_returns_405_for_non_get_method() {
        let root_dir = unique_temp_dir("serve_405");

        let response = build_serve_response(&root_dir, "index.html", &Method::Post, "/");
        assert_eq!(response.status_code, 405);
        assert_eq!(String::from_utf8_lossy(&response.body), "Method Not Allowed");

        fs::remove_dir_all(root_dir).expect("temp dir should be removed");
    }

    #[test]
    fn build_serve_response_sets_content_and_security_headers() {
        let root_dir = unique_temp_dir("serve_headers");
        let asset_path = root_dir.join("app.js.gz");
        fs::write(&asset_path, "console.log('ruff');").expect("asset should be created");

        let response = build_serve_response(&root_dir, "index.html", &Method::Get, "/app.js.gz");
        assert_eq!(response.status_code, 200);

        assert_eq!(
            get_header_value(&response, "Content-Type"),
            Some("application/javascript; charset=utf-8".to_string())
        );
        assert_eq!(
            get_header_value(&response, "Content-Encoding"),
            Some("gzip".to_string())
        );
        assert_eq!(
            get_header_value(&response, "X-Content-Type-Options"),
            Some("nosniff".to_string())
        );

        fs::remove_dir_all(root_dir).expect("temp dir should be removed");
    }
}
