// File: src/workflow_pack/renderer.rs
//
// Output renderers for workflow command results.
// Supports human-readable terminal output and machine-readable JSON.

use crate::workflow_pack::types::{CheckResult, CheckStatus, DoctorReport};
use colored::Colorize;

/// Render a DoctorReport as human-readable terminal output.
pub fn render_human(report: &DoctorReport) {
	let pack_label = format!("{} {}", report.namespace.to_uppercase(), report.command);

	println!("{}", pack_label.bold().underline());
	if let Some(ref cwd) = report.cwd {
		println!("  cwd: {}", cwd.dimmed());
	}
	println!();

	// Group checks by their implicit category based on ID prefix.
	let groups = group_checks(&report.checks);

	for (group_name, checks) in &groups {
		if checks.is_empty() {
			continue;
		}
		println!("{}", group_name.bold());
		for check in checks {
			render_check_human(check);
		}
		println!();
	}

	// Summary line.
	println!("{}", "Summary".bold());
	println!(
		"  {} passed, {} warnings, {} failed, {} skipped",
		report.summary.pass.to_string().green(),
		report.summary.warn.to_string().yellow(),
		report.summary.fail.to_string().red(),
		report.summary.skip.to_string().dimmed(),
	);

	if report.summary.info > 0 {
		println!("  {} info", report.summary.info.to_string().dimmed());
	}

	// Recommended next actions.
	if let Some(ref actions) = report.recommended_next_actions {
		if !actions.is_empty() {
			println!();
			println!("{}", "Recommended next actions".bold());
			for action in actions {
				println!("  • {}", action);
			}
		}
	}
}

/// Render a single check result in human format.
fn render_check_human(check: &CheckResult) {
	let status_str = match check.status {
		CheckStatus::Pass => "PASS".green().bold(),
		CheckStatus::Warn => "WARN".yellow().bold(),
		CheckStatus::Fail => "FAIL".red().bold(),
		CheckStatus::Skip => "SKIP".dimmed().bold(),
		CheckStatus::Info => "INFO".dimmed().bold(),
	};

	let observed = match &check.observed {
		Some(val) => format!(": {}", val),
		None => String::new(),
	};

	println!("  {} {}{}", status_str, check.label, observed);

	if let Some(message) = &check.message {
		match check.status {
			CheckStatus::Warn | CheckStatus::Fail => {
				println!("        {}", message.dimmed());
			}
			_ => {}
		}
	}

	if let Some(fix) = &check.suggested_fix {
		println!("        {} {}", "Suggested fix:".yellow(), fix.dimmed());
	}
}

/// Group checks by the prefix of their ID (before the first dot).
/// Falls back to "General" if no dot is present.
fn group_checks(checks: &[CheckResult]) -> Vec<(String, Vec<&CheckResult>)> {
	let mut groups: std::collections::BTreeMap<String, Vec<&CheckResult>> =
		std::collections::BTreeMap::new();
	let mut order: Vec<String> = Vec::new();

	for check in checks {
		let category = match check.id.split('.').next() {
			Some(prefix) if !prefix.is_empty() => prefix_to_label(prefix),
			_ => "General".to_string(),
		};

		if !groups.contains_key(&category) {
			order.push(category.clone());
		}
		groups.entry(category).or_default().push(check);
	}

	order.into_iter().map(|cat| {
		let checks = groups.remove(&cat).unwrap_or_default();
		(cat, checks)
	}).collect()
}

/// Convert a check ID prefix to a human-readable category label.
fn prefix_to_label(prefix: &str) -> String {
	match prefix {
		"env" => "Environment".to_string(),
		"repo" => "Repository".to_string(),
		"project" => "Project signals".to_string(),
		"wp" => "WordPress".to_string(),
		"build" => "Build scripts".to_string(),
		_ => {
			// Capitalize first letter, replace hyphens with spaces.
			let mut chars: Vec<char> = prefix.chars().collect();
			if !chars.is_empty() {
				chars[0] = chars[0].to_ascii_uppercase();
			}
			chars.into_iter().collect::<String>().replace('-', " ")
		}
	}
}

