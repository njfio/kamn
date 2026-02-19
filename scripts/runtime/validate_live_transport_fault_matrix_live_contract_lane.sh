#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_live_transport_fault_matrix_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_live_transport_fault_matrix_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
NEXT_STEPS_DOC="$ROOT_DIR/docs/plans/2026-02-14-production-service-next-steps.md"
RELEASE_CHECKLIST_DOC="$ROOT_DIR/docs/foundation/release-gonogo-checklist.md"
KOLME_DEVNET_OPS_DOC="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_SMOKE_MAX_SECONDS_BOUNDARY=240
RESILIENCE_GATE_REASON_TAXONOMY_VERSION="kamn.runtime.live-transport-fault-matrix-resilience-gate-reason-taxonomy.v1"
RESILIENCE_GATE_REASON_CODES_CSV="live_transport_fault_matrix_contract_ci_fast_gate_scope_mismatch,live_transport_fault_matrix_contract_ci_smoke_boundary_exceeded,live_transport_fault_matrix_contract_evidence_convergence_mismatch"
POLICY_REASON_CODES_CSV="ci_fast_gate_failed,live_transport_fault_matrix_policy_command_count_invalid,live_transport_fault_matrix_policy_command_count_mismatch,live_transport_fault_matrix_policy_elapsed_seconds_invalid,live_transport_fault_matrix_policy_execution_reason_code_mismatch,live_transport_fault_matrix_policy_final_decision_invalid,live_transport_fault_matrix_policy_final_decision_mismatch,live_transport_fault_matrix_policy_lane_mode_invalid,live_transport_fault_matrix_policy_marker_missing,live_transport_fault_matrix_policy_peer_adapter_multi_process_validation_local_heavy_status_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_projection_budget_exhausted_code_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_projection_timeout_code_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_taxonomy_version_mismatch,live_transport_fault_matrix_policy_peer_integrity_fail_closed_reason_code_mismatch,live_transport_fault_matrix_policy_reason_codes_classification_mismatch,live_transport_fault_matrix_policy_reason_codes_invalid,live_transport_fault_matrix_policy_reason_taxonomy_version_mismatch,live_transport_fault_matrix_policy_runtime_transport_mode_mismatch,live_transport_fault_matrix_policy_schema_mismatch,live_transport_fault_matrix_policy_status_invalid"

output_json=""
policy_output_json=""
max_seconds="${KAMN_LIVE_TRANSPORT_FAULT_MATRIX_CONTRACT_MAX_SECONDS:-240}"
ci_fast_gate="PASS"
mode="dry-run"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --policy-output-json)
      policy_output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    --mode)
      mode="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi
if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  echo "ci-fast-gate must be PASS or FAIL" >&2
  exit 1
fi
if [[ "$mode" != "dry-run" && "$mode" != "run" ]]; then
  echo "mode must be dry-run or run" >&2
  exit 1
fi
if [ "$mode" = "run" ] && [ "$ci_fast_gate" != "FAIL" ]; then
  echo "resilience_gate_reason_taxonomy_version=$RESILIENCE_GATE_REASON_TAXONOMY_VERSION" >&2
  echo "resilience_gate_reason_codes_csv=$RESILIENCE_GATE_REASON_CODES_CSV" >&2
  echo "resilience_gate_reason_codes_value=live_transport_fault_matrix_contract_ci_fast_gate_scope_mismatch" >&2
  echo "live_transport_fault_matrix_contract_ci_fast_gate_scope_mismatch" >&2
  exit 1
fi
if [ "$max_seconds" -gt "$CI_SMOKE_MAX_SECONDS_BOUNDARY" ]; then
  echo "resilience_gate_reason_taxonomy_version=$RESILIENCE_GATE_REASON_TAXONOMY_VERSION" >&2
  echo "resilience_gate_reason_codes_csv=$RESILIENCE_GATE_REASON_CODES_CSV" >&2
  echo "resilience_gate_reason_codes_value=live_transport_fault_matrix_contract_ci_smoke_boundary_exceeded" >&2
  echo "live_transport_fault_matrix_contract_ci_smoke_boundary_exceeded" >&2
  exit 1
