use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
    Command::new(ruff_binary())
        .args(args)
        .output()
        .expect("failed to execute ruff binary")
}

fn parse_stdout_json(output: &std::process::Output) -> Value {
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8");
    serde_json::from_str(&stdout).expect("stdout should be valid json")
}

fn write_fixture(path: &Path, content: &str) {
    fs::write(path, content).expect("failed to write fixture file");
}

#[test]
fn format_json_contract_is_stable() {
    let dir = unique_temp_dir("format_json_contract");
    let file = dir.join("format_input.ruff");
    write_fixture(&file, "let value:=1\n");

    let output = run_ruff(&[
        "format",
        file.to_str().expect("path should be utf-8"),
        "--json",
    ]);

    assert!(output.status.success(), "format --json should succeed");
    let body = parse_stdout_json(&output);

    assert_eq!(body["command"], "format");
    assert_eq!(body["status"], "preview");
    assert!(body["changed"].is_boolean());
    assert!(body["file"].is_string());
    assert!(body["options"].is_object());
    assert!(body["formatted_source"].is_string());
}

#[test]
fn lint_json_contract_is_stable() {
    let dir = unique_temp_dir("lint_json_contract");
    let file = dir.join("lint_input.ruff");
    write_fixture(&file, "let unused_value := 42\n");

    let output = run_ruff(&[
        "lint",
        file.to_str().expect("path should be utf-8"),
        "--json",
    ]);

    assert!(output.status.success(), "lint --json should succeed for warning-only input");
    let body = parse_stdout_json(&output);

    assert!(body.is_array());
    let first = body
        .as_array()
        .and_then(|items| items.first())
        .expect("lint output should include at least one issue");

    assert!(first.get("rule_id").is_some());
    assert!(first.get("line").is_some());
    assert!(first.get("column").is_some());
    assert!(first.get("severity").is_some());
    assert!(first.get("message").is_some());
    assert!(first.get("fix").is_some());
}

#[test]
fn docgen_json_contract_is_stable() {
    let dir = unique_temp_dir("docgen_json_contract");
    let file = dir.join("docgen_input.ruff");
    let out_dir = dir.join("docs_out");
    write_fixture(
        &file,
        "/// Adds one\nfunc add_one(value) {\n    return value + 1\n}\n",
    );

    let output = run_ruff(&[
        "docgen",
        file.to_str().expect("path should be utf-8"),
        "--out-dir",
        out_dir.to_str().expect("path should be utf-8"),
        "--json",
    ]);

    assert!(output.status.success(), "docgen --json should succeed");
    let body = parse_stdout_json(&output);

    assert_eq!(body["command"], "docgen");
    assert!(body["file"].is_string());
    assert!(body["output_dir"].is_string());
    assert!(body["module_doc_path"].is_string());
    assert!(body.get("builtin_doc_path").is_some());
    assert!(body["item_count"].is_number());
}

#[test]
fn lsp_cli_json_contracts_are_stable() {
    let dir = unique_temp_dir("lsp_json_contract");
    let file = dir.join("lsp_input.ruff");
    write_fixture(
        &file,
        "func greet(name) {\n    return name\n}\nlet value := greet(\"ruff\")\n",
    );

    let file_str = file.to_str().expect("path should be utf-8");

    let complete = run_ruff(&[
        "lsp-complete",
        file_str,
        "--line",
        "4",
        "--column",
        "18",
        "--json",
    ]);
    assert!(complete.status.success());
    let complete_body = parse_stdout_json(&complete);
    assert!(complete_body.is_array());

    let definition = run_ruff(&[
        "lsp-definition",
        file_str,
        "--line",
        "4",
        "--column",
        "14",
        "--json",
    ]);
    assert!(definition.status.success());
    let definition_body = parse_stdout_json(&definition);
    assert!(definition_body.is_object() || definition_body.is_null());

    let references = run_ruff(&[
        "lsp-references",
        file_str,
        "--line",
        "4",
        "--column",
        "14",
        "--json",
    ]);
    assert!(references.status.success());
    let references_body = parse_stdout_json(&references);
    assert!(references_body.is_array());

    let hover = run_ruff(&[
        "lsp-hover",
        file_str,
        "--line",
        "4",
        "--column",
        "14",
        "--json",
    ]);
    assert!(hover.status.success());
    let hover_body = parse_stdout_json(&hover);
    assert!(hover_body.is_object() || hover_body.is_null());

    let diagnostics_input = dir.join("lsp_bad_input.ruff");
    write_fixture(&diagnostics_input, "print((1 + 2)\n");
    let diagnostics = run_ruff(&[
        "lsp-diagnostics",
        diagnostics_input.to_str().expect("path should be utf-8"),
        "--json",
    ]);
    assert!(diagnostics.status.success());
    let diagnostics_body = parse_stdout_json(&diagnostics);
    assert!(diagnostics_body.is_array());

    let rename = run_ruff(&[
        "lsp-rename",
        file_str,
        "--line",
        "4",
        "--column",
        "14",
        "--new-name",
        "welcome",
        "--json",
    ]);
    assert!(rename.status.success());
    let rename_body = parse_stdout_json(&rename);
    assert!(rename_body.get("edit_count").is_some());
    assert!(rename_body.get("edits").is_some());
    assert!(rename_body.get("updated_source").is_some());

    let code_actions = run_ruff(&[
        "lsp-code-actions",
        diagnostics_input.to_str().expect("path should be utf-8"),
        "--json",
    ]);
    assert!(code_actions.status.success());
    let code_actions_body = parse_stdout_json(&code_actions);
    assert!(code_actions_body.is_array());
}
