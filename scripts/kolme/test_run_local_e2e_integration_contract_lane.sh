#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_e2e_integration_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_e2e_integration_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_e2e_integration_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_e2e_integration_contract_lane.py"
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

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local e2e integration contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local e2e integration contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/local_e2e_integration_contract_lane.py",
]:
    raise SystemExit("expected local e2e manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local e2e integration contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_e2e_integration_lane.sh"
  "check_local_e2e_integration_policy.py"
  "Regression: #1682"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected local e2e integration contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

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
