#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/dashboard/run_backend_session_auth_freshness_lane.sh \
    [--output-json <path>]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DASHBOARD_TEST_SCRIPT="$ROOT_DIR/scripts/frontend/test_dashboard_package.sh"
BACKEND_DOC="$ROOT_DIR/docs/foundation/operator-dashboard-backend-apis.md"
output_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
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

if [[ -z "$output_json" ]]; then
  usage
  fail "--output-json is required"
fi

if [ ! -x "$DASHBOARD_TEST_SCRIPT" ]; then
  fail "expected dashboard package test script to be executable"
fi

if [ ! -f "$BACKEND_DOC" ]; then
  fail "expected backend dashboard contract doc to exist"
fi

max_seconds="${KAMN_DASHBOARD_BACKEND_SESSION_MAX_SECONDS:-180}"
skip_commands="${KAMN_DASHBOARD_BACKEND_SESSION_SKIP_COMMANDS:-false}"
force_session_guard_missing="${KAMN_DASHBOARD_BACKEND_SESSION_FORCE_SESSION_GUARD_MISSING:-false}"
force_freshness_guard_missing="${KAMN_DASHBOARD_BACKEND_SESSION_FORCE_FRESHNESS_GUARD_MISSING:-false}"
force_docs_contract_missing="${KAMN_DASHBOARD_BACKEND_SESSION_FORCE_DOCS_CONTRACT_MISSING:-false}"
force_lane_failure="${KAMN_DASHBOARD_BACKEND_SESSION_FORCE_LANE_FAILURE:-false}"

if [[ ! "$max_seconds" =~ ^[0-9]+$ ]]; then
  fail "KAMN_DASHBOARD_BACKEND_SESSION_MAX_SECONDS must be a non-negative integer"
fi

for value_name in \
  skip_commands \
  force_session_guard_missing \
  force_freshness_guard_missing \
  force_docs_contract_missing \
  force_lane_failure; do
  value="${!value_name}"
  if [[ "$value" != "true" && "$value" != "false" ]]; then
    fail "invalid boolean for ${value_name}: ${value}"
  fi
done

mkdir -p "$(dirname "$output_json")"
start_epoch="$(date +%s)"

commands=()
dashboard_output=""
dashboard_exit_code=0

if [[ "$force_lane_failure" == "true" ]]; then
  dashboard_exit_code=1
elif [[ "$skip_commands" != "true" ]]; then
  commands+=("bash scripts/frontend/test_dashboard_package.sh")
  set +e
  dashboard_output="$(bash "$DASHBOARD_TEST_SCRIPT" 2>&1)"
  dashboard_exit_code=$?
  set -e
fi

frontend_contract_passed=true
if [ "$dashboard_exit_code" -ne 0 ]; then
  frontend_contract_passed=false
fi

session_guard_passed=true
freshness_guard_passed=true
if [[ "$skip_commands" != "true" ]]; then
  if [ "$frontend_contract_passed" != "true" ]; then
    session_guard_passed=false
    freshness_guard_passed=false
  else
    if ! printf '%s\n' "$dashboard_output" | grep -Fq "regression rejects live backend access without operator session"; then
      session_guard_passed=false
    fi
    if ! printf '%s\n' "$dashboard_output" | grep -Fq "regression rejects expired or unauthorized session role"; then
      session_guard_passed=false
    fi
    if ! printf '%s\n' "$dashboard_output" | grep -Fq "functional marks stale banner when snapshot age exceeds threshold"; then
      freshness_guard_passed=false
    fi
  fi
fi

if [[ "$force_session_guard_missing" == "true" ]]; then
  session_guard_passed=false
fi

if [[ "$force_freshness_guard_missing" == "true" ]]; then
  freshness_guard_passed=false
fi

docs_contract_passed=true
required_doc_snippets=(
  "## Backend Session/Auth Freshness Contract"
  "run_backend_session_auth_freshness_lane.sh"
  "check_backend_session_auth_freshness_policy.sh"
  "run_backend_session_auth_freshness_contract_lane.sh"
  "kamn.dashboard.backend-session-auth-freshness-report.v1"
  "KAMN_DASHBOARD_BACKEND_SESSION_MAX_SECONDS"
  "KAMN_DASHBOARD_BACKEND_SESSION_CONTRACT_MAX_SECONDS"
  "Regression: #941"
)

