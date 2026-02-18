# Ruff Showcase Portfolio (Advanced Developer Edition)

This folder is for portfolio-grade Ruff projects intended to impress experienced developers, technical leads, and infrastructure-minded engineers.

## Target Standard (Non-Negotiable)

Each showcase should include:

- **Production-style CLI UX**: clear flags, help output, validation, and exit codes.
- **Deterministic outputs**: machine-readable reports (JSON/CSV) and stable summaries.
- **Failure design**: retries, timeouts, typed errors, and actionable diagnostics.
- **Performance awareness**: timing data, concurrency controls, and benchmarkable behavior.
- **Testability**: at least one deterministic test mode or fixture-driven execution path.

## Current Ruff Capability Surface Used Here

- Async runtime + task orchestration
- HTTP client/server utilities
- Filesystem + path + archive operations
- Regex and string transforms
- JSON/TOML/YAML/CSV parsing + serialization
- SQLite/database APIs
- Crypto/hash helpers
- Process execution + command piping

## 25 High-Quality Showcase Projects

1. **Static Site + Markdown Blog Builder (SSG)**
   - Build markdown + front matter into templated HTML with pagination, tag indexes, and incremental rebuild hashing.
   - **Proof points**: deterministic build manifest, cache hit ratio, and build-time stats.

2. **Documentation Coverage Analyzer**
   - Scan code/docs trees to find undocumented symbols, stale references, and broken doc links.
   - **Proof points**: coverage score, stale-reference report, CI-friendly JSON output.

3. **Static Output QA Auditor**
   - Validate generated site outputs: broken links, missing assets, duplicate routes, and orphaned files.
   - **Proof points**: severity-ranked findings and reproducible fail-on-threshold mode.

4. **Release Notes Synthesizer**
   - Generate release notes from changelog + commit metadata using categorized sections and contributor summaries.
   - **Proof points**: deterministic markdown generation and diff-aware incremental mode.

5. **Authorized Web Data Extractor**
   - Crawl approved targets, normalize extracted fields, and persist structured output for downstream analysis.
   - **Proof points**: rate-limited fetch queue, retry policy metrics, schema validation report.

6. **Feed Intelligence Aggregator (RSS/Atom)**
   - Merge multiple feeds, dedupe semantically similar entries, and rank by freshness/topic.
   - **Proof points**: dedupe ratio, source reliability stats, signed digest artifact.

7. **Policy-Aware API Gateway**
   - Reverse proxy with API keys, route-level policy checks, and request accounting.
   - **Proof points**: per-key quotas, p95 latency, structured audit logs.

8. **Webhook Relay with Durable Retries**
   - Queue webhook deliveries with exponential backoff, idempotency keys, and dead-letter handling.
   - **Proof points**: delivery SLO dashboard and replay-safe recovery flow.

9. **Service Uptime + Latency Profiler**
   - Concurrent endpoint checks with windowed SLA/SLO calculations and anomaly detection.
   - **Proof points**: rolling availability metrics and p50/p95/p99 latency snapshots.

10. **Security Header Compliance Scanner**
    - Evaluate headers and TLS-related response metadata against a configurable policy baseline.
    - **Proof points**: weighted compliance score and remediation guidance export.

11. **Authorized API Surface Mapper**
    - Enumerate known endpoints, classify status/body shapes, and detect contract drift.
    - **Proof points**: response-shape clustering and change-detection diff report.

12. **SQLite Migration + Drift Manager**
    - Apply ordered migrations with checksums, drift detection, and rollback checkpoints.
    - **Proof points**: migration ledger table and idempotent rerun guarantees.

13. **Streaming Log Intelligence CLI**
    - Parse large logs, detect error bursts, correlate signatures, and summarize incidents.
    - **Proof points**: top-N root-cause signatures and time-bucketed incident timeline.

14. **Project Task Ops CLI (SQLite-backed)**
    - Advanced task workflow engine with tags, priorities, due windows, and queryable snapshots.
    - **Proof points**: filter/query DSL, export profiles, and completion trend analytics.

15. **Versioned Backup + Restore Utility**
    - Create deduplicated snapshots with archive packaging and verified restore integrity.
    - **Proof points**: restore simulation mode, checksum validation, and retention policy stats.

16. **Filesystem Integrity Monitor**
    - Track file-tree mutation with cryptographic hashing and signed baseline manifests.
    - **Proof points**: tamper report with before/after diff and confidence scoring.

17. **Duplicate Data Reclaimer**
    - Detect duplicate files via multi-stage matching (size, hash, optional content fingerprint).
    - **Proof points**: reclaimable space estimation and safe-action plan generation.

18. **Secrets + Runtime Config Gatekeeper**
    - Validate env/config contracts (required/typed/defaulted) before app start.
    - **Proof points**: startup readiness report and security-safe redaction policy.

19. **Structured ETL Engine (JSON/CSV)**
    - Execute repeatable ETL transforms with schema normalization and quality checks.
    - **Proof points**: row-level reject logs and transform lineage summary.

20. **Multi-Format Config Linter**
    - Lint JSON/TOML/YAML for structural and semantic rule violations.
    - **Proof points**: rule IDs, auto-fix suggestions, machine-consumable diagnostics.

21. **Benchmark Campaign Runner**
    - Execute benchmark suites with warmups, repeated trials, and comparative result archives.
    - **Proof points**: median/stdev reporting and regression detection thresholds.

22. **Command Pipeline Orchestrator**
    - Run multi-stage shell workflows with dependency ordering, retries, and artifact capture.
    - **Proof points**: execution DAG report, per-step timing, and failure replay commands.

23. **Unified AI Provider CLI**
    - Single interface for multiple LLM endpoints with retries, token accounting, and caching.
    - **Proof points**: provider latency/cost comparisons and normalized response schema.

24. **Prompt Regression + Drift Harness**
    - Evaluate prompt suites across model versions and compare quality metrics over time.
    - **Proof points**: pass/fail matrix, semantic drift scoring, and trend snapshots.

25. **Hybrid Moderation/Classification Pipeline**
    - Combine rules + model inference for policy classification with confidence thresholds.
    - **Proof points**: precision/recall summary on fixture sets and escalation queue output.

## Recommended Initial Portfolio Sequence

1. Static Site + Markdown Blog Builder (SSG)
2. Policy-Aware API Gateway
3. Unified AI Provider CLI
4. Prompt Regression + Drift Harness
5. SQLite Migration + Drift Manager

## Safety + Trust Notes

- Security-related projects should be defensive and explicitly authorized.
- AI integrations should use env-managed keys and redact sensitive values from logs/reports.
