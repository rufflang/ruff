# Install/Upgrade Path For Editor LSP Integrations

Status: v0.13.0

## Install From Source

```bash
git clone https://github.com/rufflang/ruff.git
cd ruff
cargo build --release
./target/release/ruff --version
```

## Verify LSP Entrypoint

```bash
./target/release/ruff lsp --help
```

If the command prints LSP usage/help, the release artifact includes LSP functionality.

## Upgrade Path

Repeat build from latest source revision:

```bash
git pull --ff-only
cargo build --release
./target/release/ruff --version
```

Then keep editor adapter command stable:

- executable: `ruff`
- args: `lsp`

## Editor Integration References

- VS Code/Cursor, Neovim, JetBrains baseline docs:
  - `docs/EDITOR_ADAPTER_BASELINES.md`
  - `docs/editor-adapters/`

## VS Code / Cursor / Codex Extension Path

Build/install the first-party Ruff extension baseline:

```bash
cd tools/vscode-ruff-extension
npm install
npm install -g @vscode/vsce
vsce package
```

Install generated `.vsix` in your editor.

After install, opening a `.ruff` file should immediately enable Ruff language mode and syntax colorization.

Optional workspace settings baseline:

- `docs/editor-adapters/vscode-cursor-settings.json`

## Clean-Environment Smoke Validation

Minimal smoke sequence:

```bash
./target/release/ruff lsp --help
cargo test --test editor_adapter_contracts
```

Extension smoke sequence:

```bash
cd tools/vscode-ruff-extension
npm install
npm run check
```

This validates shipped binary includes LSP entrypoint and adapter descriptors remain canonical.
