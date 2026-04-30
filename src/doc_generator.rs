use crate::interpreter::Interpreter;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct DocItem {
    pub name: String,
    pub line: usize,
    pub docs: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocGenerationSummary {
    pub output_dir: PathBuf,
    pub module_doc_path: PathBuf,
    pub builtin_doc_path: Option<PathBuf>,
    pub item_count: usize,
}

pub fn generate_docs_for_file(
    source_path: &Path,
    output_dir: &Path,
    include_builtins: bool,
) -> Result<DocGenerationSummary, String> {
    let source = fs::read_to_string(source_path)
        .map_err(|err| format!("Failed to read source file: {}", err))?;

    let items = extract_doc_items(&source);
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("Failed to create output directory: {}", err))?;

    let module_name = source_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("module");

    let module_doc_path = output_dir.join(format!("{}.html", module_name));
    let module_html = render_module_html(module_name, &items);
    fs::write(&module_doc_path, module_html)
        .map_err(|err| format!("Failed to write module docs: {}", err))?;

    let builtin_doc_path = if include_builtins {
        let path = output_dir.join("builtins.html");
        let html = render_builtin_html(&Interpreter::get_builtin_names());
        fs::write(&path, html).map_err(|err| format!("Failed to write builtin docs: {}", err))?;
        Some(path)
    } else {
        None
    };

    let index_path = output_dir.join("index.html");
    let index_html = render_index_html(module_name, &module_doc_path, builtin_doc_path.as_ref());
    fs::write(index_path, index_html)
        .map_err(|err| format!("Failed to write docs index: {}", err))?;

    Ok(DocGenerationSummary {
        output_dir: output_dir.to_path_buf(),
        module_doc_path,
        builtin_doc_path,
        item_count: items.len(),
    })
}

pub fn extract_doc_items(source: &str) -> Vec<DocItem> {
    let mut items = Vec::new();
    let mut pending_docs: Vec<String> = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("///") {
            let content = trimmed.trim_start_matches("///").trim_start().to_string();
            pending_docs.push(content);
            continue;
        }

        if pending_docs.is_empty() {
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if let Some(function_name) = parse_function_name(trimmed) {
            let docs = pending_docs.clone();
            let examples = extract_examples(&docs);
            items.push(DocItem {
                name: function_name,
                line: index + 1,
                docs,
                examples,
            });
        }

        pending_docs.clear();
    }

    items
}

fn parse_function_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("func ") {
        return None;
    }

    let after_keyword = trimmed.trim_start_matches("func ");
    let mut chars = after_keyword.chars().peekable();
    let mut name = String::new();

    while let Some(ch) = chars.peek().copied() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            name.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    if name.is_empty() { None } else { Some(name) }
}

fn extract_examples(docs: &[String]) -> Vec<String> {
    let mut examples = Vec::new();
    let mut in_code_block = false;
    let mut current_lines: Vec<String> = Vec::new();

    for line in docs {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_code_block {
                if !current_lines.is_empty() {
                    examples.push(current_lines.join("\n"));
                    current_lines.clear();
                }
                in_code_block = false;
            } else if trimmed == "```" || trimmed == "```ruff" {
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            current_lines.push(line.clone());
        }
    }

    if in_code_block && !current_lines.is_empty() {
        examples.push(current_lines.join("\n"));
    }

    examples
}

fn render_module_html(module_name: &str, items: &[DocItem]) -> String {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("  <meta charset=\"utf-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str(&format!(
        "  <title>Ruff Docs: {}</title>\n",
        escape_html(module_name)
    ));
    html.push_str("  <style>body{font-family:ui-sans-serif,system-ui,sans-serif;max-width:920px;margin:2rem auto;padding:0 1rem;line-height:1.5}code,pre{font-family:ui-monospace,Menlo,monospace}pre{background:#f4f4f4;padding:0.8rem;border-radius:8px;overflow:auto}a{text-decoration:none}h1,h2{line-height:1.25}</style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str(&format!(
        "  <h1>Module Documentation: {}</h1>\n",
        escape_html(module_name)
    ));

    if items.is_empty() {
        html.push_str("  <p>No documented functions found. Add <code>///</code> comments above <code>func</code> declarations.</p>\n");
    } else {
        html.push_str("  <h2>Contents</h2>\n  <ul>\n");
        for item in items {
            html.push_str(&format!(
                "    <li><a href=\"#fn-{}\">{}</a></li>\n",
                escape_html(&item.name),
                escape_html(&item.name)
            ));
        }
        html.push_str("  </ul>\n");

        for item in items {
            html.push_str(&format!(
                "  <section id=\"fn-{}\">\n",
                escape_html(&item.name)
            ));
            html.push_str(&format!(
                "    <h2>{}</h2>\n",
                escape_html(&item.name)
            ));
            html.push_str(&format!("    <p><strong>Defined at line:</strong> {}</p>\n", item.line));

            if !item.docs.is_empty() {
                html.push_str("    <p>");
                html.push_str(&escape_html(&item.docs.join("\n")));
                html.push_str("</p>\n");
            }

            if !item.examples.is_empty() {
                html.push_str("    <h3>Examples</h3>\n");
                for example in item.examples.iter() {
                    html.push_str("    <pre><code>");
                    html.push_str(&escape_html(example));
                    html.push_str("</code></pre>\n");
                }
            }

            html.push_str("  </section>\n");
        }
    }

    html.push_str("</body>\n</html>\n");
    html
}

