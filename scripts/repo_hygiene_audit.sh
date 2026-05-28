#!/usr/bin/env bash
set -euo pipefail

# Reports root-level files that are not part of the canonical project surface.
# This keeps the repository presentation clean and avoids accidental release noise.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

allowed_root_files=(
  ".editorconfig"
  ".gitignore"
  "CHANGELOG.md"
  "CONTRIBUTING.md"
  "Cargo.lock"
  "Cargo.toml"
  "INSTALLATION.md"
  "LICENSE"
  "README.md"
  "ROADMAP.md"
  "rustfmt.toml"
)

declare -A allowmap
for item in "${allowed_root_files[@]}"; do
  allowmap["$item"]=1
done

unexpected=()
while IFS= read -r tracked; do
  if [[ "$tracked" == */* ]]; then
    continue
  fi
  if [[ -z "${allowmap[$tracked]+x}" ]]; then
    unexpected+=("$tracked")
  fi
done < <(git ls-files)

if [[ ${#unexpected[@]} -eq 0 ]]; then
  echo "Repo hygiene audit: no unexpected tracked root files found."
  exit 0
fi

echo "Repo hygiene audit: unexpected tracked root files detected:"
for item in "${unexpected[@]}"; do
  echo "  - $item"
done

echo
echo "Suggested next step: move artifacts under docs/, examples/, scripts/, notes/, or tmp/ as appropriate."
exit 1
