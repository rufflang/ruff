# CHANGELOG

All notable changes to the Ruff programming language will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-01-23

### Added
- **Operator Overloading**: Full support for custom operator behavior on structs via `op_` methods
  - **Arithmetic operators**: `op_add` (+), `op_sub` (-), `op_mul` (*), `op_div` (/), `op_mod` (%)
  - **Comparison operators**: `op_eq` (==), `op_ne` (!=), `op_lt` (<), `op_gt` (>), `op_lte` (<=), `op_gte` (>=)
  - Operator methods are called automatically when using operators on struct instances
  - Methods receive the right-hand operand as a parameter and can return any type
  - Example:
    ```ruff
    struct Vector {
        x: float,
        y: float,
        
        func op_add(other) {
            return Vector { x: x + other.x, y: y + other.y };
        }
        
        func op_mul(scalar) {
            return Vector { x: x * scalar, y: y * scalar };
        }
    }
    
    v1 := Vector { x: 1.0, y: 2.0 };
    v2 := Vector { x: 3.0, y: 4.0 };
    v3 := v1 + v2;  # Calls v1.op_add(v2), result: Vector { x: 4.0, y: 6.0 }
    v4 := v1 * 2.0;  # Calls v1.op_mul(2.0), result: Vector { x: 2.0, y: 4.0 }
    ```
  - See `examples/operator_overloading.ruff` for complete examples with Vector and Money types

- **Standard Library Enhancements**: Expanded built-in functions for common programming tasks
  
  **Error Properties**: Access detailed error information in except blocks
  - `err.message` - Get the error message as a string
    - Example: `try { throw("Failed") } except err { print(err.message) }` outputs `"Failed"`
  - `err.stack` - Access the call stack trace as an array
    - Example: Stack trace array shows function call chain leading to error
    - Each stack frame shows the function name
    - Useful for debugging nested function calls
  - `err.line` - Get the line number where error occurred (when available)
    - Example: `print(err.line)` shows line number
    - Returns 0 if line information not available
  
  **Custom Error Types**: Define custom error structs for domain-specific errors
  - Throw struct instances as errors
    - Example:
      ```ruff
      struct ValidationError {
          field: string,
          message: string
      }
      
      error := ValidationError {
          field: "email",
          message: "Email is required"
      }
      throw(error)
      ```
  - Error properties automatically available in except block
  - Enables type-specific error handling patterns
  
  **Error Chaining**: Create nested error contexts with cause information
  - Add `cause` field to error structs to preserve original error
    - Example:
      ```ruff
      try {
          risky_operation()
      } except original_err {
          error := DatabaseError {
              message: "Failed to process data",
              cause: original_err.message
          }
          throw(error)
      }
      ```
  - Maintains full error context through multiple layers
  - Essential for debugging complex error scenarios
  
  **Stack Traces**: Automatic call stack tracking in errors
  - Function call chain captured when error thrown
  - Access via `err.stack` array in except blocks
  - Each array element contains function name
  - Enables detailed debugging of error origins
  
  **Examples**:
  - `examples/error_handling_enhanced.ruff` - Complete demonstration of all error handling features
  - `examples/test_errors_simple.ruff` - Quick test of error properties
  
  **Use Cases**:
  - Input validation with detailed error messages
  - API error handling with status codes
  - File operation error recovery
  - Database connection error management
  - Multi-layer error context preservation

---

## [0.4.0] - 2026-01-23

