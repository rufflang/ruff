use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn stdout_text(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr_text(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

fn tracked_root_files(root: &Path) -> Vec<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["ls-files"])
        .output()
        .expect("failed to run git ls-files");
    assert!(output.status.success(), "git ls-files failed: {}", stderr_text(&output));

    let mut files: Vec<String> = stdout_text(&output)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.contains('/'))
        .map(ToOwned::to_owned)
        .collect();
    files.sort();
    files
}

#[test]
fn tracked_root_surface_matches_hygiene_allowlist() {
    let root = repo_root();
    let tracked = tracked_root_files(&root);
    let expected = vec![
        ".editorconfig".to_string(),
        ".gitignore".to_string(),
        "CHANGELOG.md".to_string(),
        "CONTRIBUTING.md".to_string(),
        "Cargo.lock".to_string(),
        "Cargo.toml".to_string(),
        "INSTALLATION.md".to_string(),
        "LICENSE".to_string(),
        "README.md".to_string(),
        "ROADMAP.md".to_string(),
        "rustfmt.toml".to_string(),
    ];

    assert_eq!(
        tracked, expected,
        "tracked root surface drifted; update policy+test together if intentional"
    );
}

#[test]
fn repo_hygiene_policy_lists_current_root_contract() {
    let policy_path = repo_root().join("docs/REPO_HYGIENE_POLICY.md");
    let content = fs::read_to_string(&policy_path).expect("failed to read REPO_HYGIENE_POLICY.md");

    for marker in [
        "# Repository Hygiene Policy",
        "## Root Surface Contract",
        "## Retention And Cleanup",
        "cargo test --test repo_hygiene_contract",
    ] {
        assert!(content.contains(marker), "missing policy marker: {}", marker);
    }
}
