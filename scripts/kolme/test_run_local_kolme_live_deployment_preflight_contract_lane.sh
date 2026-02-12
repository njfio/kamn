#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_deployment_preflight_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py"
LANE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kolme_live_deployment_preflight_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_kolme_live_deployment_preflight_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
TMP_NEGATIVE_REPORT="$(mktemp)"
TMP_NEGATIVE_POLICY="$(mktemp)"
TMP_NEGATIVE_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT" "$TMP_NEGATIVE_REPORT" "$TMP_NEGATIVE_POLICY" "$TMP_NEGATIVE_ERR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local Kolme live deployment preflight contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local Kolme live deployment preflight policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$LANE_RUNNER" ]; then
  echo "expected local Kolme live deployment preflight lane runner to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local Kolme live deployment preflight contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local Kolme live deployment preflight contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/local_kolme_live_deployment_preflight_contract_lane.py",
]:
    raise SystemExit("expected local Kolme live deployment preflight manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local Kolme live deployment preflight contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_kolme_live_deployment_preflight_lane.sh"
  "check_local_kolme_live_deployment_preflight_policy.py"
  "run_local_kolme_live_deployment_preflight_contract_lane.sh"
  "runtime_mode_mismatch"
  "checkpoint_failed_signer_secret_contract"
  "checkpoint_failed_signer_quorum_contract"
  "checkpoint_failed_custody_evidence_contract"
  "signer_quorum_shortfall"
  "custody_evidence_missing"
  "Regression: #2226"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q -- "$marker" "$CONTRACT_IMPL"; then
    echo "expected local Kolme live deployment preflight contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

for docs_file in "$DOC_FILE" "$CI_DOC_FILE" "$README_FILE"; do
  if ! grep -q "run_local_kolme_live_deployment_preflight_lane.sh" "$docs_file"; then
    echo "expected docs parity to include deployment preflight lane marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "check_local_kolme_live_deployment_preflight_policy.py" "$docs_file"; then
    echo "expected docs parity to include deployment preflight policy checker marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "run_local_kolme_live_deployment_preflight_contract_lane.sh" "$docs_file"; then
    echo "expected docs parity to include deployment preflight contract lane marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2226" "$docs_file"; then
    echo "expected docs parity to include deployment preflight contract-lane regression marker in $docs_file" >&2
    exit 1
  fi
done

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY_REPORT" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-live-deployment-preflight-summary.v1":
    raise SystemExit("unexpected deployment preflight contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected deployment preflight contract-lane summary status ok")
if summary.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected deployment preflight dry-run reason code in contract-lane summary")
if summary.get("ci_fast_gate_eligible") is not True:
    raise SystemExit("expected deployment preflight contract-lane summary ci_fast_gate_eligible=true")
contracts = summary.get("contracts", {})
if contracts.get("ci_fast_gate_scope") != "ci-fast-gate":
    raise SystemExit("expected deployment preflight contracts ci_fast_gate_scope=ci-fast-gate")
if policy.get("schema_version") != "kamn.kolme.local-live-deployment-preflight-policy-report.v1":
    raise SystemExit("unexpected deployment preflight contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected deployment preflight contract-lane policy final_decision GO")
PY

python3 - "$TMP_REPORT" "$TMP_NEGATIVE_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
summary["runtime_mode"] = "kolme-standard"
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_NEGATIVE_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
negative_exit_code=$?
set -e

if [ "$negative_exit_code" -eq 0 ]; then
  echo "expected deployment preflight contract-lane negative proof to fail closed" >&2
  exit 1
fi

if ! grep -q "runtime_mode_mismatch" "$TMP_NEGATIVE_ERR"; then
  echo "expected runtime mode mismatch reason in deployment preflight contract-lane negative proof output" >&2
  exit 1
fi

echo "local Kolme live deployment preflight contract lane tests passed."