### Added
- **Standard Library Enhancements**: Expanded built-in functions for common programming tasks
  
  **Math & Random Functions**:
  - `random()` - Generate random float between 0.0 and 1.0
    - Example: `r := random()` returns `0.7234891`
    - Uses Rust's rand crate for cryptographically secure randomness
  - `random_int(min, max)` - Generate random integer in range (inclusive)
    - Example: `dice := random_int(1, 6)` returns random number 1-6
    - Example: `temp := random_int(-10, 35)` for temperature simulation
    - Both endpoints are inclusive
  - `random_choice(array)` - Select random element from array
    - Example: `color := random_choice(["red", "blue", "green"])` picks random color
    - Example: `card := random_choice(deck)` for card game
    - Returns 0 if array is empty
  
  **Date/Time Functions**:
  - `now()` - Get current Unix timestamp (seconds since epoch)
    - Example: `timestamp := now()` returns `1737610854`
    - Returns float for precision
  - `format_date(timestamp, format_string)` - Format timestamp as readable date
    - Example: `format_date(now(), "YYYY-MM-DD")` returns `"2026-01-23"`
    - Example: `format_date(now(), "YYYY-MM-DD HH:mm:ss")` returns `"2026-01-23 14:30:45"`
    - Supports patterns: YYYY (year), MM (month), DD (day), HH (hour), mm (minute), ss (second)
    - Custom formats: `"DD/MM/YYYY"`, `"MM-DD-YYYY HH:mm"`, etc.
  - `parse_date(date_string, format)` - Parse date string to Unix timestamp
    - Example: `ts := parse_date("2026-01-23", "YYYY-MM-DD")` converts to timestamp
    - Example: `birthday := parse_date("1990-05-15", "YYYY-MM-DD")` for age calculations
    - Returns 0.0 for invalid dates
    - Enables date arithmetic: `days_diff := (date2 - date1) / (24 * 60 * 60)`
  
  **System Operations**:
  - `env(var_name)` - Get environment variable value
    - Example: `home := env("HOME")` returns `"/Users/username"`
    - Example: `path := env("PATH")` gets system PATH
    - Returns empty string if variable not set
  - `args()` - Get command-line arguments as array
    - Example: `cli_args := args()` returns `["arg1", "arg2", "arg3"]`
    - Program name is excluded (only actual arguments)
    - Returns empty array if no arguments
  - `exit(code)` - Exit program with status code
    - Example: `exit(0)` for successful exit
    - Example: `exit(1)` for error exit
    - Standard Unix exit codes: 0 = success, non-zero = error
  - `sleep(milliseconds)` - Pause execution for specified time
    - Example: `sleep(1000)` sleeps for 1 second
    - Example: `sleep(100)` sleeps for 100ms
    - Useful for rate limiting, animations, polling
  - `execute(command)` - Execute shell command and return output
    - Example: `output := execute("ls -la")` runs shell command
    - Example: `date := execute("date")` gets system date
    - Cross-platform: uses cmd.exe on Windows, sh on Unix
    - Returns command output as string
    - Use with caution - potential security implications
  
  **Path Operations**:
  - `join_path(parts...)` - Join path components with correct separator
    - Example: `path := join_path("/home", "user", "file.txt")` returns `"/home/user/file.txt"`
    - Example: `config := join_path(home, ".config", "app", "settings.json")`
    - Handles platform-specific separators automatically
    - Variadic - accepts any number of string arguments
  - `dirname(path)` - Extract directory from path
    - Example: `dirname("/home/user/file.txt")` returns `"/home/user"`
    - Example: `dirname("src/main.rs")` returns `"src"`
    - Returns "/" for root paths
  - `basename(path)` - Extract filename from path
    - Example: `basename("/home/user/file.txt")` returns `"file.txt"`
    - Example: `basename("README.md")` returns `"README.md"`
    - Works with both absolute and relative paths
  - `path_exists(path)` - Check if file or directory exists
    - Example: `exists := path_exists("config.json")` returns boolean
    - Example: `if path_exists(log_file) { ... }` for conditional logic
    - Works for both files and directories

  **Implementation Details**:
  - Dependencies added: `rand = "0.8"`, `chrono = "0.4"`
  - All functions integrated into interpreter and type checker
  - Comprehensive error handling with descriptive messages
  - Cross-platform compatibility (Windows, macOS, Linux)
  
  **Examples & Tests**:
  - `examples/random_generator.ruff` - Random number generation, password generator, lottery numbers
  - `examples/datetime_utility.ruff` - Date formatting, parsing, calculations, age calculator
  - `examples/path_utilities.ruff` - Path building, component extraction, existence checking
  - `examples/system_info.ruff` - Environment variables, command execution, timing
  - `tests/test_stdlib_random.ruff` - 60+ test cases for random functions
  - `tests/test_stdlib_datetime.ruff` - 50+ test cases for date/time functions
  - `tests/test_stdlib_paths.ruff` - 40+ test cases for path operations
  - `tests/test_stdlib_system.ruff` - 30+ test cases for system operations

