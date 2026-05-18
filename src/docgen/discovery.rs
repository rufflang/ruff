use crate::docgen::adapters::{adapter_for_extension, adapter_for_language};
use crate::docgen::DocgenError;
use crate::path_security::{canonicalize_root, ensure_path_within_root};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    pub selected_languages: Option<Vec<String>>,
    pub max_file_size_bytes: u64,
    pub max_files: usize,
    pub max_depth: usize,
}

#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    pub language: String,
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    pub root: PathBuf,
    pub files: Vec<DiscoveredFile>,
    pub detected_languages: Vec<String>,
}

pub fn discover(input: &Path, options: &DiscoveryOptions) -> Result<DiscoveryResult, DocgenError> {
    let input_abs = if input.is_absolute() {
        input.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| DocgenError::new(format!("failed to resolve working directory: {}", e)))?
            .join(input)
    };

    let metadata = fs::metadata(&input_abs).map_err(|e| {
        DocgenError::new(format!("failed to read input path '{}': {}", input.display(), e))
    })?;

    let root_candidate = if metadata.is_dir() {
        input_abs.clone()
    } else {
        input_abs
            .parent()
            .ok_or_else(|| DocgenError::new("file input has no parent directory".to_string()))?
            .to_path_buf()
    };

    let root =
        canonicalize_root(&root_candidate, "docgen project root").map_err(DocgenError::new)?;

    let mut paths = Vec::new();
    if metadata.is_file() {
        paths.push(input_abs);
    } else {
        walk_dir(&root, &root, 0, options, &mut paths)?;
    }

    paths.sort();
    if paths.len() > options.max_files {
        return Err(DocgenError::new(format!(
            "file discovery exceeded limit ({} > {})",
            paths.len(),
            options.max_files
        )));
    }

    let selected: Option<BTreeSet<String>> = options
        .selected_languages
        .as_ref()
        .map(|langs| langs.iter().map(|lang| lang.to_ascii_lowercase()).collect());

    let mut files = Vec::new();
    let mut languages = BTreeSet::new();

    for file in paths {
        let canonical = fs::canonicalize(&file).map_err(|e| {
            DocgenError::new(format!("failed to resolve source path '{}': {}", file.display(), e))
        })?;
        ensure_path_within_root(&canonical, &root, "docgen source file")
            .map_err(DocgenError::new)?;

        let extension = canonical.extension().and_then(|value| value.to_str()).unwrap_or_default();

        let Some(adapter) = adapter_for_extension(extension) else {
            continue;
        };

        let language = adapter.language_id().to_string();
        if let Some(selected_languages) = &selected {
            if !selected_languages.contains(&language) {
                continue;
            }
        }

        let file_meta = fs::metadata(&canonical).map_err(|e| {
            DocgenError::new(format!("failed to stat source path '{}': {}", canonical.display(), e))
        })?;
        if file_meta.len() > options.max_file_size_bytes {
            continue;
        }

        let source = fs::read_to_string(&canonical).map_err(|e| {
            DocgenError::new(format!("failed to read source file '{}': {}", canonical.display(), e))
        })?;

        let relative_path = canonical
            .strip_prefix(&root)
            .map_err(|e| DocgenError::new(format!("failed to normalize source path: {}", e)))?
            .to_path_buf();

        languages.insert(language.clone());
        files.push(DiscoveredFile { language, absolute_path: canonical, relative_path, source });
    }

    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path).then(a.language.cmp(&b.language)));

    Ok(DiscoveryResult { root, files, detected_languages: languages.into_iter().collect() })
}

fn walk_dir(
    root: &Path,
    current: &Path,
    depth: usize,
    options: &DiscoveryOptions,
    out: &mut Vec<PathBuf>,
) -> Result<(), DocgenError> {
    if depth > options.max_depth {
        return Ok(());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(current).map_err(|e| {
        DocgenError::new(format!("failed to read directory '{}': {}", current.display(), e))
    })? {
        let entry = entry
            .map_err(|e| DocgenError::new(format!("failed to access directory entry: {}", e)))?;
        entries.push(entry);
    }

    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let symlink_meta = fs::symlink_metadata(&path).map_err(|e| {
            DocgenError::new(format!("failed to read metadata '{}': {}", path.display(), e))
        })?;

        if symlink_meta.file_type().is_symlink() {
            continue;
        }

        let canonical = fs::canonicalize(&path).map_err(|e| {
            DocgenError::new(format!("failed to resolve path '{}': {}", path.display(), e))
        })?;
        ensure_path_within_root(&canonical, root, "docgen discovery path")
            .map_err(DocgenError::new)?;

        if symlink_meta.is_dir() {
            walk_dir(root, &canonical, depth + 1, options, out)?;
        } else if symlink_meta.is_file() {
            out.push(canonical);
        }
    }

    Ok(())
}

pub fn parse_language_filter(
    language: Option<&str>,
    languages: Option<&str>,
) -> Result<Option<Vec<String>>, DocgenError> {
    if let Some(single) = language {
        return Ok(Some(vec![single.trim().to_ascii_lowercase()]));
    }

    if let Some(csv) = languages {
        let parsed: Vec<String> = csv
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(|part| part.to_ascii_lowercase())
            .collect();

        if parsed.is_empty() {
            return Err(DocgenError::new(
                "--languages was provided but no values were parsed".to_string(),
            ));
        }

        return Ok(Some(parsed));
    }

    Ok(None)
}

pub fn validate_languages(languages: &[String]) -> Result<(), DocgenError> {
    for language in languages {
        if adapter_for_language(language).is_none() {
            return Err(DocgenError::new(format!("unsupported language '{}' requested", language)));
        }
    }
    Ok(())
}