fi

for required_exec in "$VALIDATION_SCRIPT" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected required executable script '$required_exec'" >&2
    exit 1
  fi
done
for required_doc in "$STRATEGY_DOC" "$NEXT_STEPS_DOC" "$RELEASE_CHECKLIST_DOC" "$KOLME_DEVNET_OPS_DOC"; do
  if [ ! -f "$required_doc" ]; then
    echo "expected required documentation file '$required_doc'" >&2
    exit 1
  fi
done

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

check_evidence_convergence() {
  local summary_file="$1"
  local policy_file="$2"

  python3 - "$summary_file" "$policy_file" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))

if summary.get("reason_taxonomy_version") != policy.get("reason_taxonomy_version"):
    raise SystemExit("live_transport_fault_matrix_contract_evidence_convergence_mismatch")
if summary.get("reason_codes") != policy.get("reason_codes"):
    raise SystemExit("live_transport_fault_matrix_contract_evidence_convergence_mismatch")
if summary.get("reason_codes_value") != policy.get("reason_codes_value"):
    raise SystemExit("live_transport_fault_matrix_contract_evidence_convergence_mismatch")
if summary.get("peer_adapter_reason_taxonomy_version") != policy.get(
    "peer_adapter_reason_taxonomy_version"
):
    raise SystemExit("live_transport_fault_matrix_contract_evidence_convergence_mismatch")
if summary.get("peer_integrity_fail_closed_reason_code") != policy.get(
    "peer_integrity_fail_closed_reason_code"
):
    raise SystemExit("live_transport_fault_matrix_contract_evidence_convergence_mismatch")
if summary.get("peer_adapter_reason_projection_timeout_code") != policy.get(
    "peer_adapter_reason_projection_timeout_code"
):
    raise SystemExit("live_transport_fault_matrix_contract_evidence_convergence_mismatch")
if summary.get("peer_adapter_reason_projection_budget_exhausted_code") != policy.get(
    "peer_adapter_reason_projection_budget_exhausted_code"
):
    raise SystemExit("live_transport_fault_matrix_contract_evidence_convergence_mismatch")
if summary.get("peer_adapter_multi_process_validation_local_heavy_status") != policy.get(
    "peer_adapter_multi_process_validation_local_heavy_status"
):
    raise SystemExit("live_transport_fault_matrix_contract_evidence_convergence_mismatch")

print("evidence_convergence_status=verified")
print("evidence_convergence_reason_code=none")
PY
}

summary_report="$TMP_DIR/live-transport-fault-matrix-live-summary.json"
policy_report="$TMP_DIR/live-transport-fault-matrix-live-policy.json"
tampered_report="$TMP_DIR/live-transport-fault-matrix-live-summary.tampered.json"
tampered_policy_convergence_report="$TMP_DIR/live-transport-fault-matrix-live-policy.convergence.tampered.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode "$mode" \
    --max-seconds "$max_seconds" \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$summary_report"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected live transport fault matrix validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected live transport fault matrix validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^lane_mode=$mode$"; then
  echo "expected live transport fault matrix validation lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_transport_mode=libp2p_live_fault_matrix$'; then
  echo "expected live transport fault matrix runtime transport mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^partition_rejoin_status=verified$'; then
  echo "expected live transport fault matrix partition/rejoin marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^publish_drop_recovery_status=verified$'; then
  echo "expected live transport fault matrix publish-drop marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^replay_recovery_status=verified$'; then
  echo "expected live transport fault matrix replay marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^peer_churn_recovery_status=verified$'; then
  echo "expected live transport fault matrix peer churn marker" >&2
  exit 1
