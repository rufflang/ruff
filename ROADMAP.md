# Ruff Language - Development Roadmap

This roadmap outlines **upcoming** planned features and improvements. For completed features and bug fixes, see [CHANGELOG.md](CHANGELOG.md).

> **Current Version**: v0.10.0 (Released February 2026)  
> **Next Planned Release**: v0.11.0  
> **Status**: Roadmap tracks post-v0.10.0 work only (for completed items see CHANGELOG).

---

## 🎯 What's Next (Priority Order)

**IMMEDIATE (v0.11.0)**:
1. **🔥 Parallel Processing / Concurrency (P0)** - SSG and async runtime throughput focus
2. **Developer Experience Foundations (P1)** - LSP, formatter, linter planning/initial implementation

**AFTER v0.11.0**:
3. **📦 v0.12.0 Developer Experience Expansion** - LSP, formatter, linter, package management
4. **v1.0 Readiness** - stabilization, docs completeness, ecosystem polish

---

## Priority Levels

- **P0 (Critical)**: Highest-priority next release blockers
- **P1 (High)**: Core features needed for v1.0 production readiness
- **P2 (Medium)**: Quality-of-life improvements and developer experience
- **P3 (Low)**: Nice-to-have features for advanced use cases

---

## v0.10.0 Release Status

v0.10.0 work is complete and archived in [CHANGELOG.md](CHANGELOG.md).  
This roadmap intentionally tracks only upcoming items.

---

## v0.10.0 - Release Hardening (P1, Completed)

