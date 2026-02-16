#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_process_lifecycle_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kolme_fork_process_lifecycle_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_kolme_fork_process_lifecycle_contract_lane.py"
RUN_WRAPPER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh"
RUN_WRAPPER_IMPL="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_process_lifecycle_lane_impl.sh"
RUN_MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kolme_fork_process_lifecycle_lane.json"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
RUNBOOK_FILE="$ROOT_DIR/docs/ops/runbook_demo.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork process lifecycle contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork process lifecycle policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$RUN_WRAPPER_IMPL" ]; then
  echo "expected local fork process lifecycle implementation runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected local run lane dispatcher to be executable" >&2
  exit 1
fi

if [ ! -L "$RUN_WRAPPER" ]; then
  echo "expected local fork process lifecycle runner to be a symlink to shared runtime lane dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUN_WRAPPER")" != "run_lane_dispatch.sh" ]; then
  echo "expected local fork process lifecycle runner symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

if [ ! -f "$RUN_MANIFEST" ]; then
  echo "expected local fork process lifecycle run manifest to exist" >&2
  exit 1
fi

python3 - "$RUN_MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("expected local fork process lifecycle run manifest schema")
if payload.get("lane_id") != "kolme.local_kolme_fork_process_lifecycle.run":
    raise SystemExit("expected local fork process lifecycle run manifest lane_id")
run_command = payload.get("phases", {}).get("run")
if run_command != [
    "bash",
    "scripts/kolme/run_local_kolme_fork_process_lifecycle_lane_impl.sh",
]:
    raise SystemExit("expected local fork process lifecycle run manifest command")
PY

resolved_run_manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUN_WRAPPER")" --resolve-manifest-path)"
if [ "$resolved_run_manifest_path" != "$RUN_MANIFEST" ]; then
  echo "expected local fork process lifecycle wrapper to resolve deterministic run manifest" >&2
  exit 1
fi

if bash "$DISPATCHER" --lane-wrapper run_missing_local_kolme_fork_process_lifecycle_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected local run lane dispatcher to fail closed for unknown local fork process lifecycle wrapper" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local fork process lifecycle contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

# Regression: #1973
required_integration_finality_markers=(
  "--integration-runtime-commit-finality-command"
  "--integration-runtime-commit-finality-max-seconds"
  "--integration-runtime-commit-finality-output-file"
  "--integration-runtime-commit-live-policy-report"
  "--rollback-evidence-file"
  "--recovery-evidence-file"
)
for marker in "${required_integration_finality_markers[@]}"; do
  if ! grep -q -- "$marker" "$RUN_WRAPPER_IMPL"; then
    echo "expected local fork process lifecycle runner to expose integration pass-through marker: $marker" >&2
    exit 1
  fi
done

