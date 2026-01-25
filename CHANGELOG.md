# CHANGELOG

All notable changes to the Ruff programming language will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Type Conversion Functions** 🔄 - Convert between types with explicit conversion functions (P0 feature):
  - **to_int(value)** - Convert to integer:
    - From Float: Truncates decimal (e.g., `to_int(3.14)` → `3`)
    - From String: Parses integer (e.g., `to_int("42")` → `42`)
    - From Bool: `true` → `1`, `false` → `0`
    - From Int: Returns as-is
  - **to_float(value)** - Convert to floating-point:
    - From Int: Converts to float (e.g., `to_float(42)` → `42.0`)
    - From String: Parses float (e.g., `to_float("3.14")` → `3.14`)
    - From Bool: `true` → `1.0`, `false` → `0.0`
    - From Float: Returns as-is
  - **to_string(value)** - Convert any value to string representation:
    - Works with all types: Int, Float, Bool, Arrays, Dicts, etc.
    - Uses same formatting as `print()` function
    - Example: `to_string(42)` → `"42"`, `to_string([1, 2])` → `"[1, 2]"`
  - **to_bool(value)** - Convert to boolean with intuitive truthiness:
    - Int/Float: `0` and `0.0` → `false`, all other values → `true`
    - String: Empty string, `"false"`, `"0"` → `false`, others → `true`
    - Arrays/Dicts: Empty → `false`, non-empty → `true`
    - Null: → `false`
  - **Error Handling**: Invalid conversions (e.g., `to_int("abc")`) return error values
  - **Comprehensive Testing**: 17 new tests covering all conversion scenarios
  - **Example**:
    ```ruff
    # Numeric conversions
    x := to_int(3.14)         # 3 (truncate)
    y := to_float(42)         # 42.0
    
    # String parsing
    age := to_int("25")       # 25
    price := to_float("9.99") # 9.99
    
    # String formatting
    msg := to_string(42)      # "42"
    
    # Boolean conversion
    is_active := to_bool(1)   # true
    is_empty := to_bool("")   # false
    has_items := to_bool([1]) # true
    
    # Chaining conversions
    result := to_int(to_float(to_string(42)))  # 42
    ```

- **Type Checker Int/Float Support** 🎯 - Enhanced static type checking for integer and float types:
  - **Type Promotion**: Type checker now allows `Int` → `Float` promotion
    - Functions expecting `Float` accept `Int` arguments (e.g., `abs(5)` is valid)
    - Arithmetic operations correctly infer types: `Int + Int → Int`, `Int + Float → Float`
    - Assignment allows `Int` values to `Float` variables
  - **Accurate Type Inference**:
    - Integer literals (`42`) inferred as `Int` type
    - Float literals (`3.14`) inferred as `Float` type
    - Binary operations with mixed types promote to `Float`
  - **Function Signature Fixes**:
    - `index_of()` now correctly returns `Int` (was `Float`)
    - Math functions (`abs`, `min`, `max`, etc.) accept both `Int` and `Float` via promotion
  - **Comprehensive Testing**: 12 new tests covering all type promotion scenarios
  - **Example**:
    ```ruff
    # All these now pass type checking without warnings:
    x: int := 42              # Int literal to Int variable
    y: float := 42            # Int promoted to Float
    z: float := 5 + 2.5       # Mixed arithmetic promoted to Float
    
    result1 := abs(5)         # Int accepted, returns Float
    result2 := min(10, 20)    # Both Int accepted, returns Float
    result3 := 5 + 10         # Int + Int returns Int
    ```

- **Type Introspection** 🔍 - Runtime type checking and inspection (P0 feature):

  - **Type Inspection Function**:
    - `type(value)` - Returns the type name of any value as a string
    - Type names: `"int"`, `"float"`, `"string"`, `"bool"`, `"null"`, `"array"`, `"dict"`, `"function"`, etc.
  - **Type Predicate Functions**:
    - `is_int(value)` - Returns `true` if value is an integer
    - `is_float(value)` - Returns `true` if value is a float
    - `is_string(value)` - Returns `true` if value is a string
    - `is_array(value)` - Returns `true` if value is an array
    - `is_dict(value)` - Returns `true` if value is a dictionary
    - `is_bool(value)` - Returns `true` if value is a boolean
    - `is_null(value)` - Returns `true` if value is null
    - `is_function(value)` - Returns `true` if value is a function (user-defined or native)
  - **Use Cases**:
    - Write defensive code that handles different types gracefully
    - Build generic functions that adapt to input types
    - Validate function arguments at runtime
    - Debug and inspect values in production code
  - **Example**:
    ```ruff
    # Type inspection
    x := 42
    print(type(x))        # "int"
    
    # Type predicates for defensive coding
    func process_value(val) {
        if is_int(val) {
            return val * 2
        } else if is_string(val) {
            return len(val)
        } else {
            return 0
        }
    }
    
    print(process_value(10))        # 20
    print(process_value("hello"))   # 5
    print(process_value(true))      # 0
    
    # Type validation
    func validate_user(data) {
        if !is_dict(data) {
            return "Error: User data must be a dictionary"
        }
        if !is_string(data["name"]) {
            return "Error: Name must be a string"
        }
        if !is_int(data["age"]) {
            return "Error: Age must be an integer"
        }
        return "Valid"
    }
    ```

