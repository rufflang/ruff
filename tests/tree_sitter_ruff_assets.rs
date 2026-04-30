use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn tree_sitter_ruff_assets_exist_and_have_core_content() {
    let grammar_path = root().join("tree-sitter-ruff/grammar.js");
    let corpus_path = root().join("tree-sitter-ruff/test/corpus/core.txt");
    let highlights_path = root().join("tree-sitter-ruff/queries/highlights.scm");
    let injections_path = root().join("tree-sitter-ruff/queries/injections.scm");

    assert!(grammar_path.exists(), "missing grammar file");
    assert!(corpus_path.exists(), "missing corpus fixture");
    assert!(highlights_path.exists(), "missing highlights query file");
    assert!(injections_path.exists(), "missing injections query file");

    let grammar = fs::read_to_string(grammar_path).expect("failed to read grammar.js");
    let corpus = fs::read_to_string(corpus_path).expect("failed to read corpus fixture");
    let highlights = fs::read_to_string(highlights_path).expect("failed to read highlights query");

    assert!(grammar.contains("name: 'ruff'"));
    assert!(grammar.contains("function_definition"));
    assert!(grammar.contains("variable_declaration"));

    assert!(corpus.contains("Function definition"));
    assert!(corpus.contains("Variable declaration and call"));

    assert!(highlights.contains("@function"));
    assert!(highlights.contains("@variable"));
    assert!(highlights.contains("@keyword"));
}
