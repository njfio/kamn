#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/deploy/preflight_topology.sh"

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

valid_output="$(
  bash "$SCRIPT" \
    --processors 3 \
    --listeners 3 \
    --approvers 3 \
    --required-approvals 2
)"
assert_eq "$(extract_value "$valid_output" "status")" "ok" "valid topology must pass"

set +e
invalid_processor_output="$(
  bash "$SCRIPT" \
    --processors 0 \
    --listeners 3 \
    --approvers 3 \
    --required-approvals 2 2>&1
)"
invalid_processor_code=$?
set -e
if [ "$invalid_processor_code" -eq 0 ]; then
  echo "invalid processor topology should fail" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_processor_output" | grep -q "processors must be >= 1"; then
  echo "missing processor validation error output" >&2
  exit 1
fi

set +e
invalid_quorum_output="$(
  bash "$SCRIPT" \
    --processors 3 \
    --listeners 3 \
    --approvers 2 \
    --required-approvals 3 2>&1
)"
invalid_quorum_code=$?
set -e
if [ "$invalid_quorum_code" -eq 0 ]; then
  echo "invalid quorum topology should fail" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_quorum_output" | grep -q "required-approvals must be between 1 and approvers"; then
  echo "missing quorum validation error output" >&2
  exit 1
fi

echo "deployment preflight topology tests passed."