- **Integer Type System** 🔢 - Separate integer and floating-point types (P0 feature):
  - **Integer Literals**: `42`, `-10`, `0` are parsed as `Int(i64)` type
  - **Float Literals**: `3.14`, `-2.5`, `0.0` are parsed as `Float(f64)` type
  - **Type Preservation**: Integer arithmetic operations preserve integer type
    - `5 + 3` → `Int(8)` (not `Float(8.0)`)
    - `10 / 3` → `Int(3)` (integer division truncates)
    - `10 % 3` → `Int(1)` (modulo operation)
  - **Type Promotion**: Mixed int/float operations promote to float
    - `5 + 2.5` → `Float(7.5)`
    - `Int * Float` → `Float`
  - **Type-Aware Functions**:
    - `len()` returns `Int` for collection lengths
    - `current_timestamp()` returns `Int` (milliseconds since epoch)
    - String functions accept both Int and Float for indices/counts
    - Math functions accept both Int and Float, return Float
  - **Database & Serialization**: Types are preserved across:
    - SQLite: INTEGER vs REAL columns
    - PostgreSQL: INT8 vs FLOAT8 columns
    - MySQL: BIGINT vs DOUBLE columns
    - JSON: integers vs floats in JSON numbers
    - TOML/YAML: Integer vs Float values
  - **Example**:
    ```ruff
    # Integer arithmetic
    x := 10
    y := 3
    print(x / y)      # 3 (integer division)
    print(x % y)      # 1 (modulo)
    
    # Mixed operations
    a := 5
    b := 2.5
    print(a + b)      # 7.5 (promoted to float)
    
    # Type preservation
    numbers := [1, 2, 3]
    print(numbers[0])  # 1 (still Int)
    ```

- **Complete Timing Suite** ⏱️ - Robust benchmarking and performance measurement:
  - **High-Precision Timers**:
    - `time_us()` - Returns microseconds since program start (1/1,000 millisecond precision)
    - `time_ns()` - Returns nanoseconds since program start (1/1,000,000 millisecond precision)
    - Ideal for measuring very fast operations and CPU-level performance analysis
  - **Duration Helpers**:
    - `format_duration(ms)` - Automatically formats milliseconds to human-readable strings
      - Examples: `5.00s`, `123.45ms`, `567.89μs`, `123ns`
      - Automatically chooses the best unit for readability
    - `elapsed(start, end)` - Calculate time difference between two timestamps
  - **Use Cases**:
    - Benchmark algorithm performance with microsecond precision
    - Compare multiple implementation approaches
    - Profile code execution at different precision levels
    - Generate readable performance reports
  - **Example**:
    ```ruff
    # Microsecond precision benchmarking
    start := time_us()
    # ... fast operation ...
    end := time_us()
    print("Took: " + format_duration((end - start) / 1000.0))
    
    # Nanosecond precision for ultra-fast operations
    start_ns := time_ns()
    x := x + 1  # Single operation
    end_ns := time_ns()
    print("Single add: " + (end_ns - start_ns) + "ns")
    ```
  - See `examples/benchmark_demo.ruff` for comprehensive examples

- **Timing Functions** ⏱️:
  - `current_timestamp()` - Returns current timestamp in milliseconds since UNIX epoch (January 1, 1970)
    - Returns a large number like `1769313715178` (milliseconds)
    - Ideal for timestamps, logging, and date calculations
    - Based on system wall-clock time
  - `performance_now()` - Returns high-resolution timer in milliseconds since program start
    - Returns elapsed time like `3.142` (milliseconds)
    - Ideal for performance measurement and benchmarking
    - Monotonic timer not affected by system clock changes
  - Example usage:
    ```ruff
    # Measure execution time
    start := performance_now()
    expensive_operation()
    elapsed := performance_now() - start
    print("Took " + elapsed + "ms")
    
    # Get current timestamp
    timestamp := current_timestamp()
    print("Current time: " + timestamp)
    ```
  - See `examples/timing_demo.ruff` for working examples
  - Fixes critical bug in `examples/projects/ai_model_comparison.ruff`

## [0.6.0] - 2026-01-24

**Focus**: Production Database Features - Transactions, Connection Pooling, and Multi-Backend Support

This release completes the database foundation for production applications with SQLite, PostgreSQL, and MySQL support, plus critical features for high-traffic apps.

### Added

- **Database Transactions** 🎉:
  - `db_begin(db)` - Start a database transaction
  - `db_commit(db)` - Commit transaction changes
  - `db_rollback(db)` - Rollback transaction on error
  - `db_last_insert_id(db)` - Get auto-generated ID from last INSERT
  - Ensures atomic operations across multiple SQL statements
  - Full support for SQLite, PostgreSQL, and MySQL
  - Automatic transaction cleanup on interpreter shutdown (prevents hangs)
  - Example: Money transfers, e-commerce order processing, inventory management
  - See `examples/database_transactions.ruff` for working examples
  - See `tests/test_transactions_working.ruff` for comprehensive tests

- **Connection Pooling** 🎉:
  - `db_pool(db_type, connection_string, config)` - Create connection pool
  - `db_pool_acquire(pool)` - Acquire connection from pool (blocks if all in use)
  - `db_pool_release(pool, conn)` - Release connection back to pool
  - `db_pool_stats(pool)` - Get pool statistics (available, in_use, total, max)
  - `db_pool_close(pool)` - Close entire pool and all connections
  - Configuration options:
    - `min_connections` - Minimum pool size (reserved for future use)
    - `max_connections` - Maximum concurrent connections
    - `connection_timeout` - Seconds to wait for available connection
  - Thread-safe lazy connection creation
  - Supports all three database backends: SQLite, PostgreSQL, MySQL
  - Critical for production apps with high traffic and concurrent users
  - See `examples/database_pooling.ruff` for working examples
  - See `tests/test_connection_pooling.ruff` for comprehensive tests

