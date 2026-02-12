#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kamn_live_runtime_integration_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kamn_live_runtime_integration_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_kamn_live_runtime_integration_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local KAMN live runtime integration contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local KAMN live runtime integration policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local KAMN live runtime integration contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

# Regression: #1967
if ! grep -q "run_local_runtime_commit_live_lane.sh" "$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"; then
  echo "expected local KAMN live runtime integration runner to route runtime step through local runtime commit live lane" >&2
  exit 1
fi

# Regression: #1971
required_runtime_finality_markers=(
  "--runtime-commit-finality-command"
  "--runtime-commit-finality-max-seconds"
  "--runtime-commit-finality-output-file"
  "--runtime-commit-live-policy-report"
  "--runtime-provider-client-contract"
)
for marker in "${required_runtime_finality_markers[@]}"; do
  if ! grep -q -- "$marker" "$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"; then
    echo "expected local KAMN live runtime integration runner to expose finality pass-through marker: $marker" >&2
    exit 1
  fi
done

# Regression: #2101
if ! grep -q "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" "$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"; then
  echo "expected local KAMN live runtime integration runner to compose runtime step through runtime finality evidence contract lane" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local KAMN live runtime integration contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/local_kamn_live_runtime_integration_contract_lane.py",
]:
    raise SystemExit("expected local KAMN live runtime integration manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local KAMN live runtime integration contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_kamn_live_runtime_integration_lane.sh"
  "check_local_kamn_live_runtime_integration_policy.py"
  "run_localhost_signed_integration_contract_lane.sh"
  "run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
  "Regression: #1489"
  "Regression: #1971"
  "Regression: #2101"
  "Regression: #2112"
  "Regression: #2113"
  "Regression: #2114"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected local KAMN live runtime integration contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "check_local_kamn_live_runtime_integration_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local KAMN live runtime integration policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kamn_live_runtime_integration_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local KAMN live runtime integration contract lane" >&2
  exit 1
fi

if ! grep -q -- "--runtime-commit-finality-command" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to document runtime finality pass-through command option" >&2
  exit 1
fi

if ! grep -q -- "--runtime-provider-client-contract" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to document runtime provider contract option" >&2
  exit 1
fi

if ! grep -q "ci_fast_gate_eligible" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to document local-only fast-gate eligibility marker" >&2
  exit 1
fi

required_runbook_doc_markers=(
  "Live Provider Operator Runbook (Issue #2114)"
  "Prerequisites (Local)"
  "Execution Flow"
  "Rollback and Recovery Evidence"
  "Troubleshooting"
)
for marker in "${required_runbook_doc_markers[@]}"; do
  if ! grep -q "$marker" "$DOC_FILE"; then
    echo "expected Kolme devnet ops doc to include runbook marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference runtime finality evidence contract lane composition in local KAMN integration lane" >&2
  exit 1
fi

if ! grep -q "check_local_kamn_live_runtime_integration_policy.py" "$README_FILE"; then
  echo "expected README to reference local KAMN live runtime integration policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kamn_live_runtime_integration_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local KAMN live runtime integration contract lane" >&2
  exit 1
fi

if ! grep -q -- "--runtime-commit-finality-command" "$README_FILE"; then
  echo "expected README to document runtime finality pass-through command option" >&2
  exit 1
fi

if ! grep -q -- "--runtime-provider-client-contract" "$README_FILE"; then
  echo "expected README to document runtime provider contract option" >&2
  exit 1
fi

if ! grep -q "ci_fast_gate_eligible" "$README_FILE"; then
  echo "expected README to document local-only fast-gate eligibility marker" >&2
  exit 1
fi

if ! grep -q "Live Provider Operator Runbook (Issue #2114)" "$README_FILE"; then
  echo "expected README to reference live provider operator runbook section" >&2
  exit 1
fi

if ! grep -q "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference runtime finality evidence contract lane composition in local KAMN integration lane" >&2
  exit 1
fi

# Regression: #1489
if ! grep -q "Regression: #1489" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local KAMN live runtime integration regression marker" >&2
  exit 1
fi

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY_REPORT" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-summary.v1":
    raise SystemExit("unexpected local KAMN live runtime integration contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected local KAMN live runtime integration contract-lane summary status ok")
if summary.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry_run_no_commands_executed reason code in contract-lane summary")
runtime_policy_report = summary.get("runtime_commit_live_policy_report")
if not isinstance(runtime_policy_report, str) or not runtime_policy_report:
    raise SystemExit("expected runtime commit live policy report marker in contract-lane summary")
if runtime_policy_report not in summary.get("artifact_paths", []):
    raise SystemExit("expected runtime policy report artifact in contract-lane summary artifact paths")
checks = summary.get("checks")
if not isinstance(checks, list) or not any(
    check.get("id") == "runtime_commit_policy" and check.get("status") == "planned"
    for check in checks
):
    raise SystemExit("expected runtime commit policy planned check marker in contract-lane summary")
if summary.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected runtime provider client contract marker in contract-lane summary")
if summary.get("ci_fast_gate_eligible") is not False:
    raise SystemExit("expected local-only fast-gate exclusion marker in contract-lane summary")
contracts = summary.get("contracts", {})
if contracts.get("ci_fast_gate_scope") != "local-only":
    raise SystemExit("expected local-only fast-gate scope contract marker in contract-lane summary")
if policy.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-policy-report.v1":
    raise SystemExit("unexpected local KAMN live runtime integration contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected local KAMN live runtime integration contract-lane policy final_decision GO")
PY

TMP_DIRECT_SUMMARY="$(mktemp)"
TMP_DIRECT_RUNTIME_OUTPUT="$(mktemp)"
TMP_DIRECT_RUNTIME_POLICY="$(mktemp)"
TMP_DIRECT_RUNTIME_FINALITY_OUTPUT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT" "$TMP_DIRECT_SUMMARY" "$TMP_DIRECT_RUNTIME_OUTPUT" "$TMP_DIRECT_RUNTIME_POLICY" "$TMP_DIRECT_RUNTIME_FINALITY_OUTPUT"' EXIT

bash "$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh" \
  --mode dry-run \
  --runtime-commit-finality-command "printf 'finality=final\n'" \
  --runtime-commit-finality-max-seconds 12 \
  --runtime-commit-finality-output-file "$TMP_DIRECT_RUNTIME_FINALITY_OUTPUT" \
  --runtime-commit-live-policy-report "$TMP_DIRECT_RUNTIME_POLICY" \
  --runtime-commit-output-file "$TMP_DIRECT_RUNTIME_OUTPUT" \
  --runtime-commit-live-summary "$TMP_DIRECT_SUMMARY.runtime.json" \
  --output-json "$TMP_DIRECT_SUMMARY" >/dev/null

python3 - "$TMP_DIRECT_SUMMARY" "$TMP_DIRECT_RUNTIME_FINALITY_OUTPUT" "$TMP_DIRECT_RUNTIME_POLICY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
runtime_command = summary.get("runtime_commit_command", "")
if "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" not in runtime_command:
    raise SystemExit("expected runtime commit command to compose through runtime finality evidence contract lane")
if "--finality-command" not in runtime_command:
    raise SystemExit("expected runtime commit command to include finality command pass-through")
if "--finality-max-seconds 12" not in runtime_command:
    raise SystemExit("expected runtime commit command to include finality max seconds pass-through")
finality_output_path = pathlib.Path(sys.argv[2]).resolve()
if f"--finality-output-file {finality_output_path}" not in runtime_command:
    raise SystemExit("expected runtime commit command to include finality output pass-through")
policy_output_path = pathlib.Path(sys.argv[3]).resolve()
if f"--policy-output-json {policy_output_path}" not in runtime_command:
    raise SystemExit("expected runtime commit command to include runtime policy report pass-through")
if "--expected-provider-client-contract KolmeRuntimeCommitLiveProvider" not in runtime_command:
    raise SystemExit("expected runtime commit command to include live provider contract pass-through")
if str(finality_output_path) not in summary.get("artifact_paths", []):
    raise SystemExit("expected integration summary artifact paths to include runtime finality output file")
if str(policy_output_path) not in summary.get("artifact_paths", []):
    raise SystemExit("expected integration summary artifact paths to include runtime policy report file")
if summary.get("runtime_commit_live_policy_report") != str(policy_output_path):
    raise SystemExit("expected integration summary to expose runtime commit live policy report path")
PY

echo "local KAMN live runtime integration contract lane tests passed."
