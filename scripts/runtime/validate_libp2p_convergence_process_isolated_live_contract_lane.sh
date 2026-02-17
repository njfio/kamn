#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_convergence_process_isolated_live_policy.sh"
EVIDENCE_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_convergence_process_isolated_live_evidence_convergence.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
BLOCK_PIPELINE_DOC="$ROOT_DIR/docs/architecture/block-pipeline.md"
RUNBOOK_DOC="${KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_RUNBOOK_DOC_OVERRIDE:-$ROOT_DIR/docs/deploy/kolme_devnet_ops.md}"

EVIDENCE_REPORT_SCHEMA="kamn.runtime.libp2p-convergence-process-isolated-live-convergence-report.v1"
EVIDENCE_REASON_TAXONOMY_VERSION="kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1"
EVIDENCE_REASON_CODES_CSV="libp2p_finality_evidence_link_missing,libp2p_finality_evidence_payload_tamper_detected,libp2p_finality_promotion_decision_reason_mapping_mismatch"
PROMOTION_DECISION_REASON_TAXONOMY_VERSION="kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1"
PROMOTION_DECISION_REASON_CODES_CSV="libp2p_process_isolated_convergence_policy_required_field_missing,libp2p_process_isolated_convergence_policy_marker_missing,libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,libp2p_process_isolated_convergence_policy_expected_decision_mismatch,libp2p_process_isolated_convergence_policy_violation"
EVIDENCE_TAMPER_REASON_CODE="libp2p_finality_promotion_decision_reason_mapping_mismatch"

output_json=""
policy_output_json=""
convergence_output_json=""
summary_output_json=""
max_seconds="${KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_CONTRACT_MAX_SECONDS:-240}"
ci_fast_gate="PASS"
mode="dry-run"
lane_profile="smoke"

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
    --convergence-output-json)
      convergence_output_json="${2:-}"
      shift 2
      ;;
    --summary-output-json)
      summary_output_json="${2:-}"
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
    --lane-profile)
      lane_profile="${2:-}"
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
if [[ "$lane_profile" != "smoke" && "$lane_profile" != "deep" ]]; then
  echo "lane-profile must be smoke or deep" >&2
  exit 1
fi

for required_exec in "$VALIDATION_SCRIPT" "$POLICY_CHECKER" "$EVIDENCE_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected required executable script '$required_exec'" >&2
    exit 1
  fi
done
for required_doc in "$STRATEGY_DOC" "$BLOCK_PIPELINE_DOC" "$RUNBOOK_DOC"; do
  if [ ! -f "$required_doc" ]; then
    echo "expected required documentation file '$required_doc'" >&2
    exit 1
  fi
done

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

summary_report="$TMP_DIR/libp2p-convergence-process-isolated-live-summary.json"
policy_report="$TMP_DIR/libp2p-convergence-process-isolated-live-policy.json"
tampered_report="$TMP_DIR/libp2p-convergence-process-isolated-live-summary.tampered.json"
convergence_report="$TMP_DIR/libp2p-convergence-process-isolated-live-convergence.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode "$mode" \
    --lane-profile "$lane_profile" \
    --max-seconds "$max_seconds" \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$summary_report"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected process-isolated convergence validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected process-isolated convergence validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^lane_mode=$mode$"; then
  echo "expected process-isolated convergence validation lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^lane_profile=$lane_profile$"; then
  echo "expected process-isolated convergence validation lane profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_transport_mode=libp2p_process_isolated_convergence$'; then
  echo "expected process-isolated convergence runtime transport mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_disconnected_fail_closed_status=verified$'; then
  echo "expected process-isolated convergence disconnected fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_disconnected_fail_closed_reason_code=p2p_transport_live_socket_send_failed$'; then
  echo "expected process-isolated convergence disconnected fail-closed reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_connected_delivery_status=verified$'; then
  echo "expected process-isolated convergence connected delivery marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^no_shared_state_zero_delivery_status=verified$'; then
  echo "expected process-isolated convergence no-shared-state zero-delivery marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^no_shared_state_unexpected_delivery_reason_code=no_shared_state_unexpected_delivery_detected$'; then
  echo "expected process-isolated convergence no-shared-state unexpected-delivery reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^no_shared_state_delivery_count=0$'; then
  echo "expected process-isolated convergence no-shared-state delivery-count marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_discovery_status=verified$'; then
  echo "expected process-isolated convergence two-node discovery marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_gossip_status=verified$'; then
  echo "expected process-isolated convergence two-node gossip marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^native_compile_mode_status=verified$'; then
  echo "expected process-isolated convergence native compile-mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^three_node_partition_rejoin_status=verified$'; then
  echo "expected process-isolated convergence three-node partition/rejoin marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^three_node_publish_drop_recovery_status=verified$'; then
  echo "expected process-isolated convergence publish-drop marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^convergence_reason_code_status=verified$'; then
  echo "expected process-isolated convergence reason-code marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^convergence_reason_taxonomy_version=kamn.runtime.libp2p-convergence-reason-taxonomy.v1$'; then
  echo "expected process-isolated convergence reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^convergence_reason_codes_csv=fork_choice_stale_block_height$'; then
  echo "expected process-isolated convergence reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^finality_taxonomy_mapping_status=verified$'; then
  echo "expected process-isolated convergence finality taxonomy mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runbook_marker_parity_status=verified$'; then
  echo "expected process-isolated convergence runbook marker parity status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1$'; then
  echo "expected process-isolated convergence finality taxonomy runbook reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch$'; then
  echo "expected process-isolated convergence finality taxonomy runbook reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^transport_classification_normalization_status=verified$'; then
  echo "expected process-isolated convergence transport classification normalization marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fork_choice_stale_height_classification_status=verified$'; then
  echo "expected process-isolated convergence stale-height classification marker" >&2
  exit 1
