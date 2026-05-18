use crate::docgen::model::{DocCommentBlock, DocSymbol};
use crate::docgen::DocgenError;
use std::path::Path;

pub(crate) mod common;
pub mod go;
pub mod haskell;
pub mod javascript;
pub mod php;
pub mod python;
pub mod ruby;
pub mod ruff;
pub mod typescript;
pub mod zig;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AdapterCapability {
    pub supports_functions: bool,
    pub supports_types: bool,
    pub supports_methods: bool,
    pub supports_inline_docs: bool,
}

pub trait DocLanguageAdapter: Send + Sync {
    fn language_id(&self) -> &'static str;
    fn file_extensions(&self) -> &'static [&'static str];
    fn capabilities(&self) -> AdapterCapability;
    fn extract_symbols(&self, source: &str, path: &Path) -> Result<Vec<DocSymbol>, DocgenError>;
    fn extract_inline_docs(
        &self,
        source: &str,
        path: &Path,
    ) -> Result<Vec<DocCommentBlock>, DocgenError>;
    fn attach_docs(&self, symbols: Vec<DocSymbol>, docs: Vec<DocCommentBlock>) -> Vec<DocSymbol>;
}

pub fn registry() -> Vec<Box<dyn DocLanguageAdapter>> {
    vec![
        Box::new(ruff::RuffDocAdapter),
        Box::new(php::PhpDocAdapter),
        Box::new(python::PythonDocAdapter),
        Box::new(typescript::TypeScriptDocAdapter),
        Box::new(javascript::JavaScriptDocAdapter),
        Box::new(ruby::RubyDocAdapter),
        Box::new(go::GoDocAdapter),
        Box::new(haskell::HaskellDocAdapter),
        Box::new(zig::ZigDocAdapter),
    ]
}

pub fn adapter_for_language(language: &str) -> Option<Box<dyn DocLanguageAdapter>> {
    registry().into_iter().find(|adapter| adapter.language_id() == language)
}

pub fn adapter_for_extension(ext: &str) -> Option<Box<dyn DocLanguageAdapter>> {
    let normalized = ext.trim_start_matches('.').to_ascii_lowercase();
    registry().into_iter().find(|adapter| {
        adapter.file_extensions().iter().any(|entry| entry.eq_ignore_ascii_case(&normalized))
    })
}

#[allow(dead_code)]
pub fn language_ids() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> =
        registry().into_iter().map(|adapter| adapter.language_id()).collect();
    ids.sort_unstable();
    ids
}

pub fn capability_index() -> Vec<(String, AdapterCapability)> {
    let mut entries: Vec<(String, AdapterCapability)> = registry()
        .into_iter()
        .map(|adapter| (adapter.language_id().to_string(), adapter.capabilities()))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}
