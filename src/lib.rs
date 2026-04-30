// File: src/lib.rs
//
// Library interface for the Ruff interpreter.
// Exposes modules for integration testing and external use.

pub mod ast;
pub mod benchmarks;
pub mod builtins;
pub mod bytecode;
pub mod compiler;
pub mod errors;
pub mod interpreter;
pub mod jit;
pub mod lexer;
pub mod lsp_completion;
pub mod lsp_definition;
pub mod lsp_hover;
pub mod lsp_references;
pub mod module;
pub mod optimizer;
pub mod parser;
pub mod repl;
pub mod type_checker;
pub mod vm;