fi

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected live transport fault matrix policy status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected live transport fault matrix policy final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^live_transport_fault_matrix_policy_status=verified$'; then
  echo "expected live transport fault matrix policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_taxonomy_version=kamn.runtime.live-transport-fault-matrix-reason-taxonomy.v1$'; then
  echo "expected live transport fault matrix policy reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^reason_codes_csv=$POLICY_REASON_CODES_CSV$"; then
  echo "expected live transport fault matrix policy reason codes taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected live transport fault matrix policy normalized reason_codes_value marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^peer_adapter_reason_taxonomy_version=kamn.runtime.peer-adapter-reason-taxonomy.v1$'; then
  echo "expected live transport fault matrix policy peer-adapter reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^peer_integrity_fail_closed_reason_code=p2p_transport_unknown_sender_peer$'; then
  echo "expected live transport fault matrix policy peer-integrity fail-closed reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^peer_adapter_reason_projection_timeout_code=p2p_live_reconnect_retry_dial_timeout$'; then
  echo "expected live transport fault matrix policy retry-timeout reason projection marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^peer_adapter_reason_projection_budget_exhausted_code=p2p_live_reconnect_retry_budget_exhausted$'; then
  echo "expected live transport fault matrix policy retry-budget-exhausted reason projection marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^peer_adapter_multi_process_validation_local_heavy_status=required$'; then
  echo "expected live transport fault matrix policy peer-adapter multi-process local-heavy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^retry_reconnect_marker_contract_status=verified$'; then
  echo "expected live transport fault matrix policy retry/reconnect marker contract status marker" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["partition_rejoin_status"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$TMP_DIR/live-transport-fault-matrix-live-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e
if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered live transport fault matrix report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'live_transport_fault_matrix_policy_marker_missing:partition_rejoin_status'; then
  echo "expected deterministic fail-closed reason for tampered live transport fault matrix report" >&2
  exit 1
fi

evidence_convergence_output="$(
  check_evidence_convergence "$summary_report" "$policy_report"
)"
if ! printf '%s\n' "$evidence_convergence_output" | grep -q '^evidence_convergence_status=verified$'; then
  echo "expected live transport fault matrix evidence convergence marker" >&2
  exit 1
fi

cp "$policy_report" "$tampered_policy_convergence_report"
python3 - "$tampered_policy_convergence_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_codes_value"] = "transport_convergence_unstable"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_convergence_output="$(
  check_evidence_convergence "$summary_report" "$tampered_policy_convergence_report" 2>&1
)"
tampered_convergence_code=$?
set -e
if [ "$tampered_convergence_code" -eq 0 ]; then
  echo "expected tampered live transport fault matrix policy report to fail evidence convergence check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_convergence_output" | grep -q 'live_transport_fault_matrix_contract_evidence_convergence_mismatch'; then
  echo "expected deterministic evidence convergence mismatch marker for live transport fault matrix contract lane" >&2
  exit 1
fi

for required_ref in \
  "validate_live_transport_fault_matrix_live.sh" \
  "check_live_transport_fault_matrix_live_policy.sh" \
  "validate_live_transport_fault_matrix_live_contract_lane.sh" \
  "test_validate_live_transport_fault_matrix_live.sh" \
  "test_check_live_transport_fault_matrix_live_policy.sh" \
  "test_validate_live_transport_fault_matrix_live_contract_lane.sh"; do
  if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done
if ! grep -q "live transport fault matrix run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode." "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include live transport fault matrix run-mode exclusion marker" >&2
  exit 1
fi
if ! grep -q "#3470" "$NEXT_STEPS_DOC"; then
  echo "expected next-steps plan to reference Task #3470" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/validate_live_transport_fault_matrix_live_contract_lane.sh" "$NEXT_STEPS_DOC"; then
  echo "expected next-steps plan to reference live transport fault matrix contract lane script" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/check_live_transport_fault_matrix_live_policy.sh" "$NEXT_STEPS_DOC"; then
  echo "expected next-steps plan to reference live transport fault matrix policy checker script" >&2
  exit 1
