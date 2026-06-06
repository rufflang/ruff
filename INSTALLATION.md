# Installation Guide

Return to [README](README.md) for full language overview.

> Looking for how to get Ruff installed on your system? You're in the right place.

Ruff is the Kujo core language/runtime built with Rust. You can install Ruff either from prebuilt release artifacts (recommended for users) or by building from source with Cargo.

If another `ruff` command is already on your machine, make sure you are using the binary from this repository so you do not confuse it with unrelated tools that share the same name.

---

## Prerequisites

Ruff requires **Rust 1.86+** to build from source.

Minimum release validation assumptions for current supported install flow:

- Rust stable `1.86+`
- Linux (`ubuntu-latest` baseline)
- macOS (`macos-latest` baseline)

See `docs/RELEASE_ARTIFACT_VALIDATION.md` for cross-platform clean-environment validation and checksum verification flow.

### Install Rust

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows:**  
Download and run the installer from [rustup.rs](https://rustup.rs/)

Verify Rust installation:
```bash
rustc --version
cargo --version
```

---

## Install From Prebuilt Release Artifacts (Recommended)

Use this path when consuming a tagged Ruff release.

Set the release tag and detect platform:

```bash
RUFF_VERSION="v1.0.0"

if [[ "$(uname -s)" == "Darwin" ]]; then
   RUFF_OS="macos"
else
   RUFF_OS="linux"
fi

RUFF_ARCH="$(uname -m)"
if [[ "${RUFF_ARCH}" == "aarch64" ]]; then
   RUFF_ARCH="arm64"
fi

RUFF_TARGET="${RUFF_OS}-${RUFF_ARCH}"
```

Download binary archive and checksum:

```bash
BASE_URL="https://github.com/rufflang/ruff/releases/download/${RUFF_VERSION}"
ARCHIVE="ruff-${RUFF_VERSION}-${RUFF_TARGET}.tar.gz"

curl -sSfL "${BASE_URL}/${ARCHIVE}" -o "${ARCHIVE}"
curl -sSfL "${BASE_URL}/${ARCHIVE}.sha256" -o "${ARCHIVE}.sha256"
```

Verify checksum:

```bash
if command -v sha256sum >/dev/null 2>&1; then
   sha256sum -c "${ARCHIVE}.sha256"
else
   shasum -a 256 -c "${ARCHIVE}.sha256"
fi
```

Install and verify commands:

```bash
mkdir -p ~/.local/bin
tar -xzf "${ARCHIVE}"
cp ruff ~/.local/bin/ruff
chmod +x ~/.local/bin/ruff

export PATH="$HOME/.local/bin:$PATH"
ruff --version
ruff run examples/hello.ruff
ruff lsp --help
```

## Build from Source

### 1. Clone the Repository

```bash
git clone https://github.com/rufflang/ruff.git
cd ruff
```

### 2. Build the Project

**Development build** (faster compilation, slower runtime):
```bash
cargo build
```

**Release build** (optimized, recommended for daily use):
```bash
cargo build --release
```

### 3. Run Ruff

**Without installing** (from project directory):
```bash
# Development build
cargo run -- run examples/hello.ruff

# Release build
./target/release/ruff run examples/hello.ruff
```

### 4. Install System-Wide (Optional)

**macOS / Linux:**
```bash
cargo install --path .
# Or manually copy the binary
sudo cp target/release/ruff /usr/local/bin/
```

**Windows (PowerShell as Administrator):**
```powershell
cargo install --path .
# Or manually copy the binary to a directory in your PATH
```

Verify installation:
```bash
ruff --version
```

---

## Platform-Specific Notes

### macOS

**Supported versions**: macOS 10.15 (Catalina) or later  
**Architectures**: Intel (x86_64) and Apple Silicon (ARM64)

If you encounter permissions issues:
```bash
sudo chown -R $(whoami) /usr/local/bin
```

### Linux

**Tested distributions**: Ubuntu 20.04+, Debian 11+, Fedora 35+, Arch Linux

**Dependencies**: None required beyond Rust toolchain

If you need to install to a user directory:
```bash
mkdir -p ~/.local/bin
cp target/release/ruff ~/.local/bin/
# Add to PATH in ~/.bashrc or ~/.zshrc:
export PATH="$HOME/.local/bin:$PATH"
```

### Windows

**Supported versions**: Windows 10 or later  
**Architectures**: x64

**Common issues**:
- If you get "VCRUNTIME140.dll missing" errors, install the [Visual C++ Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe)
- Ensure your PATH includes the directory where `ruff.exe` is located

---

## Quick Start

Once installed, you can run Ruff programs:

```bash
# Run a script
ruff run examples/hello.ruff

# Run tests
ruff test

# Update test snapshots
ruff test --update
```

---

## Future Installation Methods

The following installation methods are planned for future releases:

### Homebrew (macOS / Linux)
```bash
# Coming soon
brew tap rufflang/tap
brew install ruff
```

### Scoop (Windows)
```powershell
# Coming soon
scoop bucket add ruff https://github.com/rufflang/scoop-bucket
scoop install ruff
```

### Package Managers
- **apt** (Ubuntu/Debian): Planned
- **dnf** (Fedora): Planned
- **pacman** (Arch): Planned
- **winget** (Windows): Planned

---

## Troubleshooting

### Build Failures

**"linker \`cc\` not found"** (Linux)
```bash
# Ubuntu/Debian
sudo apt install build-essential

# Fedora
sudo dnf install gcc

# Arch
sudo pacman -S base-devel
```

**"failed to run custom build command"**
- Ensure you have the latest Rust version: `rustup update`
- Clean and rebuild: `cargo clean && cargo build --release`

### Runtime Issues

**"command not found: ruff"**
- Verify the binary is in your PATH
- Try running with full path: `/usr/local/bin/ruff` or `./target/release/ruff`

**Slow compilation**
- Use `cargo build` for development (faster compile, slower runtime)
- Use `cargo build --release` only when you need performance

### Getting Help

If you encounter issues:
1. Check [GitHub Issues](https://github.com/rufflang/ruff/issues)
2. Read the [Contributing Guide](CONTRIBUTING.md)
3. Open a new issue with:
   - Your OS and version
   - Rust version (`rustc --version`)
   - Full error message
   - Steps to reproduce

---

## Updating Ruff

### Built from Source
```bash
cd ruff
git pull
cargo build --release
# If installed system-wide:
sudo cp target/release/ruff /usr/local/bin/
```

### Via Package Manager (Future)
```bash
# Homebrew
brew upgrade ruff

# Scoop
scoop update ruff
```

---

## 🗑️ Uninstalling

### Cargo Install
```bash
cargo uninstall ruff
```

### Manual Installation
```bash
# macOS/Linux
sudo rm /usr/local/bin/ruff

# Windows - Delete ruff.exe from your installation directory
```

---

## Verification

After installation, verify everything works:

```bash
# Check version
ruff --version

# Run a test script
echo 'print("Hello, Ruff!")' > test.ruff
ruff run test.ruff

# Run test suite
cd /path/to/ruff/repo
ruff test
```

Expected output:
```
Hello, Ruff!
```

---

## Development Setup

For contributors and developers:

```bash
# Clone and setup
git clone https://github.com/rufflang/ruff.git
cd ruff

# Install dev dependencies
rustup component add rustfmt clippy

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy

# Build documentation
cargo doc --open
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed development guidelines.

---

**You're ready to start coding in Ruff! 🐾**

*For examples and language features, see the [README](README.md) and [examples/](examples/) directory.*
