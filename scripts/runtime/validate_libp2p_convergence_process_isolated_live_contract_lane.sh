#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_convergence_process_isolated_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
BLOCK_PIPELINE_DOC="$ROOT_DIR/docs/architecture/block-pipeline.md"

output_json=""
policy_output_json=""
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

for required_exec in "$VALIDATION_SCRIPT" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected required executable script '$required_exec'" >&2
    exit 1
  fi
done
for required_doc in "$STRATEGY_DOC" "$BLOCK_PIPELINE_DOC"; do
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
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_discovery_status=verified$'; then
  echo "expected process-isolated convergence two-node discovery marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_gossip_status=verified$'; then
  echo "expected process-isolated convergence two-node gossip marker" >&2
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

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
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

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["two_node_disconnected_fail_closed_status"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-live-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e
if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered process-isolated convergence report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'libp2p_process_isolated_convergence_policy_marker_missing:two_node_disconnected_fail_closed_status'; then
  echo "expected deterministic fail-closed reason for tampered process-isolated convergence report" >&2
  exit 1
fi

for required_ref in \
  "validate_libp2p_convergence_process_isolated_live.sh" \
  "check_libp2p_convergence_process_isolated_live_policy.sh" \
  "validate_libp2p_convergence_process_isolated_live_contract_lane.sh" \
  "test_validate_libp2p_convergence_process_isolated_live.sh" \
  "test_check_libp2p_convergence_process_isolated_live_policy.sh" \
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
if ! grep -q "libp2p_convergence_process_isolated_live_contract.py" "$BLOCK_PIPELINE_DOC"; then
  echo "expected block-pipeline doc to reference process-isolated convergence contract implementation" >&2
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
    "fail_closed_status": "verified",
    "fail_closed_reason_code": "libp2p_process_isolated_convergence_policy_marker_missing:two_node_disconnected_fail_closed_status",
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
echo "lane_profile=$lane_profile"
echo "libp2p_process_isolated_convergence_contract_status=verified"
echo "libp2p_process_isolated_convergence_policy_status=verified"
echo "docs_contract_status=verified"
echo "runtime_transport_mode_status=verified"
echo "reason_taxonomy_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=libp2p_process_isolated_convergence_policy_marker_missing:two_node_disconnected_fail_closed_status"
echo "performance_budget_status=verified"