if [ ! -f "$MANIFEST" ]; then
  echo "expected local fork process lifecycle contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/local_kolme_fork_process_lifecycle_contract_lane.py",
]:
    raise SystemExit("expected local fork process lifecycle manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local fork process lifecycle contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_kolme_fork_process_lifecycle_lane.sh"
  "check_local_kolme_fork_process_lifecycle_policy.py"
  "run_local_kamn_live_runtime_integration_lane.sh"
  "Regression: #1494"
  "Regression: #1973"
  "Regression: #2104"
  "Regression: #2107"
  "Regression: #4495"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected local fork process lifecycle contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "check_local_kolme_fork_process_lifecycle_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork process lifecycle policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_process_lifecycle_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork process lifecycle runner" >&2
  exit 1
fi

if ! grep -q "run_lane_dispatch.sh --lane-wrapper run_local_kolme_fork_process_lifecycle_lane.sh --resolve-manifest-path" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork process lifecycle run-wrapper dispatcher mapping" >&2
  exit 1
fi

if ! grep -q "kolme_local_kolme_fork_process_lifecycle_lane.json" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork process lifecycle run manifest" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_process_lifecycle_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork process lifecycle contract lane" >&2
  exit 1
fi

if ! grep -q -- "--integration-runtime-commit-finality-command" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to document process lifecycle integration finality pass-through command option" >&2
  exit 1
fi

if ! grep -q -- "--integration-runtime-commit-live-policy-report" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to document process lifecycle runtime policy pass-through option" >&2
  exit 1
fi

if ! grep -q -- "--rollback-evidence-file" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to document process lifecycle rollback evidence option" >&2
  exit 1
fi

if ! grep -q -- "--recovery-evidence-file" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to document process lifecycle recovery evidence option" >&2
  exit 1
fi

if ! grep -q "docs/ops/runbook_demo.md" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference demo startup drift runbook" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_process_lifecycle_policy.py" "$README_FILE"; then
  echo "expected README to reference local fork process lifecycle policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_process_lifecycle_lane.sh" "$README_FILE"; then
  echo "expected README to reference local fork process lifecycle runner" >&2
  exit 1
fi

if ! grep -q "run_lane_dispatch.sh --lane-wrapper run_local_kolme_fork_process_lifecycle_lane.sh --resolve-manifest-path" "$README_FILE"; then
  echo "expected README to reference local fork process lifecycle run-wrapper dispatcher mapping" >&2
  exit 1
fi

if ! grep -q "kolme_local_kolme_fork_process_lifecycle_lane.json" "$README_FILE"; then
  echo "expected README to reference local fork process lifecycle run manifest" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_process_lifecycle_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local fork process lifecycle contract lane" >&2
  exit 1
fi

if ! grep -q -- "--integration-runtime-commit-finality-command" "$README_FILE"; then
  echo "expected README to document process lifecycle integration finality pass-through command option" >&2
  exit 1
fi

if ! grep -q -- "--integration-runtime-commit-live-policy-report" "$README_FILE"; then
  echo "expected README to document process lifecycle runtime policy pass-through option" >&2
  exit 1
fi

if ! grep -q -- "--rollback-evidence-file" "$README_FILE"; then
  echo "expected README to document process lifecycle rollback evidence option" >&2
  exit 1
fi

if ! grep -q -- "--recovery-evidence-file" "$README_FILE"; then
  echo "expected README to document process lifecycle recovery evidence option" >&2
  exit 1
fi

if ! grep -q "docs/ops/runbook_demo.md" "$README_FILE"; then
  echo "expected README to reference demo startup drift runbook" >&2
  exit 1
fi

if [ ! -f "$RUNBOOK_FILE" ]; then
  echo "expected demo startup drift runbook to exist" >&2
  exit 1
fi

if ! grep -q "startup_dependency_drift:readiness_without_process_start" "$RUNBOOK_FILE"; then
  echo "expected runbook to document startup dependency drift reason marker" >&2
  exit 1
fi

if ! grep -q "check_id_duplicate:<check-id>" "$RUNBOOK_FILE"; then
  echo "expected runbook to document duplicate orchestration check-id reason marker" >&2
  exit 1
fi

if ! grep -q "Regression: #2104" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include process lifecycle runtime policy pass-through regression marker" >&2
  exit 1
fi

if ! grep -q "Regression: #2107" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include process lifecycle rollback/recovery linkage regression marker" >&2
  exit 1
fi

# Regression: #1494
if ! grep -q "Regression: #1494" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local fork process lifecycle regression marker" >&2
  exit 1
fi

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY_REPORT" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-fork-process-lifecycle-summary.v1":
    raise SystemExit("unexpected local fork process lifecycle contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected local fork process lifecycle contract-lane summary status ok")
if summary.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry_run_no_commands_executed reason code in contract-lane summary")
if policy.get("schema_version") != "kamn.kolme.local-fork-process-lifecycle-policy-report.v1":
    raise SystemExit("unexpected local fork process lifecycle contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected local fork process lifecycle contract-lane policy final_decision GO")
PY

TMP_DIRECT_SUMMARY="$(mktemp)"
TMP_DIRECT_PROCESS_OUTPUT="$(mktemp)"
TMP_DIRECT_INTEGRATION_REPORT="$(mktemp)"
TMP_DIRECT_FINALITY_OUTPUT="$(mktemp)"
TMP_DIRECT_RUNTIME_POLICY_REPORT="$(mktemp)"
TMP_DIRECT_ROLLBACK_EVIDENCE_FILE="$(mktemp)"
TMP_DIRECT_RECOVERY_EVIDENCE_FILE="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT" "$TMP_DIRECT_SUMMARY" "$TMP_DIRECT_PROCESS_OUTPUT" "$TMP_DIRECT_INTEGRATION_REPORT" "$TMP_DIRECT_FINALITY_OUTPUT" "$TMP_DIRECT_RUNTIME_POLICY_REPORT" "$TMP_DIRECT_ROLLBACK_EVIDENCE_FILE" "$TMP_DIRECT_RECOVERY_EVIDENCE_FILE"' EXIT

bash "$ROOT_DIR/scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh" \
  --mode dry-run \
  --integration-runtime-commit-finality-command "printf 'finality=final\n'" \
  --integration-runtime-commit-finality-max-seconds 11 \
  --integration-runtime-commit-finality-output-file "$TMP_DIRECT_FINALITY_OUTPUT" \
  --integration-runtime-commit-live-policy-report "$TMP_DIRECT_RUNTIME_POLICY_REPORT" \
  --rollback-evidence-file "$TMP_DIRECT_ROLLBACK_EVIDENCE_FILE" \
  --recovery-evidence-file "$TMP_DIRECT_RECOVERY_EVIDENCE_FILE" \
  --process-output-file "$TMP_DIRECT_PROCESS_OUTPUT" \
  --integration-report "$TMP_DIRECT_INTEGRATION_REPORT" \
  --output-json "$TMP_DIRECT_SUMMARY" >/dev/null

python3 - "$TMP_DIRECT_SUMMARY" "$TMP_DIRECT_FINALITY_OUTPUT" "$TMP_DIRECT_RUNTIME_POLICY_REPORT" "$TMP_DIRECT_ROLLBACK_EVIDENCE_FILE" "$TMP_DIRECT_RECOVERY_EVIDENCE_FILE" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
checks = summary.get("checks", [])
integration_commands = [
    check.get("command", "")
    for check in checks
    if isinstance(check, dict) and check.get("id") == "kamn_live_integration"
]
if len(integration_commands) != 1:
    raise SystemExit("expected exactly one kamn_live_integration check command")
integration_command = integration_commands[0]
if "--runtime-profile real-node" not in integration_command:
    raise SystemExit("expected nested integration command to include explicit real-node runtime profile marker")
if "--runtime-provider-client-contract KolmeRuntimeCommitLiveProvider" not in integration_command:
    raise SystemExit(
        "expected nested integration command to include explicit live runtime provider contract marker"
    )
if "--runtime-commit-finality-command" not in integration_command:
    raise SystemExit("expected nested integration command to include runtime finality command pass-through")
if "--runtime-commit-finality-max-seconds 11" not in integration_command:
    raise SystemExit("expected nested integration command to include runtime finality max seconds pass-through")
finality_output_path = pathlib.Path(sys.argv[2]).resolve()
if f"--runtime-commit-finality-output-file {finality_output_path}" not in integration_command:
    raise SystemExit("expected nested integration command to include runtime finality output pass-through")
if str(finality_output_path) not in summary.get("artifact_paths", []):
    raise SystemExit("expected process lifecycle summary artifact paths to include integration finality output file")
runtime_policy_report_path = pathlib.Path(sys.argv[3]).resolve()
if f"--runtime-commit-live-policy-report {runtime_policy_report_path}" not in integration_command:
    raise SystemExit("expected nested integration command to include runtime policy report pass-through")
if summary.get("integration_runtime_commit_live_policy_report") != str(runtime_policy_report_path):
    raise SystemExit("expected summary to expose runtime policy report path")
if str(runtime_policy_report_path) not in summary.get("artifact_paths", []):
    raise SystemExit("expected process lifecycle summary artifact paths to include runtime policy report path")
contracts = summary.get("contracts")
if not isinstance(contracts, dict):
    raise SystemExit("expected process lifecycle summary contracts object")
if contracts.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
    raise SystemExit(
        "expected process lifecycle summary contracts to include live runtime provider contract marker"
    )
rollback_evidence_path = pathlib.Path(sys.argv[4]).resolve()
recovery_evidence_path = pathlib.Path(sys.argv[5]).resolve()
if summary.get("rollback_evidence_file") != str(rollback_evidence_path):
    raise SystemExit("expected summary to expose rollback evidence file path")
if summary.get("recovery_evidence_file") != str(recovery_evidence_path):
    raise SystemExit("expected summary to expose recovery evidence file path")
if summary.get("rollback_evidence_status") != "planned":
    raise SystemExit("expected dry-run rollback evidence status to be planned")
if summary.get("recovery_evidence_status") != "planned":
    raise SystemExit("expected dry-run recovery evidence status to be planned")
if str(rollback_evidence_path) not in summary.get("artifact_paths", []):
    raise SystemExit("expected process lifecycle summary artifact paths to include rollback evidence file path")
if str(recovery_evidence_path) not in summary.get("artifact_paths", []):
    raise SystemExit("expected process lifecycle summary artifact paths to include recovery evidence file path")
PY

# Regression: #4495
TMP_STARTUP_DRIFT_SUMMARY="$(mktemp)"
TMP_ORCHESTRATION_DRIFT_SUMMARY="$(mktemp)"
TMP_POLICY_OUTPUT="$(mktemp)"
TMP_POLICY_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT" "$TMP_DIRECT_SUMMARY" "$TMP_DIRECT_PROCESS_OUTPUT" "$TMP_DIRECT_INTEGRATION_REPORT" "$TMP_DIRECT_FINALITY_OUTPUT" "$TMP_DIRECT_RUNTIME_POLICY_REPORT" "$TMP_DIRECT_ROLLBACK_EVIDENCE_FILE" "$TMP_DIRECT_RECOVERY_EVIDENCE_FILE" "$TMP_STARTUP_DRIFT_SUMMARY" "$TMP_ORCHESTRATION_DRIFT_SUMMARY" "$TMP_POLICY_OUTPUT" "$TMP_POLICY_ERR"' EXIT

python3 - "$TMP_REPORT" "$TMP_STARTUP_DRIFT_SUMMARY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
summary["mode"] = "run"
summary["status"] = "ok"
summary["reason_code"] = "process_lifecycle_integration_passed"
summary["budget_status"] = "within_budget"
summary["start_reason_code"] = "process_start_failed"
summary["readiness_reason_code"] = "readiness_checks_passed"
summary["integration_reason_code"] = "kamn_live_integration_passed"
summary["teardown_reason_code"] = "process_teardown_passed"
summary["rollback_evidence_status"] = "not_required"
summary["rollback_evidence_reason_code"] = "no_failure_detected"
summary["recovery_evidence_status"] = "validated"
summary["recovery_evidence_reason_code"] = "teardown_path_validated"
for check in summary.get("checks", []):
    if not isinstance(check, dict):
        continue
    check_id = check.get("id")
    if check_id == "process_start":
        check["status"] = "fail"
        check["reason_code"] = "process_start_failed"
    elif check_id == "readiness_probe":
        check["status"] = "pass"
        check["reason_code"] = "readiness_checks_passed"
    elif check_id == "kamn_live_integration":
        check["status"] = "pass"
        check["reason_code"] = "kamn_live_integration_passed"
    elif check_id == "process_teardown":
        check["status"] = "pass"
        check["reason_code"] = "process_teardown_passed"
    elif check_id == "rollback_evidence":
        check["status"] = "pass"
        check["reason_code"] = "no_failure_detected"
    elif check_id == "recovery_evidence":
        check["status"] = "pass"
        check["reason_code"] = "teardown_path_validated"
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_STARTUP_DRIFT_SUMMARY" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_OUTPUT" >"$TMP_POLICY_ERR" 2>&1
startup_drift_code=$?
set -e
if [ "$startup_drift_code" -eq 0 ]; then
  echo "expected checker to fail when startup dependency drift is accepted in run mode" >&2
  exit 1
fi
if ! grep -q "startup_dependency_drift:readiness_without_process_start" "$TMP_POLICY_ERR"; then
  echo "expected startup dependency drift reason marker for readiness without process-start pass" >&2
  exit 1
fi

python3 - "$TMP_REPORT" "$TMP_ORCHESTRATION_DRIFT_SUMMARY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
checks = summary.get("checks", [])
checks.append(
    {
        "id": "readiness_probe",
        "command": "curl --silent --show-error --fail http://127.0.0.1:3000/healthz",
        "status": "fail",
        "reason_code": "process_readiness_failed",
    }
)
summary["checks"] = checks
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_ORCHESTRATION_DRIFT_SUMMARY" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_OUTPUT" >"$TMP_POLICY_ERR" 2>&1
orchestration_drift_code=$?
set -e
if [ "$orchestration_drift_code" -eq 0 ]; then
  echo "expected checker to fail when duplicate orchestration check ids are accepted" >&2
  exit 1
fi
if ! grep -q "check_id_duplicate:readiness_probe" "$TMP_POLICY_ERR"; then
  echo "expected duplicate orchestration check-id reason marker" >&2
  exit 1
fi

echo "local fork process lifecycle contract lane tests passed."
