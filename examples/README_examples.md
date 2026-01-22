# Ruff Examples - Comprehensive Feature Showcase

This directory contains example programs demonstrating Ruff's features.

## 🎯 Featured Examples (New v0.3.0 Features)

These examples showcase the latest features: **lexical scoping**, **user input**, **type conversion**, and **file I/O**.

### Interactive Applications

#### 📝 **note_taking_app.ruff**
A complete note-taking application with menu system.
- **Features**: Create, read, list, and append to notes
- **Demonstrates**: File I/O, user input, directory operations, loops, lexical scoping
- **Try it**: `cargo run --quiet -- run examples/note_taking_app.ruff`

#### 📊 **student_grade_tracker.ruff**
Track student grades with data persistence.
- **Features**: Add grades, view all entries, validate input
- **Demonstrates**: File I/O, parse_int, error handling, input validation
- **Try it**: `cargo run --quiet -- run examples/student_grade_tracker.ruff`

#### 💰 **expense_tracker.ruff**
Personal expense tracking system.
- **Features**: Add expenses, view history, accumulator pattern
- **Demonstrates**: parse_float, file operations, lexical scoping
- **Try it**: `cargo run --quiet -- run examples/expense_tracker.ruff`

#### 🎮 **quiz_game.ruff**
Interactive programming quiz with score tracking.
- **Features**: Multiple question types, score calculation, percentage display
- **Demonstrates**: User input, parse_int, lexical scoping, accumulators
- **Try it**: `cargo run --quiet -- run examples/quiz_game.ruff`

#### 🔐 **password_generator.ruff**
Simple password generator and storage manager.
- **Features**: Generate passwords, store credentials, view entries
- **Demonstrates**: Loops, file I/O, parse_int, conditional logic
- **Try it**: `cargo run --quiet -- run examples/password_generator.ruff`

#### 💾 **backup_tool.ruff**
Automated backup utility for directories.
- **Features**: List files, copy files, create backup logs
- **Demonstrates**: Directory operations, list_dir, file_exists, error handling
- **Try it**: `cargo run --quiet -- run examples/backup_tool.ruff`

### Quick Start Examples

#### 🎲 **guessing_game.ruff** _(Original)_
Number guessing game with input validation.
- `cargo run --quiet -- run examples/guessing_game.ruff`

#### 🧮 **interactive_calculator.ruff** _(Original)_
Calculator supporting +, -, *, / operations.
- `cargo run --quiet -- run examples/interactive_calculator.ruff`

#### 👋 **interactive_greeting.ruff** _(Original)_
Simple greeting with name and age input.
- `cargo run --quiet -- run examples/interactive_greeting.ruff`

### File I/O Basics

#### 📄 **file_logger.ruff**
Simple logging with write, append, and read operations.
- `cargo run --quiet -- run examples/file_logger.ruff`

#### 📁 **directory_tools.ruff**
Directory creation, listing, and file existence checks.
- `cargo run --quiet -- run examples/directory_tools.ruff`

#### ⚙️ **config_manager.ruff**
Configuration file management with error handling.
- `cargo run --quiet -- run examples/config_manager.ruff`

## 📚 Core Language Examples

### Data Structures
- **arrays.ruff** - Array operations and methods
- **dictionaries.ruff** - Dictionary/hash map operations
- **collections.ruff** - Collections overview

### Control Flow
- **for_loops.ruff** - For-in iteration
- **test_if_else.ruff** - Conditional statements
- **pattern_matching.ruff** - Match/case statements

### Structs & Methods
- **struct_basic.ruff** - Basic struct definitions
- **struct_methods.ruff** - Methods on structs
- **structs_comprehensive.ruff** - Complete struct features
- **struct_nested.ruff** - Nested struct instances

### Error Handling
- **error_handling.ruff** - Try/except basics
- **error_handling_comprehensive.ruff** - Advanced error handling
- **try_throw.ruff** - Throwing and catching errors

### Type System
- **type_annotations.ruff** - Type annotation examples
- **type_inference.ruff** - Type inference demonstration
- **type_errors.ruff** - Type checking errors

### Functions & Modules
- **basic_import.ruff** - Module imports
- **selective_import.ruff** - Importing specific functions
- **math_module.ruff** - Using math functions

## 🎨 Advanced Examples

### Project Templates (examples/projects/)
- **todo_manager.ruff** - Complete TODO list application
- **contact_manager.ruff** - Contact management system

## 🚀 Running Examples

```bash
# Interactive examples (with user input)
cargo run --quiet -- run examples/note_taking_app.ruff
cargo run --quiet -- run examples/quiz_game.ruff
cargo run --quiet -- run examples/expense_tracker.ruff

# Non-interactive demonstrations
cargo run --quiet -- run examples/file_logger.ruff
cargo run --quiet -- run examples/directory_tools.ruff
cargo run --quiet -- run examples/scoping.ruff
```

## 💡 Learning Path

1. **Start Here**: `hello.ruff`, `basic_import.ruff`
2. **Control Flow**: `test_if_else.ruff`, `for_loops.ruff`
3. **Data Structures**: `arrays.ruff`, `dictionaries.ruff`
4. **User Input**: `interactive_greeting.ruff`, `guessing_game.ruff`
5. **File I/O**: `file_logger.ruff`, `config_manager.ruff`
6. **Complete Apps**: `note_taking_app.ruff`, `quiz_game.ruff`

## 📖 Feature Coverage

| Feature | Examples |
|---------|----------|
| **Lexical Scoping** | scoping.ruff, quiz_game.ruff, note_taking_app.ruff |
| **User Input** | All interactive_*.ruff, quiz_game.ruff |
| **Type Conversion** | parse_int: guessing_game.ruff, student_grade_tracker.ruff |
|  | parse_float: interactive_calculator.ruff, expense_tracker.ruff |
| **File I/O (Read/Write)** | file_logger.ruff, note_taking_app.ruff, backup_tool.ruff |
| **Directory Operations** | directory_tools.ruff, backup_tool.ruff |
| **Error Handling** | All try/except examples, config_manager.ruff |
| **Structs** | struct_*.ruff examples |
| **Pattern Matching** | pattern_matching.ruff |
| **Arrays & Dicts** | arrays.ruff, dictionaries.ruff, collections.ruff |

## 🔧 Tips

- Use `--quiet` flag with cargo to hide compilation messages
- Interactive examples wait for user input - press Ctrl+C to exit
- File I/O examples create temporary files in `/tmp/`
- Check each example's comments for detailed explanations

---

**New to Ruff?** Start with `hello.ruff` and `interactive_greeting.ruff` to get a feel for the language!
