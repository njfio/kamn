#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/canary/run_launch_canary_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/launch_canary/critical_path_probe_cases.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

for required_file in "$MATRIX_SCRIPT" "$FIXTURE_FILE"; do
  if [ ! -e "$required_file" ]; then
    echo "missing required launch canary artifact: $required_file" >&2
    exit 1
  fi
done

if [ ! -x "$MATRIX_SCRIPT" ]; then
  echo "launch canary matrix runner must be executable" >&2
  exit 1
fi

python3 "$MATRIX_SCRIPT" --fixture "$FIXTURE_FILE" --output-json "$TMP_REPORT" >/dev/null

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
payload = json.loads(report_path.read_text())

if payload.get("schema_version") != "kamn.launch-canary.probe-report.v1":
    raise SystemExit("unexpected canary report schema")

if payload.get("failed_count") != 0:
    raise SystemExit("expected failed_count=0 for launch canary contract lane")

cases = payload.get("cases", [])
if not any(
    case.get("name") == "missing_probe_evidence" and case.get("derived_decision") == "NO-GO"
    for case in cases
):
    raise SystemExit("expected missing_probe_evidence regression case to derive NO-GO")
PY

echo "launch canary contract lane tests passed."