fi
if ! grep -q "## Peer Adapter Reason Projection and Multi-Process Validation Hooks (Issue #4320)" "$RELEASE_CHECKLIST_DOC"; then
  echo "expected release checklist to include peer adapter reason projection and multi-process hooks section" >&2
  exit 1
fi
if ! grep -q "peer_adapter_reason_taxonomy_version=kamn.runtime.peer-adapter-reason-taxonomy.v1" "$RELEASE_CHECKLIST_DOC"; then
  echo "expected release checklist to include peer adapter reason taxonomy marker" >&2
  exit 1
fi
if ! grep -q "peer_adapter_reason_projection_timeout_code=p2p_live_reconnect_retry_dial_timeout" "$RELEASE_CHECKLIST_DOC"; then
  echo "expected release checklist to include peer adapter timeout reason projection marker" >&2
  exit 1
fi
if ! grep -q "peer_adapter_reason_projection_budget_exhausted_code=p2p_live_reconnect_retry_budget_exhausted" "$RELEASE_CHECKLIST_DOC"; then
  echo "expected release checklist to include peer adapter retry budget exhausted reason projection marker" >&2
  exit 1
fi
if ! grep -q "peer_adapter_multi_process_validation_local_heavy_status=required" "$RELEASE_CHECKLIST_DOC"; then
  echo "expected release checklist to include peer adapter multi-process local-heavy marker" >&2
  exit 1
fi
if ! grep -q "peer sender-integrity drift fixtures must fail closed with \`p2p_transport_unknown_sender_peer\`" "$KOLME_DEVNET_OPS_DOC"; then
  echo "expected Kolme devnet ops docs to include peer sender-integrity drift fail-closed marker" >&2
  exit 1
fi
if ! grep -q "p2p_live_reconnect_retry_dial_timeout" "$KOLME_DEVNET_OPS_DOC"; then
  echo "expected Kolme devnet ops docs to include retry-timeout reason projection marker" >&2
  exit 1
fi
if ! grep -q "p2p_live_reconnect_retry_budget_exhausted" "$KOLME_DEVNET_OPS_DOC"; then
  echo "expected Kolme devnet ops docs to include retry-budget-exhausted reason projection marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "live transport fault matrix contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/live-transport-fault-matrix-live-contract-lane-report.json"
python3 - "$summary_report" "$policy_report" "$lane_report" "$elapsed_seconds" "$max_seconds" "$mode" <<'PY'
import json
import pathlib
import sys

summary_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
lane_report_file = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])
mode = sys.argv[6]

if summary_report.get("schema_version") != "kamn.runtime.live-transport-fault-matrix-report.v1":
    raise SystemExit("unexpected live transport fault matrix summary schema")
if policy_report.get("schema_version") != "kamn.runtime.live-transport-fault-matrix-policy-report.v1":
    raise SystemExit("unexpected live transport fault matrix policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected live transport fault matrix summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected live transport fault matrix policy final_decision=GO")