### Fixed

- Fixed critical bug where SQLite connections would hang on exit if in active transaction
- Added `Interpreter::cleanup()` method to rollback active transactions before drop

### Added (Previous)

- **Unified Database API**:
  - **Multi-Backend Support**:
    - Unified `db_connect(db_type, connection_string)` API that works across different databases
    - Database type parameter: `"sqlite"` ✅, `"postgres"` ✅, `"mysql"` (coming soon)
    - Same `db_execute()` and `db_query()` functions work with any database backend
    - Seamless migration path between database types without code changes
  - **SQLite Support** ✅:
    - `db_connect("sqlite", "path/to/database.db")` - Connect to SQLite database
    - `db_execute(db, sql, params)` - Execute INSERT, UPDATE, DELETE, CREATE statements
    - `db_query(db, sql, params)` - Query data and return array of dictionaries
    - Parameter binding with `?` placeholders: `["Alice", 30]`
    - Full support for NULL values, integers, floats, text, and blobs
  - **PostgreSQL Support** ✅:
    - `db_connect("postgres", "host=localhost dbname=myapp user=admin password=secret")`
    - `db_connect("postgresql", ...)` - Both "postgres" and "postgresql" accepted
    - `db_execute(db, sql, params)` - Execute SQL with `$1, $2, $3` parameter syntax
    - `db_query(db, sql, params)` - Query with full type support
    - Parameter binding: `["Alice", 30]` mapped to `$1, $2` in SQL
    - Supports: SERIAL, INTEGER, BIGINT, REAL, DOUBLE PRECISION, TEXT, BOOLEAN, NULL
    - Compatible with PostgreSQL 9.6+ features
  - **MySQL Support** ✅:
    - `db_connect("mysql", "mysql://user:pass@localhost:3306/myapp")`
    - Asynchronous driver (mysql_async) with transparent blocking interface
    - Parameter binding with `?` placeholders (MySQL style)
    - Full CRUD support: CREATE, INSERT, SELECT, UPDATE, DELETE
    - Supports: INT, AUTO_INCREMENT, VARCHAR, BOOLEAN, TIMESTAMP, NULL
    - Compatible with MySQL 5.7+ and MariaDB 10.2+
  - **Common Database Functions**:
    - `db_close(db)` - Close database connection (works for all database types)
    - Full parameter binding support prevents SQL injection
    - Automatic type conversion between Ruff and database types
    - Proper NULL value handling across all databases
  - **Transaction Support (Planned)**:
    - `db_begin(db)` - Begin transaction
    - `db_commit(db)` - Commit transaction
    - `db_rollback(db)` - Rollback transaction
    - Stub implementations show helpful messages
  - **Connection Pooling (Planned)**:
    - `db_pool(db_type, connection_string, options)` - Create connection pool
    - For high-traffic applications
    - Infrastructure designed, implementation planned for future release
  - **Use Cases**:
    - 🍽️ Restaurant menu management (SQLite for local, PostgreSQL for cloud)
    - 📝 Blog platforms with PostgreSQL
    - 💬 Forums and community sites
    - 🛒 E-commerce applications
    - 📊 Analytics dashboards
    - 🏢 Business management tools
  - **Examples**:
    ```ruff
    # SQLite with unified API
    db := db_connect("sqlite", "myapp.db")
    db_execute(db, "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)", [])
    db_execute(db, "INSERT INTO users (name) VALUES (?)", ["Alice"])
    users := db_query(db, "SELECT * FROM users WHERE id > ?", [100])
    
    # PostgreSQL with same API!
    db := db_connect("postgres", "host=localhost dbname=myapp user=admin password=secret")
    db_execute(db, "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT)", [])
    db_execute(db, "INSERT INTO users (name) VALUES ($1)", ["Alice"])
    users := db_query(db, "SELECT * FROM users WHERE id > $1", [100])
    
    # MySQL with same API!
    db := db_connect("mysql", "mysql://root@localhost:3306/myapp")
    db_execute(db, "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100))", [])
    db_execute(db, "INSERT INTO users (name) VALUES (?)", ["Alice"])
    users := db_query(db, "SELECT * FROM users WHERE id > ?", [100])
    
    # Same Ruff code works across all databases!
    for user in users {
        print(user["name"])
    }
    db_close(db)
    ```
  - See `examples/database_unified.ruff` for comprehensive SQLite examples
  - See `examples/database_postgres.ruff` for comprehensive PostgreSQL examples
  - See `examples/database_mysql.ruff` for comprehensive MySQL examples
  - Breaking change: Old `db_connect(path)` syntax replaced with `db_connect("sqlite", path)`
  - Migration: Add `"sqlite"` as first argument to existing db_connect() calls

### Fixed

- **Function Scope Handling**:
  - Fixed variable shadowing in functions - functions now properly use call-time environment
  - Fixed outer variable modification from within functions
  - Regular function definitions no longer capture environment (only closures do)
  - All 117 tests now pass (fixed 5 previously failing scope tests)

