use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("ruff_vm_runtime_mismatch_inventory_{}_{}", prefix, nanos));
    fs::create_dir_all(&path).expect("failed to create temp directory");
    path
}

fn script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("generate_vm_runtime_mismatch_inventory.sh")
}

#[test]
fn vm_runtime_mismatch_inventory_script_generates_expected_outputs() {
    let dir = unique_temp_dir("success");
    let output_md = dir.join("inventory.md");
    let output_csv = dir.join("inventory.csv");

    let output = Command::new("bash")
        .arg(script_path())
        .arg("--tests-dir")
        .arg("tests")
        .arg("--output-md")
        .arg(&output_md)
        .arg("--output-csv")
        .arg(&output_csv)
        .arg("--runner")
        .arg("target/debug/ruff")
        .arg("--max-fixtures")
        .arg("5")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to execute inventory script");

    assert!(
        output.status.success(),
        "inventory script should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let markdown = fs::read_to_string(&output_md).expect("inventory markdown should exist");
    assert!(markdown.contains("# VM Runtime Mismatch Inventory"));
    assert!(markdown.contains("| Fixture | VM Exit | Interpreter Exit | VM Matches Snapshot | Interpreter Matches Snapshot | Delta Type |"));
    assert!(markdown.contains("Summary: `5` fixtures scanned"));

    let csv = fs::read_to_string(&output_csv).expect("inventory csv should exist");
    assert!(csv.contains("fixture,vm_exit,interpreter_exit,vm_matches_snapshot,interpreter_matches_snapshot,delta_type"));
}

#[test]
fn vm_runtime_mismatch_inventory_script_is_deterministic_for_capped_scan() {
    let dir = unique_temp_dir("determinism");
    let output_md_a = dir.join("inventory_a.md");
    let output_csv_a = dir.join("inventory_a.csv");
    let output_md_b = dir.join("inventory_b.md");
    let output_csv_b = dir.join("inventory_b.csv");

    let run_once = |output_md: &PathBuf, output_csv: &PathBuf| {
        let output = Command::new("bash")
            .arg(script_path())
            .arg("--tests-dir")
            .arg("tests")
            .arg("--output-md")
            .arg(output_md)
            .arg("--output-csv")
            .arg(output_csv)
            .arg("--runner")
            .arg("target/debug/ruff")
            .arg("--max-fixtures")
            .arg("4")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("failed to execute inventory script");

        assert!(
            output.status.success(),
            "inventory script should succeed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    };

    run_once(&output_md_a, &output_csv_a);
    run_once(&output_md_b, &output_csv_b);

    let markdown_a = fs::read_to_string(output_md_a).expect("first markdown output should exist");
    let markdown_b = fs::read_to_string(output_md_b).expect("second markdown output should exist");
    assert_eq!(markdown_a, markdown_b, "markdown output should be deterministic");

    let csv_a = fs::read_to_string(output_csv_a).expect("first csv output should exist");
    let csv_b = fs::read_to_string(output_csv_b).expect("second csv output should exist");
    assert_eq!(csv_a, csv_b, "csv output should be deterministic");
}