### Completed Milestones

    - Hardened strict arity contracts for data-format and base64 APIs so trailing arguments now return deterministic contract errors (`parse_json`, `to_json`, `parse_toml`, `to_toml`, `parse_yaml`, `to_yaml`, `parse_csv`, `to_csv`, `encode_base64`, `decode_base64`)
    - Added comprehensive release-hardening contract coverage for data-format/base64 strict-arity behavior at both module and dispatcher integration layers
    - Preserved existing argument-shape error contracts while hardening extra-argument rejection

    - Hardened strict arity contracts for math APIs so trailing arguments now return deterministic contract errors (`abs`, `sqrt`, `floor`, `ceil`, `round`, `sin`, `cos`, `tan`, `log`, `exp`, `pow`, `min`, `max`)
    - Added comprehensive release-hardening contract coverage for math strict-arity behavior at both module and dispatcher integration layers
    - Preserved missing/invalid argument compatibility fallback behavior (`Int(0)`) while hardening extra-argument rejection

    - Hardened strict arity contracts for conversion and introspection APIs so trailing arguments now return deterministic contract errors (`parse_int`, `parse_float`, `to_int`, `to_float`, `to_string`, `to_bool`, `bytes`, `type`, `is_int`, `is_float`, `is_string`, `is_bool`, `is_array`, `is_dict`, `is_null`, `is_function`)
    - Added comprehensive release-hardening contract coverage for conversion/introspection strict-arity behavior at both module and dispatcher integration layers
    - Preserved missing-argument compatibility fallback behavior for `is_*` checks (`false`) while hardening extra-argument rejection

    - Hardened strict arity contracts for assertion/testing APIs so trailing arguments now return deterministic contract errors (`assert`, `assert_equal`, `assert_true`, `assert_false`, `assert_contains`)
    - Added comprehensive release-hardening contract coverage for assertion/testing strict-arity behavior at both module and dispatcher integration layers
    - Preserved `debug(...)` variadic compatibility with explicit regression coverage

    - Hardened strict arity contracts for advanced IO APIs so trailing arguments now return deterministic contract errors (`io_read_bytes`, `io_write_bytes`, `io_append_bytes`, `io_read_at`, `io_write_at`, `io_seek_read`, `io_file_metadata`, `io_truncate`, `io_copy_range`)
    - Added comprehensive release-hardening strict-arity contract coverage for advanced IO APIs at both module and dispatcher integration layers
    - Preserved existing argument-shape validation contracts and success-path behavior while hardening extra-argument rejection

    - Hardened strict arity contracts for core array and higher-order collection APIs so trailing arguments now return deterministic contract errors (`push`, `append`, `pop`, `insert`, `remove`, `remove_at`, `clear`, `slice`, `concat`, `map`, `filter`, `reduce`, `find`, `any`, `all`, `sort`, `reverse`, `unique`, `sum`, `chunk`, `flatten`, `zip`, `enumerate`, `take`, `skip`, `windows`)
    - Added comprehensive release-hardening coverage for the APIs above at both module and dispatcher integration layers, including strict-arity rejection and representative behavior contracts

    - Expanded release-hardening critical dispatcher coverage for async runtime and task/channel APIs (`channel`, `async_sleep`, `async_timeout`, `async_http_get`, `async_http_post`, `async_read_file`, `async_write_file`, `spawn_task`, `await_task`, `cancel_task`)
    - Added behavior plus argument-shape/error-shape contract coverage for async sleep/timeout behavior, async file wrapper lifecycle behavior, async HTTP wrapper argument-shape validation, and task-handle lifecycle semantics
    - Tightened async task/channel contract enforcement by requiring `channel()` zero-arity and adding explicit missing/wrong-type argument-shape coverage for `await_task(...)` and `cancel_task(...)`

    - Expanded release-hardening critical dispatcher coverage for core filesystem APIs (`read_file`, `write_file`, `append_file`, `file_exists`, `read_lines`, `list_dir`, `create_dir`, `file_size`, `delete_file`, `rename_file`, `copy_file`, `read_binary_file`, `write_binary_file`)
    - Added behavior plus argument-shape/error-shape contract coverage for text and binary file lifecycle operations, directory/listing operations, and mutation/cleanup operations
    - Hardened strict arity contracts for core filesystem APIs so trailing arguments now return deterministic contract errors

    - Added modular dispatch coverage for declared TCP/UDP APIs (`tcp_listen`, `tcp_accept`, `tcp_connect`, `tcp_send`, `tcp_receive`, `tcp_close`, `tcp_set_nonblocking`, `udp_bind`, `udp_send_to`, `udp_receive_from`, `udp_close`)
    - Added release-hardening tests for argument-shape/error-shape validation and end-to-end TCP/UDP round-trip behavior contracts
    - Hardened strict arity contracts for declared TCP/UDP APIs so trailing arguments now return deterministic contract errors
    - Added comprehensive strict-arity contract coverage for all declared TCP/UDP APIs (extra-argument rejection per entry point)
    - Reduced exhaustive dispatch known-gap list by removing migrated network APIs

    - Added modular dispatch coverage for declared image API (`load_image`)
    - Added release-hardening tests for argument-shape validation, missing-file error behavior, and successful image load contracts
    - Hardened strict arity contract for `load_image` so trailing arguments now return deterministic contract errors
    - Added explicit strict-arity extra-argument rejection coverage for `load_image(...)` at dispatcher integration level
    - Reduced exhaustive dispatch known-gap list by removing migrated `load_image`

    - Added modular dispatch coverage for declared compression/archive APIs (`zip_create`, `zip_add_file`, `zip_add_dir`, `zip_close`, `unzip`)
    - Added release-hardening tests for zip argument-shape/error-shape contracts and end-to-end zip/unzip round-trip behavior
    - Hardened strict arity contracts for compression/archive APIs so trailing arguments now return deterministic contract errors
    - Reduced exhaustive dispatch known-gap list by removing migrated compression/archive APIs

    - Added modular dispatch coverage for declared `Set(...)` constructor
    - Preserved constructor-shape behavior for empty/default construction, array-based construction, and deduplicated set semantics
    - Added release-hardening contract coverage and reduced exhaustive dispatch known-gap list by removing `Set`

    - Synchronized VM builtin name registry (`get_builtin_names`) with interpreter-native registrations for release stability
    - Added regression coverage to detect missing and duplicate builtin API entries

    - Restored modular runtime support for declared OS/path builtins and path aliases
    - Added queue/stack size API parity (`queue_size`, `stack_size`) in modular native handlers
    - Added integration tests for alias equivalence and path/collection API behavior

    - Hardened filesystem alias argument-shape behavior for `join_path(...)` and `path_join(...)`
    - Hardened collection size API argument validation for `queue_size(...)` and `stack_size(...)`
    - Added integration regression coverage for async/filesystem/collection argument-shape parity (`promise_all`/`await_all`, `join_path`/`path_join`, `queue_size`/`stack_size`)

    - Removed silent unknown-native fallback in modular native dispatcher
    - Unknown native names now return explicit runtime errors for faster API drift detection
    - Added dispatcher-level regression coverage to ensure newly introduced high-risk builtins do not regress into unknown-native fallback

    - Extended builtin API contract coverage for newly introduced concurrency/async entries in `get_builtin_names`
    - Added integration argument-shape/error-shape parity coverage for:
      - `parallel_map(...)` / `par_map(...)`
      - `shared_set/get/has/delete/add_int`
      - `set_task_pool_size(...)` / `get_task_pool_size(...)`

    - Added release-hardening dispatch coverage so `par_each` cannot regress into unknown-native fallback
    - Added argument-shape + error-shape contract coverage for `par_each(...)` and parity checks against `parallel_map(...)`

    - Added release-hardening test coverage that probes all declared builtins from `get_builtin_names()` against modular native dispatch
    - Added explicit known-gap contract guardrail plus safe-probe strategy for blocking/terminal APIs (`input`, `exit`) to detect dispatch drift early while preserving deterministic CI behavior

    - Added modular dispatch coverage for env and CLI argument APIs (`env*`, `args`, `arg_parser`)
    - Added targeted native-function tests for env contracts and ArgParser creation shape
    - Reduced exhaustive dispatch known-gap list by removing now-migrated system APIs

    - Added modular dispatch coverage for side-effecting system operation APIs (`input`, `exit`, `sleep`, `execute`)
    - Added release-hardening contract coverage for system operation argument-shape behavior (strict arity plus wrong-type validation)
    - Removed exhaustive dispatch probe skips by probing `input`/`exit` with non-side-effect argument shapes in drift coverage

    - Added modular dispatch coverage for JSON/TOML/YAML/CSV parse/serialize APIs and Base64 encode/decode APIs
    - Added targeted native-function tests for round-trip behavior and argument-shape validation
    - Reduced exhaustive dispatch known-gap list by removing now-migrated data-format/encoding APIs

    - Added modular dispatch coverage for `regex_match`, `regex_find_all`, `regex_replace`, and `regex_split`
    - Added targeted native-function tests for regex behavior and argument-shape validation
    - Reduced exhaustive dispatch known-gap list by removing now-migrated regex APIs

    - Closed modular dispatch drift so declared builtins `contains` and `index_of` no longer regress into unknown-native fallback
    - Preserved polymorphic array behavior by delegating array-first calls to collection handlers
    - Added targeted contract coverage for string success behavior plus argument-shape/error-shape validation
    - Reduced exhaustive dispatch known-gap list by removing migrated string APIs

    - Added modular dispatch coverage for advanced IO APIs (`io_read_bytes`, `io_write_bytes`, `io_append_bytes`, `io_read_at`, `io_write_at`, `io_seek_read`, `io_file_metadata`, `io_truncate`, `io_copy_range`)
    - Added native-function regression tests for round-trip, offset, metadata/truncate, range-copy, and argument-shape/error-shape contracts
    - Expanded dispatcher-level release-hardening contract coverage for migrated `io_*` APIs
    - Reduced exhaustive dispatch known-gap list by removing now-migrated IO APIs

    - Added modular dispatch coverage for declared HTTP request/response/server APIs (`http_get`, `http_post`, `http_put`, `http_delete`, `http_get_binary`, `http_get_stream`, `http_server`, `set_header`, `set_headers`, `http_response`, `json_response`, `html_response`, `redirect_response`)
    - Added native-function regression tests for response helper/header contracts, redirect/server helper behavior, and argument-shape/error-shape validation
    - Expanded dispatcher-level release-hardening contract coverage for migrated HTTP APIs
    - Reduced exhaustive dispatch known-gap list by removing now-migrated HTTP APIs

    - Added modular dispatch coverage for declared database APIs (`db_connect`, `db_execute`, `db_query`, `db_close`, `db_pool`, `db_pool_acquire`, `db_pool_release`, `db_pool_stats`, `db_pool_close`, `db_begin`, `db_commit`, `db_rollback`, `db_last_insert_id`)
    - Added native-function regression tests for SQLite-backed connect/execute/query/close behavior, transaction lifecycle, pool lifecycle, and argument-shape/error-shape validation
    - Hardened strict arity contracts for database APIs (`db_*`) so trailing arguments now return deterministic contract errors
    - Expanded dispatcher-level release-hardening contract coverage for migrated database APIs
    - Reduced exhaustive dispatch known-gap list by removing now-migrated database APIs

    - Added modular dispatch coverage for declared process APIs (`spawn_process`, `pipe_commands`)
    - Added native-function regression tests for process result struct shape, pipeline output behavior, and argument-shape/error-shape validation
    - Hardened strict arity contracts for process APIs (`spawn_process`, `pipe_commands`) so trailing arguments now return deterministic contract errors
    - Expanded dispatcher-level release-hardening contract coverage for migrated process APIs
    - Reduced exhaustive dispatch known-gap list by removing now-migrated process APIs

    - Added modular dispatch coverage for declared crypto/hash APIs (`sha256`, `md5`, `md5_file`, `hash_password`, `verify_password`, `aes_encrypt/decrypt`, `aes_encrypt_bytes/decrypt_bytes`, `rsa_generate_keypair`, `rsa_encrypt/decrypt`, `rsa_sign`, `rsa_verify`)
    - Added native-function regression tests for hash vectors, bcrypt verify behavior, AES string/bytes round trips, RSA keygen/encrypt/decrypt/sign/verify behavior, and argument-shape/key-size validation
    - Hardened strict arity contracts across crypto/hash APIs so extra arguments return deterministic contract errors instead of being silently tolerated
    - Expanded dispatcher-level release-hardening contract coverage for migrated crypto APIs
    - Reduced exhaustive dispatch known-gap list by removing now-migrated crypto APIs

