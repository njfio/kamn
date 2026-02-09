#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/dashboard/run_dashboard_stale_error_budget_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected dashboard stale/error contract lane script to be executable" >&2
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

if ! grep -q 'check_dashboard_stale_error_budget_policy.sh' "$SCRIPT"; then
  echo "expected dashboard stale/error contract lane to execute policy checker" >&2
  exit 1
fi

echo "dashboard stale/error budget contract lane script tests passed."
