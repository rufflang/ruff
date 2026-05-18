use ruff::docgen::core::{run as run_docgen, DocOutputFormat, DocgenConfig};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn symbol_visibility<'a>(symbols: &'a [Value], qualified_name: &str) -> &'a str {
    symbols
        .iter()
        .find(|symbol| symbol["qualified_name"] == qualified_name)
        .unwrap_or_else(|| panic!("missing symbol '{}'", qualified_name))["visibility"]
        .as_str()
        .unwrap_or_else(|| panic!("symbol '{}' visibility should be string", qualified_name))
}

fn has_symbol(symbols: &[Value], qualified_name: &str) -> bool {
    symbols.iter().any(|symbol| symbol["qualified_name"] == qualified_name)
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ruff_docgen_universal_{}_{}", prefix, nanos));
    fs::create_dir_all(&path).expect("failed to create temp directory");
    path
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent directory");
    }
    fs::write(path, content).expect("failed to write fixture file");
}

fn docgen_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("docgen")
        .join(name)
}

fn run_ruff(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ruff"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to execute ruff binary")
}

#[test]
fn docgen_captures_documented_and_undocumented_ruff_symbols() {
    let dir = unique_temp_dir("ruff_symbols");
    let input = dir.join("mod.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "/// Adds one\nfunc add_one(value) {\n    return value + 1\n}\n\nfunc sub_one(value) {\n    return value - 1\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input: input.clone(),
        out_dir: out.clone(),
        format: DocOutputFormat::All,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: true,
        search_index: true,
        source_links: true,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("docgen should succeed");

    assert!(summary.item_count >= 2);
    assert!(summary.project_json_path.exists());
    assert!(summary.gaps_json_path.exists());
    assert!(summary.ai_tasks_path.as_ref().expect("ai tasks path").exists());

    let project_json =
        fs::read_to_string(summary.project_json_path).expect("failed to read docgen json");
    let project: Value =
        serde_json::from_str(&project_json).expect("docgen.json should be valid json");
    let symbols = project["symbols"].as_array().expect("symbols should be an array");

    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "add_one" && symbol["docs"]["placeholder"] == false
    }));
    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "sub_one" && symbol["docs"]["placeholder"] == true
    }));
}

#[test]
fn docgen_ruff_supports_additional_doc_comment_styles() {
    let dir = unique_temp_dir("ruff_doc_comment_styles");
    let input = dir.join("doc_styles.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "//! Public API docs from bang style\npub func bang_doc_api() {\n    return 1\n}\n\n/**\n * Public API docs from block style\n */\npub func block_doc_api() {\n    return 2\n}\n\n/* Regular block comment should not be treated as docs */\npub func no_doc_api() {\n    return 3\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("docgen should succeed");

    let project_json =
        fs::read_to_string(summary.project_json_path).expect("failed to read docgen json");
    let project: Value =
        serde_json::from_str(&project_json).expect("docgen.json should be valid json");
    let symbols = project["symbols"].as_array().expect("symbols should be an array");

    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "bang_doc_api" && symbol["docs"]["placeholder"] == false
    }));
    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "block_doc_api" && symbol["docs"]["placeholder"] == false
    }));
    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "no_doc_api" && symbol["docs"]["placeholder"] == true
    }));
}

#[test]
fn docgen_ruff_attaches_docs_across_decorator_lines_without_overreaching() {
    let dir = unique_temp_dir("ruff_doc_attachment_decorators");
    let input = dir.join("doc_attachment.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "/// Decorated API docs\n@rate_limit(10)\npub func decorated_api() {\n    return 1\n}\n\n/// Internal state docs\n@memoized\nlet cached_value := 42\n\npub func missing_docs_api() {\n    return cached_value\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("docgen should succeed");

    let project_json =
        fs::read_to_string(summary.project_json_path).expect("failed to read docgen json");
    let project: Value =
        serde_json::from_str(&project_json).expect("docgen.json should be valid json");
    let symbols = project["symbols"].as_array().expect("symbols should be an array");

    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "decorated_api" && symbol["docs"]["placeholder"] == false
    }));
    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "cached_value" && symbol["docs"]["placeholder"] == false
    }));
    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "missing_docs_api" && symbol["docs"]["placeholder"] == true
    }));
}

