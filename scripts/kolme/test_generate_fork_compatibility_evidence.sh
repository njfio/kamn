#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/kolme/generate_fork_compatibility_evidence.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/kolme_compatibility/fork_compatibility_cases.json"
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

if [ ! -x "$GENERATOR" ]; then
  echo "expected Kolme fork compatibility evidence generator to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected fork compatibility fixture file to exist" >&2
  exit 1
fi

go_output="$(
  python3 "$GENERATOR" \
    --upstream-release-tag "v0.15.2" \
    --fork-release-tag "v0.15.2" \
    --fork-repo "njfio/kolme_fork" \
    --fork-ref "refs/heads/main" \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT"
)"
assert_eq "$(extract_value "$go_output" "status")" "ok" "expected matching upstream/fork tags to pass"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO decision for matching upstream/fork tags"

set +e
drift_output="$(
  python3 "$GENERATOR" \
    --upstream-release-tag "v0.15.2" \
    --fork-release-tag "v0.14.9" \
    --fork-repo "njfio/kolme_fork" \
    --fork-ref "refs/heads/main" \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT" 2>&1
)"
drift_code=$?
set -e

if [ "$drift_code" -eq 0 ]; then
  echo "expected drifted fork tuple to fail closed" >&2
  exit 1
fi

assert_eq "$(extract_value "$drift_output" "status")" "fail" "expected drifted fork tuple to report fail status"
assert_eq "$(extract_value "$drift_output" "final_decision")" "NO-GO" "expected NO-GO for drifted fork tuple"
if ! printf '%s\n' "$drift_output" | grep -q "fork_release_tag_mismatch"; then
  echo "expected fork drift reason code in no-go output" >&2
  exit 1
fi

python3 - "$TMP_REPORT" "$FIXTURE_FILE" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
fixture = json.loads(pathlib.Path(sys.argv[2]).read_text())

if report.get("schema_version") != "kamn.kolme.fork-compatibility-report.v1":
    raise SystemExit("unexpected fork compatibility report schema")
if report.get("final_decision") != "NO-GO":
    raise SystemExit("expected persisted no-go decision in report")
if fixture.get("schema_version") != "kamn.kolme.fork-compatibility-cases.v1":
    raise SystemExit("unexpected fork compatibility fixture schema")
if len(fixture.get("cases", [])) < 2:
    raise SystemExit("expected fork compatibility fixture set to include pass/fail cases")
PY

# Regression: #1401
if ! printf '%s\n' "$drift_output" | grep -q "fork_release_tag_mismatch"; then
  echo "expected fork release drift signature to remain blocked" >&2
  exit 1
fi

echo "Kolme fork compatibility evidence generator tests passed."
