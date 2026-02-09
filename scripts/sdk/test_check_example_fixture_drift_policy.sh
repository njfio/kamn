#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/check_example_fixture_drift_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected sdk example fixture drift policy checker script to be executable" >&2
  exit 1
fi

pass_report="$TMP_DIR/sdk-example-fixture-drift-report.pass.json"
cat <<'JSON' > "$pass_report"
{
  "schema_version": "kamn.sdk.example-fixture-drift-report.v1",
  "status": "pass",
  "reason_codes": ["none"],
  "fixture": "fixtures/sdk_parity/register_validation_cases.json",
  "snapshot": "fixtures/sdk_parity/register_validation_snapshot.json",
  "case_count": 4,
  "drift_case_ids": []
}
JSON

pass_output="$(bash "$SCRIPT" --report-file "$pass_report")"
if ! printf '%s\n' "$pass_output" | grep -q "^status=ok$"; then
  echo "expected sdk example fixture drift policy checker to accept pass report" >&2
  exit 1
fi

if ! printf '%s\n' "$pass_output" | grep -q "^reason_codes=none$"; then
  echo "expected sdk example fixture drift policy checker to preserve reason code marker" >&2
  exit 1
fi

invalid_schema_report="$TMP_DIR/sdk-example-fixture-drift-report.invalid-schema.json"
cat <<'JSON' > "$invalid_schema_report"
{
  "schema_version": "kamn.sdk.example-fixture-drift-report.v0",
  "status": "pass",
  "reason_codes": ["none"]
}
JSON

set +e
invalid_output="$(bash "$SCRIPT" --report-file "$invalid_schema_report" 2>&1)"
invalid_status=$?
set -e

if [ "$invalid_status" -eq 0 ]; then
  echo "expected sdk example fixture drift policy checker to reject invalid schema report" >&2
  exit 1
fi

if ! printf '%s\n' "$invalid_output" | grep -q "reason=invalid-schema-version"; then
  echo "expected explicit invalid schema reason from sdk example fixture drift policy checker" >&2
  exit 1
fi

echo "sdk example fixture drift policy checker tests passed."
