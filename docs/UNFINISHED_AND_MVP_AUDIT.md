# Unfinished / MVP / Deferred Audit

Date: 2026-05-20  
Primary source of truth: `docs/generated/PRE_V1_UNRESOLVED_INVENTORY.md`

Canonical readiness boundary: Ruff remains pre-1.0 until `ROADMAP.md` and `docs/PRE_V1_MASTER_UNFINISHED_CHECKLIST.md` release gates are closed.

This audit now mirrors the machine-generated pre-v1 inventory classification for every
`V1U-*` checklist item.

## Classification Snapshot

| Classification | Count | Meaning |
| --- | --- | --- |
| `v1-blocker` | 1 | Must be closed (or explicitly exceptioned) before v1 release sign-off. |
| `v1-should-fix` | 0 | High-value pre-v1 quality/docs/runtime work that should be completed unless explicitly deferred. |
| `post-v1` | 1 | Tag-time publication tasks that require the actual release event to close. |
| `archive` | 28 | Already completed item retained for audit traceability. |

## Category Notes

1. `v1-blocker`
   - `V1U-FINAL-003` is the only remaining blocker in this snapshot; it remains tied to the actual tag-time artifact checklist and final publish evidence.

2. `v1-should-fix`
   - No items remain in this bucket in the current snapshot.

3. `post-v1`
   - `V1U-OPEN-003`: tag-time release publication and artifact sign-off checklist completion.

4. `archive`
   - All other `V1U-*` checklist items are complete and retained in the inventory to preserve the full audit chain.

## Traceability

- Per-item classification rationale is captured directly in:
  - `docs/generated/PRE_V1_UNRESOLVED_INVENTORY.md`
  - `docs/generated/PRE_V1_UNRESOLVED_INVENTORY.csv`
- Generation path:
  - `bash scripts/generate_pre_v1_unresolved_inventory.sh`
- Contract coverage:
  - `cargo test --test pre_v1_unresolved_inventory_contract`