- **Concurrency & Parallelism** (v0.6.0):
  - **spawn Statement**:
    - `spawn { code }` - Execute code block in a background thread
    - Non-blocking execution for fire-and-forget tasks
    - Each spawn runs in isolation with its own environment
    - Perfect for background processing and long-running operations
  - **Parallel HTTP Requests**:
    - `parallel_http(urls_array)` - Make multiple HTTP GET requests concurrently
    - Returns array of response dicts in same order as input URLs
    - Each response contains `status` (number) and `body` (string) fields
    - **3x faster** than sequential requests when fetching from 3+ endpoints
    - Critical for AI tools comparing multiple model providers (OpenAI, Claude, DeepSeek)
    - Ideal for batch processing and data pipelines
  - **Channels for Thread Communication**:
    - `channel()` - Create thread-safe communication channel
    - `chan.send(value)` - Send value to channel (non-blocking)
    - `chan.receive()` - Receive value from channel (returns null if empty)
    - FIFO ordering (first in, first out)
    - Perfect for producer-consumer patterns
    - Enables coordination between spawned tasks and main thread
  - **Use Cases**:
    - **AI Model Comparison**: Query GPT-4, Claude, and DeepSeek simultaneously for 3x speedup
    - **Batch Content Generation**: Process 100+ prompts across multiple providers in parallel
    - **Background Processing**: File processing, log analysis, data transformation without blocking
    - **Web Scraping**: Fetch multiple pages concurrently
    - **API Aggregation**: Combine data from multiple services in real-time
  - **Examples**:
    ```ruff
    # Parallel HTTP requests
    urls := ["https://api1.com", "https://api2.com", "https://api3.com"]
    results := parallel_http(urls)  # All 3 requests happen simultaneously
    
    # Background tasks with spawn
    spawn {
        print("Processing in background...")
        process_large_file()
    }
    print("Main thread continues immediately")
    
    # Thread communication with channels
    chan := channel()
    
    spawn {
        result := expensive_computation()
        chan.send(result)
    }
    
    value := chan.receive()  # Get result from background thread
    ```
  - See `examples/concurrency_parallel_http.ruff` for parallel HTTP demo
  - See `examples/concurrency_spawn.ruff` for spawn examples
  - See `examples/concurrency_channels.ruff` for channel communication patterns
  - See `examples/projects/ai_model_comparison.ruff` for real-world AI tool example

- **Image Processing** (v0.6.0):
  - **Image Loading**:
    - `load_image(path)` - Load images from files (JPEG, PNG, WebP, GIF, BMP)
    - Error handling for missing or invalid image files
    - Automatic format detection from file extension
  - **Image Properties**:
    - `img.width` - Get image width in pixels
    - `img.height` - Get image height in pixels
    - `img.format` - Get image format (e.g., "jpg", "png")
  - **Image Transformations**:
    - `img.resize(width, height)` - Resize to exact dimensions
    - `img.resize(width, height, "fit")` - Resize maintaining aspect ratio
    - `img.crop(x, y, width, height)` - Extract rectangular region
    - `img.rotate(degrees)` - Rotate 90, 180, or 270 degrees
    - `img.flip("horizontal")` - Flip horizontally
    - `img.flip("vertical")` - Flip vertically
  - **Image Filters**:
    - `img.to_grayscale()` - Convert to grayscale
    - `img.blur(sigma)` - Apply Gaussian blur (sigma controls intensity)
    - `img.adjust_brightness(factor)` - Adjust brightness (1.0 = no change, >1.0 = brighter)
    - `img.adjust_contrast(factor)` - Adjust contrast (1.0 = no change, >1.0 = more contrast)
  - **Image Saving**:
    - `img.save(path)` - Save image with automatic format conversion
    - Supports JPEG, PNG, WebP, GIF, BMP output formats
    - Format automatically detected from file extension
  - **Method Chaining**:
    - All transformation methods return new Image values
    - Chain multiple operations: `img.resize(800, 600).to_grayscale().save("out.jpg")`
  - **Use Cases**:
    - AI image generation pipelines (resize, crop, watermark outputs)
    - Thumbnail generation for galleries and listings
    - Image optimization for web (format conversion, compression)
    - Social media image preparation (specific dimensions, filters)
    - Batch processing for e-commerce product photos
    - Automated image enhancement workflows
  - **Examples**:
    ```ruff
    # Load and inspect
    img := load_image("photo.jpg")
    print("Size: " + img.width + "x" + img.height)
    
    # Create thumbnail
    thumb := img.resize(200, 200, "fit")
    thumb.save("thumbnail.jpg")
    
    # Apply filters
    enhanced := img
        .adjust_brightness(1.2)
        .adjust_contrast(1.15)
        .save("enhanced.jpg")
    
    # Batch process
    for path in ["img1.jpg", "img2.jpg", "img3.jpg"] {
        img := load_image(path)
        thumb := img.resize(200, 200, "fit")
        thumb.save("thumbs/" + path)
    }
    ```
  - See `examples/image_processing.ruff` for comprehensive examples
  - See `tests/image_processing_test.ruff` for test suite
  - Full type checker support for load_image function

