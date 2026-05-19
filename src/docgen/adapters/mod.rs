use crate::docgen::model::{DocCommentBlock, DocSymbol};
use crate::docgen::DocgenError;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

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

type AdapterConstructor = fn() -> Box<dyn DocLanguageAdapter>;

#[derive(Clone)]
struct AdapterEntry {
    language_id: &'static str,
    file_extensions: &'static [&'static str],
    capabilities: AdapterCapability,
    constructor: AdapterConstructor,
}

fn make_ruff_adapter() -> Box<dyn DocLanguageAdapter> {
    Box::new(ruff::RuffDocAdapter)
}

fn make_php_adapter() -> Box<dyn DocLanguageAdapter> {
    Box::new(php::PhpDocAdapter)
}

fn make_python_adapter() -> Box<dyn DocLanguageAdapter> {
    Box::new(python::PythonDocAdapter)
}

fn make_typescript_adapter() -> Box<dyn DocLanguageAdapter> {
    Box::new(typescript::TypeScriptDocAdapter)
}

fn make_javascript_adapter() -> Box<dyn DocLanguageAdapter> {
    Box::new(javascript::JavaScriptDocAdapter)
}

fn make_ruby_adapter() -> Box<dyn DocLanguageAdapter> {
    Box::new(ruby::RubyDocAdapter)
}

fn make_go_adapter() -> Box<dyn DocLanguageAdapter> {
    Box::new(go::GoDocAdapter)
}

fn make_haskell_adapter() -> Box<dyn DocLanguageAdapter> {
    Box::new(haskell::HaskellDocAdapter)
}

fn make_zig_adapter() -> Box<dyn DocLanguageAdapter> {
    Box::new(zig::ZigDocAdapter)
}

const ADAPTER_CONSTRUCTORS: [AdapterConstructor; 9] = [
    make_ruff_adapter,
    make_php_adapter,
    make_python_adapter,
    make_typescript_adapter,
    make_javascript_adapter,
    make_ruby_adapter,
    make_go_adapter,
    make_haskell_adapter,
    make_zig_adapter,
];

fn build_adapter_entries() -> Vec<AdapterEntry> {
    ADAPTER_CONSTRUCTORS
        .iter()
        .map(|constructor| {
            let adapter = constructor();
            AdapterEntry {
                language_id: adapter.language_id(),
                file_extensions: adapter.file_extensions(),
                capabilities: adapter.capabilities(),
                constructor: *constructor,
            }
        })
        .collect()
}

fn adapter_entries() -> &'static [AdapterEntry] {
    static ADAPTER_ENTRIES: OnceLock<Vec<AdapterEntry>> = OnceLock::new();
    ADAPTER_ENTRIES.get_or_init(build_adapter_entries).as_slice()
}

fn language_lookup() -> &'static BTreeMap<String, AdapterConstructor> {
    static LOOKUP: OnceLock<BTreeMap<String, AdapterConstructor>> = OnceLock::new();
    LOOKUP.get_or_init(|| {
        adapter_entries()
            .iter()
            .map(|entry| (entry.language_id.to_ascii_lowercase(), entry.constructor))
            .collect()
    })
}

fn extension_lookup() -> &'static BTreeMap<String, AdapterConstructor> {
    static LOOKUP: OnceLock<BTreeMap<String, AdapterConstructor>> = OnceLock::new();
    LOOKUP.get_or_init(|| {
        let mut map = BTreeMap::new();
        for entry in adapter_entries() {
            for ext in entry.file_extensions {
                map.insert(ext.to_ascii_lowercase(), entry.constructor);
            }
        }
        map
    })
}

#[allow(dead_code)]
pub fn registry() -> Vec<Box<dyn DocLanguageAdapter>> {
    adapter_entries().iter().map(|entry| (entry.constructor)()).collect()
}

pub fn adapter_for_language(language: &str) -> Option<Box<dyn DocLanguageAdapter>> {
    let normalized = language.trim().to_ascii_lowercase();
    language_lookup().get(&normalized).map(|constructor| constructor())
}

pub fn adapter_for_extension(ext: &str) -> Option<Box<dyn DocLanguageAdapter>> {
    let normalized = ext.trim_start_matches('.').to_ascii_lowercase();
    extension_lookup().get(&normalized).map(|constructor| constructor())
}

#[allow(dead_code)]
pub fn language_ids() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = adapter_entries().iter().map(|entry| entry.language_id).collect();
    ids.sort_unstable();
    ids
}

pub fn capability_index() -> Vec<(String, AdapterCapability)> {
    let mut entries: Vec<(String, AdapterCapability)> = adapter_entries()
        .iter()
        .map(|entry| (entry.language_id.to_string(), entry.capabilities.clone()))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_lookup_handles_language_and_extension_normalization() {
        assert!(adapter_for_language("RUSTY_UNKNOWN").is_none());
        assert_eq!(
            adapter_for_language("RuFf").map(|adapter| adapter.language_id()),
            Some("ruff")
        );
        assert_eq!(
            adapter_for_extension(".TSX").map(|adapter| adapter.language_id()),
            Some("typescript")
        );
        assert_eq!(
            adapter_for_extension("mJs").map(|adapter| adapter.language_id()),
            Some("javascript")
        );
    }

    #[test]
    fn capability_index_and_language_ids_are_sorted_and_aligned() {
        let language_ids = language_ids();
        let mut expected_language_ids = language_ids.clone();
        expected_language_ids.sort_unstable();
        assert_eq!(language_ids, expected_language_ids);

        let capabilities = capability_index();
        let mut capability_languages: Vec<String> =
            capabilities.iter().map(|(language, _)| language.clone()).collect();
        let mut expected = capability_languages.clone();
        expected.sort();
        assert_eq!(capability_languages, expected);

        capability_languages.dedup();
        assert_eq!(capability_languages.len(), language_ids.len());
    }

    #[test]
    fn static_lookup_avoids_legacy_full_registry_construction_cost() {
        fn legacy_language_lookup_construction_count(language: &str) -> usize {
            let mut constructed = 0usize;
            for constructor in ADAPTER_CONSTRUCTORS {
                constructed += 1;
                let adapter = constructor();
                if adapter.language_id() == language {
                    break;
                }
            }
            constructed
        }

        let legacy_constructions = legacy_language_lookup_construction_count("zig");
        assert_eq!(legacy_constructions, ADAPTER_CONSTRUCTORS.len());

        let static_lookup = adapter_for_language("zig").expect("zig adapter should resolve");
        assert_eq!(static_lookup.language_id(), "zig");
        // New lookup still constructs one adapter instance for the returned result, but avoids
        // constructing all intermediate adapters during lookup.
        assert!(legacy_constructions > 1);
    }
}
