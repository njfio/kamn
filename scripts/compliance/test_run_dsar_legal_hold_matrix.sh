#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/compliance/run_dsar_legal_hold_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/compliance_dsar/legal_hold_precedence_cases.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$MATRIX_SCRIPT" ]; then
  echo "expected DSAR legal-hold matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected DSAR legal-hold fixture file to exist" >&2
  exit 1
fi

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

if payload.get("schema_version") != "kamn.compliance.dsar-legal-hold.replay-matrix.v1":
    raise SystemExit("unexpected DSAR legal-hold matrix schema version")

cases = payload.get("cases", [])
if not cases:
    raise SystemExit("expected at least one DSAR legal-hold matrix case")

if not any(case.get("case_id") == "no_go_erasure_legal_hold_active" for case in cases):
    raise SystemExit("expected no_go_erasure_legal_hold_active case in DSAR matrix report")
PY

echo "dsar legal-hold matrix tests passed."