- **Serialization Formats** (v0.6.0):
  - **TOML Support**:
    - `parse_toml(toml_string)` - Parse TOML configuration files to Ruff dictionaries and arrays
    - `to_toml(value)` - Convert Ruff dictionaries and arrays to TOML format
    - Full support for TOML data types: strings, integers, floats, booleans, datetime, arrays, and tables
    - Perfect for configuration files and structured data
  - **YAML Support**:
    - `parse_yaml(yaml_string)` - Parse YAML documents to Ruff values
    - `to_yaml(value)` - Serialize Ruff values to YAML format
    - Support for YAML sequences, mappings, scalars, and null values
    - Ideal for API specifications, data files, and cloud configs
  - **CSV Support**:
    - `parse_csv(csv_string)` - Parse CSV data into array of dictionaries (one dict per row)
    - `to_csv(array_of_dicts)` - Convert array of dictionaries to CSV string
    - Automatic header detection from first row
    - Automatic number parsing for numeric fields
    - Perfect for data analysis, reports, and spreadsheet data
  - **Use Cases**:
    - Configuration management (TOML for app configs)
    - API specifications (YAML for OpenAPI/Swagger)
    - Data import/export (CSV for spreadsheets and databases)
    - Infrastructure as code (YAML for Kubernetes, Docker Compose)
    - Data transformation pipelines
  - **Examples**:
    ```ruff
    # TOML Configuration
    config := parse_toml(read_file("config.toml"))
    port := config["server"]["port"]
    
    # YAML Processing
    api_spec := parse_yaml(read_file("openapi.yaml"))
    endpoints := api_spec["paths"]
    
    # CSV Data Analysis
    sales := parse_csv(read_file("sales.csv"))
    for row in sales {
        total := row["quantity"] * row["price"]
        print(row["product"] + ": $" + to_string(total))
    }
    ```
  - See `examples/toml_demo.ruff` for TOML configuration examples
  - See `examples/yaml_demo.ruff` for YAML processing examples
  - See `examples/csv_demo.ruff` for CSV data processing examples
  - Full type checker support for all serialization functions

- **Advanced Collections** (v0.6.0):
  - **Set Collection**:
    - `Set(array)` - Create a set from an array, automatically removing duplicates
    - `set_add(set, item)` - Add item to set if not already present
    - `set_has(set, item)` - Check if set contains item (returns boolean)
    - `set_remove(set, item)` - Remove item from set
    - `set_union(set1, set2)` - Create new set with all unique elements from both sets
    - `set_intersect(set1, set2)` - Create new set with elements present in both sets
    - `set_difference(set1, set2)` - Create new set with elements in set1 but not in set2
    - `set_to_array(set)` - Convert set back to array
    - Works with `len(set)` for counting unique elements
  - **Queue Collection (FIFO)**:
    - `Queue(array?)` - Create empty queue or initialize from array
    - `queue_enqueue(queue, item)` - Add item to back of queue
    - `queue_dequeue(queue)` - Remove and return `[modified_queue, front_item]` or `[queue, null]` if empty
    - `queue_peek(queue)` - View front item without removing (returns null if empty)
    - `queue_is_empty(queue)` - Check if queue has no items (returns boolean)
    - `queue_to_array(queue)` - Convert queue to array (front to back)
    - Works with `len(queue)` for counting items
  - **Stack Collection (LIFO)**:
    - `Stack(array?)` - Create empty stack or initialize from array
    - `stack_push(stack, item)` - Push item onto top of stack
    - `stack_pop(stack)` - Pop and return `[modified_stack, top_item]` or `[stack, null]` if empty
    - `stack_peek(stack)` - View top item without popping (returns null if empty)
    - `stack_is_empty(stack)` - Check if stack has no items (returns boolean)
    - `stack_to_array(stack)` - Convert stack to array (bottom to top)
    - Works with `len(stack)` for counting items
  - **Use Cases**:
    - **Set**: Unique visitor tracking, tag management, email deduplication, removing duplicates
    - **Queue**: Task processing systems, message queues, customer support ticketing, job scheduling
    - **Stack**: Browser history, undo/redo systems, function call stacks, depth-first traversal
  - **Examples**:
    ```ruff
    # Set - Track unique visitors
    visitors := Set(["user1", "user2", "user1", "user3"])
    print(len(visitors))  # 3 unique visitors
    
    # Queue - Task processing (FIFO)
    tasks := Queue([])
    tasks := queue_enqueue(tasks, "Task 1")
    tasks := queue_enqueue(tasks, "Task 2")
    result := queue_dequeue(tasks)  # Gets "Task 1"
    
    # Stack - Browser history (LIFO)
    history := Stack([])
    history := stack_push(history, "google.com")
    history := stack_push(history, "github.com")
    result := stack_pop(history)  # Gets "github.com"
    ```
  - See `examples/collections_advanced.ruff` for 10+ practical examples
  - See `tests/test_collections.ruff` for 30 comprehensive tests
  - Full type checker support for all collection functions

