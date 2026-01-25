# Ruff Showcase Projects

This directory contains complete, real-world projects that demonstrate the capabilities of the Ruff programming language. Each project combines multiple language features to solve practical problems.

## 🎯 Complete Projects

### 1. **Log Analyzer** (`project_log_analyzer.ruff`)
**Advanced CLI tool for analyzing log files**

**Features demonstrated:**
- Command-line argument parsing with `arg_parser()`
- File I/O operations
- Regular expression pattern matching
- Complex data structures (dictionaries, arrays)
- Statistical analysis and reporting
- JSON export capability
- Pretty-printed formatted output

**Usage:**
```bash
# Basic analysis
ruff run examples/project_log_analyzer.ruff --file server.log

# Filter by pattern and show top 20 results
ruff run examples/project_log_analyzer.ruff -f server.log -p "ERROR" -t 20

# Export as JSON with statistics
ruff run examples/project_log_analyzer.ruff -f server.log --stats --json > report.json
```

**What it does:**
- Counts log levels (ERROR, WARN, INFO, DEBUG)
- Extracts IP addresses and shows top requesters
- Identifies HTTP status codes
- Filters logs by regex patterns
- Generates comprehensive statistics

---

### 2. **Task Manager CLI** (`project_task_manager.ruff`)
**Complete project management tool with persistence**

**Features demonstrated:**
- JSON file persistence (CRUD operations)
- Data structure management
- Fluent command interface
- Progress tracking and statistics
- Pretty-printed formatted output
- Status filtering and reporting

**Usage:**
```bash
# Add tasks
ruff run examples/project_task_manager.ruff add --title "Implement feature X" --priority high --due 2026-12-31

# List tasks
ruff run examples/project_task_manager.ruff list
ruff run examples/project_task_manager.ruff list --status pending

# Complete a task
ruff run examples/project_task_manager.ruff complete --id 1

# Show detailed statistics
ruff run examples/project_task_manager.ruff stats

# Delete a task
ruff run examples/project_task_manager.ruff delete --id 2
```

**What it does:**
- Creates and manages tasks with priorities and due dates
- Persists tasks to JSON file (tasks.json)
- Tracks task status (pending/completed)
- Generates visual progress bars
- Shows comprehensive statistics

---

### 3. **API Testing Tool** (`project_api_tester.ruff`)
**HTTP endpoint testing suite with benchmarking**

**Features demonstrated:**
- HTTP client operations (GET, POST, PUT, DELETE)
- Request/response validation
- Assertion system
- Performance benchmarking
- JSON parsing and validation
- Error handling with Result types
- Statistical analysis (min/max/avg response times)

**Usage:**
```bash
# Basic API test
ruff run examples/project_api_tester.ruff --url https://api.example.com/users/1

# Test with expectations
ruff run examples/project_api_tester.ruff \
  --url https://api.example.com/users/1 \
  --expect-status 200 \
  --expect-field "id=1"

# POST request with data
ruff run examples/project_api_tester.ruff \
  --url https://api.example.com/users \
  --method POST \
  --data '{"name":"John","email":"john@example.com"}' \
  --expect-status 201

# Benchmark endpoint (run 100 times)
ruff run examples/project_api_tester.ruff \
  --url https://api.example.com/health \
  --repeat 100 \
  --benchmark
```

**What it does:**
- Makes HTTP requests with custom headers and data
- Validates response status codes and JSON fields
- Measures response times and calculates statistics
- Shows status code distribution
- Calculates requests per second
- Reports pass/fail for each assertion

---

### 4. **Data Pipeline** (`project_data_pipeline.ruff`)
**CSV to JSON transformer with validation**

**Features demonstrated:**
- CSV file parsing
- Data validation with Result types
- Data transformation and cleaning
- Filtering and column selection
- Error collection and reporting
- JSON serialization
- Comprehensive statistics

**Usage:**
```bash
# Basic transformation
ruff run examples/project_data_pipeline.ruff --input data.csv --output data.json

# With validation
ruff run examples/project_data_pipeline.ruff \
  --input users.csv \
  --output users.json \
  --validate

# Filter and select columns
ruff run examples/project_data_pipeline.ruff \
  --input users.csv \
  --output filtered.json \
  --filter "status=active" \
  --columns "id,name,email"

# Pretty-print JSON output
ruff run examples/project_data_pipeline.ruff -i data.csv -o data.json --pretty
```

