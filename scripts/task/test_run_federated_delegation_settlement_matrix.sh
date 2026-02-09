#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/task/run_federated_delegation_settlement_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/federated_task_delegation/partition_replay_cases.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$MATRIX_SCRIPT" ]; then
  echo "expected federated delegation settlement matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected federated delegation settlement fixture file to exist" >&2
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

if payload.get("schema_version") != "kamn.task.federated-delegation-settlement.partition-replay-matrix.v1":
    raise SystemExit("unexpected federated delegation settlement matrix schema version")

cases = payload.get("cases", [])
if not cases:
    raise SystemExit("expected at least one federated delegation settlement matrix case")

if not any(case.get("case_id") == "no_go_settlement_reference_drift" for case in cases):
    raise SystemExit("expected no_go_settlement_reference_drift case in matrix report")
PY

echo "federated delegation settlement matrix tests passed."