fn render_builtin_html(names: &[&str]) -> String {
    let mut sorted: Vec<&str> = names.to_vec();
    sorted.sort();

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("  <meta charset=\"utf-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("  <title>Ruff Builtin API Reference</title>\n");
    html.push_str("  <style>body{font-family:ui-sans-serif,system-ui,sans-serif;max-width:920px;margin:2rem auto;padding:0 1rem;line-height:1.5}code{font-family:ui-monospace,Menlo,monospace}li{margin:0.2rem 0}</style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str("  <h1>Ruff Builtin API Reference</h1>\n");
    html.push_str("  <p>Automatically generated from registered native builtin names.</p>\n");
    html.push_str("  <ul>\n");

    for name in sorted {
        html.push_str(&format!("    <li><code>{}</code></li>\n", escape_html(name)));
    }

    html.push_str("  </ul>\n</body>\n</html>\n");
    html
}

fn render_index_html(module_name: &str, module_doc_path: &Path, builtin_doc_path: Option<&PathBuf>) -> String {
    let module_file = module_doc_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("module.html");

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("  <meta charset=\"utf-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("  <title>Ruff Documentation Index</title>\n");
    html.push_str("  <style>body{font-family:ui-sans-serif,system-ui,sans-serif;max-width:720px;margin:2rem auto;padding:0 1rem;line-height:1.5}a{text-decoration:none}</style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str("  <h1>Ruff Documentation Index</h1>\n");
    html.push_str("  <ul>\n");
    html.push_str(&format!(
        "    <li><a href=\"{}\">Module: {}</a></li>\n",
        escape_html(module_file),
        escape_html(module_name)
    ));

    if let Some(path) = builtin_doc_path {
        let builtin_file = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("builtins.html");
        html.push_str(&format!(
            "    <li><a href=\"{}\">Builtin API reference</a></li>\n",
            escape_html(builtin_file)
        ));
    }

    html.push_str("  </ul>\n</body>\n</html>\n");
    html
}

fn escape_html(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::{extract_doc_items, generate_docs_for_file};
    use std::fs;

    #[test]
    fn extract_doc_items_reads_comments_and_examples() {
        let source = "/// Adds two numbers\n/// ```ruff\n/// add(1, 2)\n/// ```\nfunc add(a, b) {\n    return a + b\n}\n";

        let items = extract_doc_items(source);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "add");
        assert!(items[0].docs.iter().any(|line| line.contains("Adds two numbers")));
        assert_eq!(items[0].examples.len(), 1);
        assert!(items[0].examples[0].contains("add(1, 2)"));
    }

    #[test]
    fn generate_docs_writes_module_and_builtin_pages() {
        let source = "/// Echo value\nfunc echo(value) {\n    return value\n}\n";

        let unique = format!(
            "ruff_docgen_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        );

        let base_dir = std::env::temp_dir().join(unique);
        fs::create_dir_all(&base_dir).expect("temp output dir should be created");

        let source_path = base_dir.join("module.ruff");
        fs::write(&source_path, source).expect("source file should be written");

        let output_dir = base_dir.join("docs");
        let summary = generate_docs_for_file(&source_path, &output_dir, true)
            .expect("doc generation should succeed");

        assert_eq!(summary.item_count, 1);
        assert!(summary.module_doc_path.exists());
        assert!(summary
            .builtin_doc_path
            .as_ref()
            .map(|path| path.exists())
            .unwrap_or(false));
        assert!(output_dir.join("index.html").exists());
    }
}
