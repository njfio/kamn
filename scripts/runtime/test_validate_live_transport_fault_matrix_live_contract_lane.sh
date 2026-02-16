#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_live_transport_fault_matrix_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_live_transport_fault_matrix_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_live_transport_fault_matrix_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
EXPECTED_REASON_TAXONOMY_VERSION="kamn.runtime.live-transport-fault-matrix-reason-taxonomy.v1"
EXPECTED_REASON_CODES_CSV="ci_fast_gate_failed,live_transport_fault_matrix_policy_command_count_invalid,live_transport_fault_matrix_policy_command_count_mismatch,live_transport_fault_matrix_policy_elapsed_seconds_invalid,live_transport_fault_matrix_policy_execution_reason_code_mismatch,live_transport_fault_matrix_policy_final_decision_invalid,live_transport_fault_matrix_policy_final_decision_mismatch,live_transport_fault_matrix_policy_lane_mode_invalid,live_transport_fault_matrix_policy_marker_missing,live_transport_fault_matrix_policy_reason_codes_classification_mismatch,live_transport_fault_matrix_policy_reason_codes_invalid,live_transport_fault_matrix_policy_reason_taxonomy_version_mismatch,live_transport_fault_matrix_policy_runtime_transport_mode_mismatch,live_transport_fault_matrix_policy_schema_mismatch,live_transport_fault_matrix_policy_status_invalid"
EXPECTED_RESILIENCE_GATE_REASON_TAXONOMY_VERSION="kamn.runtime.live-transport-fault-matrix-resilience-gate-reason-taxonomy.v1"
EXPECTED_RESILIENCE_GATE_REASON_CODES_CSV="live_transport_fault_matrix_contract_ci_fast_gate_scope_mismatch,live_transport_fault_matrix_contract_ci_smoke_boundary_exceeded,live_transport_fault_matrix_contract_evidence_convergence_mismatch"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected live transport fault matrix contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected live transport fault matrix validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected live transport fault matrix policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/live-transport-fault-matrix-contract-lane-report.json"
policy_report="$TMP_DIR/live-transport-fault-matrix-policy-report.json"

lane_output="$({
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --ci-fast-gate PASS \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
} 2>&1)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected live transport fault matrix contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected live transport fault matrix contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^live_transport_fault_matrix_contract_status=verified$'; then
  echo "expected live transport fault matrix contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^live_transport_fault_matrix_policy_status=verified$'; then
  echo "expected live transport fault matrix policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q "^policy_reason_taxonomy_version=$EXPECTED_REASON_TAXONOMY_VERSION$"; then
  echo "expected live transport fault matrix policy reason taxonomy marker in contract lane output" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q "^policy_reason_codes_csv=$EXPECTED_REASON_CODES_CSV$"; then
  echo "expected live transport fault matrix policy reason codes taxonomy marker in contract lane output" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^policy_reason_codes_value=none$'; then
  echo "expected live transport fault matrix policy normalized reason codes marker in contract lane output" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^evidence_convergence_status=verified$'; then
  echo "expected live transport fault matrix evidence convergence marker in contract lane output" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^boundary_governance_status=verified$'; then
  echo "expected live transport fault matrix boundary governance marker in contract lane output" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q "^resilience_gate_reason_taxonomy_version=$EXPECTED_RESILIENCE_GATE_REASON_TAXONOMY_VERSION$"; then
  echo "expected live transport fault matrix resilience gate taxonomy marker in contract lane output" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q "^resilience_gate_reason_codes_csv=$EXPECTED_RESILIENCE_GATE_REASON_CODES_CSV$"; then
  echo "expected live transport fault matrix resilience gate reason codes taxonomy marker in contract lane output" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^resilience_gate_reason_codes_value=none$'; then
  echo "expected live transport fault matrix resilience gate normalized reason codes marker in contract lane output" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=live_transport_fault_matrix_policy_marker_missing:partition_rejoin_status$'; then
  echo "expected live transport fault matrix deterministic fail-closed reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^convergence_fail_closed_reason_code=live_transport_fault_matrix_contract_evidence_convergence_mismatch$'; then
  echo "expected live transport fault matrix deterministic convergence fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.live-transport-fault-matrix-contract-lane-report.v1":
    raise SystemExit("unexpected live transport fault matrix contract lane report schema")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected live transport fault matrix contract lane final_decision=GO")
if lane_payload.get("live_transport_fault_matrix_contract_status") != "verified":
    raise SystemExit("expected live transport fault matrix contract lane status marker")
