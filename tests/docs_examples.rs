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

fn expected_fail_examples_with_reason() -> [(&'static str, &'static str); 35] {
    [
        ("examples/await_test.ruff", "async/await syntax drift"),
        ("examples/benchmark_async.ruff", "legacy control-flow syntax drift"),
        (
            "examples/benchmarks/file_io.ruff",
            "benchmark fixture contains parser-incompatible syntax",
        ),
        (
            "examples/benchmarks/run_benchmarks.ruff",
            "known VM duplicate-declaration compile error in fixture",
        ),
        (
            "examples/benchmarks/sorting_algorithms.ruff",
            "benchmark fixture kept as negative-coverage debt",
        ),
        (
            "examples/benchmarks/string_processing.ruff",
            "benchmark fixture kept as negative-coverage debt",
        ),
        ("examples/csv_demo.ruff", "legacy stdlib/example syntax drift"),
        (
            "examples/database_mysql.ruff",
            "requires unsupported or drifted database demo syntax",
        ),
        (
            "examples/destructuring_demo.ruff",
            "destructuring surface still has parse drift in docs example",
        ),
        ("examples/http_streaming.ruff", "legacy loop syntax drift"),
        ("examples/io_module_demo.ruff", "legacy IO module example drift"),
        ("examples/math_module.ruff", "legacy math module example drift"),
        ("examples/minimal_async.ruff", "async/await syntax drift"),
        (
            "examples/pattern_matching.ruff",
            "pattern-matching syntax drift in legacy example",
        ),
        (
            "examples/project_api_tester.ruff",
            "named-argument style not supported by current parser",
        ),
        (
            "examples/project_data_pipeline.ruff",
            "pipeline project example has unresolved syntax debt",
        ),
        (
            "examples/project_log_analyzer.ruff",
            "named-argument style not supported by current parser",
        ),
        (
            "examples/project_markdown_converter.ruff",
            "project example has unresolved parse/runtime debt",
        ),
        (
            "examples/project_task_manager.ruff",
            "named-argument style not supported by current parser",
        ),
        (
            "examples/project_web_scraper.ruff",
            "named-argument style not supported by current parser",
        ),
        (
            "examples/projects/contact_manager.ruff",
            "project example has unresolved parse/runtime debt",
        ),
        (
            "examples/projects/log_parser.ruff",
            "project example has unresolved parse/runtime debt",
        ),
        (
            "examples/projects/oauth_github_demo.ruff",
            "uses unsupported null-coalescing operator syntax",
        ),
        (
            "examples/projects/streaming_downloader.ruff",
            "legacy loop syntax drift",
        ),
        (
            "examples/projects/weather_dashboard.ruff",
            "known VM duplicate-declaration compile error in fixture",
        ),
        (
            "examples/spread_operator_demo.ruff",
            "spread/index syntax drift in legacy example",
        ),
        ("examples/ssg/ssg_async.ruff", "async control-flow syntax drift"),
        ("examples/ssg/test_parse_perf.ruff", "intentional malformed fixture"),
        ("examples/ssg/test_trim.ruff", "intentional malformed fixture"),
        ("examples/stdlib_crypto.ruff", "legacy loop syntax drift"),
        (
            "examples/struct_self_methods.ruff",
            "struct method example has unresolved syntax debt",
        ),
        ("examples/testing_demo.ruff", "legacy test helper syntax drift"),
        ("examples/toml_demo.ruff", "intentional malformed string fixture"),
        ("examples/unary_operators.ruff", "legacy unary syntax drift"),
        ("examples/yaml_demo.ruff", "intentional malformed string fixture"),
    ]
}

fn expected_fail_examples() -> HashSet<&'static str> {
    expected_fail_examples_with_reason()
        .iter()
        .map(|(path, _reason)| *path)
        .collect()
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
    HashSet::new()
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

#[test]
fn expected_fail_examples_have_reasons_and_exist() {
    let root = repo_root();
    let expected_fails = expected_fail_examples_with_reason();
    assert!(
        !expected_fails.is_empty(),
        "expected-fail examples list should not be empty"
    );

    let mut seen = HashSet::new();
    for (path, reason) in expected_fails {
        assert!(!reason.trim().is_empty(), "missing reason for {path}");
        assert!(seen.insert(path), "duplicate expected-fail entry: {path}");
        assert!(
            root.join(path).exists(),
            "expected-fail example does not exist on disk: {path}"
        );
    }
}

#[test]
fn run_and_expected_fail_example_sets_do_not_overlap() {
    let run_set = run_examples();
    let expected_fail_set = expected_fail_examples();
    for run_example in run_set {
        assert!(
            !expected_fail_set.contains(run_example),
            "example cannot be both run and expected-fail: {run_example}"
        );
    }
}
