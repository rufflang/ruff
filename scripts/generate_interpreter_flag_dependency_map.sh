#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_PATH="${1:-$ROOT/docs/INTERPRETER_FLAG_DEPENDENCY_MAP.md}"
TMP_MATCHES="$(mktemp)"
TMP_ROWS="$(mktemp)"
trap 'rm -f "$TMP_MATCHES" "$TMP_ROWS"' EXIT

cd "$ROOT"

rg -n -- "--interpreter" src tests docs README.md ROADMAP.md examples notes .github \
    | rg -v '^docs/INTERPRETER_FLAG_DEPENDENCY_MAP.md:' >"$TMP_MATCHES"

if [[ ! -s "$TMP_MATCHES" ]]; then
    echo "error: no --interpreter references found in expected search targets" >&2
    exit 1
fi

awk -F: '
function classify(path,    category, tags) {
    if (path == "src/parser.rs") {
        category = "cli-harness"
        tags = "harness-legacy,parity-gap"
    } else if (path == "tests/native_api_security_boundaries.rs" || path == "tests/runtime_security.rs") {
        category = "integration-test"
        tags = "security-test-choice"
    } else if (path == "tests/docs_examples.rs") {
        category = "integration-test"
        tags = "docs-smoke,harness-legacy"
    } else if (path == "tests/diagnostics_golden.rs") {
        category = "integration-test"
        tags = "diagnostics-diff,harness-legacy"
    } else if (path == "tests/package_module_workflow_integration.rs") {
        category = "integration-test"
        tags = "harness-legacy,package-workflow"
    } else if (path ~ /^tests\//) {
        category = "integration-test"
        tags = "harness-legacy"
    } else if (path == "README.md" || path == "ROADMAP.md" || path ~ /^docs\//) {
        category = "documentation"
        tags = "docs-contract"
    } else if (path ~ /^examples\//) {
        category = "example-doc"
        tags = "benchmark-baseline"
    } else if (path ~ /^notes\//) {
        category = "notes-history"
        tags = "archive-note"
    } else {
        category = "other"
        tags = "manual-review"
    }
    return category "|" tags
}

{
    path = $1
    line = $2
    split(classify(path), parts, "|")
    category[path] = parts[1]
    tags[path] = parts[2]
    count[path] += 1
    if (lines[path] == "") {
        lines[path] = line
    } else {
        lines[path] = lines[path] "," line
    }
}

END {
    for (path in count) {
        printf "%s\t%s\t%s\t%d\t%s\n", path, category[path], tags[path], count[path], lines[path]
    }
}
' "$TMP_MATCHES" | LC_ALL=C sort -t$'\t' -k1,1 >"$TMP_ROWS"

parity_gap_count="$(awk -F'\t' '$3 ~ /(^|,)parity-gap($|,)/ {count++} END {print count + 0}' "$TMP_ROWS")"
parity_gap_rows="$(awk -F'\t' '$3 ~ /(^|,)parity-gap($|,)/ {printf "- `%s` (%s)\n", $1, $3}' "$TMP_ROWS")"

mkdir -p "$(dirname "$OUTPUT_PATH")"

{
    echo "# Interpreter Flag Dependency Map"
    echo
    echo "- Generated: $(date '+%Y-%m-%d %H:%M:%S %Z')"
    echo "- Command: \`rg -n -- \"--interpreter\" src tests docs README.md ROADMAP.md examples notes .github\`"
    echo
    echo "Reason tags:"
    echo "- \`harness-legacy\`: Existing harness behavior still forces interpreter mode."
    echo "- \`parity-gap\`: Runtime path currently depends on an explicitly tracked interpreter/VM parity or output-contract gap."
    echo "- \`security-test-choice\`: Security-boundary regression intentionally exercises interpreter path."
    echo "- \`diagnostics-diff\`: Diagnostic contract coverage currently pins interpreter output shape."
    echo "- \`docs-smoke\`: Docs/example smoke harness runs interpreter as canonical execution path."
    echo "- \`package-workflow\`: Package/module workflow integration still validated via interpreter runs."
    echo "- \`docs-contract\`: User-facing docs explicitly describe interpreter mode behavior."
    echo "- \`benchmark-baseline\`: Example/benchmark docs keep interpreter as baseline comparator."
    echo "- \`archive-note\`: Historical field notes mentioning interpreter usage."
    echo
    echo "| File | Category | Reason Tags | Usage Count | Line References |"
    echo "| --- | --- | --- | --- | --- |"
    while IFS=$'\t' read -r path category tags count line_refs; do
        printf '| `%s` | %s | `%s` | %s | %s |\n' "$path" "$category" "$tags" "$count" "$line_refs"
    done <"$TMP_ROWS"
    echo
    echo "## V1U-RUN-005: Parity-Gap Coverage Status"
    echo
    echo "- Current \`parity-gap\` tagged entries: ${parity_gap_count}"
    if [[ "$parity_gap_count" -gt 0 ]]; then
        echo "- Tagged surfaces:"
        printf "%s\n" "$parity_gap_rows"
        echo "- Coverage expectation: each tagged surface must have parity tests or explicit documented divergence."
        echo "- Current closure evidence paths:"
        echo "  - \`tests/cli_contracts.rs\` (bounded runtime fallback contracts)"
        echo "  - \`tests/vm_interpreter_parity_surfaces.rs\` (generator divergence contract)"
        echo "  - \`README.md\` and \`docs/VM_INTERPRETER_PARITY_MATRIX.md\` (canonical divergence docs)"
    else
        echo "- No current \`parity-gap\` tags remain in tracked interpreter-flag surfaces."
    fi
    cat <<'EOF'

## V1U-RUN-002: `ruff test` Interpreter Hardcoding Decision

Current state (`src/parser.rs::run_all_tests`): each fixture is executed via `ruff run <fixture> --interpreter`.

Root-cause evidence for keeping interpreter-pinned today:

- Snapshot corpus compatibility: `ruff test` compares fixture stdout against existing `tests/*.out` snapshots created around interpreter-first behavior.
- Runtime-path drift is still material: a local comparison sweep (`ruff run` vs `ruff run --interpreter`) found 15 mismatches in the first 21 fixtures scanned, including `tests/array_methods_test.ruff`, `tests/net_test.ruff`, `tests/error_call_stack_test.ruff`, and `tests/image_processing_test.ruff`.
- Divergence is not one class of issue: differences include runtime diagnostic code/subsystem shape (`[RUFVM001]` vs `[RUFRUN001]`), optimizer banner output, and builtin availability/behavior differences in legacy fixtures.

Decision (2026-05-20): keep `ruff test` interpreter-pinned for now, and close migration work under `V1U-RUN-003`.

Removal criteria for this hardcoding:

1. Add an explicit runtime-path strategy for `ruff test` (VM-first or dual-engine with deterministic fallback policy).
2. Normalize or rebaseline fixture expectations so runtime-path-specific diagnostics/noise do not create accidental false failures.
3. Add parity coverage for currently divergent fixture classes, then prove the selected strategy with focused command-level tests.
EOF
} >"$OUTPUT_PATH"

echo "generated interpreter dependency map: ${OUTPUT_PATH#$ROOT/}"