- **Async Alias + SSG Contract Follow-Through (✅ Complete, February 2026)**
        - Expanded release-hardening critical dispatcher coverage for async alias surfaces (`Promise.all`, `parallel_map`, `par_map`)
        - Added async alias argument-shape/error-shape parity contract coverage for `Promise.all(...)` / `promise_all(...)` / `await_all(...)` and `parallel_map(...)` / `par_map(...)` / `par_each(...)`
        - Added release-hardening contract coverage for `ssg_render_pages(...)` argument validation and successful result-shape behavior

- **Async Batch + Shared State/Task-Pool Contract Follow-Through (✅ Complete, February 2026)**
    - Added release-hardening contract coverage for async batch file wrappers (`async_read_files`, `async_write_files`) including argument-shape/error-shape validation and write/read lifecycle behavior
    - Added release-hardening contract coverage for shared-state APIs (`shared_set`, `shared_get`, `shared_has`, `shared_delete`, `shared_add_int`) including key lifecycle and integer-mutation behavior
    - Added release-hardening contract coverage for task-pool sizing APIs (`set_task_pool_size`, `get_task_pool_size`) including argument-shape/error-shape and return-shape validation

- **Advanced HTTP API Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for HTTP auth/concurrency APIs (`parallel_http`, `jwt_encode`, `jwt_decode`, `oauth2_auth_url`, `oauth2_get_token`)
    - Added argument-shape/error-shape contract coverage for all advanced HTTP APIs above
    - Hardened strict arity contracts for HTTP request/response helpers and advanced HTTP APIs so extra arguments now return deterministic contract errors instead of being silently ignored
    - Added behavior contract coverage for empty `parallel_http(...)` shape, JWT encode/decode payload integrity, and OAuth2 authorization URL output structure