- **HTTP Authentication & Streaming** (v0.6.0):
  - **JWT (JSON Web Token) Functions**:
    - `jwt_encode(payload_dict, secret_key)` - Encode a JWT token from dictionary payload
    - `jwt_decode(token, secret_key)` - Decode and verify JWT token, returns payload dictionary
    - Support for custom claims (user_id, exp, roles, etc.)
    - Automatic signature verification with HS256 algorithm
    - No expiration validation by default (flexible for various use cases)
  - **OAuth2 Helper Functions**:
    - `oauth2_auth_url(client_id, redirect_uri, auth_url, scope)` - Generate OAuth2 authorization URL
    - `oauth2_get_token(code, client_id, client_secret, token_url, redirect_uri)` - Exchange authorization code for access token
    - Automatic state parameter generation for CSRF protection
    - URL encoding of parameters for safe OAuth flows
    - Support for GitHub, Google, and custom OAuth2 providers
  - **HTTP Streaming**:
    - `http_get_stream(url)` - Fetch large HTTP responses efficiently as binary data
    - Memory-efficient downloads for large files
    - Foundation for future chunked streaming enhancements
  - **Use Cases**:
    - Secure API authentication with JWT tokens
    - Stateless session management
    - Third-party OAuth integration (GitHub, Google, Discord, etc.)
    - AI API authentication (OpenAI, Anthropic, DeepSeek)
    - Large file downloads without memory overflow
    - Multi-part file processing and streaming
  - **Examples**:
    ```ruff
    # JWT Authentication
    payload := {"user_id": 42, "role": "admin", "exp": now() + 3600}
    secret := "my-secret-key"
    token := jwt_encode(payload, secret)
    decoded := jwt_decode(token, secret)
    user_id := decoded["user_id"]
    
    # OAuth2 Flow
    auth_url := oauth2_auth_url(
        "client-123",
        "https://myapp.com/callback",
        "https://provider.com/oauth/authorize",
        "user:read user:write"
    )
    # Redirect user to auth_url, then handle callback
    token_data := oauth2_get_token(
        auth_code,
        "client-123",
        "client-secret",
        "https://provider.com/oauth/token",
        "https://myapp.com/callback"
    )
    access_token := token_data["access_token"]
    
    # HTTP Streaming
    large_file := http_get_stream("https://example.com/large-dataset.zip")
    write_binary_file("dataset.zip", large_file)
    ```
  - See `examples/jwt_auth.ruff` for JWT authentication patterns
  - See `examples/oauth_demo.ruff` for complete OAuth2 integration guide
  - See `examples/http_streaming.ruff` for streaming download examples
  - Full test coverage with 11 integration tests for JWT and OAuth2

- **Binary File Support & HTTP Downloads** (v0.6.0):
  - **Binary File I/O Functions**:
    - `read_binary_file(path)` - Read entire file as binary data (bytes)
    - `write_binary_file(path, bytes)` - Write binary data to file
    - Support for working with images, PDFs, archives, and other binary formats
  - **Binary HTTP Downloads**:
    - `http_get_binary(url)` - Download binary files via HTTP
    - Perfect for fetching images, PDFs, media files from APIs
  - **Base64 Encoding/Decoding**:
    - `encode_base64(bytes_or_string)` - Encode binary data or strings to base64 string
    - `decode_base64(base64_string)` - Decode base64 string to binary data
    - Essential for API integrations that require base64-encoded data
  - **New Value Type**:
    - `Value::Bytes` - Native binary data type for efficient byte array handling
    - `len()` function now supports binary data to get byte count
  - **Use Cases**:
    - Download AI-generated images from DALL-E, Stable Diffusion APIs
    - Fetch and store PDFs, documents, archives
    - Handle file uploads/downloads in web applications
    - Embed binary data in JSON payloads via base64
    - Process media files (images, audio, video)
    - Create backup and data migration tools
  - **Examples**:
    ```ruff
    # Download an image from a URL
    image_data := http_get_binary("https://example.com/photo.jpg")
    write_binary_file("photo.jpg", image_data)
    
    # Read binary file and convert to base64 for API
    file_bytes := read_binary_file("document.pdf")
    base64_str := encode_base64(file_bytes)
    
    # Decode base64 from API response
    received_base64 := api_response["file_data"]
    binary_data := decode_base64(received_base64)
    write_binary_file("downloaded.bin", binary_data)
    ```
  - See `examples/binary_file_demo.ruff` for comprehensive demonstrations
  - See `examples/http_download.ruff` for HTTP download patterns
  - Full test coverage in `tests/test_binary_files.ruff`

- **Method Chaining & Fluent APIs** (v0.6.0):
  - **Null Coalescing Operator (`??`)**: Returns left value if not null, otherwise returns right value
    ```ruff
    username := user?.name ?? "Anonymous"
    timeout := config?.timeout ?? 5000
    ```
  - **Optional Chaining Operator (`?.`)**: Safely access properties that might be null
    ```ruff
    email := user?.profile?.email  # Returns null if any part is null
    value := dict?.field           # Safe dictionary access
    ```
  - **Pipe Operator (`|>`)**: Pass value as first argument to function for data transformation pipelines
    ```ruff
    result := 5 |> double |> add_ten |> square
    greeting := "hello" |> to_upper |> exclaim  # "HELLO!"
    ```
  - **Null Type**: Added `null` keyword and `Value::Null` type for representing absence of value
  - **Use Cases**:
    - Safe property access without explicit null checks
    - Default value fallbacks in configuration and user data
    - Functional data transformation pipelines
    - Chainable operations for cleaner code
  - **Examples**:
    - See `examples/method_chaining.ruff` for practical demonstrations
    - See `tests/test_method_chaining.ruff` for comprehensive test coverage

- **Closures & Variable Capturing** (v0.6.0):
  - Functions now properly capture their definition environment
  - Closure state persists across multiple function calls
  - Support for counter patterns with mutable captured variables
  - Nested closures with multiple levels of variable capturing
  - Anonymous functions inherit parent scope automatically
  - **Examples**:
    ```ruff
    # Counter closure
    func make_counter() {
        let count := 0
        return func() {
            count := count + 1
            return count
        }
    }
    
    counter := make_counter()
    print(counter())  # 1
    print(counter())  # 2
    print(counter())  # 3
    
    # Adder closure
    func make_adder(x) {
        return func(y) {
            return x + y
        }
    }
    
    add5 := make_adder(5)
    print(add5(3))   # 8
    print(add5(10))  # 15
    ```
  - **Implementation**: Uses `Rc<RefCell<Environment>>` for shared mutable environment
  - **Known Limitations**: Some complex multi-variable capture scenarios need further testing

