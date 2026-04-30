# JetBrains LSP Adapter Baseline

Use a generic LSP plugin path (for example plugins that support custom external language servers).

Canonical server command:

- executable: `ruff`
- args: `lsp`

Suggested language mapping:

- file extension: `.ruff`
- language id: `ruff`

Notes:

- Keep this adapter thin. Do not duplicate parsing, linting, or symbol analysis in IDE-specific code.
- The IDE adapter should only launch/configure `ruff lsp` and forward protocol payloads.
