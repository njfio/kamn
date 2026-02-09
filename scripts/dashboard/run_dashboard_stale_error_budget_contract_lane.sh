#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/dashboard/run_dashboard_stale_error_budget_contract_lane.sh \
    [--output-file <path>]
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/dashboard/run_dashboard_stale_error_budget_lane.sh"
CHECKER="$ROOT_DIR/scripts/dashboard/check_dashboard_stale_error_budget_policy.sh"
OBSERVABILITY_DOC="$ROOT_DIR/docs/foundation/observability-slo-dashboards.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

output_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [ ! -x "$LANE_SCRIPT" ]; then
  fail "expected dashboard stale/error budget lane script to be executable"
fi

if [ ! -x "$CHECKER" ]; then
  fail "expected dashboard stale/error budget policy checker to be executable"
fi

if [ ! -f "$OBSERVABILITY_DOC" ]; then
  fail "expected observability SLO dashboard doc to exist"
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/dashboard-stale-error-contract-report.json"
fi

max_contract_seconds="${KAMN_DASHBOARD_STALE_ERROR_CONTRACT_MAX_SECONDS:-240}"
if [[ ! "$max_contract_seconds" =~ ^[1-9][0-9]*$ ]]; then
  fail "KAMN_DASHBOARD_STALE_ERROR_CONTRACT_MAX_SECONDS must be a positive integer"
fi

start_epoch="$(date +%s)"

go_output="$(
  KAMN_DASHBOARD_STALE_ERROR_MAX_SECONDS="$max_contract_seconds" \
  bash "$LANE_SCRIPT" --output-json "$output_file"
)"

if ! printf '%s\n' "$go_output" | grep -q '^status=pass$'; then
  fail "expected dashboard stale/error budget lane GO run to report pass status"
fi
if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  fail "expected dashboard stale/error budget lane GO run to report GO decision"
fi
if ! printf '%s\n' "$go_output" | grep -q '^reason_key=dashboard_stale_error_budget_reason_codes:GO:v1$'; then
  fail "expected dashboard stale/error budget lane GO run to emit deterministic GO reason key"
fi

go_policy_output="$(bash "$CHECKER" --report-file "$output_file")"
if ! printf '%s\n' "$go_policy_output" | grep -q '^status=ok$'; then
  fail "expected dashboard stale/error budget policy checker status marker for GO report"
fi
if ! printf '%s\n' "$go_policy_output" | grep -q '^final_decision=GO$'; then
  fail "expected dashboard stale/error budget policy checker GO decision for GO report"
fi
if ! printf '%s\n' "$go_policy_output" | grep -q '^failed_checks=none$'; then
  fail "expected dashboard stale/error budget policy checker no failed checks for GO report"
fi

stale_no_go_report="$TMP_DIR/dashboard-stale-error-no-go.json"
set +e
stale_no_go_output="$(
  KAMN_DASHBOARD_STALE_ERROR_SKIP_COMMANDS=true \
  KAMN_DASHBOARD_STALE_ERROR_FORCE_STALE_DATA_MISSING=true \
  bash "$LANE_SCRIPT" --output-json "$stale_no_go_report" 2>&1
)"
stale_no_go_code=$?
set -e

if [ "$stale_no_go_code" -eq 0 ]; then
  fail "expected forced stale-data-missing dashboard stale/error lane run to fail closed"
fi

if ! printf '%s\n' "$stale_no_go_output" | grep -q 'stale_data_threshold_missing'; then
  fail "expected forced stale-data-missing lane run to emit stale_data_threshold_missing reason code"
fi

stale_no_go_policy_output="$(bash "$CHECKER" --report-file "$stale_no_go_report")"
if ! printf '%s\n' "$stale_no_go_policy_output" | grep -q '^final_decision=NO-GO$'; then
  fail "expected dashboard stale/error budget policy checker NO-GO decision for stale-data-missing report"
fi
if ! printf '%s\n' "$stale_no_go_policy_output" | grep -q 'stale_data_threshold_missing'; then
  fail "expected dashboard stale/error budget policy checker failed checks to include stale_data_threshold_missing"
fi

if ! grep -q 'run_dashboard_stale_error_budget_lane.sh' "$OBSERVABILITY_DOC"; then
  fail "expected observability doc to reference dashboard stale/error lane command"
fi
if ! grep -q 'check_dashboard_stale_error_budget_policy.sh' "$OBSERVABILITY_DOC"; then
  fail "expected observability doc to reference dashboard stale/error policy checker command"
fi
if ! grep -q 'run_dashboard_stale_error_budget_contract_lane.sh' "$OBSERVABILITY_DOC"; then
  fail "expected observability doc to reference dashboard stale/error contract lane command"
fi
if ! grep -q 'kamn.dashboard.stale-error-budget-report.v1' "$OBSERVABILITY_DOC"; then
  fail "expected observability doc to reference dashboard stale/error schema marker"
fi
if ! grep -q 'Regression: #942' "$OBSERVABILITY_DOC"; then
  fail "expected observability doc to include Regression: #942 marker"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_contract_seconds" ]; then
  fail "dashboard stale/error budget contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'report_file=%s\n' "$output_file"
printf 'final_decision=GO\n'
echo "dashboard stale/error budget contract lane tests passed."