if lane_payload.get("policy_reason_taxonomy_version") != "kamn.runtime.live-transport-fault-matrix-reason-taxonomy.v1":
    raise SystemExit("expected deterministic policy reason taxonomy marker in contract lane report")
if lane_payload.get("policy_reason_codes_csv") != "ci_fast_gate_failed,live_transport_fault_matrix_policy_command_count_invalid,live_transport_fault_matrix_policy_command_count_mismatch,live_transport_fault_matrix_policy_elapsed_seconds_invalid,live_transport_fault_matrix_policy_execution_reason_code_mismatch,live_transport_fault_matrix_policy_final_decision_invalid,live_transport_fault_matrix_policy_final_decision_mismatch,live_transport_fault_matrix_policy_lane_mode_invalid,live_transport_fault_matrix_policy_marker_missing,live_transport_fault_matrix_policy_reason_codes_classification_mismatch,live_transport_fault_matrix_policy_reason_codes_invalid,live_transport_fault_matrix_policy_reason_taxonomy_version_mismatch,live_transport_fault_matrix_policy_runtime_transport_mode_mismatch,live_transport_fault_matrix_policy_schema_mismatch,live_transport_fault_matrix_policy_status_invalid":
    raise SystemExit("expected deterministic policy reason codes taxonomy marker in contract lane report")
if lane_payload.get("policy_reason_codes_value") != "none":
    raise SystemExit("expected policy_reason_codes_value=none in contract lane report")
if lane_payload.get("evidence_convergence_status") != "verified":
    raise SystemExit("expected evidence_convergence_status=verified in contract lane report")
if lane_payload.get("boundary_governance_status") != "verified":
    raise SystemExit("expected boundary_governance_status=verified in contract lane report")
if lane_payload.get("resilience_gate_reason_taxonomy_version") != "kamn.runtime.live-transport-fault-matrix-resilience-gate-reason-taxonomy.v1":
    raise SystemExit("expected deterministic resilience gate reason taxonomy marker in contract lane report")
if lane_payload.get("resilience_gate_reason_codes_csv") != "live_transport_fault_matrix_contract_ci_fast_gate_scope_mismatch,live_transport_fault_matrix_contract_ci_smoke_boundary_exceeded,live_transport_fault_matrix_contract_evidence_convergence_mismatch":
    raise SystemExit("expected deterministic resilience gate reason codes taxonomy marker in contract lane report")
if lane_payload.get("resilience_gate_reason_codes_value") != "none":
    raise SystemExit("expected resilience_gate_reason_codes_value=none in contract lane report")
if lane_payload.get("convergence_fail_closed_reason_code") != "live_transport_fault_matrix_contract_evidence_convergence_mismatch":
    raise SystemExit("expected deterministic convergence fail-closed reason marker in contract lane report")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.live-transport-fault-matrix-policy-report.v1":
    raise SystemExit("unexpected live transport fault matrix policy report schema")
if policy_payload.get("reason_taxonomy_version") != "kamn.runtime.live-transport-fault-matrix-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason taxonomy marker in policy report")
if policy_payload.get("reason_codes_value") != "none":
    raise SystemExit("expected reason_codes_value=none in policy report")
PY

set +e
boundary_overrun_output="$({
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --ci-fast-gate PASS \
    --max-seconds 241
} 2>&1)"
boundary_overrun_code=$?
set -e
if [ "$boundary_overrun_code" -eq 0 ]; then
  echo "expected live transport fault matrix contract lane to fail when ci smoke max-seconds boundary is exceeded" >&2
  exit 1
fi
if ! printf '%s\n' "$boundary_overrun_output" | grep -q 'live_transport_fault_matrix_contract_ci_smoke_boundary_exceeded'; then
  echo "expected deterministic ci smoke boundary reason marker for live transport fault matrix contract lane" >&2
  exit 1
fi

set +e
scope_mismatch_output="$({
  bash "$CONTRACT_LANE" \
    --mode run \
    --ci-fast-gate PASS \
    --max-seconds 240
} 2>&1)"
scope_mismatch_code=$?
set -e
if [ "$scope_mismatch_code" -eq 0 ]; then
  echo "expected live transport fault matrix contract lane to fail when run mode is executed with ci-fast-gate PASS" >&2
  exit 1
fi
if ! printf '%s\n' "$scope_mismatch_output" | grep -q 'live_transport_fault_matrix_contract_ci_fast_gate_scope_mismatch'; then
  echo "expected deterministic ci-fast-gate scope mismatch marker for live transport fault matrix contract lane" >&2
  exit 1
fi

echo "live transport fault matrix contract lane tests passed."