- **Regular Expressions**: Pattern matching and text processing with regex support
  
  **Regex Functions**:
  - `regex_match(text, pattern)` - Check if text matches regex pattern
    - Example: `regex_match("user@example.com", "^[a-zA-Z0-9._%+-]+@")` checks email format
    - Example: `regex_match("555-1234", "^\\d{3}-\\d{4}$")` validates phone numbers
    - Returns boolean true/false for match result
    - Use cases: input validation, format checking, data verification
  
  - `regex_find_all(text, pattern)` - Find all matches of pattern in text
    - Example: `regex_find_all("Call 555-1234 or 555-5678", "\\d{3}-\\d{4}")` returns `["555-1234", "555-5678"]`
    - Example: `regex_find_all("Extract #tags from #text", "#\\w+")` returns `["#tags", "#text"]`
    - Returns array of matched strings
    - Use cases: data extraction, parsing, finding patterns
  
  - `regex_replace(text, pattern, replacement)` - Replace pattern matches
    - Example: `regex_replace("Call 555-1234", "\\d{3}-\\d{4}", "XXX-XXXX")` returns `"Call XXX-XXXX"`
    - Example: `regex_replace("too  many   spaces", " +", " ")` normalizes whitespace
    - Replaces all occurrences of pattern
    - Use cases: data sanitization, redaction, text normalization
  
  - `regex_split(text, pattern)` - Split text by regex pattern
    - Example: `regex_split("one123two456three", "\\d+")` returns `["one", "two", "three"]`
    - Example: `regex_split("word1   word2\tword3", "\\s+")` splits by any whitespace
    - Returns array of text segments between matches
    - Use cases: tokenization, parsing structured data, CSV processing
  
  **Pattern Features**:
  - Full Rust regex syntax support
  - Character classes: `\\d` (digit), `\\w` (word), `\\s` (space)
  - Quantifiers: `+` (one or more), `*` (zero or more), `?` (optional), `{n,m}` (range)
  - Anchors: `^` (start), `$` (end), `\\b` (word boundary)
  - Groups: `(...)` for capturing, `(?:...)` for non-capturing
  - Alternation: `|` for OR patterns
  - Escape special chars: `\\.`, `\\(`, `\\)`, etc.
  
  **Implementation Details**:
  - Uses Rust's regex crate (v1.x) for performance and reliability
  - Compiled regex patterns cached internally
  - Invalid patterns return safe defaults (false/empty for matches, original text for replace)
  - Full Unicode support
  - Case-sensitive by default
  
  **Examples & Tests**:
  - `examples/validator.ruff` - Email, phone, and URL validation with contact extraction
  - `examples/log_parser_regex.ruff` - Log file parsing, filtering, and data extraction
  - `tests/test_regex.ruff` - 60+ comprehensive test cases covering all functions
  - `tests/test_regex_simple.ruff` - Basic functionality tests
  
  **Common Use Cases**:
  - Email and phone number validation
  - URL parsing and extraction
  - Log file analysis and filtering
  - Data extraction from unstructured text
  - Input sanitization and validation
  - Text normalization and cleanup
  - CSV and structured data parsing

### Fixed
- **Parser**: Fixed parser not skipping semicolons in function/method bodies
  - Previously, function bodies would stop parsing after the first statement when using semicolons
  - This bug prevented multi-statement methods and functions from working correctly
  - Now semicolons are properly skipped, allowing multiple statements in function bodies
  
- **Interpreter**: Fixed ExprStmt not routing Call expressions through eval_expr properly
  - Method calls as statements (e.g., `obj.method();`) now work correctly
  - Void methods (methods without return statements) now execute properly
  - This fix was critical for operator overloading and general struct method usage

