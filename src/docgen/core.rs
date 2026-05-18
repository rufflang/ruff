use crate::docgen::adapters::{adapter_for_language, capability_index, language_ids};
use crate::docgen::discovery::{
    discover, parse_language_filter, validate_languages, DiscoveryOptions,
};
use crate::docgen::gaps::{
    build_gaps, detect_broken_doc_links, BrokenLinkKind, LinkValidationOptions,
};
use crate::docgen::model::{
    DocComment, DocDiagnostic, DocDiagnosticSeverity, DocModule, DocProject, DocSymbol,
    DocSymbolKind, DocVisibility,
};
use crate::docgen::render;
use crate::docgen::DocgenError;
use crate::interpreter::Interpreter;
use crate::path_security::{canonicalize_root, ensure_path_within_root};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocOutputFormat {
    Html,
    Markdown,
    Json,
    All,
}

#[derive(Debug, Clone)]
pub struct DocgenConfig {
    pub input: PathBuf,
    pub out_dir: PathBuf,
    pub format: DocOutputFormat,
    pub include_builtins: bool,
    pub language: Option<String>,
    pub languages: Option<String>,
    pub emit_ai_tasks: bool,
    pub search_index: bool,
    pub source_links: bool,
    pub fail_on_undocumented: bool,
    pub fail_on_broken_links: bool,
    pub fail_on_warnings: bool,
    pub public_only: bool,
    pub include_private: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocgenRunSummary {
    pub output_dir: PathBuf,
    pub module_doc_path: PathBuf,
    pub builtin_doc_path: Option<PathBuf>,
    pub item_count: usize,
    pub project_symbol_count: usize,
    pub builtin_symbol_count: usize,
    pub symbol_kind_counts: BTreeMap<String, usize>,
    pub project_json_path: PathBuf,
    pub gaps_json_path: PathBuf,
    pub capabilities_json_path: PathBuf,
    pub ai_tasks_path: Option<PathBuf>,
    pub languages: Vec<String>,
    pub diagnostics_count: usize,
    pub undocumented_count: usize,
    pub broken_link_count: usize,
    pub warning_count: usize,
    pub discovery_skip_counts: BTreeMap<String, usize>,
    pub gate_failures: Vec<String>,
}

pub fn run(config: &DocgenConfig) -> Result<(DocProject, DocgenRunSummary), DocgenError> {
    run_with_link_validation(config, LinkValidationOptions::default())
}

pub fn run_with_link_validation(
    config: &DocgenConfig,
    link_validation: LinkValidationOptions,
) -> Result<(DocProject, DocgenRunSummary), DocgenError> {
    let selected_languages =
        parse_language_filter(config.language.as_deref(), config.languages.as_deref())?;
    if let Some(ref languages) = selected_languages {
        validate_languages(languages)?;
    }

    let discovery = discover(
        &config.input,
        &DiscoveryOptions {
            selected_languages,
            max_file_size_bytes: 2 * 1024 * 1024,
            max_files: 20_000,
            max_depth: 64,
        },
    )?;

    let mut modules = Vec::new();
    let mut symbols = Vec::new();
    let mut diagnostics: Vec<DocDiagnostic> = discovery
        .diagnostics
        .iter()
        .map(|diag| DocDiagnostic {
            severity: DocDiagnosticSeverity::Warning,
            code: diag.code.to_string(),
            message: diag.message.clone(),
            path: diag.path.clone(),
            line: None,
        })
        .collect();
    let mut source_map: BTreeMap<String, String> = BTreeMap::new();

    for file in &discovery.files {
        let Some(adapter) = adapter_for_language(&file.language) else {
            diagnostics.push(DocDiagnostic {
                severity: DocDiagnosticSeverity::Warning,
                code: "DOCGEN001".to_string(),
                message: format!("no adapter found for language '{}'", file.language),
                path: Some(file.relative_path.clone()),
                line: None,
            });
            continue;
        };

        source_map.insert(file.relative_path.display().to_string(), file.source.clone());

        let raw_symbols = adapter.extract_symbols(&file.source, &file.absolute_path)?;
        let docs = adapter.extract_inline_docs(&file.source, &file.absolute_path)?;
        let mut attached = adapter.attach_docs(raw_symbols, docs);

        for symbol in &mut attached {
            symbol.source_path = file.relative_path.clone();
        }

        modules.push(DocModule {
            name: file
                .relative_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("module")
                .to_string(),
            language: file.language.clone(),
            path: file.relative_path.clone(),
            symbols: attached.iter().map(|symbol| symbol.id.clone()).collect(),
        });

        symbols.extend(attached);
    }

    if config.include_builtins {
        add_ruff_builtins(&mut symbols);
    }

    if config.public_only && !config.include_private {
        symbols.retain(|symbol| symbol.visibility == DocVisibility::Public);
    }

    symbols.sort_by(|a, b| {
        a.language
            .cmp(&b.language)
            .then(a.source_path.cmp(&b.source_path))
            .then(a.line.cmp(&b.line))
            .then(a.qualified_name.cmp(&b.qualified_name))
    });

    let mut language_set: BTreeSet<String> = discovery.detected_languages.iter().cloned().collect();
    for symbol in &symbols {
        language_set.insert(symbol.language.clone());
    }

    let mut project = DocProject {
        name: config.input.file_name().and_then(|f| f.to_str()).map(ToOwned::to_owned),
        root: discovery.root.clone(),
        languages: language_set.into_iter().collect(),
        modules,
        symbols,
        gaps: Vec::new(),
        diagnostics,
    };

    build_gaps(&mut project, &source_map);

    if link_validation.validate_external_links && link_validation.external_link_allowlist.is_empty()
    {
        project.diagnostics.push(DocDiagnostic {
            severity: DocDiagnosticSeverity::Warning,
            code: "DOCGEN_LINK_EXTERNAL_ALLOWLIST_EMPTY".to_string(),
            message: "external link validation is enabled, but no allowlisted hosts were provided; external links were skipped".to_string(),
            path: None,
            line: None,
        });
    }
    if !link_validation.validate_external_links
        && !link_validation.external_link_allowlist.is_empty()
    {
        project.diagnostics.push(DocDiagnostic {
            severity: DocDiagnosticSeverity::Warning,
            code: "DOCGEN_LINK_EXTERNAL_ALLOWLIST_IGNORED".to_string(),
            message: "external link allowlist was provided without --validate-external-links; external links were not validated".to_string(),
            path: None,
            line: None,
        });
    }

    let broken_links = detect_broken_doc_links(&project.root, &project, link_validation.clone());
    for broken_link in &broken_links {
        let (code, mode_label) = match broken_link.kind {
            BrokenLinkKind::LocalFileMissing => ("DOCGEN_LINK_BROKEN_LOCAL_FILE", "local-file"),
            BrokenLinkKind::LocalAnchorMissing => {
                ("DOCGEN_LINK_BROKEN_LOCAL_ANCHOR", "local-anchor")
            }
            BrokenLinkKind::ExternalUnreachable => ("DOCGEN_LINK_BROKEN_EXTERNAL", "external"),
        };
        project.diagnostics.push(DocDiagnostic {
            severity: DocDiagnosticSeverity::Warning,
            code: code.to_string(),
            message: format!(
                "broken {} doc link '{}' in symbol '{}'",
                mode_label, broken_link.target, broken_link.symbol
            ),
            path: None,
            line: Some(broken_link.line),
        });
    }
    project.diagnostics.sort_by(|a, b| {
        diagnostic_severity_rank(&a.severity)
            .cmp(&diagnostic_severity_rank(&b.severity))
            .then(a.code.cmp(&b.code))
            .then(a.path.cmp(&b.path))
            .then(a.line.cmp(&b.line))
            .then(a.message.cmp(&b.message))
    });

    let output_dir = prepare_output_dir(&config.out_dir)?;
    fs::create_dir_all(&output_dir).map_err(|e| {
        DocgenError::new(format!(
            "failed to create output directory '{}': {}",
            output_dir.display(),
            e
        ))
    })?;

    let module_doc_path = output_dir.join("index.html");
    let builtin_doc_path = if config.include_builtins {
        let path = output_dir.join("builtins.html");
        write_file(&path, &render_builtins_html())?;
        Some(path)
    } else {
        None
    };

    write_outputs(config, &project, &output_dir)?;

    let gaps_json_path = output_dir.join("docgen-gaps.json");
    let gaps_json = serde_json::to_string_pretty(&project.gaps)
        .map_err(|e| DocgenError::new(format!("failed to serialize docgen gaps: {}", e)))?;
    write_file(&gaps_json_path, &gaps_json)?;

    let ai_tasks_path = if config.emit_ai_tasks {
        let path = output_dir.join("docgen-ai-tasks.md");
        write_file(&path, &render_ai_tasks(&project))?;
        Some(path)
    } else {
        None
    };

    let capabilities_json_path = output_dir.join("docgen-capabilities.json");
    write_file(&capabilities_json_path, &render_capabilities_json()?)?;

    if config.search_index {
        let search_index_path = output_dir.join("search-index.json");
        write_file(&search_index_path, &render_search_index(&project)?)?;

        let symbol_index_path = output_dir.join("symbol-index.json");
        write_file(&symbol_index_path, &render_symbol_index(&project)?)?;
    }

    let undocumented_count = project
        .gaps
        .iter()
        .filter(|gap| {
            gap.missing_sections
                .iter()
                .any(|entry| matches!(entry, crate::docgen::model::DocGapKind::MissingDocs))
        })
        .count();
    let warning_count = project
        .diagnostics
        .iter()
        .filter(|diag| diag.severity == DocDiagnosticSeverity::Warning)
        .count();

    let mut gate_failures = Vec::new();
    if config.fail_on_undocumented && undocumented_count > 0 {
        gate_failures.push(format!("{} undocumented public symbols detected", undocumented_count));
    }
    if config.fail_on_broken_links && !broken_links.is_empty() {
        let mut local_file_missing = 0usize;
        let mut local_anchor_missing = 0usize;
        let mut external_unreachable = 0usize;
        for broken_link in &broken_links {
            match broken_link.kind {
                BrokenLinkKind::LocalFileMissing => local_file_missing += 1,
                BrokenLinkKind::LocalAnchorMissing => local_anchor_missing += 1,
                BrokenLinkKind::ExternalUnreachable => external_unreachable += 1,
            }
        }
        gate_failures.push(format!(
            "{} broken links detected (local_file={}, local_anchor={}, external={})",
            broken_links.len(),
            local_file_missing,
            local_anchor_missing,
            external_unreachable
        ));
    }
    if config.fail_on_warnings && warning_count > 0 {
        gate_failures.push(format!("{} warnings detected", warning_count));
    }

    let project_json_path = output_dir.join("docgen.json");

    let item_count = project.symbols.len();
    let builtin_symbol_count = project
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == DocSymbolKind::Builtin)
        .count();
    let project_symbol_count = item_count.saturating_sub(builtin_symbol_count);
    let mut symbol_kind_counts: BTreeMap<String, usize> = BTreeMap::new();
    for symbol in &project.symbols {
        let key = doc_symbol_kind_key(&symbol.kind).to_string();
        *symbol_kind_counts.entry(key).or_insert(0) += 1;
    }
    let languages = project.languages.clone();
    let diagnostics_count = project.diagnostics.len();
    let broken_link_count = broken_links.len();
    let discovery_skip_counts = BTreeMap::from([
        ("max_depth".to_string(), discovery.skip_counts.max_depth),
        ("max_file_size".to_string(), discovery.skip_counts.max_file_size),
        ("max_files".to_string(), discovery.skip_counts.max_files),
    ]);