- **Core Alias Behavior Parity Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for core aliases (`upper`, `lower`, `replace`, `append`)
    - Added behavior parity contract coverage for `to_upper(...)`/`upper(...)`, `to_lower(...)`/`lower(...)`, `replace_str(...)`/`replace(...)`, and `push(...)`/`append(...)`

- **String Utility Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for string utility APIs (`starts_with`, `ends_with`, `repeat`, `char_at`, `is_empty`, `count_chars`)
    - Added behavior and fallback contract coverage for predicate behavior, repeat/character access behavior, missing/invalid argument-shape fallbacks, and Unicode character counting behavior

- **String Transform and Tokenization Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for string transform/tokenization APIs (`substring`, `capitalize`, `trim`, `trim_start`, `trim_end`, `split`, `join`)
    - Added behavior and fallback contract coverage for substring bounds/missing arguments, capitalization/trim-family behavior, split shape/missing-delimiter fallback, and join stringification/missing-separator fallback

- **Advanced String Methods Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for advanced string APIs (`pad_left`, `pad_right`, `lines`, `words`, `str_reverse`, `slugify`, `truncate`, `to_camel_case`, `to_snake_case`, `to_kebab_case`)
    - Added behavior and error/fallback contract coverage for padding semantics, line/word tokenization shape, Unicode reverse behavior, slug normalization behavior, truncate suffix handling, and case-conversion argument validation

