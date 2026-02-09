#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/escrow/run_settlement_reconciliation_race_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/escrow_reconciliation/finality_race_cases.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$MATRIX_SCRIPT" ]; then
  echo "expected settlement reconciliation race matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected settlement reconciliation race fixture file to exist" >&2
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

if payload.get("schema_version") != "kamn.escrow.settlement-race-matrix.v1":
    raise SystemExit("unexpected race matrix schema version")

cases = payload.get("cases", [])
if not cases:
    raise SystemExit("expected at least one race matrix case")

if not any(case.get("name") == "timeout_before_finality_pending" for case in cases):
    raise SystemExit("expected timeout_before_finality_pending case in matrix report")
PY

echo "settlement reconciliation race matrix tests passed."