- **Parser**: Fixed struct field values to support full expressions instead of just literals
  - Struct instantiation now supports computed field values: `Vec2 { x: a + b, y: c * 2.0 }`
  - Previously only literals and identifiers were allowed in struct field values
  - This enables operator overloading methods to create and return new struct instances

### Changed
- **Operator Method Naming**: Using `op_` prefix instead of Python-style `__` dunder names
  - More explicit and easier to read: `op_add` vs `__add__`
  - Consistent with Ruff's naming conventions for special methods
  - Clear indication that these are operator overload methods

---

## [0.3.0] - 2026-01-23

### Added
- **JSON Support**: Native JSON parsing and serialization functions
  - New built-in function `parse_json(json_string)` - parses JSON strings into Ruff values
  - New built-in function `to_json(value)` - converts Ruff values to JSON strings
  - Full support for JSON data types: objects, arrays, strings, numbers, booleans, null
  - JSON objects convert to/from Ruff dictionaries
  - JSON arrays convert to/from Ruff arrays
  - JSON null converts to Ruff Number(0.0) by convention
  - Handles nested structures and complex data
  - Error handling for invalid JSON with descriptive error messages
  - Round-trip conversion support (parse → modify → serialize)
  - Example: `data := parse_json("{\"name\": \"Alice\", \"age\": 30}")`
  - Example: `json_str := to_json({"status": "ok", "data": [1, 2, 3]})`
  - Uses serde_json library for reliable JSON processing
- **Multi-Line Comments**: Support for block comments spanning multiple lines
  - Syntax: `/* comment */` for single or multi-line comments
  - Example: `/* This is a comment */`
  - Example multi-line:
    ```ruff
    /*
     * This comment spans
     * multiple lines
     */
    ```
  - Useful for longer explanations, commenting out code blocks, license headers
  - Comments do not nest - first `*/` closes the comment
  - Can be placed inline: `x := 10 /* inline comment */ + 5`
  - Properly tracks line numbers for multi-line comments in error reporting
  - Lexer handles `/*` and `*/` patterns correctly
- **Doc Comments**: Documentation comments for code documentation
  - Syntax: `///` at start of line for documentation comments
  - Example:
    ```ruff
    /// Calculates the factorial of a number
    /// @param n The number to calculate factorial for
    /// @return The factorial of n
    func factorial(n) {
        if n <= 1 { return 1 }
        return n * factorial(n - 1)
    }
    ```
  - Typically used to document functions, structs, and modules
  - Supports common documentation tags: `@param`, `@return`, `@example`
  - Can be used for inline documentation of struct fields
  - Future versions may extract these for automatic documentation generation
- **Enhanced Comment Support**: All comment types work together seamlessly
  - Single-line comments: `# comment`
  - Multi-line comments: `/* comment */`
  - Doc comments: `/// comment`
  - Comments can be mixed in the same file
  - All comment types properly ignored by lexer during tokenization
  - Comprehensive test coverage: 4 test files covering all comment scenarios
  - Example file: `examples/comments.ruff` demonstrating all comment types and best practices
  - Examples include practical use cases, style guidelines, and documentation patterns
