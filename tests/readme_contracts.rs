use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn readme_covers_v1_status_cli_security_and_core_reference_links() {
    let readme_path = repo_root().join("README.md");
    let content = fs::read_to_string(&readme_path).expect("failed to read README.md");

    let required_markers = [
        "## 1.0 Readiness Status",
        "## Safety Model Snapshot",
        "## Core Reference Links",
        "cargo build --release",
        "ruff run hello.ruff",
        "ruff serve [dir]",
        "--untrusted",
        "[ROADMAP.md](ROADMAP.md)",
        "[docs/LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md)",
        "[docs/STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md)",
        "[docs/RELEASE_PROCESS.md](docs/RELEASE_PROCESS.md)",
    ];

    for marker in required_markers {
        assert!(
            content.contains(marker),
            "expected README to include marker {:?}",
            marker
        );
    }
}
