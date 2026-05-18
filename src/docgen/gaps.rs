use crate::docgen::model::{DocGap, DocGapKind, DocProject, DocSymbol, DocVisibility};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkValidationOptions {
    pub validate_local_anchors: bool,
    pub validate_external_links: bool,
    pub external_link_timeout_ms: u64,
    pub external_link_allowlist: BTreeSet<String>,
}

impl Default for LinkValidationOptions {
    fn default() -> Self {
        Self {
            validate_local_anchors: false,
            validate_external_links: false,
            external_link_timeout_ms: 1500,
            external_link_allowlist: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrokenLinkKind {
    LocalFileMissing,
    LocalAnchorMissing,
    ExternalUnreachable,
    ExternalRedirectDisallowed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenLinkFinding {
    pub symbol: String,
    pub target: String,
    pub line: usize,
    pub kind: BrokenLinkKind,
}

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

pub fn detect_broken_doc_links(
    root: &Path,
    project: &DocProject,
    options: LinkValidationOptions,
) -> Vec<BrokenLinkFinding> {
    let mut broken = Vec::new();
    for symbol in &project.symbols {
        for (idx, line) in symbol.docs.lines.iter().enumerate() {
            for link in extract_markdown_links(line) {
                if link.starts_with("mailto:") {
                    continue;
                }
                if link.starts_with("http://") || link.starts_with("https://") {
                    if !options.validate_external_links
                        || options.external_link_allowlist.is_empty()
                    {
                        continue;
                    }

                    if !external_link_in_allowlist(&link, &options.external_link_allowlist) {
                        continue;
                    }

                    match external_link_check(
                        &link,
                        &options.external_link_allowlist,
                        options.external_link_timeout_ms,
                    ) {
                        ExternalLinkCheck::Reachable => {}
                        ExternalLinkCheck::Unreachable => {
                            broken.push(BrokenLinkFinding {
                                symbol: symbol.qualified_name.clone(),
                                target: link.clone(),
                                line: idx + 1,
                                kind: BrokenLinkKind::ExternalUnreachable,
                            });
                        }
                        ExternalLinkCheck::RedirectDisallowed { next_url, blocked_host } => {
                            broken.push(BrokenLinkFinding {
                                symbol: symbol.qualified_name.clone(),
                                target: format!(
                                    "{} (redirected to non-allowlisted host '{}' via '{}')",
                                    link, blocked_host, next_url
                                ),
                                line: idx + 1,
                                kind: BrokenLinkKind::ExternalRedirectDisallowed,
                            });
                        }
                    }
                    continue;
                }

                let fragment = link.split_once('#').map(|(_, anchor)| anchor).unwrap_or_default();
                let link_without_fragment = link.split('#').next().unwrap_or_default();
                let link_without_query =
                    link_without_fragment.split('?').next().unwrap_or_default();
                if link_without_query.is_empty() {
                    continue;
                }

                let target = root.join(link_without_query);
                if !target.exists() {
                    broken.push(BrokenLinkFinding {
                        symbol: symbol.qualified_name.clone(),
                        target: target.display().to_string(),
                        line: idx + 1,
                        kind: BrokenLinkKind::LocalFileMissing,
                    });
                    continue;
                }

                if options.validate_local_anchors
                    && !fragment.is_empty()
                    && !local_anchor_exists(&target, fragment)
                {
                    broken.push(BrokenLinkFinding {
                        symbol: symbol.qualified_name.clone(),
                        target: format!("{}#{}", target.display(), fragment),
                        line: idx + 1,
                        kind: BrokenLinkKind::LocalAnchorMissing,
                    });
                }
            }
        }
    }
    broken
}

fn local_anchor_exists(target: &Path, fragment: &str) -> bool {
    let anchor = normalize_anchor(fragment);
    if anchor.is_empty() {
        return true;
    }

    let Ok(content) = std::fs::read_to_string(target) else {
        return false;
    };

    if content.contains(&format!("id=\"{}\"", anchor))
        || content.contains(&format!("name=\"{}\"", anchor))
    {
        return true;
    }

    for line in content.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }
        let heading = trimmed.trim_start_matches('#').trim();
        if markdown_anchor_slug(heading) == anchor {
            return true;
        }
    }

    false
}

fn normalize_anchor(anchor: &str) -> String {
    anchor.trim().trim_start_matches('#').trim().to_ascii_lowercase()
}

fn markdown_anchor_slug(heading: &str) -> String {
    let mut out = String::new();
    let mut last_was_dash = false;

    for ch in heading.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            out.push('-');
            last_was_dash = true;
        }
    }

    out.trim_matches('-').to_string()
}

fn external_link_in_allowlist(link: &str, allowlist: &BTreeSet<String>) -> bool {
    let Ok(url) = reqwest::Url::parse(link) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    allowlist.contains(&host.to_ascii_lowercase())
}

enum ExternalLinkCheck {
    Reachable,
    Unreachable,
    RedirectDisallowed { next_url: String, blocked_host: String },
}

fn external_link_check(
    link: &str,
    allowlist: &BTreeSet<String>,
    timeout_ms: u64,
) -> ExternalLinkCheck {
    let client = match reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_millis(timeout_ms.max(1)))
        .build()
    {
        Ok(client) => client,
        Err(_) => return ExternalLinkCheck::Unreachable,
    };

    let mut current_url = match reqwest::Url::parse(link) {
        Ok(url) => url,
        Err(_) => return ExternalLinkCheck::Unreachable,
    };

    for _ in 0..10usize {
        let response = match client.get(current_url.clone()).send() {
            Ok(response) => response,
            Err(_) => return ExternalLinkCheck::Unreachable,
        };
        let status = response.status();
        if status.is_success() {
            return ExternalLinkCheck::Reachable;
        }
        if !status.is_redirection() {
            return ExternalLinkCheck::Unreachable;
        }

        let Some(location) = response.headers().get(reqwest::header::LOCATION) else {
            return ExternalLinkCheck::Unreachable;
        };
        let Ok(location) = location.to_str() else {
            return ExternalLinkCheck::Unreachable;
        };
        let next_url = match current_url.join(location) {
            Ok(url) => url,
            Err(_) => return ExternalLinkCheck::Unreachable,
        };
        let Some(host) = next_url.host_str() else {
            return ExternalLinkCheck::Unreachable;
        };
        let blocked_host = host.to_ascii_lowercase();
        if !allowlist.contains(&blocked_host) {
            return ExternalLinkCheck::RedirectDisallowed {
                next_url: next_url.to_string(),
                blocked_host,
            };
        }
        current_url = next_url;
    }

    ExternalLinkCheck::Unreachable
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