/// Render a DoctorReport as Markdown text (no terminal colors).
pub fn render_markdown(report: &DoctorReport) -> String {
	let mut md = String::new();

	let title = format!(
		"# {} Doctor Report",
		report.namespace.to_uppercase(),
	);
	md.push_str(&title);
	md.push('\n');
	md.push('\n');

	if let Some(ref cwd) = report.cwd {
		md.push_str(&format!("**Working directory:** `{}`\n\n", cwd));
	}

	if let Some(ref schema) = report.schema_version {
		md.push_str(&format!("**Schema version:** {}\n\n", schema));
	}

	md.push_str(&format!("**Status:** {}\n\n", report.status));

	// Summary.
	md.push_str("## Summary\n\n");
	md.push_str(&format!(
		"| Status | Count |\n|--------|-------|\n"
	));
	md.push_str(&format!("| PASS | {} |\n", report.summary.pass));
	md.push_str(&format!("| WARN | {} |\n", report.summary.warn));
	md.push_str(&format!("| FAIL | {} |\n", report.summary.fail));
	md.push_str(&format!("| SKIP | {} |\n", report.summary.skip));
	md.push_str(&format!("| INFO | {} |\n", report.summary.info));
	md.push('\n');

	// Grouped checks.
	let groups = group_checks(&report.checks);
	for (group_name, checks) in &groups {
		if checks.is_empty() {
			continue;
		}
		md.push_str(&format!("## {}\n\n", group_name));

		for check in checks {
			let icon = match check.status {
				CheckStatus::Pass => "✅",
				CheckStatus::Warn => "⚠️",
				CheckStatus::Fail => "❌",
				CheckStatus::Skip => "⏭️",
				CheckStatus::Info => "ℹ️",
			};

			let observed = match &check.observed {
				Some(val) if !val.is_empty() => format!(": `{}`", val),
				_ => String::new(),
			};

			md.push_str(&format!("- {} **{}**{}\n", icon, check.label, observed));

			if let Some(message) = &check.message {
				match check.status {
					CheckStatus::Warn | CheckStatus::Fail => {
						md.push_str(&format!("  - {}\n", message));
					}
					_ => {}
				}
			}

			if let Some(fix) = &check.suggested_fix {
				md.push_str(&format!("  - *Fix:* {}\n", fix));
			}
		}
		md.push('\n');
	}

	// Recommended actions.
	if let Some(ref actions) = report.recommended_next_actions {
		if !actions.is_empty() {
			md.push_str("## Recommended Next Actions\n\n");
			for action in actions {
				md.push_str(&format!("- {}\n", action));
			}
			md.push('\n');
		}
	}

	md
}

/// Render a DoctorReport as JSON to stdout.
pub fn render_json(report: &DoctorReport) {
	match serde_json::to_string_pretty(report) {
		Ok(serialized) => println!("{}", serialized),
		Err(e) => {
			eprintln!("Failed to serialize report to JSON: {}", e);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::workflow_pack::types::{CheckResult, CheckSeverity, CheckStatus};

	#[test]
	fn group_checks_by_prefix() {
		let checks = vec![
			CheckResult {
				id: "env.git".to_string(),
				label: "Git".to_string(),
				status: CheckStatus::Pass,
				severity: CheckSeverity::Info,
				observed: Some("available".to_string()),
				expected: None,
				message: None,
				suggested_fix: None, reason: None, category: None, observed_major: None, minimum_major: None,
			},
			CheckResult {
				id: "repo.branch".to_string(),
				label: "Branch".to_string(),
				status: CheckStatus::Info,
				severity: CheckSeverity::Info,
				observed: Some("main".to_string()),
				expected: None,
				message: None,
				suggested_fix: None, reason: None, category: None, observed_major: None, minimum_major: None,
			},
		];

		let groups = group_checks(&checks);
		assert_eq!(groups.len(), 2);
		assert_eq!(groups[0].0, "Environment");
		assert_eq!(groups[1].0, "Repository");
	}
}
