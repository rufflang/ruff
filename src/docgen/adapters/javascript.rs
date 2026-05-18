use super::common::{attach_docs_by_proximity, next_nonempty_line};
use super::{AdapterCapability, DocLanguageAdapter};
use crate::docgen::model::{DocComment, DocCommentBlock, DocSymbol, DocSymbolKind, DocVisibility};
use crate::docgen::DocgenError;
use regex::Regex;
use std::path::Path;

pub struct JavaScriptDocAdapter;

impl JavaScriptDocAdapter {
    fn id(path: &Path, line: usize, name: &str, kind: &DocSymbolKind) -> String {
        format!("javascript:{}:{}:{}:{:?}", path.display(), line, name, kind)
    }
}

impl DocLanguageAdapter for JavaScriptDocAdapter {
    fn language_id(&self) -> &'static str {
        "javascript"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["js", "jsx", "mjs", "cjs"]
    }

    fn capabilities(&self) -> AdapterCapability {
        AdapterCapability {
            supports_functions: true,
            supports_types: false,
            supports_methods: true,
            supports_inline_docs: true,
        }
    }

    fn extract_symbols(&self, source: &str, path: &Path) -> Result<Vec<DocSymbol>, DocgenError> {
        let re_class = Regex::new(r"^\s*(export\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)")
            .expect("js class regex");
        let re_function =
            Regex::new(r"^\s*(export\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)")
                .expect("js function regex");
        let re_method = Regex::new(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*\{")
            .expect("js method regex");

        let mut symbols = Vec::new();
        let mut class_stack: Vec<(String, i32)> = Vec::new();
        let mut depth = 0i32;

        for (idx, line) in source.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim();

            while class_stack.last().is_some_and(|(_, d)| depth < *d) {
                class_stack.pop();
            }

            if let Some(caps) = re_class.captures(trimmed) {
                let name = caps.get(2).map(|m| m.as_str()).unwrap_or("unknown").to_string();
                symbols.push(DocSymbol {
                    id: Self::id(path, line_no, &name, &DocSymbolKind::Class),
                    language: "javascript".to_string(),
                    kind: DocSymbolKind::Class,
                    name: name.clone(),
                    qualified_name: name.clone(),
                    signature: Some(trimmed.to_string()),
                    visibility: if caps.get(1).is_some() {
                        DocVisibility::Public
                    } else {
                        DocVisibility::Private
                    },
                    source_path: path.to_path_buf(),
                    line: line_no,
                    docs: DocComment::default(),
                    examples: Vec::new(),
                    gaps: Vec::new(),
                    parent: None,
                });
                class_stack.push((name, depth + 1));
            } else if let Some(caps) = re_function.captures(trimmed) {
                let name = caps.get(2).map(|m| m.as_str()).unwrap_or("unknown").to_string();
                let args = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                symbols.push(DocSymbol {
                    id: Self::id(path, line_no, &name, &DocSymbolKind::Function),
                    language: "javascript".to_string(),
                    kind: DocSymbolKind::Function,
                    name: name.clone(),
                    qualified_name: name,
                    signature: Some(format!("function({})", args)),
                    visibility: if caps.get(1).is_some() {
                        DocVisibility::Public
                    } else {
                        DocVisibility::Private
                    },
                    source_path: path.to_path_buf(),
                    line: line_no,
                    docs: DocComment::default(),
                    examples: Vec::new(),
                    gaps: Vec::new(),
                    parent: None,
                });
            } else if let Some(caps) = re_method.captures(trimmed) {
                if let Some((class_name, _)) = class_stack.last() {
                    let name = caps.get(1).map(|m| m.as_str()).unwrap_or("unknown").to_string();
                    let args = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                    if name != "if" && name != "for" && name != "while" && name != "switch" {
                        let qualified = format!("{}.{}", class_name, name);
                        symbols.push(DocSymbol {
                            id: Self::id(path, line_no, &qualified, &DocSymbolKind::Method),
                            language: "javascript".to_string(),
                            kind: DocSymbolKind::Method,
                            name,
                            qualified_name: qualified,
                            signature: Some(format!("method({})", args)),
                            visibility: DocVisibility::Public,
                            source_path: path.to_path_buf(),
                            line: line_no,
                            docs: DocComment::default(),
                            examples: Vec::new(),
                            gaps: Vec::new(),
                            parent: Some(class_name.clone()),
                        });
                    }
                }
            }

            depth += line.chars().filter(|ch| *ch == '{').count() as i32;
            depth -= line.chars().filter(|ch| *ch == '}').count() as i32;
        }

        Ok(symbols)
    }

    fn extract_inline_docs(
        &self,
        source: &str,
        _path: &Path,
    ) -> Result<Vec<DocCommentBlock>, DocgenError> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        let mut idx = 0usize;

        while idx < lines.len() {
            let trimmed = lines[idx].trim();
            if trimmed.starts_with("/**") {
                let start = idx + 1;
                let mut content = Vec::new();
                let mut end = start;
                while idx < lines.len() {
                    let line = lines[idx].trim();
                    let cleaned = line
                        .trim_start_matches("/**")
                        .trim_start_matches("*/")
                        .trim_start_matches('*')
                        .trim()
                        .to_string();
                    if !cleaned.is_empty() {
                        content.push(cleaned);
                    }
                    if line.contains("*/") {
                        end = idx + 1;
                        break;
                    }
                    idx += 1;
                }
                blocks.push(DocCommentBlock {
                    start_line: start,
                    end_line: end,
                    target_line_hint: next_nonempty_line(source, end),
                    lines: content,
                });
            }
            idx += 1;
        }

        Ok(blocks)
    }

    fn attach_docs(&self, symbols: Vec<DocSymbol>, docs: Vec<DocCommentBlock>) -> Vec<DocSymbol> {
        attach_docs_by_proximity(symbols, docs)
    }
}