- **Array Higher-Order Functions**: Functional programming operations on arrays for data transformation and processing
  - `map(array, func)`: Transform each element by applying a function, returns new array
    - Example: `map([1, 2, 3], func(x) { return x * x })` returns `[1, 4, 9]`
    - Example: `map(["hello", "world"], func(w) { return to_upper(w) })` returns `[HELLO, WORLD]`
    - Function receives each element as parameter, return value becomes new element
    - Original array is unchanged (immutable operation)
  - `filter(array, func)`: Select elements where function returns truthy value, returns new array
    - Example: `filter([1, 2, 3, 4], func(x) { return x % 2 == 0 })` returns `[2, 4]`
    - Example: `filter(["Alice", "Bob", "Charlie"], func(n) { return len(n) < 6 })` returns `[Alice, Bob]`
    - Function returns boolean or truthy value to determine inclusion
    - Returns empty array if no elements match
  - `reduce(array, initial, func)`: Accumulate array elements into single value
    - Example: `reduce([1, 2, 3, 4, 5], 0, func(acc, x) { return acc + x })` returns `15`
    - Example: `reduce([2, 3, 4], 1, func(acc, x) { return acc * x })` returns `24`
    - Example: `reduce(["R", "u", "f", "f"], "", func(acc, l) { return acc + l })` returns `Ruff`
    - Function receives accumulator and current element, returns new accumulator value
    - Initial value sets starting accumulator and return type
  - `find(array, func)`: Return first element where function returns truthy value
    - Example: `find([10, 20, 30, 40], func(x) { return x > 25 })` returns `30`
    - Example: `find(["apple", "banana", "cherry"], func(f) { return starts_with(f, "c") })` returns `cherry`
    - Returns `0` if no element matches (null equivalent)
    - Stops searching after first match for efficiency
  - Supports chaining: `reduce(map(filter(arr, f1), f2), init, f3)` for complex transformations
  - Anonymous function expressions: `func(x) { return x * 2 }` can be used inline
  - All functions work with mixed-type arrays (numbers, strings, booleans)
  - Type checker support with function signatures
  - 20 comprehensive integration tests covering all functions and edge cases
  - Example program: `examples/array_higher_order.ruff` with practical use cases including:
    - Data transformation (temperature conversion, string manipulation)
    - Filtering and validation (even numbers, positive values, string length)
    - Aggregation (sum, product, average, max/min)
    - Search operations (first match, existence checks)
    - Real-world scenarios (student scores, price calculations, data processing)
  - Syntax:
    ```ruff
    # Transform data
    squared := map([1, 2, 3, 4, 5], func(x) { return x * x })
    
    # Filter data
    evens := filter([1, 2, 3, 4, 5, 6], func(n) { return n % 2 == 0 })
    
    # Aggregate data
    sum := reduce([1, 2, 3, 4, 5], 0, func(acc, x) { return acc + x })
    
    # Find data
    first_large := find([10, 20, 30, 40], func(x) { return x > 25 })
    
    # Chain operations
    result := reduce(
        map(
            filter(data, func(x) { return x > 0 }),
            func(x) { return x * 2 }
        ),
        0,
        func(acc, x) { return acc + x }
    )
    ```
- **Anonymous Function Expressions**: Support for inline function definitions in expression contexts
  - Syntax: `func(param1, param2) { body }` can be used as an expression
  - Compatible with all higher-order functions (map, filter, reduce, find)
  - Supports lexical scoping with access to outer variables
  - Optional type annotations: `func(x: int) -> int { return x * 2 }`
  - Functions are first-class values that can be stored, passed, and returned
- **Enhanced String Functions**: Six new string manipulation functions for common string operations
  - `starts_with(str, prefix)`: Check if string starts with prefix, returns boolean
    - Example: `starts_with("hello world", "hello")` returns `true`
    - Example: `starts_with("test.ruff", "hello")` returns `false`
  - `ends_with(str, suffix)`: Check if string ends with suffix, returns boolean
    - Example: `ends_with("test.ruff", ".ruff")` returns `true`
    - Example: `ends_with("photo.png", ".jpg")` returns `false`
  - `index_of(str, substr)`: Find first occurrence of substring, returns index or -1
    - Example: `index_of("hello world", "world")` returns `6.0`
    - Example: `index_of("hello", "xyz")` returns `-1.0`
    - Returns position of first match for repeated substrings
  - `repeat(str, count)`: Repeat string count times, returns concatenated string
    - Example: `repeat("ha", 3)` returns `"hahaha"`
    - Example: `repeat("*", 10)` returns `"**********"`
  - `split(str, delimiter)`: Split string by delimiter, returns array of strings
    - Example: `split("a,b,c", ",")` returns `["a", "b", "c"]`
    - Example: `split("one two three", " ")` returns `["one", "two", "three"]`
    - Works with multi-character delimiters: `split("hello::world", "::")`
  - `join(array, separator)`: Join array elements with separator, returns string
    - Example: `join(["a", "b", "c"], ",")` returns `"a,b,c"`
    - Example: `join([1, 2, 3], "-")` returns `"1-2-3"`
    - Converts non-string elements (numbers, booleans) to strings automatically
  - All functions implemented in Rust for performance
  - Type checker support for all functions with proper type signatures
  - 14 comprehensive integration tests covering all functions and edge cases
  - Example program: `examples/string_functions.ruff` with practical use cases
  - Syntax:
    ```ruff
    # Check file extensions
    is_ruff := ends_with("script.ruff", ".ruff")  # true
    
    # Process CSV data
    fields := split("Alice,30,Engineer", ",")
    name := fields[0]  # "Alice"
    
    # Build strings from arrays
    words := ["Ruff", "is", "awesome"]
    sentence := join(words, " ")  # "Ruff is awesome"
    
    # Search in strings
    pos := index_of("hello world", "world")  # 6
    
    # Generate patterns
    border := repeat("=", 20)  # "===================="
    
    # URL validation
    is_secure := starts_with(url, "https://")
    ```
