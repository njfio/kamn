#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_live_transport_fault_matrix_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_live_transport_fault_matrix_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_live_transport_fault_matrix_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

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
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=live_transport_fault_matrix_policy_marker_missing:partition_rejoin_status$'; then
  echo "expected live transport fault matrix deterministic fail-closed reason marker" >&2
  exit 1
fi

echo "live transport fault matrix contract lane tests passed."