#[test]
fn docgen_ruff_handles_spacing_and_proximity_edge_cases() {
    let dir = unique_temp_dir("ruff_doc_spacing_proximity");
    let input = dir.join("spacing_proximity.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "/// Spaced docs\n\n\npub func spaced_api() {\n    return 1\n}\n\n/// Blocked by regular comment\n// informational comment\npub func blocked_api() {\n    return 2\n}\n\n/// Internal state docs\nlet internal_value := 3\n\npub func missing_docs_api() {\n    return internal_value\n}\n\n/// First block\n\n/// Second block\npub func nearest_block_api() {\n    return 4\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("docgen should succeed");

    let project_json =
        fs::read_to_string(summary.project_json_path).expect("failed to read docgen json");
    let project: Value =
        serde_json::from_str(&project_json).expect("docgen.json should be valid json");
    let symbols = project["symbols"].as_array().expect("symbols should be an array");

    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "spaced_api" && symbol["docs"]["placeholder"] == false
    }));
    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "blocked_api" && symbol["docs"]["placeholder"] == true
    }));
    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "internal_value" && symbol["docs"]["placeholder"] == false
    }));
    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "missing_docs_api" && symbol["docs"]["placeholder"] == true
    }));
    assert!(symbols.iter().any(|symbol| {
        symbol["qualified_name"] == "nearest_block_api"
            && symbol["docs"]["summary"] == "Second block"
    }));
}

