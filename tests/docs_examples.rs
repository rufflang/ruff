use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SmokeMode {
    Run,
    ParseOnly,
    ExpectedFail,
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ruff_{}_{}", prefix, nanos));
    fs::create_dir_all(&path).expect("failed to create temp directory");
    path
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn ruff_binary() -> String {
    env!("CARGO_BIN_EXE_ruff").to_string()
}

fn run_ruff(args: &[&str], current_dir: &Path) -> Output {
    Command::new(ruff_binary())
        .current_dir(current_dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to execute ruff binary")
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

fn collect_ruff_files(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(dir).expect("failed to read directory");
        for entry in entries {
            let entry = entry.expect("failed to read directory entry");
            let path = entry.path();
            if path.is_dir() {
                walk(&path, files);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) == Some("ruff") {
                files.push(path);
            }
        }
    }

    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    files
}

fn relative_from_repo(path: &Path) -> String {
    let root = repo_root();
    path.strip_prefix(&root)
        .expect("path should be inside repository root")
        .to_string_lossy()
        .replace('\\', "/")
}

fn run_examples() -> HashSet<&'static str> {
    HashSet::from([
        "examples/hello.ruff",
        "examples/arrays.ruff",
        "examples/dictionaries.ruff",
        "examples/string_interpolation.ruff",
        "examples/scoping_simple.ruff",
    ])
}

fn expected_fail_examples() -> HashSet<&'static str> {
    HashSet::from([
        "examples/await_test.ruff",
        "examples/benchmark_async.ruff",
        "examples/benchmarks/file_io.ruff",
        "examples/benchmarks/run_benchmarks.ruff",
        "examples/benchmarks/sorting_algorithms.ruff",
        "examples/benchmarks/string_processing.ruff",
        "examples/csv_demo.ruff",
        "examples/database_mysql.ruff",
        "examples/destructuring_demo.ruff",
        "examples/http_streaming.ruff",
        "examples/io_module_demo.ruff",
        "examples/math_module.ruff",
        "examples/minimal_async.ruff",
        "examples/pattern_matching.ruff",
        "examples/project_api_tester.ruff",
        "examples/project_data_pipeline.ruff",
        "examples/project_log_analyzer.ruff",
        "examples/project_markdown_converter.ruff",
        "examples/project_task_manager.ruff",
        "examples/project_web_scraper.ruff",
        "examples/projects/contact_manager.ruff",
        "examples/projects/log_parser.ruff",
        "examples/projects/oauth_github_demo.ruff",
        "examples/projects/streaming_downloader.ruff",
        "examples/projects/weather_dashboard.ruff",
        "examples/spread_operator_demo.ruff",
        "examples/ssg/ssg_async.ruff",
        "examples/ssg/test_parse_perf.ruff",
        "examples/ssg/test_trim.ruff",
        "examples/stdlib_crypto.ruff",
        "examples/struct_self_methods.ruff",
        "examples/testing_demo.ruff",
        "examples/toml_demo.ruff",
        "examples/unary_operators.ruff",
        "examples/yaml_demo.ruff",
    ])
}

fn classify_example(path: &str) -> SmokeMode {
    if run_examples().contains(path) {
        return SmokeMode::Run;
    }
    if expected_fail_examples().contains(path) {
        return SmokeMode::ExpectedFail;
    }
    SmokeMode::ParseOnly
}

fn expected_fail_doc_blocks() -> HashSet<&'static str> {
    HashSet::from([
        "docs/ARCHITECTURE.md#2",
        "docs/CONCURRENCY.md#10",
        "docs/CONCURRENCY.md#12",
        "docs/CONCURRENCY.md#14",
        "docs/CONCURRENCY.md#15",
        "docs/CONCURRENCY.md#19",
        "docs/MEMORY.md#4",
        "docs/MEMORY.md#8",
        "docs/MEMORY.md#10",
        "docs/MEMORY.md#22",
        "docs/OPTIONAL_TYPING_DESIGN.md#1",
        "docs/OPTIONAL_TYPING_DESIGN.md#2",
        "docs/PERFORMANCE.md#3",
        "docs/PERFORMANCE.md#5",
    ])
}

fn classify_doc_block(doc_block_id: &str) -> SmokeMode {
    if expected_fail_doc_blocks().contains(doc_block_id) {
        return SmokeMode::ExpectedFail;
    }
    SmokeMode::ParseOnly
}

