#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
VALIDATOR="$ROOT_DIR/scripts/cutover/validate_mainnet_cutover_manifest.py"
SCHEMA_FILE="$ROOT_DIR/fixtures/mainnet_cutover/mainnet_cutover_manifest.schema.json"
VALID_FIXTURE="$ROOT_DIR/fixtures/mainnet_cutover/mainnet_cutover_manifest.valid.json"
INVALID_DEP_FIXTURE="$ROOT_DIR/fixtures/mainnet_cutover/mainnet_cutover_manifest.invalid_dependency.json"
INVALID_APPROVAL_FIXTURE="$ROOT_DIR/fixtures/mainnet_cutover/mainnet_cutover_manifest.invalid_approvals.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { sub($1 "=",""); print; exit }'
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

test_harness_require_executable "$VALIDATOR" "expected mainnet cutover validator to be executable"

for fixture in "$SCHEMA_FILE" "$VALID_FIXTURE" "$INVALID_DEP_FIXTURE" "$INVALID_APPROVAL_FIXTURE"; do
  if [ ! -f "$fixture" ]; then
    echo "expected fixture file to exist: $fixture" >&2
    exit 1
  fi
done

valid_output="$(
  python3 "$VALIDATOR" \
    --manifest "$VALID_FIXTURE" \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$valid_output" "status")" "valid" "expected validator success status"
assert_eq "$(extract_value "$valid_output" "validation_decision")" "GO" "expected validator success decision"

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
payload = json.loads(report_path.read_text())
if payload.get("schema_version") != "kamn.mainnet-cutover.validation-report.v1":
    raise SystemExit("unexpected validation report schema_version")
if payload.get("decision") != "GO":
    raise SystemExit("expected GO decision in validation report")
if payload.get("failed_count") != 0:
    raise SystemExit("expected no failed validations for valid manifest")
PY

set +e
invalid_dep_output="$(
  python3 "$VALIDATOR" \
    --manifest "$INVALID_DEP_FIXTURE" \
    --output-json "$TMP_REPORT" 2>&1
)"
invalid_dep_code=$?
set -e
if [ "$invalid_dep_code" -eq 0 ]; then
  echo "expected validator to fail dependency-ordered invalid manifest" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_dep_output" | grep -q "unresolved dependency"; then
  echo "expected unresolved dependency validation error" >&2
  exit 1
fi

set +e
invalid_approval_output="$(
  python3 "$VALIDATOR" \
    --manifest "$INVALID_APPROVAL_FIXTURE" \
    --output-json "$TMP_REPORT" 2>&1
)"
invalid_approval_code=$?
set -e
if [ "$invalid_approval_code" -eq 0 ]; then
  echo "expected validator to fail insufficient approval manifest" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_approval_output" | grep -q "insufficient approvals"; then
  echo "expected insufficient approvals validation error" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
payload = json.loads(report_path.read_text())
if payload.get("decision") != "NO-GO":
    raise SystemExit("expected NO-GO decision in invalid manifest report")
if payload.get("failed_count", 0) < 1:
    raise SystemExit("expected at least one validation failure in invalid report")
PY

echo "mainnet cutover manifest validator tests passed."