- **Data-Format/Base64 + Regex Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for data-format/base64 APIs (`parse_json`, `to_json`, `parse_toml`, `to_toml`, `parse_yaml`, `to_yaml`, `parse_csv`, `to_csv`, `encode_base64`, `decode_base64`)
    - Expanded release-hardening critical dispatcher coverage for regex APIs (`regex_match`, `regex_find_all`, `regex_replace`, `regex_split`)
    - Added behavior plus argument-shape/error-shape contract coverage for parse/serialize round trips, base64 encode/decode behavior, and regex match/find/replace/split behavior

- **Env/OS-Path + Assertion Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for env/system APIs (`env*`, `args`, `arg_parser`), OS/path APIs (`os_*`, `dirname`, `basename`, `path_*`), and testing/assertion APIs (`assert`, `debug`, `assert_equal`, `assert_true`, `assert_false`, `assert_contains`)
    - Added behavior plus argument-shape/error-shape contract coverage for env typed parsing/default/required behavior, ArgParser shape, OS directory lifecycle, path helper semantics, and assertion success/failure validation
    - Hardened strict arity contracts for env/system APIs (`env*`, `args`, `arg_parser`) so trailing arguments now return deterministic contract errors
    - Hardened strict arity contracts for OS/path APIs (`os_*`, `dirname`, `basename`, `path_*`) so trailing arguments now return deterministic contract errors

- **Random and Time API Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for system random/time/date APIs (`random`, `random_int`, `random_choice`, `set_random_seed`, `clear_random_seed`, `now`, `current_timestamp`, `performance_now`, `time_us`, `time_ns`, `format_duration`, `elapsed`, `format_date`, `parse_date`)
    - Added deterministic seeded-random parity contracts, random range/choice shape contracts, monotonic timing contracts, and formatting/parsing argument-shape contracts
    - Hardened strict arity contracts across all random/time/date APIs above so trailing arguments now return deterministic contract errors

- **Math API Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for math APIs (`abs`, `sqrt`, `pow`, `floor`, `ceil`, `round`, `min`, `max`, `sin`, `cos`, `tan`, `log`, `exp`)
    - Added deterministic behavior contracts plus compatibility fallback contracts for missing/invalid argument shapes

- **Collections and Format API Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for collection/format APIs (`range`, `keys`, `values`, `items`, `has_key`, `get`, `merge`, `invert`, `update`, `get_default`, `format`)
    - Added deterministic dict-order contracts, lookup/update behavior contracts, range argument-shape contracts, and `format(...)` placeholder/argument-shape contracts

