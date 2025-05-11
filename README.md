# 🐾 Ruff Programming Language

**Ruff** is a lean, expressive programming language built from scratch in Rust. It borrows inspiration from Go, Python, and functional design — but stands on its own.

> This repo is under active development. Consider everything experimental until v0.1.0 is released.

---

## 🧩 Installation

See [Install Guide](docs/install.md) for platform setup instructions.

---

## 🚀 Getting Started

Install Rust and run:

```bash
cargo run -- run examples/your_script.ruff
```

---

## 📄 Writing `.ruff` Scripts

Example:

```ruff
enum Result {
    Ok,
    Err
}

func check(x) {
    if x > 0 {
        return Result::Ok("great")
    }
    return Result::Err("bad")
}

res := check(42)

match res {
    case Result::Ok(msg): {
        print("✓", msg)
    }
    case Result::Err(err): {
        print("✗", err)
    }
}
```

---

## 🧪 Running Tests

Place test files in the `tests/` directory. Each `.ruff` file can have a matching `.out` file for expected output:

```bash
cargo run -- test
```

To regenerate expected `.out` snapshots:

```bash
cargo run -- test --update
```

---

## 🧠 Language Features

* ✅ Mutable/const variables
* ✅ Functions with `return`
* ✅ Pattern matching
* ✅ Enums + tagged values
* ✅ Nested matches
* ✅ `try`/`throw` error handling
* ✅ CLI testing framework

---

## 📦 Roadmap

* [ ] Type annotations
* [ ] Structs & modules
* [ ] Package manager
* [ ] WebAssembly backend
* [ ] REPL
* [ ] LSP integration

---

## 👨‍💼 Contributing

View the [CONTRIBUTING](CONTRIBUTING.md)