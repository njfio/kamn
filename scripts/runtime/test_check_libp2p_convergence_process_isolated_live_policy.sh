#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_convergence_process_isolated_live_policy.sh"
RUNBOOK_DOC="$ROOT_DIR/docs/deploy/kolme_devnet_ops.md"

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected process-isolated convergence policy checker script to be executable" >&2
  exit 1
fi
if [ ! -f "$RUNBOOK_DOC" ]; then
  echo "expected process-isolated convergence runbook doc to exist" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
report_file="$TMP_DIR/libp2p-convergence-process-isolated-summary.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_file" <<'JSON'
{
  "schema_version": "kamn.runtime.libp2p-convergence-process-isolated-live-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "lane_mode": "dry-run",
  "lane_profile": "smoke",
  "ci_fast_gate_exclusion_status": "verified",
  "runtime_transport_mode": "libp2p_process_isolated_convergence",
  "smoke_lane_status": "verified",
  "deep_lane_status": "skipped_local_only",
  "deep_lane_local_only_status": "required",
  "deep_harness_report_file": "",
  "two_node_disconnected_fail_closed_status": "verified",
  "two_node_disconnected_fail_closed_reason_code": "p2p_transport_live_socket_send_failed",
  "two_node_connected_delivery_status": "verified",
  "no_shared_state_zero_delivery_status": "verified",
  "no_shared_state_unexpected_delivery_reason_code": "no_shared_state_unexpected_delivery_detected",
  "no_shared_state_delivery_count": 0,
  "two_node_discovery_status": "verified",
  "two_node_gossip_status": "verified",
  "native_compile_mode_status": "verified",
  "three_node_partition_rejoin_status": "verified",
  "three_node_publish_drop_recovery_status": "verified",
  "convergence_reason_code_status": "verified",
  "convergence_reason_taxonomy_version": "kamn.runtime.libp2p-convergence-reason-taxonomy.v1",
  "convergence_reason_codes_csv": "fork_choice_stale_block_height",
  "finality_taxonomy_mapping_status": "verified",
  "runbook_marker_parity_status": "verified",
  "finality_taxonomy_runbook_reason_taxonomy_version": "kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1",
  "finality_taxonomy_runbook_reason_codes_csv": "finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
  "transport_classification_normalization_status": "verified",
  "fork_choice_stale_height_classification_status": "verified",
  "convergence_reason_codes": ["fork_choice_stale_block_height"],
  "evidence_keys": [
    "no_shared_state_zero_delivery_status",
    "two_node_disconnected_fail_closed_status",
    "two_node_connected_delivery_status",
    "two_node_discovery_status",
    "two_node_gossip_status",
    "native_compile_mode_status",
    "three_node_partition_rejoin_status",
    "three_node_publish_drop_recovery_status",
    "convergence_reason_code_status"
  ],
  "performance_budget_status": "verified",
  "execution_reason_code": "dry_run_no_commands_executed",
  "command_count": 0,
  "elapsed_seconds": 0
}
JSON