    Ok((
        project,
        DocgenRunSummary {
            output_dir,
            module_doc_path,
            builtin_doc_path,
            item_count,
            project_symbol_count,
            builtin_symbol_count,
            symbol_kind_counts,
            project_json_path,
            gaps_json_path,
            capabilities_json_path,
            ai_tasks_path,
            languages,
            diagnostics_count,
            undocumented_count,
            broken_link_count,
            warning_count,
            discovery_skip_counts,
            gate_failures,
        },
    ))
}

fn diagnostic_severity_rank(severity: &DocDiagnosticSeverity) -> u8 {
    match severity {
        DocDiagnosticSeverity::Info => 0,
        DocDiagnosticSeverity::Warning => 1,
        DocDiagnosticSeverity::Error => 2,
    }
}

fn doc_symbol_kind_key(kind: &DocSymbolKind) -> &'static str {
    match kind {
        DocSymbolKind::Module => "module",
        DocSymbolKind::Function => "function",
        DocSymbolKind::Method => "method",
        DocSymbolKind::Class => "class",
        DocSymbolKind::Struct => "struct",
        DocSymbolKind::Enum => "enum",
        DocSymbolKind::EnumVariant => "enum_variant",
        DocSymbolKind::Interface => "interface",
        DocSymbolKind::Trait => "trait",
        DocSymbolKind::TypeAlias => "type_alias",
        DocSymbolKind::Constant => "constant",
        DocSymbolKind::Variable => "variable",
        DocSymbolKind::Property => "property",
        DocSymbolKind::Builtin => "builtin",
        DocSymbolKind::Unknown => "unknown",
    }
}