fn markdown_files_for_doc_snippets() -> Vec<PathBuf> {
    let root = repo_root();
    let docs_dir = root.join("docs");
    let mut files = vec![root.join("README.md")];

    let entries = fs::read_dir(&docs_dir).expect("failed to read docs directory");
    for entry in entries {
        let entry = entry.expect("failed to read docs directory entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.push(path);
        }
    }

    files.sort();
    files
}

fn extract_ruff_blocks(markdown_path: &Path) -> Vec<(usize, String)> {
    let content = fs::read_to_string(markdown_path).expect("failed to read markdown file");
    let mut blocks = Vec::new();
    let mut in_ruff_block = false;
    let mut index = 0usize;
    let mut current = String::new();

    for line in content.lines() {
        if !in_ruff_block {
            if line.trim() == "```ruff" {
                in_ruff_block = true;
                index += 1;
                current.clear();
            }
            continue;
        }

        if line.trim() == "```" {
            in_ruff_block = false;
            blocks.push((index, current.clone()));
            current.clear();
            continue;
        }

        current.push_str(line);
        current.push('\n');
    }

    blocks
}

#[test]
fn examples_smoke_parse_run_or_expected_fail() {
    let root = repo_root();
    let examples_root = root.join("examples");
    let files = collect_ruff_files(&examples_root);
    assert!(!files.is_empty(), "expected at least one Ruff example file");

    let mut failures = Vec::new();

    for file in files {
        let rel = relative_from_repo(&file);
        let mode = classify_example(&rel);
        match mode {
            SmokeMode::Run => {
                let output = run_ruff(
                    &["run", "--interpreter", file.to_str().expect("path should be utf-8")],
                    &root,
                );
                if !output.status.success() {
                    failures.push(format!(
                        "RUN {} failed: status={:?} stdout={} stderr={}",
                        rel,
                        output.status.code(),
                        stdout_text(&output),
                        stderr_text(&output)
                    ));
                }
            }
            SmokeMode::ParseOnly => {
                let output = run_ruff(
                    &["check", file.to_str().expect("path should be utf-8"), "--quiet"],
                    &root,
                );
                if !output.status.success() {
                    failures.push(format!(
                        "PARSE {} failed unexpectedly: status={:?} stdout={} stderr={}",
                        rel,
                        output.status.code(),
                        stdout_text(&output),
                        stderr_text(&output)
                    ));
                }
            }
            SmokeMode::ExpectedFail => {
                let output = run_ruff(
                    &["check", file.to_str().expect("path should be utf-8"), "--quiet"],
                    &root,
                );
                if output.status.success() {
                    failures.push(format!(
                        "EXPECTED_FAIL {} now passes; reclassify as parse/run example",
                        rel
                    ));
                }
            }
        }
    }

    assert!(failures.is_empty(), "example smoke mismatches:\n{}", failures.join("\n"));
}

#[test]
fn docs_ruff_snippets_parse_or_expected_fail() {
    let root = repo_root();
    let temp_dir = unique_temp_dir("docs_snippet_smoke");
    let mut failures = Vec::new();

    for markdown_path in markdown_files_for_doc_snippets() {
        let rel = relative_from_repo(&markdown_path);
        let blocks = extract_ruff_blocks(&markdown_path);
        for (index, snippet) in blocks {
            let block_id = format!("{}#{}", rel, index);
            let mode = classify_doc_block(&block_id);
            let snippet_file =
                temp_dir.join(format!("{}_{}.ruff", rel.replace(['/', '.'], "_"), index));
            fs::write(&snippet_file, snippet).expect("failed to write snippet file");

            let output = run_ruff(
                &["check", snippet_file.to_str().expect("snippet path should be utf-8"), "--quiet"],
                &root,
            );

            match mode {
                SmokeMode::ParseOnly | SmokeMode::Run => {
                    if !output.status.success() {
                        failures.push(format!(
                            "DOC {} failed unexpectedly: status={:?} stdout={} stderr={}",
                            block_id,
                            output.status.code(),
                            stdout_text(&output),
                            stderr_text(&output)
                        ));
                    }
                }
                SmokeMode::ExpectedFail => {
                    if output.status.success() {
                        failures.push(format!(
                            "DOC expected-fail {} now passes; reclassify this snippet",
                            block_id
                        ));
                    }
                }
            }
        }
    }

    assert!(failures.is_empty(), "docs snippet smoke mismatches:\n{}", failures.join("\n"));
}
