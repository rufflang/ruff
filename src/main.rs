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
mod errors;
mod interpreter;
mod jit;
mod lexer;
mod module;
mod optimizer;
mod parser;
mod repl;
mod type_checker;
mod vm;

use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::path::PathBuf;

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

        /// Arguments to pass to the script
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        script_args: Vec<String>,
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, interpreter, script_args } => {
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

                            vm.set_globals(env);

                            (vm.execute(chunk), vm.get_call_stack())
                        })
                        .await;

                        match result {
                            Ok((Ok(_result), _)) => {
                                // Success - program executed
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
    }
}