- **String Interpolation**: Embed expressions directly in strings with `${}` syntax
  - Interpolate variables: `"Hello, ${name}!"` produces `"Hello, World!"`
  - Interpolate numbers: `"The answer is ${x}"` produces `"The answer is 42"`
  - Interpolate expressions: `"Result: ${x * 2}"` produces `"Result: 84"`
  - Interpolate function calls: `"Double of ${n} is ${double(n)}"`
  - Interpolate comparisons: `"Valid: ${x > 5}"` produces `"Valid: true"`
  - Multiple interpolations: `"Name: ${first} ${last}, Age: ${age}"`
  - Struct field access: `"Hello, ${person.name}!"`
  - Parenthesized expressions: `"Result: ${(a + b) * c}"`
  - Lexer tokenizes interpolated strings as `InterpolatedString` with text and expression parts
  - Parser converts expression strings to AST nodes for evaluation
  - Interpreter evaluates embedded expressions and converts to strings
  - Type checker validates embedded expressions and infers String type
  - 15 comprehensive integration tests covering all interpolation patterns
  - Example program: `examples/string_interpolation.ruff`
  - Syntax:
    ```ruff
    name := "Alice"
    age := 30
    message := "Hello, ${name}! You are ${age} years old."
    print(message)  # "Hello, Alice! You are 30 years old."
    
    # With expressions
    x := 10
    y := 5
    result := "Sum: ${x + y}, Product: ${x * y}"
    print(result)  # "Sum: 15, Product: 50"
    ```
- **Parenthesized Expression Grouping**: Parser now supports `(expr)` for grouping expressions
  - Enables precedence control: `(a + b) * c` evaluates addition first
  - Works in all expression contexts including string interpolation
  - Properly handles nested parentheses
- **Loop Control Statements**: Full support for `while` loops, `break`, and `continue`
  - `while condition { ... }`: Execute loop while condition is truthy
  - `break`: Exit current loop immediately
  - `continue`: Skip to next iteration of current loop
  - Works in both `for` and `while` loops
  - Properly handles nested loops (break/continue only affect innermost loop)
  - Control flow tracking with `ControlFlow` enum in interpreter
  - 14 comprehensive integration tests covering: basic while loops, break in for/while, continue in for/while, nested loops, edge cases
  - Example programs: `loop_control_simple.ruff`, `while_loops_simple.ruff`
  - Syntax:
    ```ruff
    # While loop
    x := 0
    while x < 10 {
        print(x)
        x := x + 1
    }
    
    # Break statement
    for i in 100 {
        if i > 10 {
            break
        }
        print(i)
    }
    
    # Continue statement
    for i in 10 {
        if i % 2 == 0 {
            continue
        }
        print(i)  # Only odd numbers
    }
    ```
- **Modulo Operator**: Added `%` operator for modulo arithmetic
  - Works on numeric values: `5 % 2` returns `1.0`
  - Same precedence as `*` and `/`
  - Lexer tokenizes `%` as operator
  - Parser handles in multiplicative expressions