**What it does:**
- Parses CSV files with headers
- Validates email formats and data types
- Filters rows based on column values
- Selects specific columns for output
- Normalizes data (lowercase emails, etc.)
- Generates computed fields (full_name from first/last)
- Reports validation errors

---

### 5. **Web Scraper** (`project_web_scraper.ruff`)
**Extract data from websites with pattern matching**

**Features demonstrated:**
- HTTP requests with custom User-Agent
- Regular expression data extraction
- Link discovery and following
- Email and phone number extraction
- JSON export
- Error handling for network failures
- Deduplication

**Usage:**
```bash
# Basic scraping
ruff run examples/project_web_scraper.ruff --url https://example.com

# Extract specific pattern
ruff run examples/project_web_scraper.ruff \
  --url https://example.com \
  --pattern "\\d{4}-\\d{2}-\\d{2}" \
  --output dates.json

# Follow links and scrape
ruff run examples/project_web_scraper.ruff \
  --url https://example.com \
  --follow-links \
  --max-depth 2 \
  --output data.json
```

**What it does:**
- Fetches web pages over HTTP
- Extracts emails, phone numbers, and links
- Follows links to scrape multiple pages
- Extracts data using custom regex patterns
- Saves results as structured JSON
- Reports scraping statistics

---

### 6. **Markdown to HTML Converter** (`project_markdown_converter.ruff`)
**Convert Markdown files to styled HTML**

**Features demonstrated:**
- Text parsing and transformation
- Regular expression replacements
- String manipulation (slice, split, join)
- HTML generation
- CSS styling
- Table of contents generation
- Pattern matching for syntax elements

**Usage:**
```bash
# Basic conversion
ruff run examples/project_markdown_converter.ruff \
  --input README.md \
  --output README.html

# With custom title and TOC
ruff run examples/project_markdown_converter.ruff \
  --input document.md \
  --output document.html \
  --title "My Document" \
  --toc

# Use custom CSS
ruff run examples/project_markdown_converter.ruff \
  --input article.md \
  --output article.html \
  --css styles.css \
  --toc
```

**What it does:**
- Converts Markdown syntax to HTML
- Supports headers, lists, links, images
- Handles bold, italic, and inline code
- Processes code blocks with language tags
- Generates table of contents from headers
- Adds default responsive CSS styling
- Creates anchor links for headers

---

## 🚀 Why These Projects Matter

These showcase projects demonstrate that Ruff is **production-ready** for:

1. **CLI Tools** - Full-featured command-line applications with arg parsing
2. **Data Processing** - CSV/JSON transformation, validation, filtering
3. **Web Integration** - HTTP clients, API testing, web scraping
4. **File Processing** - Log analysis, markdown conversion, data pipelines
5. **Task Automation** - Task management, project tracking, reporting

## 📚 Language Features Demonstrated

Across these projects, you'll see:

- ✅ Command-line argument parsing (`arg_parser()`)
- ✅ File I/O operations (read/write files)
- ✅ HTTP requests (GET, POST, PUT, DELETE)
- ✅ JSON parsing and serialization
- ✅ Regular expressions (pattern matching, replacement)
- ✅ Error handling with Result types (`Ok`/`Err`)
- ✅ Pattern matching (`match` expressions)
- ✅ Data structures (arrays, dictionaries)
- ✅ String manipulation (split, join, slice, trim)
- ✅ Loops and iterators (`for`, `range`)
- ✅ Functions and closures
- ✅ Conditional logic (`if`/`else`)
- ✅ Type conversions
- ✅ Pretty-printed output with Unicode emojis

## 🎓 Learning Path

**Beginner**: Start with individual examples in the examples/ folder  
**Intermediate**: Study these complete projects to see features combined  
**Advanced**: Use these as templates for your own projects

## 💡 Using as Templates

Each project is designed to be:
- **Self-contained** - Run directly without dependencies
- **Well-commented** - Clear explanations of what code does
- **Modular** - Easy to extract and reuse functions
- **Extensible** - Add your own features on top

Feel free to copy, modify, and adapt these projects for your own use cases!

## 📖 More Examples

See also:
- `examples/` - Individual feature demonstrations
- `tests/` - Comprehensive test suite showing all language features
- `CHANGELOG.md` - Latest features and capabilities
- `ROADMAP.md` - Upcoming features

---

**Built with Ruff v0.8.0-dev** 🐾
