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
        tags = "harness-legacy"
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

mkdir -p "$(dirname "$OUTPUT_PATH")"

{
    echo "# Interpreter Flag Dependency Map"
    echo
    echo "- Generated: $(date '+%Y-%m-%d %H:%M:%S %Z')"
    echo "- Command: \`rg -n -- \"--interpreter\" src tests docs README.md ROADMAP.md examples notes .github\`"
    echo
    echo "Reason tags:"
    echo "- \`harness-legacy\`: Existing harness behavior still forces interpreter mode."
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
} >"$OUTPUT_PATH"

echo "generated interpreter dependency map: ${OUTPUT_PATH#$ROOT/}"