policy_report="$TMP_DIR/libp2p-convergence-process-isolated-policy.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$RUNBOOK_DOC" \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected process-isolated convergence policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected process-isolated convergence policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^libp2p_process_isolated_convergence_policy_status=verified$'; then
  echo "expected process-isolated convergence policy checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^finality_taxonomy_mapping_status=verified$'; then
  echo "expected deterministic finality taxonomy mapping status marker in policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^runbook_marker_parity_status=verified$'; then
  echo "expected deterministic runbook marker parity status marker in policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1$'; then
  echo "expected deterministic finality taxonomy runbook reason taxonomy marker in policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch$'; then
  echo "expected deterministic finality taxonomy runbook reason codes marker in policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^finality_taxonomy_runbook_reason_code=none$'; then
  echo "expected deterministic finality taxonomy runbook reason code marker on GO path in policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^promotion_decision_reason_mapping_status=verified$'; then
  echo "expected deterministic promotion decision reason mapping status marker in policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^promotion_decision_reason_taxonomy_version=kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1$'; then
  echo "expected deterministic promotion decision reason taxonomy version marker in policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^promotion_decision_reason_codes_csv=libp2p_process_isolated_convergence_policy_required_field_missing,libp2p_process_isolated_convergence_policy_marker_missing,libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,libp2p_process_isolated_convergence_policy_expected_decision_mismatch,libp2p_process_isolated_convergence_policy_violation$'; then
  echo "expected deterministic promotion decision reason taxonomy csv marker in policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^promotion_decision_reason_code=none$'; then
  echo "expected deterministic promotion decision reason code marker on GO path in policy output" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.libp2p-convergence-process-isolated-live-policy-report.v1":
    raise SystemExit("unexpected process-isolated convergence policy report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if payload.get("libp2p_process_isolated_convergence_policy_status") != "verified":
    raise SystemExit("expected libp2p_process_isolated_convergence_policy_status=verified")
if payload.get("convergence_reason_taxonomy_version") != "kamn.runtime.libp2p-convergence-reason-taxonomy.v1":
    raise SystemExit("expected deterministic convergence_reason_taxonomy_version marker")
if payload.get("convergence_reason_codes_csv") != "fork_choice_stale_block_height":
    raise SystemExit("expected deterministic convergence_reason_codes_csv marker")
if payload.get("finality_taxonomy_mapping_status") != "verified":
    raise SystemExit("expected deterministic finality_taxonomy_mapping_status marker")
if payload.get("runbook_marker_parity_status") != "verified":
    raise SystemExit("expected deterministic runbook_marker_parity_status marker")
if payload.get("finality_taxonomy_runbook_reason_taxonomy_version") != "kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1":
    raise SystemExit("expected deterministic finality_taxonomy_runbook_reason_taxonomy_version marker")
if payload.get("finality_taxonomy_runbook_reason_codes_csv") != "finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch":
    raise SystemExit("expected deterministic finality_taxonomy_runbook_reason_codes_csv marker")
if payload.get("finality_taxonomy_runbook_reason_code") != "none":
    raise SystemExit("expected deterministic finality_taxonomy_runbook_reason_code marker on GO path")
if payload.get("promotion_decision_reason_mapping_status") != "verified":
    raise SystemExit("expected deterministic promotion_decision_reason_mapping_status marker")
if payload.get("promotion_decision_reason_taxonomy_version") != "kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1":
    raise SystemExit("expected deterministic promotion_decision_reason_taxonomy_version marker")
if payload.get("promotion_decision_reason_codes_csv") != "libp2p_process_isolated_convergence_policy_required_field_missing,libp2p_process_isolated_convergence_policy_marker_missing,libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,libp2p_process_isolated_convergence_policy_expected_decision_mismatch,libp2p_process_isolated_convergence_policy_violation":
    raise SystemExit("expected deterministic promotion_decision_reason_codes_csv marker")
if payload.get("promotion_decision_reason_code") != "none":
    raise SystemExit("expected deterministic promotion_decision_reason_code marker on GO path")
if payload.get("transport_classification_normalization_status") != "verified":
    raise SystemExit("expected deterministic transport_classification_normalization_status marker")
if payload.get("fork_choice_stale_height_classification_status") != "verified":
    raise SystemExit("expected deterministic fork_choice_stale_height_classification_status marker")
PY

tampered_report="$TMP_DIR/libp2p-convergence-process-isolated-summary.tampered.json"
cp "$report_file" "$tampered_report"
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
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$RUNBOOK_DOC" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered process-isolated convergence report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'libp2p_process_isolated_convergence_policy_marker_missing:no_shared_state_zero_delivery_status'; then
  echo "expected deterministic mismatch reason code for tampered process-isolated convergence policy validation" >&2
  exit 1
fi

tampered_classification_report="$TMP_DIR/libp2p-convergence-process-isolated-summary.classification.tampered.json"
cp "$report_file" "$tampered_classification_report"
python3 - "$tampered_classification_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fork_choice_stale_height_classification_status"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_classification_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_classification_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$RUNBOOK_DOC" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-policy.classification.tampered.json" 2>&1
)"
tampered_classification_code=$?
set -e
if [ "$tampered_classification_code" -eq 0 ]; then
  echo "expected tampered stale-height classification report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_classification_output" | grep -q 'libp2p_process_isolated_convergence_policy_fork_choice_stale_height_classification_status_mismatch'; then
  echo "expected deterministic stale-height classification mismatch reason code for policy validation" >&2
  exit 1