- **Not-Equal Operator**: Added `!=` comparison operator
  - Works on all comparable types
  - Returns boolean value: `5 != 3` returns `true`
  - Lexer tokenizes `!=` as two-character operator
- **Boolean Type as First-Class Value**: Booleans are now proper runtime values
  - Added `Value::Bool(bool)` variant to replace string-based "true"/"false"
  - Added `Expr::Bool(bool)` to AST for boolean literals
  - Lexer tokenizes `true` and `false` as `TokenKind::Bool` instead of identifiers
  - Parser creates `Expr::Bool` for boolean tokens
  - All comparison operators (`==`, `!=`, `<`, `>`, `<=`, `>=`) now return `Value::Bool`
  - Type checker recognizes `TypeAnnotation::Bool` and infers boolean types from comparisons
  - Boolean values work directly in if conditions: `if my_bool { }`
  - Print function correctly displays boolean values as "true" or "false"
  - File I/O functions (`write_file`, `append_file`, `create_dir`, `file_exists`) return proper booleans
  - Backwards compatible: string-based "true"/"false" still work in if conditions
  - 10 comprehensive integration tests covering: literals, comparisons, if conditions, equality, variables, structs, arrays
  - Enhanced `examples/test_bool.ruff` with comprehensive demonstrations
  - Fixed parser bug where `if x {` was incorrectly parsed as struct instantiation
- **File I/O Functions**: Complete filesystem operations support
  - `read_file(path)`: Reads entire file as string
  - `write_file(path, content)`: Writes/overwrites file content
  - `append_file(path, content)`: Appends content to existing file
  - `file_exists(path)`: Checks if file or directory exists
  - `read_lines(path)`: Reads file and returns array of lines
  - `list_dir(path)`: Lists all files in directory
  - `create_dir(path)`: Creates directory with parents (like mkdir -p)
  - All functions return `Value::Error` on failure, caught by try/except
  - 6 comprehensive unit tests for all file operations
  - Fixed `Expr::Tag` evaluation to check for native/user functions before treating as enum constructors
  - Example programs: `file_logger.ruff`, `config_manager.ruff`, `directory_tools.ruff`, `backup_tool.ruff`, `note_taking_app.ruff`
- **User Input Functions**: Added interactive I/O capabilities
  - `input(prompt)`: Reads a line from stdin, displays prompt without newline
  - `parse_int(str)`: Converts string to integer (returns Error on failure)
  - `parse_float(str)`: Converts string to float (returns Error on failure)
  - All functions integrate with try/except error handling
  - Example programs: `interactive_greeting.ruff`, `guessing_game.ruff`, `interactive_calculator.ruff`, `quiz_game.ruff`
- **Lexical Scoping**: Implemented proper lexical scoping with environment stack
  - Variables now correctly update across scope boundaries
  - Accumulator pattern works: `sum := sum + n` in loops
  - Function local variables properly isolated
  - Nested functions can read and modify outer variables
  - For-loop variables don't leak to outer scope
  - `let` keyword creates shadowed variables in current scope
- **Scope Management**: Environment now uses Vec<HashMap> scope stack
  - `push_scope()`/`pop_scope()` for nested contexts
  - Variable lookup walks up scope chain (innermost to outermost)
  - Assignment updates in correct scope or creates in current
- **Comprehensive Tests**: 12 new integration tests for scoping
  - Nested function scopes
  - For-loop variable isolation
  - Variable shadowing with `let`
  - Function modifying outer variables
  - Scope chain lookup
  - Try/except scoping
  - Accumulator patterns
  - Multiple assignments in loops
- **Example File**: `examples/scoping.ruff` demonstrates all scoping features
  - Accumulator pattern (sum in loop)
  - Function counters
  - Variable shadowing
  - Nested functions
  - Loop variable isolation
  - Factorial-like patterns