fn write_outputs(
    config: &DocgenConfig,
    project: &DocProject,
    out_dir: &Path,
) -> Result<(), DocgenError> {
    let html_path = out_dir.join("index.html");
    let markdown_path = out_dir.join("docgen.md");
    let json_path = out_dir.join("docgen.json");

    match config.format {
        DocOutputFormat::Html => {
            write_file(&html_path, &render::html::render(project, config.source_links))?;
            write_file(&json_path, &render::json::render(project).map_err(DocgenError::new)?)?;
        }
        DocOutputFormat::Markdown => {
            write_file(&markdown_path, &render::markdown::render(project))?;
            write_file(&json_path, &render::json::render(project).map_err(DocgenError::new)?)?;
            write_file(&html_path, &render::html::render(project, config.source_links))?;
        }
        DocOutputFormat::Json => {
            write_file(&json_path, &render::json::render(project).map_err(DocgenError::new)?)?;
            write_file(&html_path, &render::html::render(project, config.source_links))?;
        }
        DocOutputFormat::All => {
            write_file(&html_path, &render::html::render(project, config.source_links))?;
            write_file(&markdown_path, &render::markdown::render(project))?;
            write_file(&json_path, &render::json::render(project).map_err(DocgenError::new)?)?;
        }
    }

    Ok(())
}