- **Set/Queue/Stack API Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for collection constructor/method APIs (`set_add`, `set_has`, `set_remove`, `set_union`, `set_intersect`, `set_difference`, `set_to_array`, `Queue`, `queue_enqueue`, `queue_dequeue`, `queue_peek`, `queue_is_empty`, `queue_to_array`, `Stack`, `stack_push`, `stack_pop`, `stack_peek`, `stack_is_empty`, `stack_to_array`)
    - Added behavior and argument-shape contract coverage for mutation, membership, conversion, and dequeue/pop result-shape behavior
    - Added explicit queue/stack size type-validation checks (`queue_size(...)`, `stack_size(...)`) in the same hardening slice

- **Polymorphic `len(...)` Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage to include `len`
    - Added polymorphic contract coverage for `len(...)` on `String`, `Array`, `Dict`, `Bytes`, `Set`, `Queue`, and `Stack`
    - Added fallback contract coverage for unsupported/missing arguments (`len(null)` and `len()` return `0`)

- **`type(...)` + `is_*` Introspection Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for `type`, `is_int`, `is_float`, `is_string`, `is_bool`, `is_array`, `is_dict`, `is_null`, and `is_function`
    - Added contract coverage for `type(...)` result-shape behavior (`string`, `array`, `null`) and missing-argument error shape
    - Added `is_*` bool-return contract coverage for matching/non-matching values plus missing-argument fallback behavior

- **Conversion + `bytes(...)` Contract Follow-Through (✅ Complete, February 2026)**
    - Expanded release-hardening critical dispatcher coverage for conversion/validation builtins (`parse_int`, `parse_float`, `to_int`, `to_float`, `to_string`, `to_bool`, `bytes`)
    - Added contract coverage for valid conversion behavior and invalid parse/conversion error-shape behavior
    - Added `bytes(...)` validation coverage for value-range and argument-shape errors

- **Optional Static Typing Design Package (✅ Complete, February 2026)**
    - Added a consolidated exploratory design document at `docs/OPTIONAL_TYPING_DESIGN.md`
    - Finalized proposed annotation surface for functions, variables, and collection types with parser/type-check impact notes
    - Defined optional runtime type-check mode contract (opt-in behavior, expected error shape, and compatibility posture)
    - Documented typed-JIT optimization boundaries, explicit non-goals, migration compatibility notes, and open decisions

### Remaining Focus

- Stabilize external-facing runtime APIs and aliases ahead of v1.0
- Continue compatibility contract expansion as future builtins and aliases are introduced

### v0.10 Remaining Work Queue (Use in Future Chats)

#### 1) Release Hardening Follow-Through (P1)

**Status**: Ready to execute (iterative)

**Run This in Future Chats**:
- Pick newly added or recently modified builtins/aliases since last hardening pass.
- Add/extend compatibility contract tests for:
    - builtin-name list parity (`get_builtin_names`)
    - alias behavior parity
    - argument-shape + error-shape parity
    - unknown-native dispatch safety
- Run full validation (`cargo test`, warning-free build) and update docs.

**Done Criteria (v0.10)**:
- No known drift between declared, registered, and dispatched external APIs.
- Contract coverage exists for each newly introduced public builtin/alias in v0.10 scope.
- Changelog and README hardening notes stay synchronized with implemented coverage.

#### 2) Optional Static Typing Design Package (Exploratory)

**Status**: ✅ Complete (design package delivered, February 2026)

**Output**:
- Consolidated design document: `docs/OPTIONAL_TYPING_DESIGN.md`
- Includes annotation surface, optional runtime type-check contract, typed-JIT boundaries, migration notes, open decisions, and non-goals

**Done Criteria (v0.10)**:
- A concrete design document exists for optional typing (syntax, semantics, migration/compatibility notes).
- Open decisions and non-goals are explicitly listed.
- No commitment to mandatory typing or v1.0 breaking changes.

---

