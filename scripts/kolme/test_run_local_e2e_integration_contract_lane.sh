#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_e2e_integration_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_e2e_integration_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local e2e integration contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local e2e integration policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "check_local_e2e_integration_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local e2e integration policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_e2e_integration_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local e2e integration contract lane" >&2
  exit 1
fi

if ! grep -q "check_local_e2e_integration_policy.py" "$README_FILE"; then
  echo "expected README to reference local e2e integration policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_e2e_integration_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local e2e integration contract lane" >&2
  exit 1
fi

if ! grep -q "Regression: #1682" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local e2e integration policy regression marker" >&2
  exit 1
fi

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY_REPORT" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-e2e-integration-summary.v1":
    raise SystemExit("unexpected local e2e contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected local e2e contract-lane summary status ok")
if summary.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry-run reason code in local e2e contract-lane summary")
if policy.get("schema_version") != "kamn.kolme.local-e2e-integration-policy-report.v1":
    raise SystemExit("unexpected local e2e contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected local e2e contract-lane policy final_decision GO")
PY

echo "local e2e integration contract lane tests passed."
