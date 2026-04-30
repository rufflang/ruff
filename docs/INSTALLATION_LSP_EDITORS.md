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

## Clean-Environment Smoke Validation

Minimal smoke sequence:

```bash
./target/release/ruff lsp --help
cargo test --test editor_adapter_contracts
```

This validates shipped binary includes LSP entrypoint and adapter descriptors remain canonical.
