use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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

fn parse_stdout_json(output: &Output) -> Value {
    let stdout = stdout_text(output);
    serde_json::from_str(&stdout).expect("stdout should be valid json")
}

#[test]
fn package_module_workflow_end_to_end_contract() {
    let project_root = unique_temp_dir("package_module_workflow");
    let project_root_str = project_root.to_str().expect("path should be utf-8");

    let init =
        run_ruff(&["init", "--dir", project_root_str, "--name", "workflow_demo"], &project_root);
    assert!(
        init.status.success(),
        "ruff init failed: stdout={} stderr={}",
        stdout_text(&init),
        stderr_text(&init)
    );

    let manifest_path = project_root.join("ruff.toml");
    let manifest_path_str = manifest_path.to_str().expect("path should be utf-8");
    assert!(manifest_path.exists(), "expected init to create ruff.toml");
    assert!(project_root.join("src/main.ruff").exists(), "expected init to create src/main.ruff");

    let package_add = run_ruff(
        &["package-add", "example_dep", "--version", "1.2.3", "--manifest", manifest_path_str],
        &project_root,
    );
    assert!(
        package_add.status.success(),
        "ruff package-add failed: stdout={} stderr={}",
        stdout_text(&package_add),
        stderr_text(&package_add)
    );

    let package_install =
        run_ruff(&["package-install", "--manifest", manifest_path_str], &project_root);
    assert!(
        package_install.status.success(),
        "ruff package-install failed: stdout={} stderr={}",
        stdout_text(&package_install),
        stderr_text(&package_install)
    );
    assert!(
        stdout_text(&package_install).contains("install\texample_dep\t1.2.3"),
        "expected package-install output to include declared dependency, got: {}",
        stdout_text(&package_install)
    );

    let module_path = project_root.join("math_helper.ruff");
    fs::write(&module_path, "export answer := 42\n").expect("failed to write module file");

    let workflow_path = project_root.join("workflow.ruff");
    fs::write(&workflow_path, "from math_helper import answer\nprint(answer)\n")
        .expect("failed to write workflow script");

    let run_output = run_ruff(
        &["run", workflow_path.to_str().expect("path should be utf-8"), "--interpreter"],
        &project_root,
    );
    assert!(
        run_output.status.success(),
        "ruff run workflow failed: stdout={} stderr={}",
        stdout_text(&run_output),
        stderr_text(&run_output)
    );
    assert!(
        stdout_text(&run_output).contains("42"),
        "expected import/export workflow to print imported value, got stdout={} stderr={}",
        stdout_text(&run_output),
        stderr_text(&run_output)
    );

    let format_output = run_ruff(
        &["format", workflow_path.to_str().expect("path should be utf-8"), "--json"],
        &project_root,
    );
    assert!(
        format_output.status.success(),
        "ruff format --json failed: stdout={} stderr={}",
        stdout_text(&format_output),
        stderr_text(&format_output)
    );
    let format_json = parse_stdout_json(&format_output);
    assert_eq!(format_json["command"], "format");

    let lint_output = run_ruff(
        &["lint", workflow_path.to_str().expect("path should be utf-8"), "--json"],
        &project_root,
    );
    assert!(
        lint_output.status.success(),
        "ruff lint --json failed: stdout={} stderr={}",
        stdout_text(&lint_output),
        stderr_text(&lint_output)
    );
    let lint_json = parse_stdout_json(&lint_output);
    assert!(lint_json.is_array());

    let docs_input = project_root.join("docs_input.ruff");
    fs::write(&docs_input, "/// Adds one\nfunc add_one(value) {\n\treturn value + 1\n}\n")
        .expect("failed to write doc input file");
    let docs_out_dir = project_root.join("generated_docs");

    let docgen_output = run_ruff(
        &[
            "docgen",
            docs_input.to_str().expect("path should be utf-8"),
            "--out-dir",
            docs_out_dir.to_str().expect("path should be utf-8"),
            "--json",
        ],
        &project_root,
    );
    assert!(
        docgen_output.status.success(),
        "ruff docgen --json failed: stdout={} stderr={}",
        stdout_text(&docgen_output),
        stderr_text(&docgen_output)
    );
    let docgen_json = parse_stdout_json(&docgen_output);
    assert_eq!(docgen_json["command"], "docgen");
    assert!(docgen_json["item_count"].as_u64().unwrap_or(0) >= 1);
}

#[test]
fn package_module_cycle_error_reports_import_chain() {
    let project_root = unique_temp_dir("package_module_cycle_error");

    let module_a = "cycle_a";
    let module_b = "cycle_b";
    let module_a_path = project_root.join(format!("{}.ruff", module_a));
    let module_b_path = project_root.join(format!("{}.ruff", module_b));
    let workflow_path = project_root.join("cycle_workflow.ruff");

    fs::write(&module_a_path, format!("import {}\nexport a := 1\n", module_b))
        .expect("failed to write module A");
    fs::write(&module_b_path, format!("import {}\nexport b := 2\n", module_a))
        .expect("failed to write module B");
    fs::write(&workflow_path, format!("import {}\nprint(\"unreachable\")\n", module_a))
        .expect("failed to write cycle workflow script");

    let run_output = run_ruff(
        &["run", workflow_path.to_str().expect("path should be utf-8"), "--interpreter"],
        &project_root,
    );

    assert!(
        !run_output.status.success(),
        "expected circular import run to fail, stdout={} stderr={}",
        stdout_text(&run_output),
        stderr_text(&run_output)
    );

    let stderr = stderr_text(&run_output);
    assert!(
        stderr.contains("Circular import detected: cycle_a -> cycle_b -> cycle_a"),
        "expected circular import chain in stderr, got: {}",
        stderr
    );
}

#[test]
fn package_module_run_refreshes_after_module_file_change() {
    let project_root = unique_temp_dir("package_module_refresh");

    let module_path = project_root.join("math_helper.ruff");
    let workflow_path = project_root.join("refresh_workflow.ruff");

    fs::write(&module_path, "export answer := 10\n").expect("failed to write initial module");
    fs::write(&workflow_path, "from math_helper import answer\nprint(answer)\n")
        .expect("failed to write workflow script");

    let first_run = run_ruff(
        &["run", workflow_path.to_str().expect("path should be utf-8"), "--interpreter"],
        &project_root,
    );
    assert!(
        first_run.status.success(),
        "first run failed: stdout={} stderr={}",
        stdout_text(&first_run),
        stderr_text(&first_run)
    );
    assert!(
        stdout_text(&first_run).contains("10"),
        "expected first run output to include 10, got stdout={} stderr={}",
        stdout_text(&first_run),
        stderr_text(&first_run)
    );

    fs::write(&module_path, "export answer := 25\n")
        .expect("failed to update module export value");

    let second_run = run_ruff(
        &["run", workflow_path.to_str().expect("path should be utf-8"), "--interpreter"],
        &project_root,
    );
    assert!(
        second_run.status.success(),
        "second run failed: stdout={} stderr={}",
        stdout_text(&second_run),
        stderr_text(&second_run)
    );
    assert!(
        stdout_text(&second_run).contains("25"),
        "expected second run output to include updated value 25, got stdout={} stderr={}",
        stdout_text(&second_run),
        stderr_text(&second_run)
    );
}