lane_report = {
    "schema_version": "kamn.runtime.live-transport-fault-matrix-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "lane_mode": mode,
    "live_transport_fault_matrix_contract_status": "verified",
    "live_transport_fault_matrix_policy_status": policy_report.get(
        "live_transport_fault_matrix_policy_status"
    ),
    "docs_contract_status": "verified",
    "runtime_transport_mode_status": "verified",
    "reason_taxonomy_status": "verified",
    "evidence_convergence_status": "verified",
    "boundary_governance_status": "verified",
    "resilience_gate_reason_taxonomy_version": "kamn.runtime.live-transport-fault-matrix-resilience-gate-reason-taxonomy.v1",
    "resilience_gate_reason_codes_csv": "live_transport_fault_matrix_contract_ci_fast_gate_scope_mismatch,live_transport_fault_matrix_contract_ci_smoke_boundary_exceeded,live_transport_fault_matrix_contract_evidence_convergence_mismatch",
    "resilience_gate_reason_codes_value": "none",
    "policy_reason_taxonomy_version": policy_report.get("reason_taxonomy_version"),
    "policy_reason_codes_csv": policy_report.get("reason_codes_csv"),
    "policy_reason_codes_value": policy_report.get("reason_codes_value"),
    "peer_adapter_reason_taxonomy_version": policy_report.get(
        "peer_adapter_reason_taxonomy_version"
    ),
    "peer_integrity_fail_closed_reason_code": policy_report.get(
        "peer_integrity_fail_closed_reason_code"
    ),
    "peer_adapter_reason_projection_timeout_code": policy_report.get(
        "peer_adapter_reason_projection_timeout_code"
    ),
    "peer_adapter_reason_projection_budget_exhausted_code": policy_report.get(
        "peer_adapter_reason_projection_budget_exhausted_code"
    ),
    "peer_adapter_multi_process_validation_local_heavy_status": policy_report.get(
        "peer_adapter_multi_process_validation_local_heavy_status"
    ),
    "retry_reconnect_marker_contract_status": policy_report.get(
        "retry_reconnect_marker_contract_status"
    ),
    "peer_reason_docs_contract_status": "verified",
    "fail_closed_status": "verified",
    "convergence_fail_closed_reason_code": "live_transport_fault_matrix_contract_evidence_convergence_mismatch",
    "fail_closed_reason_code": "live_transport_fault_matrix_policy_marker_missing:partition_rejoin_status",
    "performance_budget_status": "verified",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
}
lane_report_file.write_text(json.dumps(lane_report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

if [[ -n "$output_json" ]]; then
  cp "$lane_report" "$output_json"
fi
if [[ -n "$policy_output_json" ]]; then
  cp "$policy_report" "$policy_output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "lane_mode=$mode"
echo "live_transport_fault_matrix_contract_status=verified"
echo "live_transport_fault_matrix_policy_status=verified"
echo "docs_contract_status=verified"
echo "runtime_transport_mode_status=verified"
echo "reason_taxonomy_status=verified"
echo "evidence_convergence_status=verified"
echo "boundary_governance_status=verified"
echo "resilience_gate_reason_taxonomy_version=kamn.runtime.live-transport-fault-matrix-resilience-gate-reason-taxonomy.v1"
echo "resilience_gate_reason_codes_csv=live_transport_fault_matrix_contract_ci_fast_gate_scope_mismatch,live_transport_fault_matrix_contract_ci_smoke_boundary_exceeded,live_transport_fault_matrix_contract_evidence_convergence_mismatch"
echo "resilience_gate_reason_codes_value=none"
echo "policy_reason_taxonomy_version=kamn.runtime.live-transport-fault-matrix-reason-taxonomy.v1"
echo "policy_reason_codes_csv=$POLICY_REASON_CODES_CSV"
echo "policy_reason_codes_value=none"
echo "peer_adapter_reason_taxonomy_version=kamn.runtime.peer-adapter-reason-taxonomy.v1"
echo "peer_integrity_fail_closed_reason_code=p2p_transport_unknown_sender_peer"
echo "peer_adapter_reason_projection_timeout_code=p2p_live_reconnect_retry_dial_timeout"
echo "peer_adapter_reason_projection_budget_exhausted_code=p2p_live_reconnect_retry_budget_exhausted"
echo "peer_adapter_multi_process_validation_local_heavy_status=required"
echo "retry_reconnect_marker_contract_status=verified"
echo "peer_reason_docs_contract_status=verified"
echo "fail_closed_status=verified"
echo "convergence_fail_closed_reason_code=live_transport_fault_matrix_contract_evidence_convergence_mismatch"
echo "fail_closed_reason_code=live_transport_fault_matrix_policy_marker_missing:partition_rejoin_status"
echo "performance_budget_status=verified"
