// File: src/workflow_pack/manifest.rs
//
// Workflow pack manifest parsing and validation.
// Uses YAML format (ruff-pack.yaml) with serde_yaml.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// The canonical filename for a workflow pack manifest.
pub const MANIFEST_FILENAME: &str = "ruff-pack.yaml";

/// A single command definition in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandDef {
	pub name: String,
	pub summary: String,
	pub entry: String,
	#[serde(default)]
	pub safe: bool,
	#[serde(default)]
	pub writes_files: bool,
	#[serde(default)]
	pub runs_processes: bool,
	#[serde(default)]
	pub requires_network: bool,
}

/// The full workflow pack manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackManifest {
	pub id: String,
	pub namespace: String,
	pub name: String,
	pub version: String,
	#[serde(default)]
	pub description: String,
	#[serde(default)]
	pub commands: Vec<CommandDef>,
}

/// Validation error with a human-readable message.
#[derive(Debug, Clone)]
pub struct ManifestError {
	pub message: String,
}

impl std::fmt::Display for ManifestError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.message)
	}
}

impl From<String> for ManifestError {
	fn from(message: String) -> Self {
		Self { message }
	}
}

/// Parse a manifest from YAML string content.
pub fn parse_manifest(content: &str) -> Result<PackManifest, ManifestError> {
	let manifest: PackManifest = serde_yaml::from_str(content)
		.map_err(|e| ManifestError { message: format!("Failed to parse manifest: {}", e) })?;

	validate_manifest(&manifest)?;
	Ok(manifest)
}

/// Parse a manifest from a file path.
pub fn parse_manifest_file(path: &Path) -> Result<PackManifest, ManifestError> {
	let content = std::fs::read_to_string(path).map_err(|e| ManifestError {
		message: format!("Failed to read manifest file '{}': {}", path.display(), e),
	})?;
	parse_manifest(&content)
}

/// Validate a parsed manifest.
pub fn validate_manifest(manifest: &PackManifest) -> Result<(), ManifestError> {
	if manifest.id.trim().is_empty() {
		return Err(ManifestError { message: "Manifest 'id' is required and must not be empty.".to_string() });
	}

	if manifest.namespace.trim().is_empty() {
		return Err(ManifestError { message: "Manifest 'namespace' is required and must not be empty.".to_string() });
	}

	if !is_cli_safe_name(&manifest.namespace) {
		return Err(ManifestError {
			message: format!(
				"Manifest 'namespace' '{}' is not CLI-safe. Namespace must match [a-z][a-z0-9-]*.",
				manifest.namespace
			),
		});
	}

	if manifest.name.trim().is_empty() {
		return Err(ManifestError { message: "Manifest 'name' is required and must not be empty.".to_string() });
	}

	if manifest.version.trim().is_empty() {
		return Err(ManifestError { message: "Manifest 'version' is required and must not be empty.".to_string() });
	}

	if manifest.commands.is_empty() {
		return Err(ManifestError { message: "Manifest must declare at least one command.".to_string() });
	}

	let mut seen_command_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
	for cmd in &manifest.commands {
		if cmd.name.trim().is_empty() {
			return Err(ManifestError { message: "Each command must have a non-empty 'name'.".to_string() });
		}

		if !is_cli_safe_command_name(&cmd.name) {
			return Err(ManifestError {
				message: format!(
					"Command name '{}' is not CLI-safe. Use lowercase letters, digits, hyphens, and spaces for nested commands.",
					cmd.name
				),
			});
		}

		if cmd.entry.trim().is_empty() {
			return Err(ManifestError {
				message: format!("Command '{}' must have a non-empty 'entry'.", cmd.name),
			});
		}

		if !seen_command_names.insert(cmd.name.clone()) {
			return Err(ManifestError {
				message: format!("Duplicate command name '{}' in manifest.", cmd.name),
			});
		}
	}

	Ok(())
}

/// Check if a name is safe for CLI usage (namespace).
/// Must match [a-z][a-z0-9-]* (lowercase start, lowercase, digits, hyphens only).
fn is_cli_safe_name(name: &str) -> bool {
	if name.is_empty() {
		return false;
	}

	let bytes = name.as_bytes();

	// First character must be lowercase letter.
	if !bytes[0].is_ascii_lowercase() {
		return false;
	}

	// Remaining characters must be lowercase, digit, or hyphen.
	for &b in &bytes[1..] {
		if !b.is_ascii_lowercase() && !b.is_ascii_digit() && b != b'-' {
			return false;
		}
	}

	true
}

/// Check if a command name is CLI-safe.
/// Allows lowercase, digits, hyphens, and spaces (for nested commands like "card check").
fn is_cli_safe_command_name(name: &str) -> bool {
	if name.is_empty() {
		return false;
	}

	let bytes = name.as_bytes();

	for &b in bytes {
		if !b.is_ascii_lowercase() && !b.is_ascii_digit() && b != b'-' && b != b' ' {
			return false;
		}
	}

	// No leading/trailing spaces.
	if bytes[0] == b' ' || bytes[bytes.len() - 1] == b' ' {
		return false;
	}

	// No double spaces.
	for window in bytes.windows(2) {
		if window[0] == b' ' && window[1] == b' ' {
			return false;
		}
	}

	true
}

