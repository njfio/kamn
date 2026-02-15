#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_convergence_process_isolated_live_policy.sh"

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected process-isolated convergence policy checker script to be executable" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
report_file="$TMP_DIR/libp2p-convergence-process-isolated-summary.json"
cat > "$report_file" <<'JSON'
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

set +e
deep_fast_gate_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate FAIL \
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
cat > "$deep_harness_report" <<'JSON'
{
  "schema_version": "kamn.runtime.libp2p-process-isolated-harness-report.v1",
  "status": "pass",
  "final_decision": "GO"
}
JSON

deep_report="$TMP_DIR/libp2p-convergence-process-isolated-summary.deep.json"
cat > "$deep_report" <<JSON
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

echo "process-isolated libp2p convergence live policy checker tests passed."
