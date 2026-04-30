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
mod lsp_completion;
mod lsp_definition;
mod lsp_diagnostics;
mod lsp_hover;
mod lsp_rename;
mod lsp_references;
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

        /// Cooperative scheduler timeout in milliseconds (overrides env/default)
        #[arg(long)]
        scheduler_timeout_ms: Option<u64>,

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
    }
}

#[cfg(test)]
mod tests {
    use super::{cooperative_scheduler_timeout, DEFAULT_COOPERATIVE_SCHEDULER_TIMEOUT_MS};
    use std::sync::Mutex;
    use std::time::Duration;

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
}
