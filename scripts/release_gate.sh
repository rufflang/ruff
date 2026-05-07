#!/usr/bin/env bash
set -euo pipefail

run_cmd() {
  echo ""
  echo "+ $*"
  "$@"
}

run_optional_cmd() {
  local tool="$1"
  shift

  if command -v "${tool}" >/dev/null 2>&1; then
    run_cmd "$@"
  else
    echo ""
    echo "- Skipping optional command: $* (missing ${tool})"
  fi
}

run_cmd cargo test --lib -- --test-threads=1

if [[ "${RUFF_ENABLE_SOCKET_TESTS:-0}" == "1" ]]; then
  run_cmd cargo test --tests
else
  echo ""
  echo "- Skipping socket-bound serve integration tests (set RUFF_ENABLE_SOCKET_TESTS=1 to enable)"
  run_cmd cargo test --tests -- \
    --skip serve_head_returns_headers_without_body \
    --skip serve_range_returns_partial_content_and_content_range_header \
    --skip serve_if_none_match_returns_304_for_matching_etag \
    --skip serve_accept_encoding_prefers_gzip_sibling_asset \
    --skip serve_mime_policy_covers_known_unknown_and_extensionless_assets
fi

run_cmd cargo test --test native_api_security_boundaries
run_cmd cargo test --test package_module_workflow_integration
run_cmd cargo test --test vm_interpreter_parity_surfaces

if [[ "${RUFF_ENABLE_SOCKET_TESTS:-0}" == "1" ]]; then
  run_cmd cargo test --test serve_command_integration
fi

run_optional_cmd cargo-audit cargo audit
run_optional_cmd cargo-deny cargo deny check
