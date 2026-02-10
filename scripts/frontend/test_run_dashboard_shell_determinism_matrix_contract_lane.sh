#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/frontend/run_dashboard_shell_determinism_matrix_contract_lane.sh"
SHARED_SCRIPT="$ROOT_DIR/scripts/frontend/dashboard_shell_determinism_matrix_contract_lane_contract.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected dashboard shell matrix contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q 'dashboard_shell_determinism_matrix_contract_lane_contract.py' "$SCRIPT"; then
  echo "expected dashboard shell matrix contract lane wrapper to delegate to shared implementation" >&2
  exit 1
fi

if [ ! -x "$SHARED_SCRIPT" ]; then
  echo "expected shared dashboard shell matrix contract lane implementation to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/dashboard-shell-matrix-contract-report.json"
output="$(
  KAMN_FRONTEND_SHELL_MATRIX_CONTRACT_MAX_SECONDS=240 \
  bash "$SCRIPT" --output-file "$report_file"
)"

if ! printf '%s\n' "$output" | grep -q 'dashboard shell determinism matrix contract lane tests passed.'; then
  echo "expected success output from dashboard shell matrix contract lane" >&2
  exit 1
fi

if [ ! -f "$report_file" ]; then
  echo "expected dashboard shell matrix contract lane to emit report file" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.frontend.shell-matrix-report.v1"' "$report_file"; then
  echo "expected dashboard shell matrix report schema marker in contract lane output" >&2
  exit 1
fi

if ! grep -q '"final_decision": "GO"' "$report_file"; then
  echo "expected GO final decision in dashboard shell matrix contract lane report" >&2
  exit 1
fi

if ! grep -q 'check_dashboard_shell_determinism_matrix_policy.sh' "$SHARED_SCRIPT"; then
  echo "expected dashboard shell matrix contract lane implementation to execute policy checker" >&2
  exit 1
fi

echo "dashboard shell determinism matrix contract lane script tests passed."
