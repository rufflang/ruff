# Native API Security Posture

Status: v1.0.0 baseline draft (active, not a release-ready claim)
Last updated: 2026-05-12

This document defines the security trust model and operational caveats for high-risk native APIs.

This posture document records current trust boundaries only; final 1.0 release readiness is tracked in `ROADMAP.md` and depends on outstanding security/runtime roadmap items.

## Trust Model

Ruff now supports an explicit runtime capability policy for host-effect native APIs in `ruff run` and `ruff test-run`.

Mode defaults:

- default mode (no capability flags): trusted, all host-effect capabilities enabled
- restricted mode (`--untrusted`): deny-by-default for host-effect capabilities
- explicit allow mode (any `--allow-*` flag): restricted baseline with only requested capabilities enabled
- override trusted mode (`--allow-all`): force-enable all host-effect capabilities

Implications:

- Trusted mode behaves like pre-policy Ruff and should be treated as equivalent to running a local program with the current process privileges.
- Restricted mode blocks host-effect API calls until explicitly enabled by capability flags.
- Untrusted scripts should be run with `--untrusted` and only the minimum required `--allow-*` flags, plus external controls (containerization, OS sandboxing, seccomp/apparmor, network egress controls, readonly filesystems, least-privilege credentials).

Capability flags:

- `--allow-fs-read`
- `--allow-fs-write`
- `--allow-fs-delete`
- `--allow-process-exec`
- `--allow-shell-exec`
- `--allow-env-read`
- `--allow-env-write`
- `--allow-net-client`
- `--allow-net-server`
- `--allow-net` (enables both net client/server)
- `--allow-database`
- `--allow-clock`
- `--allow-random`
- `--allow-all`

## High-Risk Surface Areas

### Process APIs

Relevant builtins: `execute`, `execute_status`, `spawn_process`, `pipe_commands`.

Risk profile:

- command execution with host user privileges
- shell/toolchain dependency and environment-variable influence

Execution model:

- `spawn_process` and `pipe_commands` execute direct argv arrays (no shell token expansion)
- `execute` and `execute_status` execute shell command strings (`sh -c` / `cmd /C`) and remain high-risk surfaces
- shell-string execution requires `--allow-shell-exec`; direct argv process execution requires `--allow-process-exec`

Bounded process controls:

- process builtins accept an optional options dict: `timeout_ms`, `max_output_bytes`, `inherit_env`, `env_allow`, `env_deny`, `env`
- defaults: timeout `30000` ms, max captured stdout/stderr `1048576` bytes each
- hard max for `max_output_bytes`: `16777216` bytes
- `execute` preserves legacy string-return behavior on success but now fails with deterministic error objects on timeout, output-limit overflow, or non-zero exits
- `execute_status` / `spawn_process` return a `ProcessResult` struct (`exitcode`, `stdout`, `stderr`, `success`, `timed_out`, `stdout_truncated`, `stderr_truncated`)
- `pipe_commands` enforces the same timeout/output/env policy per stage and fails fast on boundary violations

Operational guidance:

- avoid interpolating untrusted input into command arguments
- prefer explicit argument arrays (`spawn_process`, `pipe_commands`) over shell-style command strings (`execute`, `execute_status`)
- run Ruff in least-privilege execution contexts for automation workloads

### Network APIs

Relevant builtins: TCP and UDP helpers in `src/interpreter/native_functions/network.rs`.

Risk profile:

- outbound connections can exfiltrate data
- listeners can expose local services
- blocking behavior can impact availability if misused

Operational guidance:

- enforce host-level ingress/egress restrictions
- bind listeners to explicit interfaces and non-privileged ports
- validate payload boundaries and size parameters

### Filesystem APIs

Relevant builtins: file read/write/delete, directory operations, archive helpers, path helpers.

Risk profile:

- arbitrary file read/write/delete within process permissions
- archive extraction remains a high-impact host-write surface even with built-in hardening

Current archive-extraction guardrails (`unzip`):

- rejects unsafe entry names: absolute paths, parent traversal (`..`), drive-prefixed segments, null-byte names, and symlink entries
- fails the extraction on the first unsafe entry
- enforces extraction limits: max 1024 entries, max 16 MiB per entry (uncompressed), max 64 MiB total uncompressed bytes

Current file-operation guardrails:

- whole-file reads (`read_file`, `read_lines`, `read_binary_file`, and async read wrapper `read_file_async`) reject files larger than `8 MiB`
- write payloads (`write_file`, `write_file_sync`, `write_file_async`, `write_binary_file`, `append_file`) reject payloads larger than `8 MiB`
- `write_file` and `write_binary_file` default to no-overwrite behavior; replacing an existing file requires explicit `overwrite=true`
- `delete_file` refuses directory paths and only removes regular files (directory removal remains `os_rmdir`, which is non-recursive)

Operational guidance:

- run with least-privilege filesystem permissions
- avoid operating on attacker-controlled archive inputs without pre-validation, especially when expected payloads may exceed the built-in extraction limits
- constrain working directories in production jobs

### Crypto APIs

Relevant builtins: hashing, password hashing/verification, AES, RSA helpers.

Risk profile:

- misuse can create false security guarantees
- key-material handling is caller responsibility

Operational guidance:

- store keys outside source code and rotate through external secret management
- treat crypto helper errors as hard failures, not soft warnings
- avoid custom crypto protocol design in Ruff scripts

### Database APIs

Relevant builtins: `db_connect`, `db_execute`, `db_query`, pools, transactions.

Risk profile:

- credential misuse and lateral movement if connection strings are over-privileged
- destructive SQL execution when inputs are not controlled

Operational guidance:

- use least-privileged database users per workload
- parameterize queries where possible
- isolate network access from Ruff execution environments to only required database endpoints

## Runtime Guardrail Expectations

The current baseline guarantees:

- capability checks at native host-effect boundaries for filesystem/process/shell/environment/network/database/clock/random APIs
- deterministic capability-denied runtime errors in restricted mode
- deterministic misuse errors when capability is enabled but arguments are invalid

This policy is runtime-level gating, not a full sandbox. Scripts still run in-process and should be isolated externally for high-risk deployments.

## Regression Coverage Requirement

Security-sensitive behavior changes to these native categories must include targeted regression tests for failure and misuse boundaries, and update this document if trust assumptions or caveats change.
