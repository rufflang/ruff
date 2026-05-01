use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "ruff_{}_{}_{}_{}",
        prefix,
        std::process::id(),
        nanos,
        counter
    ));
    fs::create_dir_all(&path).expect("failed to create temp directory");
    path
}

fn ruff_binary() -> String {
    env!("CARGO_BIN_EXE_ruff").to_string()
}

fn run_ruff(args: &[&str], current_dir: &Path) -> Output {
    Command::new(ruff_binary())
        .current_dir(current_dir)
        .args(args)
        .output()
        .expect("failed to execute ruff binary")
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

fn assert_runtime_boundary_failure(script_source: &str, expected_runtime_error: &str) {
    let project_root = unique_temp_dir("native_api_security_boundary");
    let script_path = project_root.join("boundary.ruff");
    fs::write(&script_path, script_source).expect("failed to write script");

    let output = run_ruff(
        &["run", script_path.to_str().expect("script path should be utf-8"), "--interpreter"],
        &project_root,
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "expected runtime misuse to exit with code 1, got status={:?}, stdout={}, stderr={}",
        output.status.code(),
        stdout_text(&output),
        stderr_text(&output)
    );

    let combined_output = format!("{}\n{}", stdout_text(&output), stderr_text(&output));
    assert!(
        combined_output.contains(expected_runtime_error),
        "expected runtime error text '{}' in output, got stdout={} stderr={}",
        expected_runtime_error,
        stdout_text(&output),
        stderr_text(&output)
    );
}

#[test]
fn process_native_api_misuse_reports_deterministic_error() {
    assert_runtime_boundary_failure("execute(123)\n", "execute() requires a string command");
}

#[test]
fn network_native_api_misuse_reports_deterministic_error() {
    assert_runtime_boundary_failure(
        "tcp_receive(1, 10)\n",
        "tcp_receive requires (TcpStream, int_size) arguments",
    );
}

#[test]
fn filesystem_native_api_misuse_reports_deterministic_error() {
    assert_runtime_boundary_failure("write_file(1, 2)\n", "write_file requires string arguments");
}

#[test]
fn crypto_native_api_misuse_reports_deterministic_error() {
    assert_runtime_boundary_failure(
        "rsa_generate_keypair(1024)\n",
        "RSA key size must be 2048 or 4096 bits",
    );
}

#[test]
fn database_native_api_misuse_reports_deterministic_error() {
    assert_runtime_boundary_failure(
        "db_connect(\"sqlite\")\n",
        "db_connect requires database type ('sqlite'|'postgres'|'mysql') and connection string",
    );
}