fn write_file(path: &Path, content: &str) -> Result<(), DocgenError> {
    fs::write(path, content)
        .map_err(|e| DocgenError::new(format!("failed to write '{}': {}", path.display(), e)))
}

fn prepare_output_dir(path: &Path) -> Result<PathBuf, DocgenError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| DocgenError::new(format!("failed to resolve cwd: {}", e)))?
            .join(path)
    };

    let parent = absolute
        .parent()
        .ok_or_else(|| DocgenError::new("output directory has no parent".to_string()))?;
    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|e| {
            DocgenError::new(format!(
                "failed to create output parent '{}': {}",
                parent.display(),
                e
            ))
        })?;
    }

    let canonical_parent =
        canonicalize_root(parent, "docgen output parent").map_err(DocgenError::new)?;
    let candidate = canonical_parent.join(
        absolute
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| DocgenError::new("invalid output directory name".to_string()))?,
    );
    ensure_path_within_root(&candidate, &canonical_parent, "docgen output directory")
        .map_err(DocgenError::new)?;
    Ok(candidate)
}

fn add_ruff_builtins(symbols: &mut Vec<DocSymbol>) {
    let mut names: Vec<&str> = Interpreter::get_builtin_names().to_vec();
    names.sort_unstable();
    for name in names {
        symbols.push(DocSymbol {
            id: format!("ruff:builtin:{}", name),
            language: "ruff".to_string(),
            kind: DocSymbolKind::Builtin,
            name: name.to_string(),
            qualified_name: format!("builtin::{}", name),
            signature: None,
            visibility: DocVisibility::Public,
            source_path: PathBuf::from("<builtins>"),
            line: 0,
            docs: DocComment {
                lines: vec!["Ruff builtin/native API.".to_string()],
                summary: Some("Ruff builtin/native API.".to_string()),
                placeholder: false,
            },
            examples: Vec::new(),
            gaps: Vec::new(),
            parent: Some("builtins".to_string()),
        });
    }
}

