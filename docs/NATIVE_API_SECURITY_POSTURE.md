# Native API Security Posture

Status: v1.0.0 baseline draft (active)
Last updated: 2026-05-01

This document defines the security trust model and operational caveats for high-risk native APIs.

## Trust Model

Ruff scripts are trusted code by default. The runtime does not currently sandbox native APIs.

Implications:

- Any script that can call process/network/filesystem/crypto/database builtins can perform host-impacting actions.
- Production environments should treat Ruff script execution as equivalent to running a local program with the current process privileges.
- Untrusted scripts must be isolated with external controls (containerization, OS sandboxing, seccomp/apparmor, network egress controls, readonly filesystems, least-privilege credentials).

## High-Risk Surface Areas

### Process APIs

Relevant builtins: `execute`, `spawn_process`, `pipe_commands`.

Risk profile:

- command execution with host user privileges
- shell/toolchain dependency and environment-variable influence

Operational guidance:

- avoid interpolating untrusted input into command arguments
- prefer explicit argument arrays (`spawn_process`) over shell-style command strings
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

The current baseline guarantees argument-shape validation and deterministic misuse errors for high-risk API entrypoints. It does not provide in-runtime sandboxing, capability-scoped permissions, or policy enforcement.

## Regression Coverage Requirement

Security-sensitive behavior changes to these native categories must include targeted regression tests for failure and misuse boundaries, and update this document if trust assumptions or caveats change.