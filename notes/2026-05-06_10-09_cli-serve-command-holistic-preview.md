# Ruff Field Notes — CLI serve command for holistic local preview

**Date:** 2026-05-06
**Session:** 10:09 local
**Branch/Commit:** main / ad31700
**Scope:** Added a first-class `ruff serve` CLI command so local static preview is available to any Ruff user without project-specific scripts. Replaced `ruff-ssg` docs usage of a one-off preview script with the new shared command.

---

## What I Changed
- Added `Serve` subcommand to `src/main.rs`:
  - `ruff serve [dir] --host <host> --port <port> --index <file>`
- Implemented static file serving in CLI runtime using `tiny_http`:
  - canonical root resolution
  - GET-only request handling
  - safe path boundary check (`canonical_target.starts_with(root_dir)`)
  - index-file resolution for `/` and directory paths
  - extension-based content type header
- Updated Ruff CLI docs table in `README.md` to include `ruff serve [dir]`.
- Updated `ruff-ssg/README.md` preview section to use `ruff serve output --port 8080`.
- Removed `ruff-ssg/serve.ruff` to avoid maintaining a project-specific server implementation.

## Gotchas (Read This Next Time)
- **Gotcha:** `ruff run --interpreter` and `http_server(...).route(...)` behavior is not parity-complete for all server patterns.
  - **Symptom:** Earlier SSG preview script failed with `Unknown method: route` in interpreter mode.
  - **Root cause:** Method dispatch parity for HTTP server helper paths differs between runtime modes.
  - **Fix:** Move preview server responsibility to a CLI-native command (`ruff serve`) rather than script-level server code.
  - **Prevention:** For universal user workflows, prefer Ruff CLI subcommands over runtime-mode-sensitive script hacks.

- **Gotcha:** Security constraints must be explicit when serving arbitrary paths.
  - **Symptom:** Naive path joins allow traversal outside serve root.
  - **Root cause:** URL path segments can reference parent directories.
  - **Fix:** Canonicalize requested path and reject if it escapes root (`!starts_with(root_dir)`).
  - **Prevention:** Treat root-boundary checks as a non-optional invariant for any file-serving primitive.

## Things I Learned
- CLI-level primitives are the correct place for cross-project workflows that should “just work” for all Ruff users.
- Rule: if a capability is expected for every user (local preview/static serving), put it in `ruff <subcommand>` rather than embedding it in a single project script.
- Existing `tiny_http` dependency made this addition cheap and low-risk to introduce.

## Debug Notes (Only if applicable)
- **Failing test / error:** `Runtime Error: Unknown method: route` when launching preview script with `--interpreter`.
- **Repro steps:**
  - `cd /Users/robertdevore/2026/ruff-ssg`
  - `/Users/robertdevore/2026/ruff/target/release/ruff run ./serve.ruff --interpreter`
- **Breakpoints / logs used:** CLI help output + runtime smoke test with `curl`.
- **Final diagnosis:** The one-off script approach depended on runtime mode behavior; a CLI command is the stable holistic surface.

## Follow-ups / TODO (For Future Agents)
- [ ] Add dedicated Rust unit/integration tests for `ruff serve` status codes, path traversal rejection, and content-type headers.
- [ ] Consider optional SPA fallback flag (`--spa`) to map 404 paths to index file when needed.

## Links / References
- Files touched:
  - `src/main.rs`
  - `README.md`
  - `../ruff-ssg/README.md`
  - `../ruff-ssg/serve.ruff`
- Related docs:
  - `README.md`
  - `ROADMAP.md`
  - `notes/FIELD_NOTES_SYSTEM.md`
