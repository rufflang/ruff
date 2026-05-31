use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn import_heavy_perf_comparison_artifact_contains_required_markers() {
    let path =
        repo_root().join("docs").join("generated").join("VM_IMPORT_HEAVY_PERF_COMPARISON.md");
    let content = fs::read_to_string(&path)
        .expect("failed to read docs/generated/VM_IMPORT_HEAVY_PERF_COMPARISON.md");

    for marker in [
        "# VM Import-Heavy Nested Startup Perf Comparison",
        "Benchmark target: `module_resolution/import_heavy_nested_dotted_startup_cold_loader`",
        "## Tolerance Policy",
        "Unacceptable regression threshold:",
        "## Result",
        "Interpretation: `PASS`",
    ] {
        assert!(
            content.contains(marker),
            "perf comparison artifact should contain marker {:?}",
            marker
        );
    }
}