fi

set +e
deep_fast_gate_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate FAIL \
    --runbook-file "$RUNBOOK_DOC" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-policy.deep-fast-gate.json" 2>&1
)"
deep_fast_gate_code=$?
set -e
if [ "$deep_fast_gate_code" -eq 0 ]; then
  echo "expected smoke profile with ci-fast-gate FAIL to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$deep_fast_gate_output" | grep -q 'libp2p_process_isolated_convergence_policy_ci_fast_gate_failed'; then
  echo "expected deterministic ci-fast-gate mismatch reason code for smoke profile policy validation" >&2
  exit 1
fi

deep_harness_report="$TMP_DIR/libp2p-process-isolated-harness-summary.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$deep_harness_report" <<'JSON'
{
  "schema_version": "kamn.runtime.libp2p-process-isolated-harness-report.v1",
  "status": "pass",
  "final_decision": "GO"
}
JSON

deep_report="$TMP_DIR/libp2p-convergence-process-isolated-summary.deep.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$deep_report" <<JSON
{
  "schema_version": "kamn.runtime.libp2p-convergence-process-isolated-live-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "lane_mode": "run",
  "lane_profile": "deep",
  "ci_fast_gate_exclusion_status": "verified",
  "runtime_transport_mode": "libp2p_process_isolated_convergence",
  "smoke_lane_status": "verified",
  "deep_lane_status": "verified",
  "deep_lane_local_only_status": "required",
  "deep_harness_report_file": "$deep_harness_report",
  "two_node_disconnected_fail_closed_status": "verified",
  "two_node_disconnected_fail_closed_reason_code": "p2p_transport_live_socket_send_failed",
  "two_node_connected_delivery_status": "verified",
  "no_shared_state_zero_delivery_status": "verified",
  "no_shared_state_unexpected_delivery_reason_code": "no_shared_state_unexpected_delivery_detected",
  "no_shared_state_delivery_count": 0,
  "two_node_discovery_status": "verified",
  "two_node_gossip_status": "verified",
  "native_compile_mode_status": "verified",
  "three_node_partition_rejoin_status": "verified",
  "three_node_publish_drop_recovery_status": "verified",
  "convergence_reason_code_status": "verified",
  "convergence_reason_taxonomy_version": "kamn.runtime.libp2p-convergence-reason-taxonomy.v1",
  "convergence_reason_codes_csv": "fork_choice_stale_block_height",
  "finality_taxonomy_mapping_status": "verified",
  "runbook_marker_parity_status": "verified",
  "finality_taxonomy_runbook_reason_taxonomy_version": "kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1",
  "finality_taxonomy_runbook_reason_codes_csv": "finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
  "transport_classification_normalization_status": "verified",
  "fork_choice_stale_height_classification_status": "verified",
  "convergence_reason_codes": ["fork_choice_stale_block_height"],
  "evidence_keys": [
    "no_shared_state_zero_delivery_status",
    "two_node_disconnected_fail_closed_status",
    "two_node_connected_delivery_status",
    "two_node_discovery_status",
    "two_node_gossip_status",
    "native_compile_mode_status",
    "three_node_partition_rejoin_status",
    "three_node_publish_drop_recovery_status",
    "convergence_reason_code_status"
  ],
  "performance_budget_status": "verified",
  "execution_reason_code": "run_mode_deep_harness_executed",
  "command_count": 1,
  "elapsed_seconds": 1
}
JSON

deep_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$deep_report" \
    --expected-final-decision GO \
    --ci-fast-gate FAIL \
    --runbook-file "$RUNBOOK_DOC" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-policy.deep.json"
)"
if ! printf '%s\n' "$deep_policy_output" | grep -q '^status=ok$'; then
  echo "expected deep profile policy checker status=ok marker with ci-fast-gate FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$deep_policy_output" | grep -q '^libp2p_process_isolated_convergence_policy_status=verified$'; then
  echo "expected deep profile policy checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$deep_policy_output" | grep -q '^finality_taxonomy_mapping_status=verified$'; then
  echo "expected deep profile finality taxonomy mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$deep_policy_output" | grep -q '^runbook_marker_parity_status=verified$'; then
  echo "expected deep profile runbook marker parity status marker" >&2
  exit 1