#[test]
fn docgen_emits_discovery_diagnostics_for_oversized_files() {
    let dir = unique_temp_dir("docgen_discovery_max_file_size");
    let oversized = dir.join("oversized.ruff");
    let small = dir.join("small.ruff");
    let out = dir.join("docs");

    write_file(&oversized, &"a".repeat((2 * 1024 * 1024) + 1));
    write_file(&small, "pub func kept_api() {\n    return 1\n}\n");

    let (_project, summary) = run_docgen(&DocgenConfig {
        input: dir.clone(),
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("docgen should succeed");

    assert!(summary.diagnostics_count > 0, "expected discovery warning diagnostics");
    assert_eq!(summary.discovery_skip_counts.get("max_file_size").copied().unwrap_or_default(), 1);
    assert_eq!(summary.discovery_skip_counts.get("max_depth").copied().unwrap_or_default(), 0);
    assert_eq!(summary.discovery_skip_counts.get("max_files").copied().unwrap_or_default(), 0);

    let project_json =
        fs::read_to_string(summary.project_json_path).expect("failed to read docgen json");
    let project: Value =
        serde_json::from_str(&project_json).expect("docgen.json should be valid json");
    let symbols = project["symbols"].as_array().expect("symbols should be an array");
    let diagnostics = project["diagnostics"].as_array().expect("diagnostics should be an array");

    assert!(has_symbol(symbols, "kept_api"));
    assert!(diagnostics.iter().any(|diag| {
        diag["code"] == "DOCGEN_DISCOVERY_MAX_FILE_SIZE"
            && diag["severity"] == "Warning"
            && diag["path"].as_str().unwrap_or_default().ends_with("oversized.ruff")
    }));
}

#[test]
fn docgen_diagnostics_order_is_deterministic_across_sources() {
    let dir = unique_temp_dir("docgen_diagnostic_ordering");
    let input = dir.join("module.ruff");
    let oversized = dir.join("oversized.ruff");
    let out_a = dir.join("docs_a");
    let out_b = dir.join("docs_b");

    write_file(
        &input,
        "/// Uses a broken doc link: [missing](./does-not-exist.md)\npub func public_api() {\n    return 1\n}\n",
    );
    write_file(&oversized, &"z".repeat((2 * 1024 * 1024) + 1));

    let (_project_a, summary_a) = run_docgen(&DocgenConfig {
        input: dir.clone(),
        out_dir: out_a,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("first docgen run should succeed");

    let (_project_b, summary_b) = run_docgen(&DocgenConfig {
        input: dir,
        out_dir: out_b,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("second docgen run should succeed");

    let project_a: Value = serde_json::from_str(
        &fs::read_to_string(summary_a.project_json_path).expect("read first project json"),
    )
    .expect("first project json should be valid");
    let project_b: Value = serde_json::from_str(
        &fs::read_to_string(summary_b.project_json_path).expect("read second project json"),
    )
    .expect("second project json should be valid");

    let diagnostics_a = project_a["diagnostics"]
        .as_array()
        .expect("diagnostics should be an array");
    let diagnostics_b = project_b["diagnostics"]
        .as_array()
        .expect("diagnostics should be an array");

    assert_eq!(
        diagnostics_a, diagnostics_b,
        "diagnostic ordering should remain stable across repeated runs"
    );

    let codes: Vec<&str> = diagnostics_a
        .iter()
        .map(|diag| diag["code"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(
        codes,
        vec!["DOCGEN_DISCOVERY_MAX_FILE_SIZE", "DOCGEN_LINK_BROKEN"],
        "diagnostics should be deterministically ordered by severity/code/path/line/message"
    );
}

#[test]
fn docgen_ruff_visibility_tracks_top_level_functions_and_struct_methods() {
    let dir = unique_temp_dir("ruff_visibility_matrix");
    let input = dir.join("visibility.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "func internal_helper() {\n    return 1\n}\n\npub func exported_api() {\n    return 2\n}\n\nstruct Worker {\n    func hidden_method(self) {\n        return 3\n    }\n\n    pub func visible_method(self) {\n        return 4\n    }\n}\n\npub struct PublicWorker {\n    pub func visible_method(self) {\n        return 5\n    }\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input: input.clone(),
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("docgen should succeed");

    let project_json =
        fs::read_to_string(summary.project_json_path).expect("failed to read docgen json");
    let project: Value =
        serde_json::from_str(&project_json).expect("docgen.json should be valid json");
    let symbols = project["symbols"].as_array().expect("symbols should be an array");

    assert_eq!(symbol_visibility(symbols, "internal_helper"), "Private");
    assert_eq!(symbol_visibility(symbols, "exported_api"), "Public");
    assert_eq!(symbol_visibility(symbols, "Worker::hidden_method"), "Private");
    assert_eq!(symbol_visibility(symbols, "Worker::visible_method"), "Private");
    assert_eq!(symbol_visibility(symbols, "PublicWorker::visible_method"), "Public");
}

#[test]
fn docgen_strict_public_gate_ignores_private_undocumented_ruff_functions() {
    let dir = unique_temp_dir("ruff_visibility_strict_gate_private");
    let input = dir.join("strict_private.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "func private_helper() {\n    return 1\n}\n\n/// API docs\npub func public_api() {\n    return 2\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: true,
        fail_on_broken_links: true,
        fail_on_warnings: true,
        public_only: true,
        include_private: false,
    })
    .expect("strict docgen run should complete");

    assert_eq!(summary.undocumented_count, 0);
    assert_eq!(summary.warning_count, 0);
    assert_eq!(summary.broken_link_count, 0);
    assert!(summary.gate_failures.is_empty(), "strict gate should pass");
}

#[test]
fn docgen_strict_public_gate_still_fails_on_undocumented_explicit_public_ruff_function() {
    let dir = unique_temp_dir("ruff_visibility_strict_gate_public");
    let input = dir.join("strict_public.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "func private_helper() {\n    return 1\n}\n\npub func public_missing_docs() {\n    return 2\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: true,
        fail_on_broken_links: true,
        fail_on_warnings: true,
        public_only: true,
        include_private: false,
    })
    .expect("strict docgen run should complete");

    assert_eq!(summary.undocumented_count, 1);
    assert!(
        summary
            .gate_failures
            .iter()
            .any(|failure| failure == "1 undocumented public symbols detected"),
        "strict gate should fail on undocumented explicit public symbols"
    );
}

#[test]
fn docgen_public_only_visibility_matrix_keeps_internal_helpers_out_of_public_gate() {
    let dir = unique_temp_dir("ruff_public_only_visibility_matrix");
    let input = dir.join("visibility_matrix_public_only.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "func internal_helper() {\n    return 1\n}\n\n/// Exported API docs\npub func exported_api() {\n    return 2\n}\n\nstruct InternalWorker {\n    pub func visible_but_private(self) {\n        return 3\n    }\n\n    func hidden_method(self) {\n        return 4\n    }\n}\n\n/// Public worker docs\npub struct PublicWorker {\n    /// Public method docs\n    pub func documented_public_method(self) {\n        return 5\n    }\n\n    func private_method(self) {\n        return 6\n    }\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: true,
        fail_on_broken_links: true,
        fail_on_warnings: true,
        public_only: true,
        include_private: false,
    })
    .expect("strict public-only docgen run should complete");

    assert_eq!(summary.undocumented_count, 0);
    assert!(summary.gate_failures.is_empty(), "strict gate should pass");

    let project_json =
        fs::read_to_string(summary.project_json_path).expect("failed to read docgen json");
    let project: Value =
        serde_json::from_str(&project_json).expect("docgen.json should be valid json");
    let symbols = project["symbols"].as_array().expect("symbols should be an array");

    assert!(has_symbol(symbols, "exported_api"));
    assert!(has_symbol(symbols, "PublicWorker"));
    assert!(has_symbol(symbols, "PublicWorker::documented_public_method"));

    assert!(!has_symbol(symbols, "internal_helper"));
    assert!(!has_symbol(symbols, "InternalWorker"));
    assert!(!has_symbol(symbols, "InternalWorker::visible_but_private"));
    assert!(!has_symbol(symbols, "InternalWorker::hidden_method"));
    assert!(!has_symbol(symbols, "PublicWorker::private_method"));
}

#[test]
fn docgen_public_only_excludes_methods_on_private_structs() {
    let dir = unique_temp_dir("ruff_private_struct_methods_gate");
    let input = dir.join("private_struct_methods.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "struct InternalWorker {\n    pub func internal_api(self) {\n        return 1\n    }\n}\n\n/// Public worker docs\npub struct PublicWorker {\n    /// Public worker API\n    pub func exposed_api(self) {\n        return 2\n    }\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: true,
        fail_on_broken_links: true,
        fail_on_warnings: true,
        public_only: true,
        include_private: false,
    })
    .expect("strict docgen run should complete");

    assert_eq!(summary.undocumented_count, 0);
    assert!(summary.gate_failures.is_empty(), "strict gate should pass");

    let project_json =
        fs::read_to_string(summary.project_json_path).expect("failed to read docgen json");
    let project: Value =
        serde_json::from_str(&project_json).expect("docgen.json should be valid json");
    let symbols = project["symbols"].as_array().expect("symbols should be an array");

    assert!(
        !has_symbol(symbols, "InternalWorker::internal_api"),
        "public-only output should not include methods on private structs"
    );
    assert!(
        has_symbol(symbols, "PublicWorker::exposed_api"),
        "public-only output should include methods on public structs"
    );
}

#[test]
fn docgen_public_only_excludes_variants_of_private_enums() {
    let dir = unique_temp_dir("ruff_private_enum_variants_gate");
    let input = dir.join("private_enum_variants.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "enum InternalState {\n    Idle,\n    Busy,\n}\n\npub enum ApiState {\n    /// Public variant docs\n    Ready,\n    Running,\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: true,
        include_private: false,
    })
    .expect("public-only docgen run should complete");

    let project_json =
        fs::read_to_string(summary.project_json_path).expect("failed to read docgen json");
    let project: Value =
        serde_json::from_str(&project_json).expect("docgen.json should be valid json");
    let symbols = project["symbols"].as_array().expect("symbols should be an array");

    assert!(
        !has_symbol(symbols, "InternalState::Idle"),
        "public-only output should not include variants from private enums"
    );
    assert!(
        !has_symbol(symbols, "InternalState::Busy"),
        "public-only output should not include variants from private enums"
    );
    assert!(
        has_symbol(symbols, "ApiState"),
        "public enum should be included in public-only output"
    );
    assert!(
        has_symbol(symbols, "ApiState::Ready"),
        "public enum variants should be included in public-only output"
    );
}

#[test]
fn docgen_extracts_async_ruff_functions_and_methods_with_visibility() {
    let dir = unique_temp_dir("ruff_async_visibility_matrix");
    let input = dir.join("async_visibility.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "async func internal_async() {\n    return 1\n}\n\npub async func exported_async() {\n    return 2\n}\n\nstruct PrivateWorker {\n    pub async func visible_but_private(self) {\n        return 3\n    }\n}\n\npub struct PublicWorker {\n    async func hidden_async(self) {\n        return 4\n    }\n\n    pub async func visible_async(self) {\n        return 5\n    }\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("docgen should succeed");

    let project_json =
        fs::read_to_string(summary.project_json_path).expect("failed to read docgen json");
    let project: Value =
        serde_json::from_str(&project_json).expect("docgen.json should be valid json");
    let symbols = project["symbols"].as_array().expect("symbols should be an array");

    assert_eq!(symbol_visibility(symbols, "internal_async"), "Private");
    assert_eq!(symbol_visibility(symbols, "exported_async"), "Public");
    assert_eq!(symbol_visibility(symbols, "PrivateWorker::visible_but_private"), "Private");
    assert_eq!(symbol_visibility(symbols, "PublicWorker::hidden_async"), "Private");
    assert_eq!(symbol_visibility(symbols, "PublicWorker::visible_async"), "Public");
}

#[test]
fn docgen_strict_public_gate_handles_async_ruff_functions() {
    let dir = unique_temp_dir("ruff_async_visibility_strict_gate");
    let input = dir.join("async_strict_public.ruff");
    let out = dir.join("docs");

    write_file(
        &input,
        "async func private_async_helper() {\n    return 1\n}\n\npub async func public_async_missing_docs() {\n    return 2\n}\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: true,
        fail_on_broken_links: true,
        fail_on_warnings: true,
        public_only: true,
        include_private: false,
    })
    .expect("strict docgen run should complete");

    assert_eq!(summary.undocumented_count, 1);
    assert!(
        summary
            .gate_failures
            .iter()
            .any(|failure| failure == "1 undocumented public symbols detected"),
        "strict gate should fail on undocumented explicit public async symbols"
    );
}

#[test]
fn docgen_ruff_extraction_edge_fixture_async_visibility_contract() {
    let dir = unique_temp_dir("ruff_async_fixture_visibility");
    let input = dir.join("fixture_async_visibility.ruff");
    let out = dir.join("docs");

    let source = fs::read_to_string(docgen_fixture_path("ruff_async_visibility.ruff"))
        .expect("failed to read async visibility fixture");
    write_file(&input, &source);

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("docgen should succeed");

    let expected_json =
        fs::read_to_string(docgen_fixture_path("ruff_async_visibility.expected.json"))
            .expect("failed to read expected async visibility fixture");
    let expected: Value =
        serde_json::from_str(&expected_json).expect("expected visibility fixture should be json");
    let expected_map =
        expected.as_object().expect("expected visibility fixture should be an object map");

    let project_json =
        fs::read_to_string(summary.project_json_path).expect("failed to read docgen json");
    let project: Value =
        serde_json::from_str(&project_json).expect("docgen.json should be valid json");
    let symbols = project["symbols"].as_array().expect("symbols should be an array");

    for (qualified_name, visibility) in expected_map {
        let expected_visibility = visibility.as_str().unwrap_or_else(|| {
            panic!("visibility fixture for '{}' should be string", qualified_name)
        });
        assert_eq!(
            symbol_visibility(symbols, qualified_name),
            expected_visibility,
            "unexpected visibility for fixture symbol '{}'",
            qualified_name
        );
    }
}

#[test]
fn docgen_ruff_extraction_edge_fixture_async_strict_gate_contract() {
    let dir = unique_temp_dir("ruff_async_fixture_strict_gate");
    let input = dir.join("fixture_async_strict.ruff");
    let out = dir.join("docs");

    let source = fs::read_to_string(docgen_fixture_path("ruff_async_strict_public.ruff"))
        .expect("failed to read async strict fixture");
    write_file(&input, &source);

    let expected_json =
        fs::read_to_string(docgen_fixture_path("ruff_async_strict_public.expected.json"))
            .expect("failed to read expected async strict fixture");
    let expected: Value =
        serde_json::from_str(&expected_json).expect("expected strict fixture should be json");
    let expected_undocumented =
        expected["undocumented_count"].as_u64().expect("expected undocumented_count should be u64")
            as usize;
    let expected_gate_failure = expected["gate_failure_contains"]
        .as_str()
        .expect("expected gate_failure_contains should be string");

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: true,
        fail_on_broken_links: true,
        fail_on_warnings: true,
        public_only: true,
        include_private: false,
    })
    .expect("strict docgen run should complete");

    assert_eq!(summary.undocumented_count, expected_undocumented);
    assert!(
        summary.gate_failures.iter().any(|failure| failure.contains(expected_gate_failure)),
        "strict gate failures should contain '{}'",
        expected_gate_failure
    );
}

#[test]
fn docgen_cli_json_contract_preserves_legacy_fields() {
    let dir = unique_temp_dir("cli_contract");
    let input = dir.join("file.ruff");
    let out = dir.join("docs");

    write_file(&input, "func hello() {\n    return 1\n}\n");

    let output = run_ruff(
        &[
            "docgen",
            input.to_str().expect("path utf-8"),
            "--out-dir",
            out.to_str().expect("path utf-8"),
            "--json",
        ],
        &dir,
    );

    assert!(output.status.success(), "docgen cli should succeed");
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be json");

    assert_eq!(json["command"], "docgen");
    assert!(json["file"].is_string());
    assert!(json["output_dir"].is_string());
    assert!(json["module_doc_path"].is_string());
    assert!(json["item_count"].is_number());
    assert!(json["project_json_path"].is_string());
    assert!(json["gaps_json_path"].is_string());
}

#[test]
fn docgen_supports_mixed_language_projects_deterministically() {
    let dir = unique_temp_dir("mixed");
    let out_a = dir.join("docs_a");
    let out_b = dir.join("docs_b");

    write_file(&dir.join("main.ruff"), "func ruff_fn() { return 1 }\n");
    write_file(&dir.join("a.py"), "def py_fn(x):\n    return x\n");
    write_file(&dir.join("b.php"), "<?php\nfunction php_fn($x) { return $x; }\n");
    write_file(&dir.join("c.ts"), "export function tsFn(x: number): number { return x; }\n");
    write_file(&dir.join("d.js"), "export function jsFn(x) { return x; }\n");
    write_file(&dir.join("e.rb"), "def rb_fn(x)\n  x\nend\n");
    write_file(&dir.join("f.go"), "package main\nfunc GoFn(x int) int { return x }\n");
    write_file(&dir.join("g.hs"), "-- | doc\nhFn :: Int -> Int\nhFn x = x\n");
    write_file(&dir.join("h.zig"), "pub fn zigFn(x: i32) i32 { return x; }\n");

    let run_config = |out_dir: PathBuf| DocgenConfig {
        input: dir.clone(),
        out_dir,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: None,
        languages: Some("ruff,php,python,typescript,javascript,ruby,go,haskell,zig".to_string()),
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    };

    let (_, summary_a) =
        run_docgen(&run_config(out_a.clone())).expect("first docgen run should succeed");
    let (_, summary_b) =
        run_docgen(&run_config(out_b.clone())).expect("second docgen run should succeed");

    let json_a = fs::read_to_string(summary_a.project_json_path).expect("read first project json");
    let json_b = fs::read_to_string(summary_b.project_json_path).expect("read second project json");
    assert_eq!(json_a, json_b, "docgen output should be deterministic");

    let parsed: Value = serde_json::from_str(&json_a).expect("valid json");
    let langs: BTreeSet<String> = parsed["languages"]
        .as_array()
        .expect("languages should be array")
        .iter()
        .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
        .collect();

    for expected in
        ["ruff", "php", "python", "typescript", "javascript", "ruby", "go", "haskell", "zig"]
    {
        assert!(langs.contains(expected), "missing language {}", expected);
    }
}

#[test]
fn docgen_does_not_follow_symlink_escape() {
    #[cfg(unix)]
    {
        use std::os::unix::fs as unix_fs;

        let root = unique_temp_dir("symlink_root");
        let outside = unique_temp_dir("symlink_outside");
        let out_dir = root.join("docs");
        let outside_file = outside.join("escape.py");
        write_file(&outside_file, "def stolen():\n    return 1\n");
        let symlink = root.join("linked_escape.py");
        unix_fs::symlink(&outside_file, &symlink).expect("failed to create symlink");

        let (_project, summary) = run_docgen(&DocgenConfig {
            input: root.clone(),
            out_dir,
            format: DocOutputFormat::Json,
            include_builtins: false,
            language: None,
            languages: Some("python".to_string()),
            emit_ai_tasks: false,
            search_index: false,
            source_links: false,
            fail_on_undocumented: false,
            fail_on_broken_links: false,
            fail_on_warnings: false,
            public_only: false,
            include_private: true,
        })
        .expect("docgen should succeed");

        let json = fs::read_to_string(summary.project_json_path).expect("read json");
        assert!(!json.contains("stolen"), "symlink escape file should not be discovered");
    }
}

#[test]
fn docgen_never_executes_source_code() {
    let dir = unique_temp_dir("no_exec");
    let out = dir.join("docs");
    let marker = dir.join("should_not_exist.txt");

    write_file(
        &dir.join("danger.ruff"),
        &format!("write_file(\"{}\", \"owned\")\nfunc safe() {{ return 1 }}\n", marker.display()),
    );

    let output = run_ruff(
        &[
            "docgen",
            dir.to_str().expect("dir utf-8"),
            "--language",
            "ruff",
            "--out-dir",
            out.to_str().expect("out utf-8"),
        ],
        &dir,
    );

    assert!(output.status.success(), "docgen should parse without executing code");
    assert!(
        !marker.exists(),
        "marker file should not exist because docgen must not execute source"
    );
}

#[test]
fn docgen_html_escapes_untrusted_docs() {
    let dir = unique_temp_dir("escape_html");
    let out = dir.join("docs");
    let input = dir.join("x.ruff");
    write_file(&input, "/// <script>alert('xss')</script>\nfunc test() { return 1 }\n");

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Html,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("docgen should succeed");

    let html = fs::read_to_string(summary.module_doc_path).expect("read html");
    assert!(html.contains("&lt;script&gt;alert"), "html output should be escaped");
    assert!(!html.contains("<script>alert"), "raw script should not be present");
}

#[test]
fn docgen_adapter_conformance_smoke_extracts_symbols_for_all_languages() {
    let dir = unique_temp_dir("adapter_conformance");
    let out = dir.join("docs");

    write_file(&dir.join("sample.ruff"), "func ruff_fn() { return 1 }\n");
    write_file(&dir.join("sample.php"), "<?php\nfunction php_fn($x) { return $x; }\n");
    write_file(&dir.join("sample.py"), "def py_fn(x):\n    return x\n");
    write_file(&dir.join("sample.ts"), "export function tsFn(x: number): number { return x; }\n");
    write_file(&dir.join("sample.js"), "export function jsFn(x) { return x; }\n");
    write_file(&dir.join("sample.rb"), "def rb_fn(x)\n  x\nend\n");
    write_file(&dir.join("sample.go"), "package main\nfunc GoFn(x int) int { return x }\n");
    write_file(&dir.join("sample.hs"), "hFn :: Int -> Int\nhFn x = x\n");
    write_file(&dir.join("sample.zig"), "pub fn zigFn(x: i32) i32 { return x; }\n");

    let (project, _summary) = run_docgen(&DocgenConfig {
        input: dir.clone(),
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: None,
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("docgen should succeed");

    let present: BTreeSet<String> =
        project.symbols.iter().map(|symbol| symbol.language.clone()).collect();
    for language in
        ["ruff", "php", "python", "typescript", "javascript", "ruby", "go", "haskell", "zig"]
    {
        assert!(
            present.contains(language),
            "expected at least one extracted symbol for language {}",
            language
        );
    }
}

#[test]
fn docgen_strict_gates_fail_as_expected() {
    let dir = unique_temp_dir("strict_gates");
    let out = dir.join("docs");
    let input = dir.join("gate.ruff");
    write_file(
        &input,
        "/// See [Missing](missing.md)\npub func documented() { return 1 }\npub func undocumented() { return 2 }\n",
    );

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: true,
        fail_on_broken_links: true,
        fail_on_warnings: true,
        public_only: true,
        include_private: false,
    })
    .expect("docgen run itself should complete and report gate failures");

    assert!(!summary.gate_failures.is_empty(), "strict gates should report failure messages");
    assert!(
        summary.gate_failures.iter().any(|entry| entry.contains("undocumented")),
        "expected undocumented gate failure"
    );
    assert!(
        summary.gate_failures.iter().any(|entry| entry.contains("broken links")),
        "expected broken-link gate failure"
    );
}

#[test]
fn docgen_large_repo_smoke_completes_with_deterministic_counts() {
    let dir = unique_temp_dir("large_repo");
    let out = dir.join("docs");

    for idx in 0..250usize {
        write_file(
            &dir.join(format!("pkg/mod_{}.py", idx)),
            &format!("def fn_{}(x):\n    return x\n", idx),
        );
    }

    let (_project, summary) = run_docgen(&DocgenConfig {
        input: dir.clone(),
        out_dir: out,
        format: DocOutputFormat::Json,
        include_builtins: false,
        language: Some("python".to_string()),
        languages: None,
        emit_ai_tasks: false,
        search_index: false,
        source_links: false,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("large repo docgen should succeed");

    assert_eq!(summary.item_count, 250);
}

#[test]
fn docgen_snapshot_stable_core_outputs() {
    let dir = unique_temp_dir("snapshot");
    let out = dir.join("docs");
    let input = dir.join("snap.ruff");
    write_file(&input, "/// Add\nfunc add(a, b) { return a + b }\nconst VALUE := 1\n");

    let (_project, summary) = run_docgen(&DocgenConfig {
        input,
        out_dir: out,
        format: DocOutputFormat::All,
        include_builtins: false,
        language: Some("ruff".to_string()),
        languages: None,
        emit_ai_tasks: true,
        search_index: true,
        source_links: true,
        fail_on_undocumented: false,
        fail_on_broken_links: false,
        fail_on_warnings: false,
        public_only: false,
        include_private: true,
    })
    .expect("snapshot docgen should succeed");

    let json = fs::read_to_string(summary.project_json_path).expect("read project json");
    assert!(json.contains("\"qualified_name\": \"add\""));
    assert!(json.contains("\"qualified_name\": \"VALUE\""));

    let html = fs::read_to_string(summary.module_doc_path).expect("read html");
    assert!(html.contains("Universal Ruff DocGen"));

    let markdown = fs::read_to_string(summary.output_dir.join("docgen.md")).expect("read markdown");
    assert!(markdown.contains("# Ruff DocGen"));
}
