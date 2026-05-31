use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn architecture_doc_matches_current_runtime_posture() {
    let path = repo_root().join("docs").join("ARCHITECTURE.md");
    let content = fs::read_to_string(&path).expect("failed to read docs/ARCHITECTURE.md");

    for marker in [
        "# Ruff Architecture",
        "Current crate version: `0.14.0`",
        "VM (default `ruff run` path)",
        "Tree-walking interpreter (explicit fallback path)",
        "Supports `--runtime dual|vm|interpreter`.",
        "Top-level generator iteration (`func*` + `yield`) is intentionally divergent",
        "docs/VM_INTERPRETER_PARITY_MATRIX.md",
    ] {
        assert!(content.contains(marker), "architecture doc should contain marker {:?}", marker);
    }

    for stale in [
        "Tree-walking interpreter** (current primary execution path)",
        "Bytecode VM** (experimental, not yet default)",
        "Version**: v0.8.0",
        "v0.9.0 modularization in progress",
    ] {
        assert!(
            !content.contains(stale),
            "architecture doc should not contain stale marker {:?}",
            stale
        );
    }
}