fi

taxonomy_tampered_report="$TMP_DIR/libp2p-convergence-process-isolated-summary.taxonomy.tampered.json"
cp "$report_file" "$taxonomy_tampered_report"
python3 - "$taxonomy_tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["convergence_reason_codes_csv"] = "fork_choice_stale_block_height,unknown_code"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
taxonomy_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$taxonomy_tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$RUNBOOK_DOC" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-policy.taxonomy.tampered.json" 2>&1
)"
taxonomy_tampered_code=$?
set -e
if [ "$taxonomy_tampered_code" -eq 0 ]; then
  echo "expected taxonomy-tampered process-isolated convergence report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$taxonomy_tampered_output" | grep -q 'finality_taxonomy_mapping_drift_detected'; then
  echo "expected deterministic finality taxonomy drift reason marker for taxonomy tamper path" >&2
  exit 1
fi
if ! printf '%s\n' "$taxonomy_tampered_output" | grep -q 'finality_taxonomy_runbook_reason_code=finality_taxonomy_mapping_drift_detected'; then
  echo "expected deterministic finality taxonomy runbook reason-code projection for taxonomy tamper path" >&2
  exit 1
fi

runbook_tampered_doc="$TMP_DIR/kolme_devnet_ops.runbook.tampered.md"
cp "$RUNBOOK_DOC" "$runbook_tampered_doc"
python3 - "$runbook_tampered_doc" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace(
    "finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected",
)
path.write_text(text, encoding="utf-8")
PY

set +e
runbook_divergence_output_first="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$runbook_tampered_doc" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-policy.runbook-divergence.first.json" 2>&1
)"
runbook_divergence_code_first=$?
runbook_divergence_output_second="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$runbook_tampered_doc" \
    --output-json "$TMP_DIR/libp2p-convergence-process-isolated-policy.runbook-divergence.second.json" 2>&1
)"
runbook_divergence_code_second=$?
set -e
if [ "$runbook_divergence_code_first" -eq 0 ] || [ "$runbook_divergence_code_second" -eq 0 ]; then
  echo "expected runbook-divergence process-isolated convergence report to fail policy checker deterministically" >&2
  exit 1
fi
if ! printf '%s\n' "$runbook_divergence_output_first" | grep -q 'runbook_marker_parity_mismatch'; then
  echo "expected deterministic runbook marker parity mismatch reason for first runbook divergence run" >&2
  exit 1
fi
if ! printf '%s\n' "$runbook_divergence_output_second" | grep -q 'runbook_marker_parity_mismatch'; then
  echo "expected deterministic runbook marker parity mismatch reason for second runbook divergence run" >&2
  exit 1
fi
if ! printf '%s\n' "$runbook_divergence_output_first" | grep -q 'finality_taxonomy_runbook_reason_code=runbook_marker_parity_mismatch'; then
  echo "expected deterministic finality taxonomy runbook reason-code projection for runbook divergence path" >&2
  exit 1
fi

python3 - "$runbook_divergence_output_first" "$runbook_divergence_output_second" <<'PY'
import sys

first = sys.argv[1]
second = sys.argv[2]

def marker(output: str, key: str) -> str:
    for line in output.splitlines():
        if line.startswith(f"{key}="):
            return line
    raise SystemExit(f"missing {key} marker in policy output")

if marker(first, "reason_codes") != marker(second, "reason_codes"):
    raise SystemExit("expected deterministic reason_codes projection across repeated runbook divergence checks")
if marker(first, "finality_taxonomy_runbook_reason_code") != marker(second, "finality_taxonomy_runbook_reason_code"):
    raise SystemExit("expected deterministic finality_taxonomy_runbook_reason_code projection across repeated runbook divergence checks")
PY

echo "process-isolated libp2p convergence live policy checker tests passed."
