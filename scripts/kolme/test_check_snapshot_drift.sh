#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_snapshot_drift.py"
BASELINE="$ROOT_DIR/fixtures/kolme_compatibility/snapshot_baseline.json"
MATCH="$ROOT_DIR/fixtures/kolme_compatibility/snapshot_candidate_match.json"
DRIFT="$ROOT_DIR/fixtures/kolme_compatibility/snapshot_candidate_drift.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

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

if [ ! -x "$CHECKER" ]; then
  echo "expected Kolme snapshot drift checker to be executable" >&2
  exit 1
fi

if [ ! -f "$BASELINE" ] || [ ! -f "$MATCH" ] || [ ! -f "$DRIFT" ]; then
  echo "expected Kolme snapshot fixtures to exist" >&2
  exit 1
fi

pass_output="$(
  python3 "$CHECKER" \
    --baseline-file "$BASELINE" \
    --candidate-file "$MATCH" \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$pass_output" "status")" "pass" "expected matching Kolme snapshot case to pass"
assert_eq "$(extract_value "$pass_output" "changed_fields_count")" "0" "expected no changed fields for matching Kolme snapshot case"

set +e
fail_output="$(
  python3 "$CHECKER" \
    --baseline-file "$BASELINE" \
    --candidate-file "$DRIFT" \
    --output-json "$TMP_REPORT" 2>&1
)"
fail_code=$?
set -e

if [ "$fail_code" -eq 0 ]; then
  echo "expected drifted Kolme snapshot case to fail" >&2
  exit 1
fi

assert_eq "$(extract_value "$fail_output" "status")" "fail" "expected drifted Kolme snapshot case to report fail status"

if ! printf '%s\n' "$fail_output" | grep -q "changed_fields=docs.docs/src/SUMMARY.md,protocols.sync_manager.version"; then
  echo "expected drift output to list deterministic changed fields" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
payload = json.loads(report_path.read_text())

if payload.get("schema_version") != "kamn.kolme.snapshot-drift-report.v1":
    raise SystemExit("unexpected Kolme drift report schema version")

if payload.get("status") != "fail":
    raise SystemExit("expected persisted Kolme drift report status to be fail")
PY

# Regression: #775
if ! printf '%s\n' "$fail_output" | grep -q "protocols.sync_manager.version"; then
  echo "expected sync manager drift signal to remain fail-closed" >&2
  exit 1
fi

echo "Kolme snapshot drift checker tests passed."