- **HTTP Headers Support**: Full control over HTTP response headers
  - **Header Manipulation Functions**:
    - `set_header(response, key, value)` - Add or update a single header on an HTTP response
    - `set_headers(response, headers_dict)` - Set multiple headers at once from a dictionary
  - **Automatic Headers**:
    - `json_response()` now automatically includes `Content-Type: application/json` header
  - **Enhanced Functions**:
    - `redirect_response(url, headers_dict)` - Now accepts optional second parameter for custom headers
  - **Request Headers**:
    - HTTP server requests now include `request.headers` dictionary with all incoming headers
    - Access headers like: `content_type := request.headers["Content-Type"]`
  - **Use Cases**:
    - CORS headers: `Access-Control-Allow-Origin`, `Access-Control-Allow-Methods`
    - Security headers: `X-Content-Type-Options`, `X-Frame-Options`, `Strict-Transport-Security`
    - Caching: `Cache-Control`, `ETag`, `Last-Modified`
    - Custom metadata: `X-Request-ID`, `X-API-Version`, `X-Rate-Limit`
  - **Examples**:
    ```ruff
    response := http_response(200, "OK")
    response := set_header(response, "X-API-Version", "1.0")
    response := set_header(response, "Cache-Control", "max-age=3600")
    ```
  - See `examples/http_headers_demo.ruff` for complete examples
  - Comprehensive test coverage in `tests/test_http_headers.ruff`

---

## [0.5.0] - 2026-01-23

### Added
- **HTTP Server & Networking**: Full-featured HTTP client and server capabilities
  - **HTTP Client Functions**:
    - `http_get(url)` - Send GET requests and receive responses
    - `http_post(url, body)` - Send POST requests with JSON body
    - `http_put(url, body)` - Send PUT requests with JSON body
    - `http_delete(url)` - Send DELETE requests
    - Returns `Result<dict, string>` with status code and response body
  - **HTTP Server Creation**:
    - `http_server(port)` - Create HTTP server on specified port
    - `server.route(method, path, handler)` - Register route handlers
    - `server.listen()` - Start server and handle incoming requests
  - **Request/Response Objects**:
    - `http_response(status, body)` - Create HTTP response with status code and text body
    - `json_response(status, data)` - Create HTTP response with JSON body
    - Request object includes: `method`, `path`, `body` fields
  - **Features**:
    - Method-based routing (GET, POST, PUT, DELETE, etc.)
    - Path-based routing with exact matching
    - JSON request/response handling
    - Automatic request body parsing
    - Error handling with proper status codes
  - **Example applications**:
    - `examples/http_server_simple.ruff` - Basic hello world server
    - `examples/http_rest_api.ruff` - Full REST API with in-memory data
    - `examples/http_client.ruff` - HTTP client usage examples
    - `examples/http_webhook.ruff` - Webhook receiver implementation
  - Example:
    ```ruff
    let server = http_server(8080);
    
    server.route("GET", "/hello", func(request) {
        return http_response(200, "Hello, World!");
    });
    
    server.route("POST", "/data", func(request) {
        return json_response(200, {"received": request.body});
    });
    
    server.listen();  // Start serving requests
    ```

- **SQLite Database Support**: Built-in SQLite database functions for persistent data storage
  - `db_connect(path)` - Connect to a SQLite database file (creates if not exists)
  - `db_execute(db, sql, params)` - Execute INSERT, UPDATE, DELETE, CREATE statements
  - `db_query(db, sql, params)` - Execute SELECT queries, returns array of dicts
  - `db_close(db)` - Close database connection
  - Parameters use `?` placeholders: `db_execute(db, "INSERT INTO t (a, b) VALUES (?, ?)", [val1, val2])`
  - Query results are arrays of dicts with column names as keys
  - Thread-safe with `Arc<Mutex<Connection>>` wrapper

- **HTTP redirect_response()**: New function for creating HTTP 302 redirect responses
  - `redirect_response(url)` - Returns HTTP response with Location header
  - Used for URL shorteners and OAuth flows

- **Dynamic route path parameters**: HTTP server routes now support parameterized paths like `/:code`
  - New `match_route_pattern()` function extracts path parameters from URLs
  - Request object now includes a `params` dict with extracted path values
  - Example: `server.route("GET", "/:code", func(request) { code := request.params["code"] })`
  - Exact routes take priority over parameterized routes (e.g., `/health` matches before `/:code`)

- **Interactive REPL (Read-Eval-Print Loop)**: Full-featured interactive shell for Ruff
  - **Launch with `ruff repl`** - Start interactive mode for quick experimentation and learning
  - **Multi-line input support** - Automatically detects incomplete statements (unclosed braces, brackets, parentheses)
    - Type opening brace `{` and continue on next line with `....>` prompt
    - Close brace `}` to execute the complete statement
    - Works for functions, loops, conditionals, and any multi-line construct
  - **Command history** - Navigate previous commands with up/down arrow keys
  - **Line editing** - Full readline support with cursor movement and editing
  - **Special commands**:
    - `:help` or `:h` - Display help information
    - `:quit` or `:q` - Exit the REPL (or use Ctrl+D)
    - `:clear` or `:c` - Clear the screen
    - `:vars` or `:v` - Show defined variables
    - `:reset` or `:r` - Reset environment to clean state
    - `Ctrl+C` - Interrupt current input
  - **Persistent state** - Variables and functions remain defined across inputs
  - **Pretty-printed output** - Colored, formatted display of values
    - Numbers: `=> 42`
    - Strings: `=> "Hello, World"`
    - Booleans: `=> true`
    - Arrays: `=> [1, 2, 3, 4]`
    - Dictionaries: `=> {"name": "Alice", "age": 30}`
    - Functions: `=> <function(x, y)>`
    - Structs: `=> Point { x: 3, y: 4 }`
  - **Expression evaluation** - Any expression automatically prints its result
  - **Error handling** - Errors display clearly without crashing the REPL
  - Example session:
    ```
    ruff> let x := 42
    ruff> x + 10
    => 52
    ruff> func greet(name) {
    ....>     print("Hello, " + name)
    ....> }
    ruff> greet("World")
    Hello, World
    => 0
    ruff> let nums := [1, 2, 3, 4, 5]
    ruff> nums
    => [1, 2, 3, 4, 5]
    ruff> :quit
    Goodbye!
    ```
  - See `tests/test_repl_*.txt` for comprehensive examples

