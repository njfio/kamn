#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/dashboard/run_dashboard_stale_error_budget_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/dashboard/stale_error_budget_contract_lane_contract.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected dashboard stale/error contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected dashboard stale/error shared contract-lane module to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/dashboard-stale-error-contract-report.json"
output="$(
  KAMN_DASHBOARD_STALE_ERROR_CONTRACT_MAX_SECONDS=240 \
  bash "$SCRIPT" --output-file "$report_file"
)"

if ! printf '%s\n' "$output" | grep -q 'dashboard stale/error budget contract lane tests passed.'; then
  echo "expected success output from dashboard stale/error contract lane" >&2
  exit 1
fi

if [ ! -f "$report_file" ]; then
  echo "expected dashboard stale/error contract lane to emit report file" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.dashboard.stale-error-budget-report.v1"' "$report_file"; then
  echo "expected dashboard stale/error report schema marker in contract lane output" >&2
  exit 1
fi

if ! grep -q '"final_decision": "GO"' "$report_file"; then
  echo "expected GO final decision in dashboard stale/error contract lane report" >&2
  exit 1
fi

if ! grep -q 'stale_error_budget_contract_lane_contract.py' "$SCRIPT"; then
  echo "expected dashboard stale/error contract lane wrapper to dispatch to shared module" >&2
  exit 1
fi

if ! grep -q 'check_dashboard_stale_error_budget_policy.sh' "$SHARED_CONTRACT"; then
  echo "expected dashboard stale/error shared contract-lane module to execute policy checker" >&2
  exit 1
fi

if ! grep -q 'KAMN_DASHBOARD_STALE_ERROR_CONTRACT_MAX_SECONDS' "$SHARED_CONTRACT"; then
  echo "expected dashboard stale/error shared contract-lane module to enforce runtime guard env marker" >&2
  exit 1
fi

if ! grep -q 'KAMN_DASHBOARD_STALE_ERROR_FORCE_STALE_DATA_MISSING' "$SHARED_CONTRACT"; then
  echo "expected dashboard stale/error shared contract-lane module to cover forced stale-data path" >&2
  exit 1
fi

echo "dashboard stale/error budget contract lane script tests passed."
