// Simple benchmark to measure JIT compilation overhead and potential speedup
// This is a Rust program that will demonstrate JIT performance

use std::time::Instant;
use ruff::bytecode::{BytecodeChunk, Constant, OpCode};
use ruff::jit::JitCompiler;
use ruff::compiler::Compiler;
use ruff::lexer::Lexer;
use ruff::parser::Parser;
use ruff::vm::VM;

fn main() {
    println!("=== JIT Performance Benchmark ===\n");

    // Test 1: Simple arithmetic loop
    benchmark_arithmetic_loop();

    println!("\n=== Benchmark Complete ===");
}

fn benchmark_arithmetic_loop() {
    println!("Benchmark: Simple Arithmetic Loop (1000 iterations)");

    let code = r#"
        sum := 0
        for i in range(1000) {
            sum := sum + i
        }
        sum
    "#;

    // Compile to bytecode
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().expect("Tokenization failed");

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("Parsing failed");

    let mut compiler = Compiler::new();
    let chunk = compiler.compile(&ast).expect("Compilation failed");

    println!("  Bytecode size: {} instructions", chunk.instructions.len());

    // Run with bytecode VM (baseline)
    let start = Instant::now();
    let mut vm = VM::new();
    vm.set_jit_enabled(false); // Disable JIT for baseline
    let result1 = vm.execute(chunk.clone()).expect("VM execution failed");
    let bytecode_time = start.elapsed();

    println!("  Bytecode VM time: {:?}", bytecode_time);
    println!("  Result: {:?}", result1);

    // Run with JIT enabled
    let start = Instant::now();
    let mut vm = VM::new();
    vm.set_jit_enabled(true); // Enable JIT
    let result2 = vm.execute(chunk).expect("VM execution failed");
    let jit_time = start.elapsed();

    println!("  JIT-enabled VM time: {:?}", jit_time);
    println!("  Result: {:?}", result2);

    // Calculate speedup
    let speedup = bytecode_time.as_nanos() as f64 / jit_time.as_nanos() as f64;
    println!("  Speedup: {:.2}x", speedup);

    if speedup > 1.0 {
        println!("  ✓ JIT is faster!");
    } else {
        println!("  Note: JIT compilation overhead may dominate for small loops");
    }
}
