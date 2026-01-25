// File: src/main.rs
//
// Main entry point for the Ruff programming language interpreter.
// Handles command-line argument parsing and dispatches to the appropriate
// subcommand (run, repl, or test).

mod ast;
mod builtins;
mod bytecode;
mod compiler;
mod errors;
mod interpreter;
mod lexer;
mod module;
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
        
        /// Use bytecode VM instead of tree-walking interpreter
        #[arg(long)]
        vm: bool,
        
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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, vm, script_args } => {
            // Store script arguments in environment for args() function to retrieve
            // We need to prepend the script filename so the filtering logic works correctly
            if !script_args.is_empty() {
                // Create a modified args list: [script_name, ...script_args]
                let mut full_args: Vec<String> = std::env::args().take(3).collect(); // ruff, run, script_name
                full_args.extend(script_args);
                
                // Clear current args and set new ones
                // Note: This is a workaround since we can't directly modify env::args()
                // Instead, we'll pass these to the interpreter through the environment
                std::env::set_var("RUFF_SCRIPT_ARGS", full_args[3..].join("\x1f")); // Use unit separator
            }
            
            let code = fs::read_to_string(&file).expect("Failed to read .ruff file");
            let filename = file.to_string_lossy().to_string();
            let tokens = lexer::tokenize(&code);
            let mut parser = parser::Parser::new(tokens);
            let stmts = parser.parse();
            
            // Debug: print AST for inspection
            if vm && std::env::var("DEBUG_AST").is_ok() {
                eprintln!("DEBUG AST: {:#?}", stmts);
            }

            if vm {
                // Use bytecode compiler and VM
                use std::rc::Rc;
                use std::cell::RefCell;
                
                let mut compiler = compiler::Compiler::new();
                match compiler.compile(&stmts) {
                    Ok(chunk) => {
                        let mut vm = vm::VM::new();
                        
                        // Set up global environment with built-in functions
                        // We need to populate it with NativeFunction values for all built-ins
                        let env = Rc::new(RefCell::new(interpreter::Environment::new()));
                        
                        // Register all built-in functions as NativeFunction values
                        // Get the complete list from the interpreter
                        let builtins = interpreter::Interpreter::get_builtin_names();
                        
                        for builtin_name in builtins {
                            env.borrow_mut().set(
                                builtin_name.to_string(),
                                interpreter::Value::NativeFunction(builtin_name.to_string())
                            );
                        }
                        
                        vm.set_globals(env);
                        
                        match vm.execute(chunk) {
                            Ok(_result) => {
                                // Success - program executed
                            }
                            Err(e) => {
                                eprintln!("Runtime error: {}", e);
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
                // Use tree-walking interpreter (default)
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
                interpreter.eval_stmts(&stmts);
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
    }
}
