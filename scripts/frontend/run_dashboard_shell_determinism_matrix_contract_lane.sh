#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/frontend/run_dashboard_shell_determinism_matrix_contract_lane.sh \
    [--output-file <path>]
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/frontend/run_dashboard_shell_determinism_matrix_lane.sh"
CHECKER="$ROOT_DIR/scripts/frontend/check_dashboard_shell_determinism_matrix_policy.sh"
UI_DOC="$ROOT_DIR/docs/foundation/operator-dashboard-ui-mvp.md"
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
  fail "expected dashboard shell matrix lane script to be executable"
fi

if [ ! -x "$CHECKER" ]; then
  fail "expected dashboard shell matrix policy checker to be executable"
fi

if [ ! -f "$UI_DOC" ]; then
  fail "expected operator dashboard UI doc to exist"
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/dashboard-shell-matrix-contract-report.json"
fi

max_contract_seconds="${KAMN_FRONTEND_SHELL_MATRIX_CONTRACT_MAX_SECONDS:-240}"
if [[ ! "$max_contract_seconds" =~ ^[1-9][0-9]*$ ]]; then
  fail "KAMN_FRONTEND_SHELL_MATRIX_CONTRACT_MAX_SECONDS must be a positive integer"
fi

start_epoch="$(date +%s)"

go_output="$(
  KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS="$max_contract_seconds" \
  bash "$LANE_SCRIPT" --output-json "$output_file"
)"

if ! printf '%s\n' "$go_output" | grep -q '^status=pass$'; then
  fail "expected dashboard shell matrix lane GO run to report pass status"
fi
if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  fail "expected dashboard shell matrix lane GO run to report GO decision"
fi
if ! printf '%s\n' "$go_output" | grep -q '^reason_key=frontend_shell_matrix_reason_codes:GO:v1$'; then
  fail "expected dashboard shell matrix lane GO run to emit deterministic GO reason key"
fi

go_policy_output="$(bash "$CHECKER" --report-file "$output_file")"
if ! printf '%s\n' "$go_policy_output" | grep -q '^status=ok$'; then
  fail "expected dashboard shell matrix policy checker status marker for GO report"
fi
if ! printf '%s\n' "$go_policy_output" | grep -q '^final_decision=GO$'; then
  fail "expected dashboard shell matrix policy checker GO decision for GO report"
fi
if ! printf '%s\n' "$go_policy_output" | grep -q '^failed_checks=none$'; then
  fail "expected dashboard shell matrix policy checker no failed checks for GO report"
fi

stale_critical_no_go_report="$TMP_DIR/dashboard-shell-matrix-no-go.json"
set +e
stale_critical_no_go_output="$(
  KAMN_FRONTEND_SHELL_MATRIX_SKIP_COMMANDS=true \
  KAMN_FRONTEND_SHELL_MATRIX_FORCE_STALE_CRITICAL_STATE_MISSING=true \
  bash "$LANE_SCRIPT" --output-json "$stale_critical_no_go_report" 2>&1
)"
stale_critical_no_go_code=$?
set -e

if [ "$stale_critical_no_go_code" -eq 0 ]; then
  fail "expected forced stale/critical missing dashboard shell matrix lane run to fail closed"
fi

if ! printf '%s\n' "$stale_critical_no_go_output" | grep -q 'stale_critical_state_missing'; then
  fail "expected forced stale/critical missing lane run to emit stale_critical_state_missing reason code"
fi

stale_critical_no_go_policy_output="$(bash "$CHECKER" --report-file "$stale_critical_no_go_report")"
if ! printf '%s\n' "$stale_critical_no_go_policy_output" | grep -q '^final_decision=NO-GO$'; then
  fail "expected dashboard shell matrix policy checker NO-GO decision for stale/critical-missing report"
fi
if ! printf '%s\n' "$stale_critical_no_go_policy_output" | grep -q 'stale_critical_state_missing'; then
  fail "expected dashboard shell matrix policy checker failed checks to include stale_critical_state_missing"
fi

if ! grep -q 'run_dashboard_shell_determinism_matrix_lane.sh' "$UI_DOC"; then
  fail "expected operator dashboard UI doc to reference dashboard shell matrix lane command"
fi
if ! grep -q 'check_dashboard_shell_determinism_matrix_policy.sh' "$UI_DOC"; then
  fail "expected operator dashboard UI doc to reference dashboard shell matrix policy checker command"
fi
if ! grep -q 'run_dashboard_shell_determinism_matrix_contract_lane.sh' "$UI_DOC"; then
  fail "expected operator dashboard UI doc to reference dashboard shell matrix contract lane command"
fi
if ! grep -q 'kamn.frontend.shell-matrix-report.v1' "$UI_DOC"; then
  fail "expected operator dashboard UI doc to reference dashboard shell matrix schema marker"
fi
if ! grep -q 'Regression: #943' "$UI_DOC"; then
  fail "expected operator dashboard UI doc to include Regression: #943 marker"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_contract_seconds" ]; then
  fail "dashboard shell determinism matrix contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'report_file=%s\n' "$output_file"
printf 'final_decision=GO\n'
echo "dashboard shell determinism matrix contract lane tests passed."
