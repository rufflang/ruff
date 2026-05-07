use ruff::lexer::tokenize;
use ruff::parser::{ParseOutput, Parser};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn parse_output(source: &str) -> ParseOutput {
    let tokens = tokenize(source).expect("test source should tokenize");
    let mut parser = Parser::new(tokens);
    parser.parse_with_diagnostics()
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ruff_{}_{}", prefix, nanos));
    fs::create_dir_all(&path).expect("failed to create temp directory");
    path
}

fn write_fixture(path: &Path, content: &str) {
    fs::write(path, content).expect("failed to write fixture file");
}

fn run_ruff(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ruff"))
        .args(args)
        .output()
        .expect("failed to execute ruff binary")
}

#[test]
fn parser_accepts_valid_program_without_diagnostics() {
    let output = parse_output("let value := 1\nprint(value)\n");
    assert!(output.diagnostics.is_empty());
    assert_eq!(output.stmts.len(), 2);
}

#[test]
fn parser_reports_missing_closing_parenthesis() {
    let output = parse_output("print((1 + 2)\n");
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Expected ')'")));
}

#[test]
fn parser_reports_missing_closing_bracket() {
    let output = parse_output("values := [1, 2\n");
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Expected ']'")));
}

#[test]
fn parser_reports_missing_closing_brace() {
    let output = parse_output("if true {\n  print(1)\n");
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Expected '}'")));
}

#[test]
fn parser_reports_invalid_assignment_target() {
    let output = parse_output("foo() := 1\n");
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Invalid assignment target")));
}

#[test]
fn parser_reports_unexpected_eof_in_function_body() {
    let output = parse_output("func greet(name) {\n");
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Expected '}'")));
}

#[test]
fn parser_recovery_reports_multiple_independent_errors() {
    let output = parse_output("print((1 + 2\nvalues := [1, 2\nok := 1\n");

    let messages: Vec<&str> =
        output.diagnostics.iter().map(|diagnostic| diagnostic.message.as_str()).collect();
    assert!(messages.iter().any(|message| message.contains("Expected ')'")));
    assert!(messages.iter().any(|message| message.contains("Expected ']'")));
    assert!(output.stmts.iter().any(|stmt| matches!(stmt, ruff::ast::Stmt::Assign { .. })));
}

#[test]
fn cli_run_exits_non_zero_on_parse_diagnostics() {
    let dir = unique_temp_dir("cli_run_parse_error");
    let file = dir.join("broken.ruff");
    write_fixture(&file, "print((1 + 2)\n");

    let output = run_ruff(&["run", file.to_str().expect("path should be utf-8")]);
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("[RUFPARSE001]"));
    assert!(stderr.contains("Expected ')'"));
}

#[test]
fn cli_test_run_exits_non_zero_on_parse_diagnostics() {
    let dir = unique_temp_dir("cli_test_run_parse_error");
    let file = dir.join("broken_test.ruff");
    write_fixture(&file, "test \"broken\" {\n    print((1 + 2)\n}\n");

    let output = run_ruff(&["test-run", file.to_str().expect("path should be utf-8")]);
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("[RUFPARSE001]"));
    assert!(stderr.contains("Expected ')'"));
}
