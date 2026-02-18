#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
MATRIX_SCRIPT="$ROOT_DIR/scripts/canary/run_launch_canary_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/launch_canary/critical_path_probe_cases.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

test_harness_require_executable "$MATRIX_SCRIPT" "expected launch canary matrix runner to be executable"

test_harness_require_file "$FIXTURE_FILE" "expected launch canary fixture file to exist"

python3 "$MATRIX_SCRIPT" \
  --fixture "$FIXTURE_FILE" \
  --output-json "$TMP_REPORT" \
  >/dev/null

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
payload = json.loads(report_path.read_text())

if payload.get("schema_version") != "kamn.launch-canary.probe-report.v1":
    raise SystemExit("unexpected launch canary matrix schema version")

cases = payload.get("cases", [])
if not cases:
    raise SystemExit("expected at least one launch canary case")

if not any(case.get("name") == "missing_probe_evidence" for case in cases):
    raise SystemExit("expected missing_probe_evidence case in canary matrix report")
PY

echo "launch canary matrix tests passed."
