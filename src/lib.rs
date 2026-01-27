// File: src/lib.rs
//
// Library interface for the Ruff interpreter.
// Exposes modules for integration testing and external use.

pub mod ast;
pub mod builtins;
pub mod bytecode;
pub mod compiler;
pub mod errors;
pub mod interpreter;
pub mod lexer;
pub mod module;
pub mod parser;
pub mod repl;
pub mod type_checker;
pub mod vm;
