#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/deploy/preflight_topology.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

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

valid_bundle="$TMP_DIR/valid.bundle.env"
cat >"$valid_bundle" <<'EOF'
PROCESSORS=3
LISTENERS=3
APPROVERS=3
REQUIRED_APPROVALS=2
EOF

bundle_output="$(bash "$SCRIPT" --bundle-file "$valid_bundle")"
assert_eq "$(extract_value "$bundle_output" "status")" "ok" "valid bundle topology must pass"

missing_required_bundle="$TMP_DIR/missing_required.bundle.env"
cat >"$missing_required_bundle" <<'EOF'
PROCESSORS=3
LISTENERS=3
APPROVERS=3
EOF

set +e
missing_required_output="$(bash "$SCRIPT" --bundle-file "$missing_required_bundle" 2>&1)"
missing_required_code=$?
set -e
if [ "$missing_required_code" -eq 0 ]; then
  echo "bundle with missing required fields should fail" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_required_output" | grep -q "missing bundle field: REQUIRED_APPROVALS"; then
  echo "missing required field error output for bundle topology" >&2
  exit 1
fi

non_integer_bundle="$TMP_DIR/non_integer.bundle.env"
cat >"$non_integer_bundle" <<'EOF'
PROCESSORS=x
LISTENERS=3
APPROVERS=3
REQUIRED_APPROVALS=2
EOF

set +e
non_integer_output="$(bash "$SCRIPT" --bundle-file "$non_integer_bundle" 2>&1)"
non_integer_code=$?
set -e
if [ "$non_integer_code" -eq 0 ]; then
  echo "bundle with non-integer processor count should fail" >&2
  exit 1
fi
if ! printf '%s\n' "$non_integer_output" | grep -q "processors must be an integer"; then
  echo "missing non-integer processor validation error output for bundle topology" >&2
  exit 1
fi

echo "deployment preflight topology tests passed."
