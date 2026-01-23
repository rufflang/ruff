# Example Projects

This directory contains demonstration scripts that showcase Ruff's language features through practical examples. These are **code demonstrations**, not interactive applications - they run once and show you what Ruff can do.

## Projects

### 1. Todo Manager (`todo_manager.ruff`)
A demonstration script showing:
- **Structs**: `Todo` with typed fields  
- **Arrays**: Managing lists of todos
- **Control flow**: Loops and conditionals
- **Field mutation**: `todos[0].done := true`

**What it does:**
- Creates hardcoded todos in the script
- Displays them with checkbox format
- Mutates a field to mark one complete
- Shows the updated list

*Note: Not interactive - no user input yet. Edit the .ruff file to change todos.*

**Run it:**
```bash
cargo run --quiet -- run examples/projects/todo_manager.ruff
```

### 2. Contact Manager (`contact_manager.ruff`)
A demonstration script showcasing:
- **Dictionaries**: Fast contact lookup by name
- **Structs**: Contact data structure
- **String functions**: Search with `contains()`, case handling
- **Error handling**: Try/except for missing data
- **Dict operations**: `keys()`, `has_key()`

**What it does:**
- Creates hardcoded contacts in the script
- Demonstrates dictionary operations
- Shows filtering and searching
- Displays error handling patterns

*Note: Not interactive - demonstrates language features with preset data.*

**Run it:**
```bash
cargo run --quiet -- run examples/projects/contact_manager.ruff
```

### 3. Data Analyzer (`data_analyzer.ruff`) ✨ NEW
A demonstration script showcasing:
- **Array higher-order functions**: `map()`, `filter()`, `reduce()`, `find()`
- **Anonymous functions**: `func(x) { return x * 2 }`
- **Functional programming**: Data transformation and aggregation
- **Statistics**: Calculate sum, average, min, max
- **Advanced filtering**: Multi-step data processing

**What it does:**
- Analyzes dataset with validation
- Filters valid data points
- Calculates comprehensive statistics
- Performs category-based analysis
- Demonstrates function chaining
- Computes performance scores

*Note: Perfect example of functional programming in Ruff!*

**Run it:**
```bash
cargo run --quiet -- run examples/projects/data_analyzer.ruff
```

### 4. Log Parser (`log_parser.ruff`) ✨ NEW
A demonstration script showcasing:
- **File I/O**: `read_file()`, `write_file()`, `read_lines()`
- **String functions**: `split()`, `join()`, `contains()`, `substring()`
- **Data extraction**: Parse structured log entries
- **Pattern matching**: Find specific log patterns
- **Report generation**: Create summary reports

**What it does:**
- Creates sample log file
- Parses log entries into structured data
- Filters by log level (ERROR, WARNING, INFO)
- Searches for specific patterns
- Extracts email addresses
- Generates summary report file

*Note: Demonstrates real-world text processing and file operations.*

**Run it:**
```bash
cargo run --quiet -- run examples/projects/log_parser.ruff
```

### 5. Inventory System (`inventory_system.ruff`) ✨ NEW
A demonstration script showcasing:
- **Complex structs**: Product data with multiple fields
- **Enum patterns**: Product status tracking
- **Advanced filtering**: Multi-condition product searches
- **Calculations**: Inventory valuation and analytics
- **Business logic**: Stock management rules

**What it does:**
- Manages product inventory with 8 sample products
- Tracks stock levels with status indicators
- Filters by category and stock level
- Identifies low stock and out of stock items
- Calculates total inventory value
- Finds most/least expensive products
- Simulates restocking operations

*Note: Great example of a business application structure.*

**Run it:**
```bash
cargo run --quiet -- run examples/projects/inventory_system.ruff
```

## Learning Path

1. **Start with Todo Manager** - Simplest introduction to structs and arrays
2. **Try Contact Manager** - Learn dictionary operations and string functions
3. **Explore Data Analyzer** - Master functional programming with higher-order functions
4. **Study Log Parser** - Learn file I/O and text processing techniques
5. **Examine Inventory System** - See complex business logic and calculations

## What These Projects Demonstrate

All projects use real-world patterns and demonstrate:
- ✅ Struct design with typed fields
- ✅ Collection manipulation (arrays and dicts)
- ✅ Function organization and modularity
- ✅ Control flow (loops, conditionals)
- ✅ String operations (`contains`, `split`, `join`, `substring`, etc.)
- ✅ Error handling with try/except
- ✅ Dictionary operations (`keys`, `has_key`)
- ✅ Array iteration
- ✅ **NEW:** Array higher-order functions (`map`, `filter`, `reduce`, `find`)
- ✅ **NEW:** Anonymous function expressions
- ✅ **NEW:** File I/O operations
- ✅ **NEW:** Complex data processing and analysis
- ✅ **NEW:** Text parsing and pattern matching

## Notes

**These are demonstrations, not applications:** The scripts run once to show language features. They use hardcoded data and don't accept user input. User input functions like `input()` are planned for future versions of Ruff.

These projects use only the standard Ruff features available in v0.2.0+. They demonstrate practical programming patterns that work reliably with the current implementation.

## Tips for Your Own Projects

- Use `[]` for array literals: `[1, 2, 3]`
- Use `{}` for dict literals: `{"key": "value"}`
- Use `#` for comments (not `//`)
- Print supports multiple args: `print("Value:", x)`
- Structs work great for organizing data
- Dicts provide fast lookups by key
- Arrays are perfect for ordered collections
- Field assignment works: `obj.field := value` and `arr[0].field := value`
- Booleans (`true`/`false`) work in if conditions
- If you see compiler warnings about unused code, those are harmless internal warnings