### Changed
- **URL Shortener example**: Updated to use SQLite database for secure URL storage
  - URLs no longer exposed via public `/list` JSON endpoint
  - Stats endpoint now requires POST with code in body
  - New `/count` endpoint shows total URLs without exposing data
  - Database file: `urls.db` in working directory

### Fixed
- **Critical: Logical AND (&&) and OR (||) operators not working**: The `&&` and `||` operators were completely broken - they always returned `false` regardless of operands.
  - **Lexer**: Added tokenization for `|` and `&` characters to produce `||` and `&&` tokens
  - **Parser**: Added `parse_or()` and `parse_and()` functions with proper operator precedence (`||` lowest, then `&&`, then comparisons)
  - **Interpreter**: Added `&&` and `||` cases to the Bool/Bool match arm
  - Also added `!=` operator support for Bool and String comparisons
  - This fixes URL validation in `url_shortener.ruff` which uses `starts_with(url, "http://") || starts_with(url, "https://")`
  - Example: `true || false` now correctly returns `true` (previously returned `false`)

- **URL shortener /list endpoint**: Changed from `for code in keys(urls)` to `for code in urls`
  - The `keys()` function inside closures causes hangs
  - Direct dict iteration works correctly and iterates over keys

- **Critical: Function cleanup hang bug**: Fixed stack overflow that occurred when functions containing loops were cleaned up during program shutdown. Functions can now safely contain loops, nested control structures, and complex logic without hanging.
  - Introduced `LeakyFunctionBody` wrapper type using `ManuallyDrop` to prevent deep recursion during Rust's automatic drop
  - Function bodies are intentionally leaked at program shutdown (OS reclaims all memory anyway)
  - Updated `url_shortener.ruff` example to use proper random code generation with loops
  - Added comprehensive tests in `tests/test_function_drop_fix.ruff`

- **HTTP functions type checking warnings**: Fixed "Undefined function" warnings for HTTP functions in the type checker.
  - Registered all HTTP client functions: `http_get`, `http_post`, `http_put`, `http_delete`
  - Registered all HTTP server functions: `http_server`, `http_response`, `json_response`
  - HTTP examples now run without type checking warnings
  - Added test file `tests/test_http_type_checking.ruff`


## [0.4.0] - 2026-01-23

### Added
- **Unary Operator Overloading**: Structs can now overload unary operators for custom behavior
  - **`op_neg`** for unary minus (`-value`) - enables vector negation, complex number negation, etc.
  - **`op_not`** for logical not (`!value`) - enables custom boolean logic, flag toggling, etc.
  - Works with nested unary operators: `--value`, `!!flag`, etc.
  - Combines seamlessly with binary operators: `-a + b`, `!flag && condition`, etc.
  - Falls back to default behavior for built-in types (Number, Bool)
  - Example:
    ```ruff
    struct Vector {
        x: float,
        y: float,
        
        fn op_neg(self) {
            return Vector { x: -self.x, y: -self.y };
        }
    }
    
    let v = Vector { x: 3.0, y: 4.0 };
    let neg_v = -v;  // Returns Vector { x: -3.0, y: -4.0 }
    ```
  - Boolean toggle example:
    ```ruff
    struct Flag {
        value: bool,
        
        fn op_not(self) {
            return Flag { value: !self.value };
        }
    }
    
    let f = Flag { value: true };
    let toggled = !f;  // Returns Flag { value: false }
    ```

- **Explicit `self` Parameter for Struct Methods**: Methods can now use explicit `self` parameter for clearer code and method composition
  - Add `self` as the first parameter to access the struct instance within methods
  - Enables calling other methods: `self.method_name()` works within method bodies
  - Supports builder patterns and fluent interfaces
  - Fully backward compatible - methods without `self` still work (use implicit field access)
  - Example:
    ```ruff
    struct Calculator {
        base: float,
        
        func add(self, x) {
            return self.base + x;
        }
        
        func chain(self, x) {
            temp := self.add(x);  # Call another method on self
            return temp * 2.0;
        }
    }
    
    calc := Calculator { base: 10.0 };
    result := calc.chain(5.0);  # Returns 30.0: (10 + 5) * 2
    ```
  - Builder pattern example:
    ```ruff
    struct Builder {
        x: float,
        y: float,
        
        func set_x(self, value) {
            return Builder { x: value, y: self.y };
        }
        
        func set_y(self, value) {
            return Builder { x: self.x, y: value };
        }
    }
    
    result := Builder { x: 0.0, y: 0.0 }
        .set_x(10.0)
        .set_y(20.0);
    ```
  - Works seamlessly with operator overloading methods
  - See `examples/struct_self_methods.ruff` for comprehensive examples

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
