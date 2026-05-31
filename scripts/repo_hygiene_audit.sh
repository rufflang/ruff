#!/usr/bin/env bash
set -euo pipefail

# Cross-check tracked root files against the canonical policy list in
# docs/REPO_HYGIENE_POLICY.md. This is a fast guard intended to fail before
# broader compile/test work when hygiene drifts.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
policy_file="${repo_root}/docs/REPO_HYGIENE_POLICY.md"

if [[ ! -f "${policy_file}" ]]; then
  echo "Repo hygiene audit: missing policy file: ${policy_file}" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

policy_root_files="${tmp_dir}/policy_root_files.txt"
tracked_root_files="${tmp_dir}/tracked_root_files.txt"

awk '
  /Current allowed tracked root files:/ {capture=1; next}
  capture && /^## / {exit}
  capture && /^- `[^`]+`$/ {
    line=$0
    sub(/^- `/, "", line)
    sub(/`$/, "", line)
    print line
  }
' "${policy_file}" | sort > "${policy_root_files}"

if [[ ! -s "${policy_root_files}" ]]; then
  echo "Repo hygiene audit: failed to parse root file allowlist from policy." >&2
  exit 1
fi

git -C "${repo_root}" ls-files | awk -F/ 'NF==1 {print $0}' | sort > "${tracked_root_files}"

if diff -u "${policy_root_files}" "${tracked_root_files}" > "${tmp_dir}/drift.diff"; then
  echo "Repo hygiene audit: tracked root files match docs/REPO_HYGIENE_POLICY.md."
  exit 0
fi

echo "Repo hygiene audit: tracked root surface drifted from docs/REPO_HYGIENE_POLICY.md."
echo
cat "${tmp_dir}/drift.diff"
echo
echo "Suggested next step: update docs/REPO_HYGIENE_POLICY.md and tests/repo_hygiene_contract.rs together if the drift is intentional."
exit 1
