use crate::docgen::model::{DocGap, DocGapKind, DocProject, DocSymbol, DocVisibility};
use std::collections::BTreeMap;
use std::path::Path;

pub fn build_gaps(project: &mut DocProject, source_map: &BTreeMap<String, String>) {
    let mut gaps = Vec::new();

    for symbol in &project.symbols {
        if symbol.visibility == DocVisibility::Private {
            continue;
        }

        if symbol.gaps.is_empty() {
            continue;
        }

        let key = symbol.source_path.display().to_string();
        let context = source_map
            .get(&key)
            .map(|source| bounded_context(source, symbol.line, 2))
            .unwrap_or_default();

        let call_sites = known_call_sites(source_map, &symbol.name, 6);

        gaps.push(DocGap {
            id: format!("gap:{}:{}:{}", symbol.language, symbol.qualified_name, symbol.line),
            language: symbol.language.clone(),
            symbol_id: symbol.id.clone(),
            symbol_name: symbol.qualified_name.clone(),
            symbol_kind: symbol.kind.clone(),
            signature: symbol.signature.clone(),
            source_path: symbol.source_path.clone(),
            line: symbol.line,
            missing_sections: symbol.gaps.clone(),
            existing_docs: symbol.docs.lines.clone(),
            bounded_source_context: context,
            known_call_sites: call_sites,
            suggested_ai_prompt: ai_prompt(symbol),
        });
    }

    gaps.sort_by(|a, b| {
        a.language
            .cmp(&b.language)
            .then(a.source_path.cmp(&b.source_path))
            .then(a.line.cmp(&b.line))
            .then(a.symbol_name.cmp(&b.symbol_name))
    });

    project.gaps = gaps;
}

fn bounded_context(source: &str, line: usize, radius: usize) -> Vec<String> {
    if line == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let start = line.saturating_sub(radius + 1) + 1;
    let end = line + radius;

    for (idx, text) in source.lines().enumerate() {
        let line_no = idx + 1;
        if line_no < start || line_no > end {
            continue;
        }
        out.push(format!("{}: {}", line_no, text));
    }

    out
}

fn known_call_sites(
    source_map: &BTreeMap<String, String>,
    symbol_name: &str,
    limit: usize,
) -> Vec<String> {
    if symbol_name.is_empty() {
        return Vec::new();
    }

    let mut calls = Vec::new();
    let needle = format!("{}(", symbol_name);
    for (path, source) in source_map {
        for (idx, line) in source.lines().enumerate() {
            if line.contains(&needle) {
                calls.push(format!("{}:{}: {}", path, idx + 1, line.trim()));
                if calls.len() >= limit {
                    return calls;
                }
            }
        }
    }

    calls
}

fn ai_prompt(symbol: &DocSymbol) -> String {
    let missing = if symbol.gaps.is_empty() {
        "none".to_string()
    } else {
        symbol.gaps.iter().map(gap_name).collect::<Vec<&'static str>>().join(", ")
    };

    format!(
        "Document this {} '{}' with missing sections: {}. Use only the provided source context. Do not invent behavior. Mark uncertainty. Keep docs concise. Prefer examples only when the source supports them.",
        kind_name(&symbol.kind),
        symbol.qualified_name,
        missing
    )
}

fn gap_name(kind: &DocGapKind) -> &'static str {
    match kind {
        DocGapKind::MissingSummary => "summary",
        DocGapKind::MissingExamples => "examples",
        DocGapKind::MissingDocs => "docs",
    }
}

fn kind_name(kind: &crate::docgen::model::DocSymbolKind) -> &'static str {
    match kind {
        crate::docgen::model::DocSymbolKind::Module => "module",
        crate::docgen::model::DocSymbolKind::Function => "function",
        crate::docgen::model::DocSymbolKind::Method => "method",
        crate::docgen::model::DocSymbolKind::Class => "class",
        crate::docgen::model::DocSymbolKind::Struct => "struct",
        crate::docgen::model::DocSymbolKind::Enum => "enum",
        crate::docgen::model::DocSymbolKind::EnumVariant => "enum variant",
        crate::docgen::model::DocSymbolKind::Interface => "interface",
        crate::docgen::model::DocSymbolKind::Trait => "trait",
        crate::docgen::model::DocSymbolKind::TypeAlias => "type alias",
        crate::docgen::model::DocSymbolKind::Constant => "constant",
        crate::docgen::model::DocSymbolKind::Variable => "variable",
        crate::docgen::model::DocSymbolKind::Property => "property",
        crate::docgen::model::DocSymbolKind::Builtin => "builtin",
        crate::docgen::model::DocSymbolKind::Unknown => "symbol",
    }
}

pub fn detect_broken_doc_links(root: &Path, project: &DocProject) -> Vec<(String, String, usize)> {
    let mut broken = Vec::new();
    for symbol in &project.symbols {
        for (idx, line) in symbol.docs.lines.iter().enumerate() {
            for link in extract_markdown_links(line) {
                if link.starts_with("http://")
                    || link.starts_with("https://")
                    || link.starts_with("mailto:")
                {
                    continue;
                }
                let target = root.join(link);
                if !target.exists() {
                    broken.push((
                        symbol.qualified_name.clone(),
                        target.display().to_string(),
                        idx + 1,
                    ));
                }
            }
        }
    }
    broken
}

fn extract_markdown_links(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("](") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find(')') {
            let link = &after[..end];
            out.push(link.to_string());
            rest = &after[end + 1..];
        } else {
            break;
        }
    }
    out
}