#[cfg(test)]
mod tests {
	use super::*;

	fn valid_manifest_yaml() -> String {
		r#"id: team-updraft
namespace: tud
name: Team Updraft
version: 0.1.0
description: Workflow commands for Team Updraft.

commands:
  - name: doctor
    summary: Check whether the local development environment is ready.
    entry: builtin
    safe: true
    writes_files: false
    runs_processes: true
    requires_network: false
"#
		.to_string()
	}

	#[test]
	fn parse_valid_manifest() {
		let manifest = parse_manifest(&valid_manifest_yaml()).expect("valid manifest should parse");
		assert_eq!(manifest.id, "team-updraft");
		assert_eq!(manifest.namespace, "tud");
		assert_eq!(manifest.name, "Team Updraft");
		assert_eq!(manifest.version, "0.1.0");
		assert_eq!(manifest.commands.len(), 1);
		assert_eq!(manifest.commands[0].name, "doctor");
	}

	#[test]
	fn reject_empty_id() {
		let yaml = valid_manifest_yaml().replace("id: team-updraft", "id: ''");
		let err = parse_manifest(&yaml).expect_err("empty id should fail");
		assert!(err.message.contains("id"));
	}

	#[test]
	fn reject_empty_namespace() {
		let yaml = valid_manifest_yaml().replace("namespace: tud", "namespace: ''");
		let err = parse_manifest(&yaml).expect_err("empty namespace should fail");
		assert!(err.message.contains("namespace"));
	}

	#[test]
	fn reject_invalid_namespace() {
		let yaml = valid_manifest_yaml().replace("namespace: tud", "namespace: TeamUpdraft");
		let err = parse_manifest(&yaml).expect_err("camelCase namespace should fail");
		assert!(err.message.contains("CLI-safe"));
	}

	#[test]
	fn reject_namespace_with_underscore() {
		let yaml = valid_manifest_yaml().replace("namespace: tud", "namespace: team_updraft");
		let err = parse_manifest(&yaml).expect_err("namespace with underscore should fail");
		assert!(err.message.contains("CLI-safe"));
	}

	#[test]
	fn reject_namespace_starting_with_digit() {
		let yaml = valid_manifest_yaml().replace("namespace: tud", "namespace: 2team");
		let err = parse_manifest(&yaml).expect_err("namespace starting with digit should fail");
		assert!(err.message.contains("CLI-safe"));
	}

	#[test]
	fn accept_namespace_with_hyphens() {
		let yaml = valid_manifest_yaml().replace("namespace: tud", "namespace: my-team");
		let manifest = parse_manifest(&yaml).expect("hyphenated namespace should parse");
		assert_eq!(manifest.namespace, "my-team");
	}

	#[test]
	fn reject_empty_command_name() {
		let yaml = valid_manifest_yaml().replace("name: doctor", "name: ''");
		let err = parse_manifest(&yaml).expect_err("empty command name should fail");
		assert!(err.message.contains("non-empty"));
	}

	#[test]
	fn reject_missing_commands() {
		let yaml = r#"id: test
namespace: test
name: Test
version: 0.1.0
commands: []
"#;
		let err = parse_manifest(yaml).expect_err("empty commands should fail");
		assert!(err.message.contains("at least one command"));
	}

	#[test]
	fn reject_duplicate_command_names() {
		let yaml = r#"id: test
namespace: test
name: Test
version: 0.1.0
commands:
  - name: doctor
    summary: Check env.
    entry: builtin
  - name: doctor
    summary: Duplicate.
    entry: builtin
"#;
		let err = parse_manifest(yaml).expect_err("duplicate command names should fail");
		assert!(err.message.contains("Duplicate"));
	}

	#[test]
	fn accept_nested_command_name() {
		let yaml = r#"id: test
namespace: test
name: Test
version: 0.1.0
commands:
  - name: card check
    summary: Check cards.
    entry: builtin
"#;
		let manifest = parse_manifest(yaml).expect("nested command name should parse");
		assert_eq!(manifest.commands[0].name, "card check");
	}

	#[test]
	fn reject_command_name_with_leading_space() {
		let yaml = r#"id: test
namespace: test
name: Test
version: 0.1.0
commands:
  - name: " doctor"
    summary: Bad.
    entry: builtin
"#;
		let err = parse_manifest(yaml).expect_err("leading space should fail");
		assert!(err.message.contains("CLI-safe"));
	}

	#[test]
	fn reject_missing_entry() {
		let yaml = r#"id: test
namespace: test
name: Test
version: 0.1.0
commands:
  - name: doctor
    summary: Check env.
    entry: ''
"#;
		let err = parse_manifest(yaml).expect_err("empty entry should fail");
		assert!(err.message.contains("entry"));
	}
}
