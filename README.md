# 🐾 Ruff Programming Language

**Ruff** is a lean, expressive programming language built from scratch in Rust. It borrows inspiration from Go, Python, and functional design — but stands on its own.

> **Status**: Core functionality complete and tested. Ready for semi-production use with basic programs.

---

## 🎯 Project Status

### ✅ Implemented Features

* **Variables & Constants**
  - `let` and `mut` for mutable variables
  - `const` for constants
  - Shorthand assignment with `:=` (e.g., `x := 5`)

* **Functions**
  - Function definitions with `func` keyword
  - Parameter passing
  - Return values
  - Lexical scoping

* **Control Flow**
  - `if`/`else` statements
  - Pattern matching with `match`/`case`
  - `loop` and `for` loops
  - `try`/`except`/`throw` error handling

* **Data Types**
  - Numbers (f64)
  - Strings with escape sequences
  - Enums with tagged variants
  - Functions as first-class values

* **Operators**
  - Arithmetic: `+`, `-`, `*`, `/`
  - Comparison: `==`, `>`, `<`, `>=`, `<=`
  - String concatenation with `+`

* **Built-in Functions**
  - `print()` for output
  - `throw()` for error handling

* **Testing Framework**
  - Built-in test runner
  - Snapshot testing with `.out` files
  - Test result reporting

---

## 🧩 Installation

See [Install Guide](INSTALLATION.md) for platform setup instructions.

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