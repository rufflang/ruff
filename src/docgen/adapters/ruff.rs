use super::common::{attach_docs_by_proximity, next_nonempty_line};
use super::{AdapterCapability, DocLanguageAdapter};
use crate::docgen::model::{DocComment, DocCommentBlock, DocSymbol, DocSymbolKind, DocVisibility};
use crate::docgen::DocgenError;
use regex::Regex;
use std::path::Path;

pub struct RuffDocAdapter;

impl RuffDocAdapter {
    fn symbol_id(path: &Path, line: usize, name: &str, kind: &DocSymbolKind) -> String {
        format!("ruff:{}:{}:{}:{:?}", path.display(), line, name, kind)
    }
}

impl DocLanguageAdapter for RuffDocAdapter {
    fn language_id(&self) -> &'static str {
        "ruff"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["ruff"]
    }

    fn capabilities(&self) -> AdapterCapability {
        AdapterCapability {
            supports_functions: true,
            supports_types: true,
            supports_methods: true,
            supports_inline_docs: true,
        }
    }

    fn extract_symbols(&self, source: &str, path: &Path) -> Result<Vec<DocSymbol>, DocgenError> {
        let re_func =
            Regex::new(r"^\s*(pub\s+)?(async\s+)?func\*?\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)")
            .expect("valid ruff function regex");
        let re_struct = Regex::new(r"^\s*(pub\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)")
            .expect("valid ruff struct regex");
        let re_enum = Regex::new(r"^\s*(pub\s+)?enum\s+([A-Za-z_][A-Za-z0-9_]*)")
            .expect("valid ruff enum regex");
        let re_const = Regex::new(r"^\s*(pub\s+)?(const|let)\s+([A-Za-z_][A-Za-z0-9_]*)\s*[:=]")
            .expect("valid ruff const regex");
        let re_variant = Regex::new(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*,?\s*$")
            .expect("valid ruff enum variant regex");

        let mut symbols = Vec::new();
        let mut brace_depth: i32 = 0;
        let mut active_struct: Option<(String, i32, bool)> = None;
        let mut active_enum: Option<(String, i32, bool)> = None;

        for (idx, line) in source.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim();

            if let Some((_, end_depth, _)) = active_struct.clone() {
                if brace_depth < end_depth {
                    active_struct = None;
                }
            }
            if let Some((_, end_depth, _)) = active_enum.clone() {
                if brace_depth < end_depth {
                    active_enum = None;
                }
            }

            if let Some(caps) = re_struct.captures(trimmed) {
                let name = caps.get(2).map(|m| m.as_str()).unwrap_or("unknown").to_string();
                let visibility = if caps.get(1).is_some() {
                    DocVisibility::Public
                } else {
                    DocVisibility::Private
                };
                let is_public = visibility == DocVisibility::Public;
                symbols.push(DocSymbol {
                    id: Self::symbol_id(path, line_no, &name, &DocSymbolKind::Struct),
                    language: "ruff".to_string(),
                    kind: DocSymbolKind::Struct,
                    name: name.clone(),
                    qualified_name: name.clone(),
                    signature: Some(trimmed.to_string()),
                    visibility,
                    source_path: path.to_path_buf(),
                    line: line_no,
                    docs: DocComment::default(),
                    examples: Vec::new(),
                    gaps: Vec::new(),
                    parent: None,
                });
                if line.contains('{') {
                    active_struct = Some((name, brace_depth + 1, is_public));
                }
            } else if let Some(caps) = re_enum.captures(trimmed) {
                let name = caps.get(2).map(|m| m.as_str()).unwrap_or("unknown").to_string();
                let visibility = if caps.get(1).is_some() {
                    DocVisibility::Public
                } else {
                    DocVisibility::Private
                };
                let is_public = visibility == DocVisibility::Public;
                symbols.push(DocSymbol {
                    id: Self::symbol_id(path, line_no, &name, &DocSymbolKind::Enum),
                    language: "ruff".to_string(),
                    kind: DocSymbolKind::Enum,
                    name: name.clone(),
                    qualified_name: name.clone(),
                    signature: Some(trimmed.to_string()),
                    visibility,
                    source_path: path.to_path_buf(),
                    line: line_no,
                    docs: DocComment::default(),
                    examples: Vec::new(),
                    gaps: Vec::new(),
                    parent: None,
                });
                if line.contains('{') {
                    active_enum = Some((name, brace_depth + 1, is_public));
                }
            } else if let Some(caps) = re_func.captures(trimmed) {
                let name = caps.get(3).map(|m| m.as_str()).unwrap_or("unknown").to_string();
                let args = caps.get(4).map(|m| m.as_str()).unwrap_or("");
                let is_method = active_struct.is_some();
                let kind = if is_method { DocSymbolKind::Method } else { DocSymbolKind::Function };
                let parent = active_struct.as_ref().map(|(name, _, _)| name.clone());
                let is_async = caps.get(2).is_some();
                let explicit_public = caps.get(1).is_some();
                let visibility = if explicit_public {
                    DocVisibility::Public
                } else {
                    DocVisibility::Private
                };
                let visibility = if let Some((_, _, parent_public)) = &active_struct {
                    if explicit_public && *parent_public {
                        DocVisibility::Public
                    } else {
                        DocVisibility::Private
                    }
                } else {
                    visibility
                };
                let qualified_name = if let Some(parent_name) = &parent {
                    format!("{}::{}", parent_name, name)
                } else {
                    name.clone()
                };
                symbols.push(DocSymbol {
                    id: Self::symbol_id(path, line_no, &qualified_name, &kind),
                    language: "ruff".to_string(),
                    kind,
                    name,
                    qualified_name,
                    signature: Some(if is_async {
                        format!("async func({})", args)
                    } else {
                        format!("func({})", args)
                    }),
                    visibility,
                    source_path: path.to_path_buf(),
                    line: line_no,
                    docs: DocComment::default(),
                    examples: Vec::new(),
                    gaps: Vec::new(),
                    parent,
                });
            } else if let Some(caps) = re_const.captures(trimmed) {
                let name = caps.get(3).map(|m| m.as_str()).unwrap_or("unknown").to_string();
                let visibility = if caps.get(1).is_some() {
                    DocVisibility::Public
                } else {
                    DocVisibility::Private
                };
                symbols.push(DocSymbol {
                    id: Self::symbol_id(path, line_no, &name, &DocSymbolKind::Constant),
                    language: "ruff".to_string(),
                    kind: DocSymbolKind::Constant,
                    name: name.clone(),
                    qualified_name: name,
                    signature: Some(trimmed.to_string()),
                    visibility,
                    source_path: path.to_path_buf(),
                    line: line_no,
                    docs: DocComment::default(),
                    examples: Vec::new(),
                    gaps: Vec::new(),
                    parent: None,
                });
            } else if let Some((enum_name, _, enum_public)) = active_enum.clone() {
                if re_variant.is_match(trimmed)
                    && !trimmed.starts_with('#')
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("/*")
                    && !trimmed.starts_with('}')
                {
                    let variant_name = trimmed.trim_end_matches(',').trim().to_string();
                    let qualified = format!("{}::{}", enum_name, variant_name);
                    symbols.push(DocSymbol {
                        id: Self::symbol_id(path, line_no, &qualified, &DocSymbolKind::EnumVariant),
                        language: "ruff".to_string(),
                        kind: DocSymbolKind::EnumVariant,
                        name: variant_name,
                        qualified_name: qualified,
                        signature: Some(trimmed.to_string()),
                        visibility: if enum_public {
                            DocVisibility::Public
                        } else {
                            DocVisibility::Private
                        },
                        source_path: path.to_path_buf(),
                        line: line_no,
                        docs: DocComment::default(),
                        examples: Vec::new(),
                        gaps: Vec::new(),
                        parent: Some(enum_name),
                    });
                }
            }

            let opens = line.chars().filter(|ch| *ch == '{').count() as i32;
            let closes = line.chars().filter(|ch| *ch == '}').count() as i32;
            brace_depth += opens;
            brace_depth -= closes;
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

            if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                let start_line = idx + 1;
                let mut content = Vec::new();
                let mut end_line = start_line;

                while idx < lines.len() {
                    let candidate = lines[idx].trim();
                    if !(candidate.starts_with("///") || candidate.starts_with("//!")) {
                        break;
                    }
                    content.push(
                        candidate
                            .trim_start_matches("///")
                            .trim_start_matches("//!")
                            .trim_start()
                            .to_string(),
                    );
                    end_line = idx + 1;
                    idx += 1;
                }

                blocks.push(DocCommentBlock {
                    start_line,
                    end_line,
                    target_line_hint: next_nonempty_line(source, end_line),
                    lines: content,
                });
                continue;
            }

            if trimmed.starts_with("/**") {
                let start_line = idx + 1;
                let mut content = Vec::new();
                let mut end_line = start_line;

                while idx < lines.len() {
                    let candidate = lines[idx].trim();
                    let cleaned = candidate
                        .trim_start_matches("/**")
                        .trim_start_matches("*/")
                        .trim_start_matches('*')
                        .trim()
                        .to_string();
                    if !cleaned.is_empty() {
                        content.push(cleaned);
                    }
                    end_line = idx + 1;
                    if candidate.contains("*/") {
                        idx += 1;
                        break;
                    }
                    idx += 1;
                }

                blocks.push(DocCommentBlock {
                    start_line,
                    end_line: end_line.max(start_line),
                    target_line_hint: next_nonempty_line(source, end_line.max(start_line)),
                    lines: content,
                });
                continue;
            }

            idx += 1;
        }

        Ok(blocks)
    }

    fn attach_docs(&self, symbols: Vec<DocSymbol>, docs: Vec<DocCommentBlock>) -> Vec<DocSymbol> {
        attach_docs_by_proximity(symbols, docs)
    }
}
