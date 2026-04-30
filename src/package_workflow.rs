use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RuffManifest {
    pub package: PackageMetadata,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
}

pub fn default_manifest(package_name: &str) -> String {
    let manifest = RuffManifest {
        package: PackageMetadata {
            name: package_name.to_string(),
            version: "0.1.0".to_string(),
        },
        dependencies: BTreeMap::new(),
    };

    toml::to_string_pretty(&manifest).expect("manifest serialization should succeed")
}

pub fn parse_manifest(content: &str) -> Result<RuffManifest, String> {
    toml::from_str::<RuffManifest>(content).map_err(|error| format!("Invalid ruff.toml: {}", error))
}

pub fn add_dependency(content: &str, name: &str, version: &str) -> Result<String, String> {
    if name.trim().is_empty() {
        return Err("Dependency name must not be empty".to_string());
    }
    if version.trim().is_empty() {
        return Err("Dependency version must not be empty".to_string());
    }

    let mut manifest = parse_manifest(content)?;
    manifest.dependencies.insert(name.to_string(), version.to_string());

    toml::to_string_pretty(&manifest).map_err(|error| format!("Failed to serialize ruff.toml: {}", error))
}

#[cfg(test)]
mod tests {
    use super::{add_dependency, default_manifest, parse_manifest};

    #[test]
    fn default_manifest_contains_package_section() {
        let content = default_manifest("demo");
        assert!(content.contains("[package]"));
        assert!(content.contains("name = \"demo\""));
        assert!(content.contains("version = \"0.1.0\""));
    }

    #[test]
    fn add_dependency_updates_manifest() {
        let content = default_manifest("demo");
        let updated = add_dependency(&content, "http", "1.2.3").expect("dependency update should succeed");

        let manifest = parse_manifest(&updated).expect("updated manifest should parse");
        assert_eq!(manifest.dependencies.get("http").map(|value| value.as_str()), Some("1.2.3"));
    }

    #[test]
    fn add_dependency_rejects_empty_values() {
        let content = default_manifest("demo");
        assert!(add_dependency(&content, "", "1.0.0").is_err());
        assert!(add_dependency(&content, "http", "").is_err());
    }
}