fn render_builtins_html() -> String {
    let mut names: Vec<&str> = Interpreter::get_builtin_names().to_vec();
    names.sort_unstable();
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Ruff Builtins</title></head><body>");
    html.push_str("<h1>Ruff Builtin API Reference</h1><ul>");
    for name in names {
        html.push_str(&format!("<li><code>{}</code></li>", name));
    }
    html.push_str("</ul></body></html>");
    html
}

fn render_ai_tasks(project: &DocProject) -> String {
    let mut out = String::new();
    out.push_str("# DocGen AI Tasks\n\n");
    for gap in &project.gaps {
        out.push_str(&format!("## {} ({:?})\n\n", gap.symbol_name, gap.symbol_kind));
        out.push_str(&format!("- Language: {}\n", gap.language));
        out.push_str(&format!("- File: `{}`:{}\n", gap.source_path.display(), gap.line));
        if let Some(signature) = &gap.signature {
            out.push_str(&format!("- Signature: `{}`\n", signature));
        }
        out.push_str(&format!("- Missing sections: {:?}\n\n", gap.missing_sections));
        out.push_str("### Existing docs\n\n");
        if gap.existing_docs.is_empty() {
            out.push_str("_none_\n\n");
        } else {
            for line in &gap.existing_docs {
                out.push_str(&format!("- {}\n", line));
            }
            out.push('\n');
        }
        out.push_str("### Source context\n\n```text\n");
        for line in &gap.bounded_source_context {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str("```\n\n");
        out.push_str("### Suggested task prompt\n\n");
        out.push_str(&gap.suggested_ai_prompt);
        out.push_str("\n\n");
    }
    out
}

fn render_search_index(project: &DocProject) -> Result<String, DocgenError> {
    #[derive(Serialize)]
    struct SearchEntry {
        language: String,
        name: String,
        kind: String,
        path: String,
        line: usize,
        summary: Option<String>,
    }

    let entries: Vec<SearchEntry> = project
        .symbols
        .iter()
        .map(|symbol| SearchEntry {
            language: symbol.language.clone(),
            name: symbol.qualified_name.clone(),
            kind: format!("{:?}", symbol.kind),
            path: symbol.source_path.display().to_string(),
            line: symbol.line,
            summary: symbol.docs.summary.clone(),
        })
        .collect();

    serde_json::to_string_pretty(&entries)
        .map_err(|e| DocgenError::new(format!("failed to serialize search index: {}", e)))
}

fn render_symbol_index(project: &DocProject) -> Result<String, DocgenError> {
    let mut index: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for symbol in &project.symbols {
        index.entry(symbol.language.clone()).or_default().push(symbol.qualified_name.clone());
    }
    for names in index.values_mut() {
        names.sort();
        names.dedup();
    }

    serde_json::to_string_pretty(&index)
        .map_err(|e| DocgenError::new(format!("failed to serialize symbol index: {}", e)))
}

fn render_capabilities_json() -> Result<String, DocgenError> {
    let payload = capability_index();
    serde_json::to_string_pretty(&payload)
        .map_err(|e| DocgenError::new(format!("failed to serialize adapter capabilities: {}", e)))
}

pub fn parse_output_format(value: Option<&str>) -> Result<DocOutputFormat, DocgenError> {
    match value.unwrap_or("html").to_ascii_lowercase().as_str() {
        "html" => Ok(DocOutputFormat::Html),
        "markdown" | "md" => Ok(DocOutputFormat::Markdown),
        "json" => Ok(DocOutputFormat::Json),
        "all" => Ok(DocOutputFormat::All),
        other => Err(DocgenError::new(format!(
            "unsupported docgen format '{}' (supported: html, markdown, json, all)",
            other
        ))),
    }
}

#[allow(dead_code)]
pub fn supported_languages() -> Vec<&'static str> {
    language_ids()
}
