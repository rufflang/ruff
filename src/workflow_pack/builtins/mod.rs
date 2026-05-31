// File: src/workflow_pack/builtins/mod.rs
//
// Built-in workflow pack registrations.
//
// Built-in packs are compiled into the Ruff binary and have the highest
// priority during namespace resolution. They cannot be overridden by
// external packs.
//
// To add a built-in pack:
//   1. Create a new module under builtins/ (e.g., myteam.rs)
//   2. Add `pub mod myteam;` below
//   3. Implement command handlers as fn(&CommandContext) -> DoctorReport
//   4. Register the pack in register_all() below
//
// Currently no built-in packs are registered. The workflow pack system
// is designed so that teams can create external packs without modifying
// Ruff core. See docs/WORKFLOW_PACKS.md for details.

use crate::workflow_pack::discovery::DiscoveredPack;
use crate::workflow_pack::registry::WorkflowRegistry;

/// Register all built-in workflow packs into the registry.
pub fn register_all(registry: &mut WorkflowRegistry) -> Result<(), String> {
	// No built-in packs are registered by default.
	// Teams should create external workflow packs in .ruff/packs/,
	// ~/.ruff/packs/, or via RUFF_PACK_PATH.
	let _ = registry;
	Ok(())
}

/// Build the list of built-in packs for discovery priority tracking.
pub fn builtin_packs_list() -> Vec<DiscoveredPack> {
	// No built-in packs currently ship with Ruff.
	Vec::new()
}

