# 🧩 Installation

📚 Return to [README](../README.md) for full language overview.

> Looking for how to get Ruff installed on your system? You’re in the right place.

You can install the Ruff programming language on **macOS**, **Linux**, or **Windows** using Homebrew, Scoop, or direct binary download.

---

## 🥜 Homebrew (macOS / Linux)

```bash
brew tap Rufflang/tap
brew install Ruff
```

Verify:

```bash
Ruff --version
```

---

## 💪 Scoop (Windows)

```powershell
scoop bucket add Ruff https://github.com/Rufflang/scoop-bucket
scoop install Ruff
```

Verify:

```powershell
Ruff --version
```

---

## 📄 Manual Download

Go to the [Releases](https://github.com/Rufflang/Ruff/releases) page and download the appropriate zip:

* `Ruff-vX.Y.Z-linux-x64.zip`
* `Ruff-vX.Y.Z-macos-universal.zip`
* `Ruff-vX.Y.Z-win64.zip`

Then unzip and move the binary into your `PATH`:

```bash
mv Ruff /usr/local/bin/Ruff
```

---

## 🧠 From the README

To quickly return here, see the [Install Guide](docs/install.md) from the main README.

---

## 🚀 Running

```bash
Ruff run yourfile.Ruff
```

To run tests:

```bash
Ruff test
```

To update expected test output:

```bash
Ruff test --update
```

---

You're ready to Ruff.