## v0.11.0 - Parallel Processing & Concurrency (P0)

**Focus**: Deliver production-grade async throughput for large I/O-bound workloads (SSG priority)  
**Timeline**: Q2-Q3 2026  
**Priority**: P0 - CRITICAL for production performance perception  
**Status**: In Progress

### Completed Milestones

- **Async VM Integration Completion (✅ Complete, February 2026)**
  - Made cooperative suspend/resume the default execution model for async-heavy workloads
  - Enabled `cooperative_suspend_enabled: true` in VM constructor to activate non-blocking await semantics by default
  - Integrated cooperative scheduler loop into main execution path (`execute_until_suspend()` + `run_scheduler_until_complete()`)
  - Replaced blocking `vm.execute()` with cooperative scheduler in VM execution entry points
  - User-defined async functions now execute with true concurrency semantics
  - Eliminated remaining blocking `block_on()` bottleneck in critical VM await paths
  - Added comprehensive integration tests for cooperative default behavior (7 new tests, all passing)
  - Result: Production-ready non-blocking async VM ready for SSG and I/O-bound workloads

### Scope (Forward Work Only)

Existing async/runtime groundwork is tracked in [CHANGELOG.md](CHANGELOG.md).
v0.11.0 tracks only the remaining performance and architecture work.

### Remaining High-Priority Workstreams

1. **SSG Throughput Focus (Primary Benchmark Gate)**
     - Continue reducing render/write overhead in `bench-ssg` execution path.
     - Add additional native bulk helpers where script-level hot loops dominate.
     - Keep checksum/file-count equivalence validation intact for all benchmark path changes.

2. **Benchmark Stability & Measurement Quality**
     - Add repeat-run/median reporting to reduce one-off benchmark noise.
     - Keep Ruff-only stage profiling (`--profile-async`) as the optimization signal.
     - Keep cross-language runs (`--compare-python`) for directional trend tracking.

### Success Criteria

- `bench-ssg` Ruff build time: **<10 seconds** (phase target)
- Stretch target: **<5 seconds**
- No regressions in correctness (`cargo test` remains green)
- Async API surface remains stable and documented

### Performance Snapshot (Tracking)

- Baseline (synchronous): ~55 seconds
- Current (`ruff bench-ssg --compare-python`): ~36 seconds (Ruff local sample)
- Target (v0.11.0): <10 seconds

---

## v0.10.0 - Optional Static Typing (Exploratory)

**Status**: ✅ Design package completed in v0.10.0  
**Timeline**: Completed (February 2026)  
**Priority**: Exploratory

**Key Question**: Should Ruff adopt optional static typing?

### Stage 1: Type Annotations (Documentation Only)

```ruff
func calculate(x: int, y: float) -> float {
    return x * y
}

let name: string := "Alice"
let scores: Array<int> := [95, 87, 92]
```

### Stage 2: Optional Runtime Type Checking

```ruff
@type_check
func calculate(x: int, y: float) -> float {
    return x * y
}
```

### Stage 3: JIT Optimization for Typed Code

Typed code could enable 10-50x performance improvements through:
- Unboxed arithmetic
- Stack allocation
- SIMD vectorization

**Status**: 🔬 EXPLORATORY - Not committed for v1.0

---

## v0.12.0 - Developer Experience

**Focus**: World-class tooling for productivity  
**Timeline**: Q3 2026  
**Priority**: P1

### Language Server Protocol (LSP) (P1)

**Estimated Effort**: Large (4-6 weeks)

**Features**:
- Autocomplete (built-ins, variables, functions)
- Go to definition
- Find references
- Hover documentation
- Real-time diagnostics
- Rename refactoring
- Code actions

**Implementation**: Use `tower-lsp` Rust framework

---

### Code Formatter (ruff-fmt) (P1)

**Estimated Effort**: Medium (2-3 weeks)

**Features**:
- Opinionated formatting (like gofmt, black)
- Configurable indentation
- Line length limits
- Import sorting

---

### Linter (ruff-lint) (P1)

