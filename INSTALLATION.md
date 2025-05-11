# 🧩 Installation

📚 Return to [README](../README.md) for full language overview.

> Looking for how to get RUFF installed on your system? You’re in the right place.

You can install the RUFF programming language on **macOS**, **Linux**, or **Windows** using Homebrew, Scoop, or direct binary download.

---

## 🥜 Homebrew (macOS / Linux)

```bash
brew tap rufflang/tap
brew install ruff
```

Verify:

```bash
ruff --version
```

---

## 💪 Scoop (Windows)

```powershell
scoop bucket add ruff https://github.com/rufflang/scoop-bucket
scoop install ruff
```

Verify:

```powershell
ruff --version
```

---

## 📄 Manual Download

Go to the [Releases](https://github.com/rufflang/ruff/releases) page and download the appropriate zip:

* `ruff-vX.Y.Z-linux-x64.zip`
* `ruff-vX.Y.Z-macos-universal.zip`
* `ruff-vX.Y.Z-win64.zip`

Then unzip and move the binary into your `PATH`:

```bash
mv ruff /usr/local/bin/ruff
```

---

## 🧠 From the README

To quickly return here, see the [Install Guide](docs/install.md) from the main README.

---

## 🚀 Running

```bash
ruff run yourfile.ruff
```

To run tests:

```bash
ruff test
```

To update expected test output:

```bash
ruff test --update
```

---

You're ready to RUFF.
