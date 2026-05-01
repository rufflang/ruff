#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

if command -v sha256sum >/dev/null 2>&1; then
	checksum_cmd="sha256sum"
	verify_cmd="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
	checksum_cmd="shasum -a 256"
	verify_cmd="shasum -a 256"
else
	echo "[artifact-validate] ERROR: no SHA-256 tool found (sha256sum/shasum)"
	exit 1
fi

echo "[artifact-validate] building release binary"
cargo build --release

install_root="$(mktemp -d)/ruff-install-root"

echo "[artifact-validate] installing from clean root: $install_root"
cargo install --path . --root "$install_root" --force --locked --offline

ruff_bin="$install_root/bin/ruff"
if [[ ! -x "$ruff_bin" ]]; then
	echo "[artifact-validate] ERROR: expected binary not found at $ruff_bin"
	exit 1
fi

echo "[artifact-validate] running installed binary checks"
"$ruff_bin" --version
"$ruff_bin" run examples/hello.ruff

checksum_file="target/release/ruff.sha256"

echo "[artifact-validate] generating checksum: $checksum_file"
(
	cd target/release
	$checksum_cmd ruff > ruff.sha256
)

echo "[artifact-validate] verifying checksum"
(
	cd target/release
	$verify_cmd -c ruff.sha256
)

artifact_root="target/release-artifacts"
mkdir -p "$artifact_root"
cp target/release/ruff "$artifact_root/ruff"
tarball="$artifact_root/ruff-local.tar.gz"

echo "[artifact-validate] packaging local release tarball: $tarball"
tar -czf "$tarball" -C "$artifact_root" ruff

echo "[artifact-validate] generating tarball checksum"
(
	cd "$artifact_root"
	$checksum_cmd ruff-local.tar.gz > ruff-local.tar.gz.sha256
	$verify_cmd -c ruff-local.tar.gz.sha256
)

extract_root="$(mktemp -d)/ruff-artifact-extract"
mkdir -p "$extract_root"
tar -xzf "$tarball" -C "$extract_root"

echo "[artifact-validate] validating repository-independent artifact run"
"$extract_root/ruff" --version

artifact_script="$extract_root/hello.ruff"
cat > "$artifact_script" <<'RUFF_SCRIPT'
print("artifact-ok")
RUFF_SCRIPT

"$extract_root/ruff" run "$artifact_script"

echo "[artifact-validate] OK: install flow and checksum verification passed"