fi

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --runbook-file "$RUNBOOK_DOC" \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected process-isolated convergence policy status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected process-isolated convergence policy final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^libp2p_process_isolated_convergence_policy_status=verified$'; then
  echo "expected process-isolated convergence policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^finality_taxonomy_mapping_status=verified$'; then
  echo "expected process-isolated convergence policy finality taxonomy mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^runbook_marker_parity_status=verified$'; then
  echo "expected process-isolated convergence policy runbook marker parity status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1$'; then
  echo "expected process-isolated convergence policy finality taxonomy runbook reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch$'; then
  echo "expected process-isolated convergence policy finality taxonomy runbook reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^finality_taxonomy_runbook_reason_code=none$'; then
  echo "expected process-isolated convergence policy finality taxonomy runbook reason code marker on GO path" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^promotion_decision_reason_mapping_status=verified$'; then
  echo "expected process-isolated convergence policy promotion decision reason mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^promotion_decision_reason_taxonomy_version=${PROMOTION_DECISION_REASON_TAXONOMY_VERSION}$"; then
  echo "expected process-isolated convergence policy promotion decision reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^promotion_decision_reason_codes_csv=${PROMOTION_DECISION_REASON_CODES_CSV}$"; then
  echo "expected process-isolated convergence policy promotion decision reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^promotion_decision_reason_code=none$'; then
  echo "expected process-isolated convergence policy promotion decision reason code marker on GO path" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["no_shared_state_zero_delivery_status"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --runbook-file "$RUNBOOK_DOC" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-live-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e
if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered process-isolated convergence report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'libp2p_process_isolated_convergence_policy_marker_missing:no_shared_state_zero_delivery_status'; then
  echo "expected deterministic fail-closed reason for tampered process-isolated convergence report" >&2
  exit 1
fi

for required_ref in \
  "validate_libp2p_convergence_process_isolated_live.sh" \
  "check_libp2p_convergence_process_isolated_live_policy.sh" \
  "check_libp2p_convergence_process_isolated_live_evidence_convergence.sh" \
  "validate_libp2p_convergence_process_isolated_live_contract_lane.sh" \
  "test_validate_libp2p_convergence_process_isolated_live.sh" \
  "test_check_libp2p_convergence_process_isolated_live_policy.sh" \
  "test_check_libp2p_convergence_process_isolated_live_evidence_convergence.sh" \
  "test_validate_libp2p_convergence_process_isolated_live_contract_lane.sh"; do
  if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done
if ! grep -q "process-isolated convergence deep run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode." "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include process-isolated convergence deep run-mode exclusion marker" >&2
  exit 1
fi
if ! grep -q "finality taxonomy and runbook-marker parity remains deterministic via:" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include finality taxonomy and runbook-marker parity heading" >&2
  exit 1
fi
if ! grep -q "finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include finality taxonomy runbook reason taxonomy marker" >&2
  exit 1
fi
if ! grep -q "finality evidence convergence remains deterministic via:" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include finality evidence convergence heading" >&2
  exit 1
fi
if ! grep -q "libp2p_finality_evidence_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include libp2p finality evidence reason taxonomy marker" >&2
  exit 1
fi
if ! grep -q "promotion_decision_reason_taxonomy_version=kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include libp2p promotion decision reason taxonomy marker" >&2
  exit 1
fi
if ! grep -q "libp2p_convergence_process_isolated_live_contract.py" "$BLOCK_PIPELINE_DOC"; then
  echo "expected block-pipeline doc to reference process-isolated convergence contract implementation" >&2
  exit 1
fi
if ! grep -q "## Fork-Choice Finality Taxonomy and Runbook Marker Parity Contracts (Issue #4252)" "$RUNBOOK_DOC"; then
  echo "expected runbook doc to include finality taxonomy and runbook marker parity section" >&2
  exit 1
fi
if ! grep -q "finality_taxonomy_mapping_status=verified" "$RUNBOOK_DOC"; then
  echo "expected runbook doc to include finality taxonomy mapping status marker" >&2
  exit 1
fi
if ! grep -q "runbook_marker_parity_status=verified" "$RUNBOOK_DOC"; then
  echo "expected runbook doc to include runbook marker parity status marker" >&2
  exit 1
fi
if ! grep -q "finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1" "$RUNBOOK_DOC"; then
  echo "expected runbook doc to include finality taxonomy runbook reason taxonomy marker" >&2
  exit 1
fi
if ! grep -q "finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch" "$RUNBOOK_DOC"; then
  echo "expected runbook doc to include finality taxonomy runbook reason codes marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "process-isolated convergence contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/libp2p-convergence-process-isolated-live-contract-lane-report.json"
python3 - "$summary_report" "$policy_report" "$lane_report" "$elapsed_seconds" "$max_seconds" "$mode" "$lane_profile" <<'PY'
import json
import pathlib
import sys

summary_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
lane_report_file = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])
mode = sys.argv[6]
lane_profile = sys.argv[7]

