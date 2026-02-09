#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/check_example_fixture_drift.py"
FIXTURE="$ROOT_DIR/fixtures/sdk_parity/register_validation_cases.json"
SNAPSHOT="$ROOT_DIR/fixtures/sdk_parity/register_validation_snapshot.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected sdk example fixture drift checker script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/sdk-example-fixture-drift-report.json"
output="$(python3 "$SCRIPT" --fixture "$FIXTURE" --snapshot "$SNAPSHOT" --output-json "$report_file")"

if ! printf '%s\n' "$output" | grep -q "^status=pass$"; then
  echo "expected sdk example fixture drift checker to pass on baseline snapshot" >&2
  exit 1
fi

if ! printf '%s\n' "$output" | grep -q "^reason_codes=none$"; then
  echo "expected sdk example fixture drift checker baseline reason codes to be none" >&2
  exit 1
fi

if [ ! -f "$report_file" ]; then
  echo "expected sdk example fixture drift checker to emit report file" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.sdk.example-fixture-drift-report.v1"' "$report_file"; then
  echo "expected sdk example fixture drift checker report schema marker" >&2
  exit 1
fi

tampered_snapshot="$TMP_DIR/register_validation_snapshot.tampered.json"
cp "$SNAPSHOT" "$tampered_snapshot"
python3 - "$tampered_snapshot" <<'PY'
import json
import pathlib
import sys

snapshot_file = pathlib.Path(sys.argv[1])
payload = json.loads(snapshot_file.read_text(encoding="utf-8"))
payload["cases"][0]["rust"]["error_code"] = "tampered_error_code"
snapshot_file.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  python3 "$SCRIPT" \
    --fixture "$FIXTURE" \
    --snapshot "$tampered_snapshot" \
    --output-json "$TMP_DIR/tampered-report.json" \
    2>&1
)"
tampered_status=$?
set -e

if [ "$tampered_status" -eq 0 ]; then
  echo "expected sdk example fixture drift checker to fail on tampered snapshot" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "reason_codes=sdk_example_fixture_snapshot_drift"; then
  echo "expected explicit snapshot drift reason code marker from sdk example fixture drift checker" >&2
  exit 1
fi

echo "sdk example fixture drift checker script tests passed."
