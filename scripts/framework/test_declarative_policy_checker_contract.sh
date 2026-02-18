#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/framework/declarative_policy_checker.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected declarative policy checker to be executable: $CHECKER" >&2
  exit 1
fi

PASS_POLICY="$TMP_DIR/pass-policy.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$PASS_POLICY" <<'JSON'
{
  "schema_version": "kamn.framework.declarative-policy.v1",
  "policy_name": "example.policy",
  "reason_taxonomy_version": "kamn.example.reason-taxonomy.v1",
  "reason_key_prefix": "example_reason_codes",
  "success_reason_code": "none",
  "checks": [
    {
      "field": "status",
      "op": "equals",
      "expected": "pass",
      "reason_code": "status_not_pass"
    },
    {
      "field": "error_count",
      "op": "lte",
      "expected": 0,
      "reason_code": "error_count_exceeded"
    },
    {
      "field": "markers",
      "op": "contains",
      "expected": "ready",
      "reason_code": "marker_ready_missing"
    }
  ]
}
JSON

PASS_REPORT="$TMP_DIR/pass-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$PASS_REPORT" <<'JSON'
{
  "status": "pass",
  "error_count": 0,
  "markers": ["ready", "stable"]
}
JSON

PASS_OUTPUT_JSON="$TMP_DIR/pass-output.json"
pass_output="$(python3 "$CHECKER" \
  --policy-file "$PASS_POLICY" \
  --report-file "$PASS_REPORT" \
  --expected-final-decision GO \
  --output-json "$PASS_OUTPUT_JSON")"

printf '%s\n' "$pass_output" | grep -q '^status=ok$'
printf '%s\n' "$pass_output" | grep -q '^final_decision=GO$'
printf '%s\n' "$pass_output" | grep -q '^reason_codes=none$'
printf '%s\n' "$pass_output" | grep -q '^reason_key=example_reason_codes:GO:v1$'

python3 - "$PASS_OUTPUT_JSON" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.framework.declarative-policy-report.v1":
    raise SystemExit("unexpected output schema")
if payload.get("status") != "pass":
    raise SystemExit("expected pass status in output JSON")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected success reason code list")
PY

FAIL_REPORT="$TMP_DIR/fail-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$FAIL_REPORT" <<'JSON'
{
  "status": "fail",
  "error_count": 2,
  "markers": ["stable"]
}
JSON

set +e
fail_output="$(python3 "$CHECKER" \
  --policy-file "$PASS_POLICY" \
  --report-file "$FAIL_REPORT" \
  --expected-final-decision GO 2>&1)"
fail_status=$?
set -e

if [ "$fail_status" -eq 0 ]; then
  echo "expected checker to fail when evaluated final decision does not match expected GO" >&2
  exit 1
fi

printf '%s\n' "$fail_output" | grep -q '^status=ok$'
printf '%s\n' "$fail_output" | grep -q '^final_decision=NO-GO$'
printf '%s\n' "$fail_output" | grep -q 'error_count_exceeded'
printf '%s\n' "$fail_output" | grep -q 'marker_ready_missing'
printf '%s\n' "$fail_output" | grep -q 'status_not_pass'

INVALID_POLICY="$TMP_DIR/invalid-policy.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$INVALID_POLICY" <<'JSON'
{
  "schema_version": "kamn.framework.declarative-policy.v0",
  "checks": []
}
JSON

set +e
invalid_output="$(python3 "$CHECKER" \
  --policy-file "$INVALID_POLICY" \
  --report-file "$PASS_REPORT" 2>&1)"
invalid_status=$?
set -e

if [ "$invalid_status" -eq 0 ]; then
  echo "expected checker to fail on invalid policy schema_version" >&2
  exit 1
fi

printf '%s\n' "$invalid_output" | grep -q 'policy schema_version mismatch'

INVALID_TAXONOMY_POLICY="$TMP_DIR/invalid-taxonomy-policy.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$INVALID_TAXONOMY_POLICY" <<'JSON'
{
  "schema_version": "kamn.framework.declarative-policy.v1",
  "policy_name": "invalid.taxonomy.policy",
  "reason_taxonomy_version": "kamn.example.reason-taxonomy",
  "reason_key_prefix": "example_reason_codes",
  "success_reason_code": "none",
  "checks": [
    {
      "field": "status",
      "op": "equals",
      "expected": "pass",
      "reason_code": "status_not_pass"
    }
  ]
}
JSON

set +e
invalid_taxonomy_output="$(python3 "$CHECKER" \
  --policy-file "$INVALID_TAXONOMY_POLICY" \
  --report-file "$PASS_REPORT" 2>&1)"
invalid_taxonomy_status=$?
set -e

if [ "$invalid_taxonomy_status" -eq 0 ]; then
  echo "expected checker to fail on invalid reason_taxonomy_version marker format" >&2
  exit 1
fi

printf '%s\n' "$invalid_taxonomy_output" | grep -q "policy field 'reason_taxonomy_version' must end with .v<integer>"

echo "declarative policy checker contract tests passed."