if summary_report.get("schema_version") != "kamn.runtime.libp2p-convergence-process-isolated-live-report.v1":
    raise SystemExit("unexpected process-isolated convergence summary schema")
if policy_report.get("schema_version") != "kamn.runtime.libp2p-convergence-process-isolated-live-policy-report.v1":
    raise SystemExit("unexpected process-isolated convergence policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected process-isolated convergence summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected process-isolated convergence policy final_decision=GO")
if summary_report.get("convergence_reason_taxonomy_version") != "kamn.runtime.libp2p-convergence-reason-taxonomy.v1":
    raise SystemExit("expected summary convergence_reason_taxonomy_version marker")
if summary_report.get("convergence_reason_codes_csv") != "fork_choice_stale_block_height":
    raise SystemExit("expected summary convergence_reason_codes_csv marker")
if summary_report.get("finality_taxonomy_mapping_status") != "verified":
    raise SystemExit("expected summary finality_taxonomy_mapping_status=verified")
if summary_report.get("runbook_marker_parity_status") != "verified":
    raise SystemExit("expected summary runbook_marker_parity_status=verified")
if summary_report.get("finality_taxonomy_runbook_reason_taxonomy_version") != "kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1":
    raise SystemExit("expected summary finality_taxonomy_runbook_reason_taxonomy_version marker")
if summary_report.get("finality_taxonomy_runbook_reason_codes_csv") != "finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch":
    raise SystemExit("expected summary finality_taxonomy_runbook_reason_codes_csv marker")
if summary_report.get("transport_classification_normalization_status") != "verified":
    raise SystemExit("expected summary transport_classification_normalization_status=verified")
if summary_report.get("fork_choice_stale_height_classification_status") != "verified":
    raise SystemExit("expected summary fork_choice_stale_height_classification_status=verified")
if policy_report.get("finality_taxonomy_mapping_status") != "verified":
    raise SystemExit("expected policy finality_taxonomy_mapping_status=verified")
if policy_report.get("runbook_marker_parity_status") != "verified":
    raise SystemExit("expected policy runbook_marker_parity_status=verified")
if policy_report.get("finality_taxonomy_runbook_reason_taxonomy_version") != "kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1":
    raise SystemExit("expected policy finality_taxonomy_runbook_reason_taxonomy_version marker")
if policy_report.get("finality_taxonomy_runbook_reason_codes_csv") != "finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch":
    raise SystemExit("expected policy finality_taxonomy_runbook_reason_codes_csv marker")
if policy_report.get("finality_taxonomy_runbook_reason_code") != "none":
    raise SystemExit("expected policy finality_taxonomy_runbook_reason_code=none")

