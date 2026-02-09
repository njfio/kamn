#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/governance/run_stake_slash_risk_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/governance_stake_slash/risk_threshold_cases.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$MATRIX_SCRIPT" ]; then
  echo "expected stake/slash risk matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected stake/slash risk fixture file to exist" >&2
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

if payload.get("schema_version") != "kamn.governance.stake-slash-risk.replay-matrix.v1":
    raise SystemExit("unexpected stake/slash risk matrix schema version")

cases = payload.get("cases", [])
if not cases:
    raise SystemExit("expected at least one stake/slash risk matrix case")

if not any(case.get("case_id") == "no_go_stake_at_risk_breach" for case in cases):
    raise SystemExit("expected no_go_stake_at_risk_breach case in stake/slash risk matrix report")
PY

echo "stake/slash risk matrix tests passed."