### Fixed
- **Assignment Operator**: Fixed `:=` to update existing variables instead of always creating new
  - Changed parser to emit `Stmt::Assign` instead of `Stmt::Let` for `:=`
  - `Stmt::Assign` uses `Environment::set()` which updates existing or creates new
  - `let x :=` still creates new variable (shadowing)
  - Fixes critical bug where `sum := sum + n` created new local variable
- **Function Call Cleanup**: Fixed `return_value` not being cleared after function calls
  - Functions now properly clear return state after execution
  - Prevents early termination of parent statement evaluation
  - Allows multiple statements after function calls to execute

### Changed
- **Environment Architecture**: Replaced single HashMap with Vec<HashMap> scope stack
  - Stack index 0 is global scope
  - Higher indices are nested scopes (functions, loops, try/except)
  - All statement handlers updated to use push_scope/pop_scope

## [0.2.0] - 2026-01-22

### Added
- **Field Assignment**: Full support for mutating struct fields with `:=` operator
  - Direct field mutation: `person.age := 26`
  - Nested field mutation: `todos[0].done := true`
  - Works with array indexing and dictionary keys
- **Truthy/Falsy Evaluation**: If conditions now properly handle boolean values and collections
  - Boolean identifiers (`true`/`false`) work in conditionals
  - Strings: "true" → truthy, "false" → falsy, empty → falsy
  - Arrays: empty → falsy, non-empty → truthy
  - Dictionaries: empty → falsy, non-empty → truthy
- **Test Suite**: Added 10 comprehensive integration tests covering:
  - Field assignment for structs and arrays
  - Boolean conditions and truthy values
  - Array and dict operations
  - String concatenation
  - For-in loops
  - Variable assignment behavior
  - Struct field access
- **Example Projects**: Two demonstration projects showcasing language features
  - Todo Manager: struct mutation, arrays, control flow
  - Contact Manager: dictionaries, string functions, error handling
- **Clean Build**: Zero compiler warnings - all infrastructure code properly annotated

### Fixed
- **Variable Assignment**: `:=` operator now consistently creates or updates variables
  - Previously would fail if variable didn't exist in certain contexts
  - Now always inserts/updates in environment
- **Boolean Handling**: Fixed if statements not recognizing boolean struct fields
  - Was only checking numeric values for truthiness
  - Now properly evaluates boolean identifiers and other types
- **Pattern Matching**: Corrected struct pattern matching syntax in field assignment
  - Changed from incorrect `Value::Struct(ref mut fields)` 
  - To correct `Value::Struct { name: _, fields }`

### Changed
- **Documentation**: Clarified that example projects are demonstrations, not interactive applications
- **Build Output**: Added `--quiet` flag recommendation for clean execution output
- **README**: Updated with clearer feature descriptions and usage examples

### Known Limitations (Documented)
- No lexical scoping - uses single global environment
- Variable shadowing in blocks doesn't update outer scope (design limitation)
- Booleans stored as string identifiers internally (architectural choice)
- No user input function yet (`input()` planned for future release)

### Technical Details
- Total tests: 14 (up from 4)
- Compiler warnings: 0 (down from 14)
- Lines of test code added: ~200
- Files modified: interpreter.rs, ast.rs, errors.rs, builtins.rs, module.rs

---

## [0.1.0] - 2026-01-21

### Added
- Initial release of Ruff programming language
- Core language features:
  - Variables and constants
  - Functions with optional type annotations
  - Control flow (if/else, loops, pattern matching)
  - Data types (numbers, strings, enums, arrays, dicts, structs)
  - Struct definitions with methods
  - Type system with inference and checking
  - Module system with imports/exports
  - Error handling (try/except/throw)
- Built-in functions:
  - Math: abs, sqrt, pow, floor, ceil, round, min, max, trig functions
  - Strings: len, to_upper, to_lower, trim, substring, contains, replace
  - Arrays: push, pop, slice, concat
  - Dicts: keys, values, has_key, remove
- Command-line interface with run and test commands
- Comprehensive documentation and examples

---

[Unreleased]: https://github.com/rufflang/ruff/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/rufflang/ruff/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/rufflang/ruff/releases/tag/v0.2.0