for snippet in "${required_doc_snippets[@]}"; do
  if ! grep -Fq "$snippet" "$BACKEND_DOC"; then
    docs_contract_passed=false
    break
  fi
done

if [[ "$force_docs_contract_missing" == "true" ]]; then
  docs_contract_passed=false
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"

reason_codes=()
if [ "$frontend_contract_passed" != "true" ]; then
  reason_codes+=("backend_lane_failed")
fi
if [ "$session_guard_passed" != "true" ]; then
  reason_codes+=("session_guard_missing")
fi
if [ "$freshness_guard_passed" != "true" ]; then
  reason_codes+=("freshness_guard_missing")
fi
if [ "$docs_contract_passed" != "true" ]; then
  reason_codes+=("docs_contract_missing")
fi
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  reason_codes+=("runtime_budget_exceeded")
fi

if [ "${#reason_codes[@]}" -gt 0 ]; then
  mapfile -t reason_codes < <(printf '%s\n' "${reason_codes[@]}" | sort -u)
fi

status="pass"
final_decision="GO"
if [ "${#reason_codes[@]}" -gt 0 ]; then
  status="fail"
  final_decision="NO-GO"
fi
reason_key="dashboard_backend_session_auth_freshness_reason_codes:${final_decision}:v1"

reason_codes_csv="none"
if [ "${#reason_codes[@]}" -gt 0 ]; then
  reason_codes_csv="$(printf '%s,' "${reason_codes[@]}" | sed 's/,$//')"
fi

python3 - "$output_json" "$status" "$final_decision" "$reason_key" "$elapsed_seconds" "$max_seconds" "$skip_commands" "$dashboard_exit_code" "$frontend_contract_passed" "$session_guard_passed" "$freshness_guard_passed" "$docs_contract_passed" "$reason_codes_csv" "${commands[@]}" <<'PY'
import json
import pathlib
import sys

output_file = pathlib.Path(sys.argv[1])
status = sys.argv[2]
final_decision = sys.argv[3]
reason_key = sys.argv[4]
elapsed_seconds = int(sys.argv[5])
max_seconds = int(sys.argv[6])
skip_commands = sys.argv[7] == "true"
dashboard_exit_code = int(sys.argv[8])
frontend_contract_passed = sys.argv[9] == "true"
session_guard_passed = sys.argv[10] == "true"
freshness_guard_passed = sys.argv[11] == "true"
docs_contract_passed = sys.argv[12] == "true"
reason_codes_csv = sys.argv[13]
commands = sys.argv[14:]

payload = {
    "schema_version": "kamn.dashboard.backend-session-auth-freshness-report.v1",
    "evidence_key": "dashboard_backend_session_auth_freshness:v1",
    "status": status,
    "final_decision": final_decision,
    "reason_key": reason_key,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "skip_commands": skip_commands,
    "dashboard_package_exit_code": dashboard_exit_code,
    "command_count": len(commands),
    "commands": commands,
    "frontend_contract_passed": frontend_contract_passed,
    "session_guard_passed": session_guard_passed,
    "freshness_guard_passed": freshness_guard_passed,
    "docs_contract_passed": docs_contract_passed,
    "reason_codes": [] if reason_codes_csv == "none" else reason_codes_csv.split(","),
}
output_file.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

printf 'status=%s\n' "$status"
printf 'final_decision=%s\n' "$final_decision"
printf 'elapsed_seconds=%s\n' "$elapsed_seconds"
printf 'reason_codes=%s\n' "$reason_codes_csv"
printf 'reason_key=%s\n' "$reason_key"
printf 'report_file=%s\n' "$output_json"

if [ "$status" != "pass" ]; then
  fail "dashboard backend session/auth freshness lane failed closed: ${reason_codes_csv}"
fi

echo "dashboard backend session/auth freshness lane tests passed."
