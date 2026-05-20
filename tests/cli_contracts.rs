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

fn run_ruff_in_dir(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(ruff_binary())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to execute ruff binary")
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
fn cli_run_runtime_error_json_mode_emits_stdout_payload() {
    let dir = unique_temp_dir("cli_run_runtime_json_error");
    let file = dir.join("runtime_error_json.ruff");
    write_fixture(&file, "denom := 0\nprint(1 / denom)\n");

    let output = run_ruff(&[
        "run",
        file.to_str().expect("path should be utf-8"),
        "--json-runtime-diagnostics",
    ]);
    assert_eq!(output.status.code(), Some(EXIT_RUNTIME_ERROR));
    assert!(output.stderr.is_empty(), "json runtime diagnostics should suppress stderr");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value =
        serde_json::from_str(&stdout).expect("run --json-runtime-diagnostics should emit JSON");
    assert_eq!(parsed["command"], "run");
    assert_eq!(parsed["status"], "error");
    assert_eq!(parsed["kind"], "runtime_diagnostic");
    assert_eq!(parsed["exit_code"], EXIT_RUNTIME_ERROR);
    assert!(parsed["diagnostic"].is_object());
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

#[test]
fn cli_check_does_not_execute_script_side_effects() {
    let dir = unique_temp_dir("cli_check_no_side_effects");
    let file = dir.join("check_only.ruff");
    let marker = dir.join("marker.txt");
    let source = format!("write_file(\"{}\", \"created\", true)\n", marker.to_string_lossy());
    write_fixture(&file, &source);

    let output = run_ruff(&["check", file.to_str().expect("path should be utf-8")]);

    assert_eq!(output.status.code(), Some(0));
    assert!(!marker.exists(), "check command must not execute runtime side effects");
}

#[test]
fn cli_run_executes_program_output() {
    let dir = unique_temp_dir("cli_run_executes_output");
    let file = dir.join("prints.ruff");
    write_fixture(&file, "print(\"run-ok\")\n");

    let output = run_ruff(&["run", file.to_str().expect("path should be utf-8")]);

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty(), "successful run should not emit stderr");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("run-ok"), "run should execute and print script output");
}

#[test]
fn cli_test_discovers_and_runs_expected_fixtures() {
    let workspace = unique_temp_dir("cli_test_discovers_fixtures");
    let tests_dir = workspace.join("tests");
    fs::create_dir_all(&tests_dir).expect("failed to create tests directory");

    let fixture = tests_dir.join("sample.ruff");
    let snapshot = tests_dir.join("sample.out");
    write_fixture(&fixture, "print(\"fixture-ok\")\n");
    write_fixture(&snapshot, "fixture-ok\n");

    let output = run_ruff_in_dir(&["test"], &workspace);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty(), "test command should report results on stdout");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Passed 1/1 tests"), "test should discover and run fixture files");
    assert!(stdout.contains("[✓]"), "test should report passing fixture");
}

#[test]
fn cli_test_runtime_vm_mode_reports_mismatch_for_vm_drift_fixture() {
    let workspace = unique_temp_dir("cli_test_runtime_vm_mode");
    let tests_dir = workspace.join("tests");
    fs::create_dir_all(&tests_dir).expect("failed to create tests directory");

    let fixture = tests_dir.join("sample.ruff");
    let snapshot = tests_dir.join("sample.out");
    write_fixture(
        &fixture,
        "print(\"start\")\nresult := assert_equal(5, 5)\nprint(\"after first\")\ntry {\n    result := assert_equal(5, 10)\n    print(\"unexpected\")\n} except error {\n    print(\"caught\")\n}\nprint(\"end\")\n",
    );
    write_fixture(&snapshot, "start\nafter first\ncaught\nend\n");

    let output = run_ruff_in_dir(&["test", "--runtime", "vm"], &workspace);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty(), "test command should report results on stdout");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Passed 0/1 tests"), "vm mode should report failed snapshot match");
    assert!(stdout.contains("[✗]"), "vm mode should mark mismatched fixture as failed");
    assert!(
        stdout.contains("Runtime strategy: vm"),
        "vm mode should print runtime strategy summary"
    );
}

#[test]
fn cli_test_runtime_dual_mode_falls_back_to_interpreter_for_vm_drift_fixture() {
    let workspace = unique_temp_dir("cli_test_runtime_dual_mode");
    let tests_dir = workspace.join("tests");
    fs::create_dir_all(&tests_dir).expect("failed to create tests directory");

    let fixture = tests_dir.join("sample.ruff");
    let snapshot = tests_dir.join("sample.out");
    write_fixture(
        &fixture,
        "print(\"start\")\nresult := assert_equal(5, 5)\nprint(\"after first\")\ntry {\n    result := assert_equal(5, 10)\n    print(\"unexpected\")\n} except error {\n    print(\"caught\")\n}\nprint(\"end\")\n",
    );
    write_fixture(&snapshot, "start\nafter first\ncaught\nend\n");

    let output = run_ruff_in_dir(&["test", "--runtime", "dual"], &workspace);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty(), "test command should report results on stdout");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("Passed 1/1 tests"),
        "dual mode should recover via interpreter fallback"
    );
    assert!(stdout.contains("[✓]"), "dual mode should report passing fixture");
    assert!(
        stdout.contains("Runtime strategy: dual"),
        "dual mode should print runtime strategy summary"
    );
    assert!(
        stdout.contains("interpreter_fallback=1"),
        "dual mode should report fallback count for drift fixtures"
    );
}

#[test]
fn cli_check_verbose_and_quiet_output_are_deterministic() {
    let dir = unique_temp_dir("cli_check_verbosity");
    let file = dir.join("valid.ruff");
    write_fixture(&file, "let value := args()\n");

    let quiet = run_ruff(&["check", file.to_str().expect("path should be utf-8"), "--quiet"]);
    assert_eq!(quiet.status.code(), Some(0));
    assert!(quiet.stdout.is_empty(), "check --quiet should suppress success output on stdout");
    assert!(quiet.stderr.is_empty(), "check --quiet success should not write stderr");

    let verbose = run_ruff(&["check", file.to_str().expect("path should be utf-8"), "--verbose"]);
    assert_eq!(verbose.status.code(), Some(0));
    assert!(verbose.stderr.is_empty(), "check --verbose success should not write stderr");
    let verbose_stdout = String::from_utf8(verbose.stdout).expect("stdout should be utf-8");
    assert!(
        verbose_stdout.contains("check passed"),
        "check --verbose should emit deterministic success summary"
    );
    assert!(
        verbose_stdout.contains("statements="),
        "check --verbose should include statement-count metadata"
    );
}

#[test]
fn cli_check_json_success_is_valid_json() {
    let dir = unique_temp_dir("cli_check_json");
    let file = dir.join("valid.ruff");
    write_fixture(&file, "print(\"json-check\")\n");

    let output = run_ruff(&["check", file.to_str().expect("path should be utf-8"), "--json"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty(), "check --json success should not write stderr");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value = serde_json::from_str(&stdout).expect("check --json output should be JSON");
    assert_eq!(parsed["command"], "check");
    assert_eq!(parsed["status"], "ok");
    assert_eq!(parsed["file"], file.display().to_string());
    assert!(parsed["statement_count"].is_u64());
    assert!(parsed["bytecode_instruction_count"].is_u64());
}
