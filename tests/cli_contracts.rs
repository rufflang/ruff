use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const EXIT_USAGE_ERROR: i32 = 2;
const EXIT_LEX_PARSE_ERROR: i32 = 3;
const EXIT_RUNTIME_ERROR: i32 = 4;
const EXIT_IO_ERROR: i32 = 5;

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ruff_{}_{}", prefix, nanos));
    fs::create_dir_all(&path).expect("failed to create temp directory");
    path
}

fn ruff_binary() -> String {
    env!("CARGO_BIN_EXE_ruff").to_string()
}

fn run_ruff(args: &[&str]) -> std::process::Output {
    Command::new(ruff_binary()).args(args).output().expect("failed to execute ruff binary")
}

fn write_fixture(path: &Path, content: &str) {
    fs::write(path, content).expect("failed to write fixture file");
}

#[test]
fn cli_help_exits_zero() {
    let output = run_ruff(&["--help"]);
    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Ruff: A modern programming language"));
}

#[test]
fn cli_version_exits_zero_and_prints_crate_version() {
    let output = run_ruff(&["--version"]);
    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn cli_run_missing_file_exits_with_io_error_code() {
    let dir = unique_temp_dir("cli_run_missing_file");
    let missing = dir.join("missing.ruff");

    let output = run_ruff(&["run", missing.to_str().expect("path should be utf-8")]);
    assert_eq!(output.status.code(), Some(EXIT_IO_ERROR));
    assert!(output.stdout.is_empty(), "run missing-file failure should not write stdout");

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("Failed to read .ruff file"));
}

#[test]
fn cli_run_parse_error_exits_with_lex_parse_code() {
    let dir = unique_temp_dir("cli_run_parse_exit_code");
    let file = dir.join("broken.ruff");
    write_fixture(&file, "print((1 + 2)\n");

    let output = run_ruff(&["run", file.to_str().expect("path should be utf-8")]);
    assert_eq!(output.status.code(), Some(EXIT_LEX_PARSE_ERROR));
    assert!(output.stdout.is_empty(), "parse failure should not write stdout");

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("[RUFPARSE001]"));
}

#[test]
fn cli_run_runtime_error_exits_with_runtime_code() {
    let dir = unique_temp_dir("cli_run_runtime_exit_code");
    let file = dir.join("runtime_error.ruff");
    write_fixture(&file, "let denom := 0\nprint(1 / denom)\n");

    let output = run_ruff(&["run", file.to_str().expect("path should be utf-8")]);
    assert_eq!(output.status.code(), Some(EXIT_RUNTIME_ERROR));
    assert!(output.stdout.is_empty(), "runtime failure should not write stdout");

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("Division by zero") || stderr.contains("divide by zero"));
}

#[test]
fn cli_usage_errors_use_usage_exit_code() {
    let output = run_ruff(&["run"]);
    assert_eq!(output.status.code(), Some(EXIT_USAGE_ERROR));
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("Usage:"));
}

#[test]
fn cli_lsp_diagnostics_json_is_valid_json() {
    let dir = unique_temp_dir("cli_lsp_diagnostics_json");
    let file = dir.join("broken.ruff");
    write_fixture(&file, "print((1 + 2)\n");

    let output =
        run_ruff(&["lsp-diagnostics", file.to_str().expect("path should be utf-8"), "--json"]);

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert!(parsed.is_array());
    assert!(output.stderr.is_empty(), "successful JSON diagnostics should not write stderr");
}
