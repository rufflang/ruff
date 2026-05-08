use ruff::interpreter::Interpreter;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn documented_functions_from_reference(content: &str) -> HashSet<String> {
    let mut names = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("| `") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split('|').collect();
        if parts.len() < 3 {
            continue;
        }
        let function_cell = parts[1].trim();
        if !(function_cell.starts_with('`') && function_cell.ends_with('`')) {
            continue;
        }
        let name = function_cell.trim_matches('`');
        if !name.is_empty() {
            names.insert(name.to_string());
        }
    }

    names
}

#[test]
fn stdlib_reference_documents_runtime_builtins() {
    let reference_path = Path::new("docs/STANDARD_LIBRARY_REFERENCE.md");
    let reference = fs::read_to_string(reference_path)
        .expect("failed to read standard library reference markdown");

    let documented = documented_functions_from_reference(&reference);
    assert!(
        documented.len() >= 100,
        "expected broad builtin coverage in docs, got {} entries",
        documented.len()
    );

    let runtime: HashSet<String> =
        Interpreter::get_builtin_names().into_iter().map(|name| name.to_string()).collect();

    for function_name in &documented {
        assert!(
            runtime.contains(function_name),
            "documented builtin '{}' is not registered by runtime",
            function_name
        );
    }

    for required in [
        "print",
        "to_upper",
        "map",
        "parse_json",
        "read_file",
        "execute",
        "parallel_map",
        "http_get",
        "db_query",
        "sha256",
        "load_image",
    ] {
        assert!(
            documented.contains(required),
            "expected required canonical builtin '{}' to be documented",
            required
        );
    }
}
