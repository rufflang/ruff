# Unfinished / MVP / Deferred Audit

Date: 2026-05-20  
Primary source of truth: `docs/generated/PRE_V1_UNRESOLVED_INVENTORY.md`

This audit now mirrors the machine-generated pre-v1 inventory classification for every
`V1U-*` checklist item.

## Classification Snapshot

| Classification | Count | Meaning |
| --- | --- | --- |
| `v1-blocker` | 16 | Must be closed (or explicitly exceptioned) before v1 release sign-off. |
| `v1-should-fix` | 11 | High-value pre-v1 quality/docs/runtime work that should be completed unless explicitly deferred. |
| `post-v1` | 2 | Tag-time publication tasks that require the actual release event to close. |
| `archive` | 1 | Already completed item retained for audit traceability. |

## Category Notes

1. `v1-blocker`
   - Includes unresolved truth-set governance (`V1U-RES-002/003`), release-gate determinism (`V1U-GATE-*`), runtime-path parity burn-down (`V1U-RUN-*`), high-risk runtime TODO closure (`V1U-CODE-002`), and final release evidence tasks (`V1U-FINAL-*`).

2. `v1-should-fix`
   - Includes external-doc drift handoff (`V1U-OPEN-001`), docgen roadmap execution (`V1U-OPEN-004`, `V1U-DG-*`), stale-doc and consistency alignment (`V1U-DOC-*`), and broader runtime TODO triage/optional-typing boundary verification (`V1U-CODE-001`, `V1U-CODE-003`).

3. `post-v1`
   - `V1U-OPEN-002`: final roadmap checklist items that hinge on intentional version-bump/tag-phase execution.
   - `V1U-OPEN-003`: tag-time release publication and artifact sign-off checklist completion.

4. `archive`
   - `V1U-RES-001` completed; kept in inventory to preserve the full audit chain.

## Traceability

- Per-item classification rationale is captured directly in:
  - `docs/generated/PRE_V1_UNRESOLVED_INVENTORY.md`
  - `docs/generated/PRE_V1_UNRESOLVED_INVENTORY.csv`
- Generation path:
  - `bash scripts/generate_pre_v1_unresolved_inventory.sh`
- Contract coverage:
  - `cargo test --test pre_v1_unresolved_inventory_contract`
