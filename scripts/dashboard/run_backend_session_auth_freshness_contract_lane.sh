#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/dashboard/run_backend_session_auth_freshness_contract_lane.sh \
    [--output-file <path>]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/dashboard/run_backend_session_auth_freshness_lane.sh"
CHECKER="$ROOT_DIR/scripts/dashboard/check_backend_session_auth_freshness_policy.sh"
BACKEND_DOC="$ROOT_DIR/docs/foundation/operator-dashboard-backend-apis.md"
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
  fail "expected dashboard backend session/auth freshness lane script to be executable"
fi

if [ ! -x "$CHECKER" ]; then
  fail "expected dashboard backend session/auth freshness policy checker to be executable"
fi

if [ ! -f "$BACKEND_DOC" ]; then
  fail "expected backend dashboard contract doc to exist"
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/dashboard-backend-session-auth-freshness-contract-report.json"
fi

max_contract_seconds="${KAMN_DASHBOARD_BACKEND_SESSION_CONTRACT_MAX_SECONDS:-240}"
if [[ ! "$max_contract_seconds" =~ ^[1-9][0-9]*$ ]]; then
  fail "KAMN_DASHBOARD_BACKEND_SESSION_CONTRACT_MAX_SECONDS must be a positive integer"
fi

start_epoch="$(date +%s)"

go_output="$(
  KAMN_DASHBOARD_BACKEND_SESSION_MAX_SECONDS="$max_contract_seconds" \
  bash "$LANE_SCRIPT" --output-json "$output_file"
)"

if ! printf '%s\n' "$go_output" | grep -q '^status=pass$'; then
  fail "expected dashboard backend session/auth freshness lane GO run to report pass status"
fi

if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  fail "expected dashboard backend session/auth freshness lane GO run to report GO decision"
fi

if ! printf '%s\n' "$go_output" | grep -q '^reason_key=dashboard_backend_session_auth_freshness_reason_codes:GO:v1$'; then
  fail "expected dashboard backend session/auth freshness lane GO run to emit deterministic GO reason key"
fi

go_policy_output="$(bash "$CHECKER" --report-file "$output_file")"
if ! printf '%s\n' "$go_policy_output" | grep -q '^status=ok$'; then
  fail "expected dashboard backend session/auth freshness policy checker status marker for GO report"
fi
if ! printf '%s\n' "$go_policy_output" | grep -q '^final_decision=GO$'; then
  fail "expected dashboard backend session/auth freshness policy checker GO decision for GO report"
fi
if ! printf '%s\n' "$go_policy_output" | grep -q '^failed_checks=none$'; then
  fail "expected dashboard backend session/auth freshness policy checker no failed checks for GO report"
fi

session_no_go_report="$TMP_DIR/dashboard-backend-session-auth-freshness-session-no-go.json"
set +e
session_no_go_output="$(
  KAMN_DASHBOARD_BACKEND_SESSION_SKIP_COMMANDS=true \
  KAMN_DASHBOARD_BACKEND_SESSION_FORCE_SESSION_GUARD_MISSING=true \
  bash "$LANE_SCRIPT" --output-json "$session_no_go_report" 2>&1
)"
session_no_go_code=$?
set -e

if [ "$session_no_go_code" -eq 0 ]; then
  fail "expected forced session-guard-missing dashboard backend session/auth freshness lane run to fail closed"
fi

if ! printf '%s\n' "$session_no_go_output" | grep -q 'session_guard_missing'; then
  fail "expected forced session-guard-missing lane run to emit session_guard_missing reason code"
fi

session_no_go_policy_output="$(bash "$CHECKER" --report-file "$session_no_go_report")"
if ! printf '%s\n' "$session_no_go_policy_output" | grep -q '^final_decision=NO-GO$'; then
  fail "expected dashboard backend session/auth freshness policy checker NO-GO decision for session-guard-missing report"
fi
if ! printf '%s\n' "$session_no_go_policy_output" | grep -q 'session_guard_missing'; then
  fail "expected dashboard backend session/auth freshness policy checker failed checks to include session_guard_missing"
fi

if ! grep -q 'run_backend_session_auth_freshness_lane.sh' "$BACKEND_DOC"; then
  fail "expected backend dashboard contract doc to reference backend session/auth freshness lane command"
fi
if ! grep -q 'check_backend_session_auth_freshness_policy.sh' "$BACKEND_DOC"; then
  fail "expected backend dashboard contract doc to reference backend session/auth freshness policy checker command"
fi
if ! grep -q 'run_backend_session_auth_freshness_contract_lane.sh' "$BACKEND_DOC"; then
  fail "expected backend dashboard contract doc to reference backend session/auth freshness contract lane command"
fi
if ! grep -q 'kamn.dashboard.backend-session-auth-freshness-report.v1' "$BACKEND_DOC"; then
  fail "expected backend dashboard contract doc to reference backend session/auth freshness schema marker"
fi
if ! grep -q 'Regression: #941' "$BACKEND_DOC"; then
  fail "expected backend dashboard contract doc to include Regression: #941 marker"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_contract_seconds" ]; then
  fail "dashboard backend session/auth freshness contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'report_file=%s\n' "$output_file"
printf 'final_decision=GO\n'
echo "dashboard backend session/auth freshness contract lane tests passed."
