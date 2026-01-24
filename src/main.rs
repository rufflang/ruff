// File: src/main.rs
//
// Main entry point for the Ruff programming language interpreter.
// Handles command-line argument parsing and dispatches to the appropriate
// subcommand (run, repl, or test).

mod ast;
mod builtins;
mod errors;
mod interpreter;
mod lexer;
mod module;
mod parser;
mod repl;
mod type_checker;

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
        Commands::Run { file } => {
            let code = fs::read_to_string(&file).expect("Failed to read .ruff file");
            let filename = file.to_string_lossy().to_string();
            let tokens = lexer::tokenize(&code);
            let mut parser = parser::Parser::new(tokens);
            let stmts = parser.parse();

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
