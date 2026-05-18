# DOCGEN

`ruff docgen` is Ruff's universal documentation generator.

It is model-driven and adapter-based:

- Universal core pipeline
- Language adapters
- Shared symbol model
- Gap analyzer
- Renderers (HTML, Markdown, JSON)
- CI quality gates
- Optional AI task emission

## Supported Languages

- Ruff (`.ruff`)
- PHP (`.php`)
- Python (`.py`)
- TypeScript (`.ts`, `.tsx`)
- JavaScript (`.js`, `.jsx`, `.mjs`, `.cjs`)
- Ruby (`.rb`)
- Go (`.go`)
- Haskell (`.hs`, `.lhs`)
- Zig (`.zig`)

## Core Design

The core lives under `src/docgen/` and is language-agnostic.

- `core.rs`: orchestration + output + gates
- `discovery.rs`: safe deterministic file discovery
- `model.rs`: shared project/module/symbol/gap model
- `gaps.rs`: missing-doc and link-gap analysis
- `render/*`: HTML/Markdown/JSON rendering
- `adapters/*`: language-specific symbol/doc extraction

## Ruff Visibility Policy

For Ruff symbols, DocGen visibility is explicit and gate-oriented:

1. Top-level symbols are `Public` only when declared with `pub`:
   - `pub func ...`
   - `pub struct ...`
   - `pub enum ...`
   - `pub const ...` / `pub let ...`
2. Non-`pub` top-level symbols are `Private`.
3. Struct method visibility requires both:
   - method declaration is `pub`
   - containing struct is `Public`
   Methods under private structs remain `Private` even if the method itself is declared `pub`.
4. Enum variants inherit visibility from the containing enum:
   - variants under `pub enum` are `Public`
   - variants under non-`pub enum` are `Private`
5. `--public-only` with `--include-private` disabled filters to `Public` symbols only and is the intended strict CI gate surface.

## Ruff Doc Comment Styles

Ruff DocGen currently attaches inline documentation from these Ruff comment forms:

1. `///` line doc comments
2. `//!` line doc comments
3. `/** ... */` block doc comments

Non-doc block comments (`/* ... */`) are not treated as API documentation.
Attachment matching is decorator-aware: DocGen will skip Ruff decorator/attribute lines (for example `@...` and `#[...]`) between a doc block and its symbol target.
Proximity behavior is stable and test-locked:
1. Blank lines between a doc block and symbol are allowed.
2. Regular non-doc comment lines break attachment.
3. The nearest eligible doc block is attached when multiple blocks appear before a symbol.

## Ruff Extraction Decision (Workstream B-3)

Decision: keep a hybrid Ruff extraction strategy for now, with regex-based symbol discovery as the production path and fixture-driven edge-case hardening as the reliability guard.

Rationale:

1. `ruff docgen` is expected to be best-effort and resilient even when repositories contain partially invalid Ruff sources; a hard parser-only path would reject symbol extraction for files that fail full parse.
2. Current parser surfaces are optimized for program execution semantics and require broader AST/lowering context than doc extraction needs, increasing migration risk for CI gate stability.
3. Recent Workstream B improvements plus fixture-backed regressions (`tests/fixtures/docgen/*`) close concrete extraction misses (for example `async func`) without introducing parser-coupled failure modes.

Follow-through policy:

1. Continue expanding fixture-backed Ruff extraction edge cases as they are discovered.
2. Revisit parser-assisted extraction when a bounded adapter can consume parser output with graceful fallback behavior on parse diagnostics.
3. Preserve deterministic output and strict-gate stability as non-negotiable acceptance criteria for any parser-assisted rollout.

## Security Model

DocGen is scan-only.

- No source code execution
- No imports/build steps
- No external AI calls by default
- Symlink traversal is skipped during discovery
- File size, depth, and file count limits are enforced
- Deterministic ordering for CI stability
- HTML output escapes documentation content by default

## Discovery Skip Diagnostics

When discovery limits skip input, DocGen emits warning diagnostics in `docgen.json`:

1. `DOCGEN_DISCOVERY_MAX_FILE_SIZE`
2. `DOCGEN_DISCOVERY_MAX_DEPTH`
3. `DOCGEN_DISCOVERY_MAX_FILES`

`ruff docgen --json` also reports per-reason skip counters under `discovery_skip_counts`:
1. `max_file_size`
2. `max_depth`
3. `max_files`

Discovery and project diagnostics are emitted in deterministic sorted order for CI-stable JSON comparisons.

## CLI

### Basic Ruff docs

```bash
ruff docgen src/ --language ruff --out-dir docs/generated
```

### Auto-detect languages

```bash
ruff docgen . --out-dir docs/generated
```

### Explicit multi-language run

```bash
ruff docgen . --languages ruff,php,python,typescript,javascript,ruby,go,haskell,zig --out-dir docs/generated
```

### Strict CI gates

```bash
ruff docgen . --public-only --fail-on-undocumented --fail-on-broken-links
```

### AI-ready gap files

```bash
ruff docgen . --emit-ai-tasks --out-dir docs/generated
```

## Output Files

The output directory includes:

- `index.html`
- `docgen.md` (when format includes markdown)
- `docgen.json`
- `docgen-gaps.json`
- `docgen-capabilities.json`
- `docgen-ai-tasks.md` (with `--emit-ai-tasks`)
- `builtins.html` (unless `--no-builtins`)
- `search-index.json` + `symbol-index.json` (with `--search-index`)

## Gaps and Placeholders

Public symbols are documented even when no inline docs exist.

Missing docs are rendered as:

- `Documentation needed.`
- `This symbol was discovered from the source code, but no human-authored documentation was found.`

`docgen-gaps.json` and `docgen-ai-tasks.md` include bounded source context and constrained prompts:

- Use only provided context
- Do not invent behavior
- Mark uncertainty
- Keep docs concise
- Add examples only when source supports them