**Estimated Effort**: Medium (3-4 weeks)

**Rules**:
- Unused variables
- Unreachable code
- Type mismatches
- Missing error handling
- Auto-fix for simple issues

---

### Package Manager (P1)

**Estimated Effort**: Large (8-12 weeks)

**Features**:
- `ruff.toml` project configuration
- Dependency management with semver
- Package registry
- CLI: `ruff init`, `ruff add`, `ruff install`, `ruff publish`

---

### REPL Improvements (P2)

**Estimated Effort**: Medium (1-2 weeks)

**Features**:
- Tab completion
- Syntax highlighting
- Multi-line editing
- `.help <function>` documentation

---

### Documentation Generator (P2)

**Estimated Effort**: Medium (2-3 weeks)

Generate HTML documentation from doc comments:

```ruff
/// Calculates the square of a number.
/// 
/// # Examples
/// ```ruff
/// result := square(5)  # 25
/// ```
func square(n) {
    return n * n
}
```

---

## v0.11.0+ - Stub Module Completion

**Status**: Planned  
**Priority**: P2 - Implement on-demand

### JSON Module (json.rs)
- `parse_json()`, `to_json()`, `json_get()`, `json_merge()`
- **Trigger**: When users need JSON API integration

### Crypto Module (crypto.rs)
- Hashing: MD5, SHA256, SHA512
- Encryption: AES, RSA
- Digital signatures
- **Trigger**: When users need secure authentication

### Database Module (database.rs)
- MySQL, PostgreSQL, SQLite connections
- Query execution, transactions
- Connection pooling
- **Trigger**: When users need persistent storage

### Network Module (network.rs)
- TCP/UDP socket operations
- **Trigger**: When users need low-level networking

---

## v1.0.0 - Production Ready

**Focus**: Polish, documentation, community  
**Timeline**: Q4 2026  
**Goal**: Production-ready language competitive with Python/Go

**Requirements**:
- All v0.10.0 release-hardening and stability targets met
- All v0.11.0 tooling complete
- Comprehensive documentation
- Stable API (no breaking changes)

---

## Future Versions (v1.0.0+)

### Generic Types (P2)
```ruff
func first<T>(arr: Array<T>) -> Option<T> {
    if len(arr) > 0 { return Some(arr[0]) }
    return None
}
```

### Union Types (P2)
```ruff
func process(value: int | string | null) {
    match type(value) {
        case "int": print("Number")
        case "string": print("Text")
    }
}
```

### Enums with Methods (P2)
```ruff
enum Status {
    Pending,
    Active { user_id: int },
    Completed { result: string }
    
    func is_done(self) {
        return match self {
            case Status::Completed: true
            case _: false
        }
    }
}
```

### Macros & Metaprogramming (P3)
```ruff
macro debug_print(expr) {
    print("${expr} = ${eval(expr)}")
}
```

### Foreign Function Interface (FFI) (P3)
```ruff
lib := load_library("libmath.so")
extern func cos(x: float) -> float from lib
```

### WebAssembly Compilation (P3)
```bash
ruff build --target wasm script.ruff
```

### AI/ML Built-in (P3)
```ruff
import ml
model := ml.linear_regression()
model.train(x_train, y_train)
```

---

## 🤝 Contributing

**Good First Issues**:
- String utility functions
- Array utility functions
- Test coverage improvements

**Medium Complexity**:
- JIT opcode coverage expansion
- Error message improvements
- Standard library modules

**Advanced Projects**:
- LSP implementation
- Package manager
- JIT performance optimization
- Debugger implementation

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## Version Strategy

- **v0.10.0**: Architecture cleanup + release hardening + typing exploration
- **v0.11.0**: Parallel processing / concurrency performance (SSG focus)
- **v0.12.0**: Developer experience (LSP, package manager)
- **v1.0.0**: Production-ready 🎉

**See Also**:
- [CHANGELOG.md](CHANGELOG.md) - Completed features
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md) - Performance guide
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - System architecture

---

*Last Updated: February 18, 2026*