lane_report = {
    "schema_version": "kamn.runtime.libp2p-convergence-process-isolated-live-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "lane_mode": mode,
    "lane_profile": lane_profile,
    "libp2p_process_isolated_convergence_contract_status": "verified",
    "libp2p_process_isolated_convergence_policy_status": policy_report.get(
        "libp2p_process_isolated_convergence_policy_status"
    ),
    "docs_contract_status": "verified",
    "runtime_transport_mode_status": "verified",
    "reason_taxonomy_status": "verified",
    "convergence_reason_taxonomy_version": summary_report.get(
        "convergence_reason_taxonomy_version"
    ),
    "convergence_reason_codes_csv": summary_report.get("convergence_reason_codes_csv"),
    "finality_taxonomy_mapping_status": policy_report.get(
        "finality_taxonomy_mapping_status"
    ),
    "runbook_marker_parity_status": policy_report.get("runbook_marker_parity_status"),
    "finality_taxonomy_runbook_reason_taxonomy_version": policy_report.get(
        "finality_taxonomy_runbook_reason_taxonomy_version"
    ),
    "finality_taxonomy_runbook_reason_codes_csv": policy_report.get(
        "finality_taxonomy_runbook_reason_codes_csv"
    ),
    "finality_taxonomy_runbook_reason_code": policy_report.get(
        "finality_taxonomy_runbook_reason_code"
    ),
    "promotion_decision_reason_mapping_status": policy_report.get(
        "promotion_decision_reason_mapping_status"
    ),
    "promotion_decision_reason_taxonomy_version": policy_report.get(
        "promotion_decision_reason_taxonomy_version"
    ),
    "promotion_decision_reason_codes_csv": policy_report.get(
        "promotion_decision_reason_codes_csv"
    ),
    "promotion_decision_reason_code": policy_report.get(
        "promotion_decision_reason_code"
    ),
    "transport_classification_normalization_status": summary_report.get(
        "transport_classification_normalization_status"
    ),
    "fork_choice_stale_height_classification_status": summary_report.get(
        "fork_choice_stale_height_classification_status"
    ),
    "fail_closed_status": "verified",
    "fail_closed_reason_code": "libp2p_process_isolated_convergence_policy_marker_missing:no_shared_state_zero_delivery_status",
    "performance_budget_status": "verified",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
}
lane_report_file.write_text(json.dumps(lane_report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

source_report_file="$summary_report"
if [[ -n "$summary_output_json" ]]; then
  mkdir -p "$(dirname "$summary_output_json")"
  cp "$summary_report" "$summary_output_json"
  source_report_file="$summary_output_json"
fi
python3 - "$policy_report" "$source_report_file" <<'PY'
import json
import pathlib
import sys

policy_report_file = pathlib.Path(sys.argv[1])
source_report_file = sys.argv[2]
payload = json.loads(policy_report_file.read_text(encoding="utf-8"))
payload["source_report_file"] = source_report_file
policy_report_file.write_text(
    json.dumps(payload, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

convergence_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$policy_report" \
    --output-json "$convergence_report"
)"
for marker in \
  "status=ok" \
  "final_decision=GO" \
  "evidence_convergence_status=verified" \
  "promotion_decision_reason_mapping_status=verified" \
  "reason_taxonomy_version=${EVIDENCE_REASON_TAXONOMY_VERSION}" \
  "reason_codes_csv=${EVIDENCE_REASON_CODES_CSV}" \
  "reason_codes_value=none" \
  "promotion_decision_reason_taxonomy_version=${PROMOTION_DECISION_REASON_TAXONOMY_VERSION}" \
  "promotion_decision_reason_codes_csv=${PROMOTION_DECISION_REASON_CODES_CSV}" \
  "promotion_decision_reason_code=none"; do
  if ! printf '%s\n' "$convergence_output" | grep -q "^${marker}$"; then
    echo "expected process-isolated convergence evidence marker ${marker}" >&2
    exit 1
  fi
done

tampered_policy_report="$TMP_DIR/libp2p-convergence-process-isolated-live-policy.tampered-mapping.json"
cp "$policy_report" "$tampered_policy_report"
python3 - "$tampered_policy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["promotion_decision_reason_code"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_convergence_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$tampered_policy_report" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-live-convergence.tampered-mapping.json" 2>&1
)"
tampered_convergence_code=$?
set -e
if [[ "$tampered_convergence_code" -eq 0 ]]; then
  echo "expected tampered process-isolated convergence promotion mapping to fail evidence checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_convergence_output" | grep -q "$EVIDENCE_TAMPER_REASON_CODE"; then
  echo "expected deterministic fail-closed reason for tampered process-isolated convergence promotion mapping" >&2
  exit 1
fi

python3 - "$lane_report" "$convergence_report" "$EVIDENCE_REPORT_SCHEMA" "$EVIDENCE_REASON_TAXONOMY_VERSION" "$EVIDENCE_REASON_CODES_CSV" "$PROMOTION_DECISION_REASON_TAXONOMY_VERSION" "$PROMOTION_DECISION_REASON_CODES_CSV" <<'PY'
import json
import pathlib
import sys

lane_report_file = pathlib.Path(sys.argv[1])
convergence_report_file = pathlib.Path(sys.argv[2])
expected_schema = sys.argv[3]
expected_reason_taxonomy_version = sys.argv[4]
expected_reason_codes_csv = sys.argv[5]
expected_promotion_reason_taxonomy_version = sys.argv[6]
expected_promotion_reason_codes_csv = sys.argv[7]

lane_payload = json.loads(lane_report_file.read_text(encoding="utf-8"))
convergence_payload = json.loads(convergence_report_file.read_text(encoding="utf-8"))
if convergence_payload.get("schema_version") != expected_schema:
    raise SystemExit("unexpected process-isolated convergence evidence report schema")
if convergence_payload.get("reason_taxonomy_version") != expected_reason_taxonomy_version:
    raise SystemExit("unexpected process-isolated convergence evidence reason taxonomy marker")
if convergence_payload.get("reason_codes_csv") != expected_reason_codes_csv:
    raise SystemExit("unexpected process-isolated convergence evidence reason codes marker")
if (
    convergence_payload.get("promotion_decision_reason_taxonomy_version")
    != expected_promotion_reason_taxonomy_version
):
    raise SystemExit("unexpected process-isolated convergence promotion reason taxonomy marker")
if (
    convergence_payload.get("promotion_decision_reason_codes_csv")
    != expected_promotion_reason_codes_csv
):
    raise SystemExit("unexpected process-isolated convergence promotion reason codes marker")

lane_payload["libp2p_finality_evidence_convergence_status"] = convergence_payload.get(
    "evidence_convergence_status"
)
lane_payload["promotion_decision_reason_mapping_status"] = convergence_payload.get(
    "promotion_decision_reason_mapping_status"
)
lane_payload["libp2p_finality_evidence_reason_taxonomy_version"] = (
    convergence_payload.get("reason_taxonomy_version")
)
lane_payload["libp2p_finality_evidence_reason_codes_csv"] = convergence_payload.get(
    "reason_codes_csv"
)
lane_payload["promotion_decision_reason_taxonomy_version"] = (
    convergence_payload.get("promotion_decision_reason_taxonomy_version")
)
lane_payload["promotion_decision_reason_codes_csv"] = convergence_payload.get(
    "promotion_decision_reason_codes_csv"
)
lane_payload["promotion_decision_reason_code"] = convergence_payload.get(
    "promotion_decision_reason_code"
)

lane_report_file.write_text(
    json.dumps(lane_payload, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

if [[ -n "$output_json" ]]; then
  cp "$lane_report" "$output_json"
fi
if [[ -n "$policy_output_json" ]]; then
  cp "$policy_report" "$policy_output_json"
fi
if [[ -n "$convergence_output_json" ]]; then
  cp "$convergence_report" "$convergence_output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "lane_mode=$mode"
echo "lane_profile=$lane_profile"
echo "libp2p_process_isolated_convergence_contract_status=verified"
echo "libp2p_process_isolated_convergence_policy_status=verified"
echo "docs_contract_status=verified"
echo "runtime_transport_mode_status=verified"
echo "reason_taxonomy_status=verified"
echo "convergence_reason_taxonomy_version=kamn.runtime.libp2p-convergence-reason-taxonomy.v1"
echo "convergence_reason_codes_csv=fork_choice_stale_block_height"
echo "finality_taxonomy_mapping_status=verified"
echo "runbook_marker_parity_status=verified"
echo "finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1"
echo "finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
echo "finality_taxonomy_runbook_reason_code=none"
echo "libp2p_finality_evidence_convergence_status=verified"
echo "promotion_decision_reason_mapping_status=verified"
echo "libp2p_finality_evidence_reason_taxonomy_version=${EVIDENCE_REASON_TAXONOMY_VERSION}"
echo "libp2p_finality_evidence_reason_codes_csv=${EVIDENCE_REASON_CODES_CSV}"
echo "promotion_decision_reason_taxonomy_version=${PROMOTION_DECISION_REASON_TAXONOMY_VERSION}"
echo "promotion_decision_reason_codes_csv=${PROMOTION_DECISION_REASON_CODES_CSV}"
echo "promotion_decision_reason_code=none"
echo "transport_classification_normalization_status=verified"
echo "fork_choice_stale_height_classification_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=libp2p_process_isolated_convergence_policy_marker_missing:no_shared_state_zero_delivery_status"
echo "performance_budget_status=verified"
