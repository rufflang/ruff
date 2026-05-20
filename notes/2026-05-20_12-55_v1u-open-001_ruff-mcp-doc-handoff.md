# V1U-OPEN-001 External Handoff: Ruff MCP Closure-Mutation Docs Drift

Date: 2026-05-20  
Checklist link: `docs/PRE_V1_MASTER_UNFINISHED_CHECKLIST.md` (`V1U-OPEN-001`)  
Source checklist item: `docs/PRE_V1_ACTION_CHECKLIST.md` (`PREV1-RUN-002`)

## Why This Is External

This repository does not contain editable `ruff-mcp` source docs.

Validation evidence:

- `rg -n "ruff-mcp|mcp.ruff" README.md docs notes -g '*.md'`
- `rg --files | rg "mcp|MCP|ruff-mcp|mcp.ruff"`

Result: only references and generated external outputs are present here; no source-doc edit target exists for `ruff-mcp` docs.

## Handoff Target

- External repository: `ruff-mcp` (source repo where `README`/`mcp.ruff` docs live)
- Suggested owner: Ruff MCP maintainers / docs owner
- Handoff ticket ID: `V1U-OPEN-001-HANDOFF-2026-05-20`

## Requested Doc Update

Update any stale wording that claims closure mutation behavior is still limited.

Proposed replacement guidance:

1. State that Ruff runtime now supports named nested closure capture and mutation behavior.
2. Replace legacy caveats that imply this remains unsupported.
3. Link parity evidence to Ruff note:
   - `notes/2026-05-12_23-23_NO-ROADMAP_named-nested-closure-capture-parity.md`

## Validation Steps (in `ruff-mcp` repo)

1. Update docs (`README`, `mcp.ruff`, or equivalent canonical page).
2. Confirm no stale closure-mutation limitation language remains.
3. Record commit hash / PR URL.
4. Back-link resulting PR/commit into Ruff notes for traceability.

## Completion Condition For Ruff Side

Once external PR/commit URL is available, append it to this note and to the matching checklist evidence line.
