# Ruff Language Tools VS Code Extension

This extension provides:

- Ruff language registration for `.ruff` files
- TextMate-based syntax highlighting for Ruff source
- Optional `ruff lsp` client wiring for language intelligence

## Development Setup

```bash
cd tools/vscode-ruff-extension
npm install
```

## Run Locally In Extension Host

1. Open this extension folder in VS Code.
2. Press `F5` to launch an Extension Development Host.
3. Open any `.ruff` file in the host window.

Expected result:

- Language mode shows `Ruff`
- Syntax highlighting is active

## LSP Configuration

Default command:

```json
"ruff.lsp.command": ["ruff", "lsp"]
```

If Ruff is not on PATH, point to an explicit binary path:

```json
"ruff.lsp.command": ["/absolute/path/to/ruff", "lsp"]
```

## Package As VSIX

```bash
npm install -g @vscode/vsce
vsce package
```

Then install the generated `.vsix` in VS Code/Cursor/Codex-compatible editors.

## Extension Settings

- `ruff.lsp.enabled`
- `ruff.lsp.command`
- `ruff.lsp.trace.server`
