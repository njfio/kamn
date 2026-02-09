#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/frontend/run_dashboard_shell_determinism_matrix_lane.sh \
    --output-json <path>
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DASHBOARD_TEST_SCRIPT="$ROOT_DIR/scripts/frontend/test_dashboard_package.sh"
UI_DOC="$ROOT_DIR/docs/foundation/operator-dashboard-ui-mvp.md"
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

if [ ! -f "$UI_DOC" ]; then
  fail "expected operator dashboard UI doc to exist"
fi

max_seconds="${KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS:-180}"
skip_commands="${KAMN_FRONTEND_SHELL_MATRIX_SKIP_COMMANDS:-false}"
force_healthy_state_missing="${KAMN_FRONTEND_SHELL_MATRIX_FORCE_HEALTHY_STATE_MISSING:-false}"
force_stale_critical_state_missing="${KAMN_FRONTEND_SHELL_MATRIX_FORCE_STALE_CRITICAL_STATE_MISSING:-false}"
force_error_state_missing="${KAMN_FRONTEND_SHELL_MATRIX_FORCE_ERROR_STATE_MISSING:-false}"
force_docs_contract_missing="${KAMN_FRONTEND_SHELL_MATRIX_FORCE_DOCS_CONTRACT_MISSING:-false}"
force_lane_failure="${KAMN_FRONTEND_SHELL_MATRIX_FORCE_LANE_FAILURE:-false}"

if [[ ! "$max_seconds" =~ ^[0-9]+$ ]]; then
  fail "KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS must be a non-negative integer"
fi

for value_name in \
  skip_commands \
  force_healthy_state_missing \
  force_stale_critical_state_missing \
  force_error_state_missing \
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
frontend_lane_passed=true

if [[ "$force_lane_failure" == "true" ]]; then
  frontend_lane_passed=false
  dashboard_exit_code=1
elif [[ "$skip_commands" != "true" ]]; then
  commands+=("bash scripts/frontend/test_dashboard_package.sh")
  set +e
  dashboard_output="$(bash "$DASHBOARD_TEST_SCRIPT" 2>&1)"
  dashboard_exit_code=$?
  set -e
  if [ "$dashboard_exit_code" -ne 0 ]; then
    frontend_lane_passed=false
  fi
fi

healthy_state_passed=true
stale_critical_state_passed=true
error_state_passed=true

if [[ "$skip_commands" != "true" ]]; then
  if [ "$frontend_lane_passed" != "true" ]; then
    healthy_state_passed=false
    stale_critical_state_passed=false
    error_state_passed=false
  else
    if ! printf '%s\n' "$dashboard_output" | grep -Fq "functional builds dashboard shell from live backend snapshot"; then
      healthy_state_passed=false
    fi
    if ! printf '%s\n' "$dashboard_output" | grep -Fq "regression renders critical badge and stale banner together"; then
      stale_critical_state_passed=false
    fi
    if ! printf '%s\n' "$dashboard_output" | grep -Fq "integration renders explicit error state shell"; then
      error_state_passed=false
    fi
    if ! printf '%s\n' "$dashboard_output" | grep -Fq "regression renders error shell when live backend request fails"; then
      error_state_passed=false
    fi
  fi
fi

if [[ "$force_healthy_state_missing" == "true" ]]; then
  healthy_state_passed=false
fi
if [[ "$force_stale_critical_state_missing" == "true" ]]; then
  stale_critical_state_passed=false
fi
if [[ "$force_error_state_missing" == "true" ]]; then
  error_state_passed=false
fi

docs_contract_passed=true
required_doc_snippets=(
  "## Frontend Shell Determinism Matrix Contract"
  "run_dashboard_shell_determinism_matrix_lane.sh"
  "check_dashboard_shell_determinism_matrix_policy.sh"
  "run_dashboard_shell_determinism_matrix_contract_lane.sh"
  "kamn.frontend.shell-matrix-report.v1"
  "KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS"
  "KAMN_FRONTEND_SHELL_MATRIX_CONTRACT_MAX_SECONDS"
  "Regression: #943"
)

for snippet in "${required_doc_snippets[@]}"; do
  if ! grep -Fq "$snippet" "$UI_DOC"; then
    docs_contract_passed=false
    break
  fi
done

if [[ "$force_docs_contract_missing" == "true" ]]; then
  docs_contract_passed=false
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"

reason_codes=()
if [ "$frontend_lane_passed" != "true" ]; then
  reason_codes+=("frontend_lane_failed")
fi
if [ "$healthy_state_passed" != "true" ]; then
  reason_codes+=("healthy_state_missing")
fi
if [ "$stale_critical_state_passed" != "true" ]; then
  reason_codes+=("stale_critical_state_missing")
fi
if [ "$error_state_passed" != "true" ]; then
  reason_codes+=("error_state_missing")
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
reason_key="frontend_shell_matrix_reason_codes:${final_decision}:v1"

reason_codes_csv="none"
if [ "${#reason_codes[@]}" -gt 0 ]; then
  reason_codes_csv="$(printf '%s,' "${reason_codes[@]}" | sed 's/,$//')"
fi

python3 - "$output_json" "$status" "$final_decision" "$reason_key" "$elapsed_seconds" "$max_seconds" "$skip_commands" "$dashboard_exit_code" "$frontend_lane_passed" "$healthy_state_passed" "$stale_critical_state_passed" "$error_state_passed" "$docs_contract_passed" "$reason_codes_csv" "${commands[@]}" <<'PY'
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
frontend_lane_passed = sys.argv[9] == "true"
healthy_state_passed = sys.argv[10] == "true"
stale_critical_state_passed = sys.argv[11] == "true"
error_state_passed = sys.argv[12] == "true"
docs_contract_passed = sys.argv[13] == "true"
reason_codes_csv = sys.argv[14]
commands = sys.argv[15:]

payload = {
    "schema_version": "kamn.frontend.shell-matrix-report.v1",
    "evidence_key": "frontend_shell_matrix:v1",
    "status": status,
    "final_decision": final_decision,
    "reason_key": reason_key,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "skip_commands": skip_commands,
    "dashboard_package_exit_code": dashboard_exit_code,
    "command_count": len(commands),
    "commands": commands,
    "frontend_lane_passed": frontend_lane_passed,
    "healthy_state_passed": healthy_state_passed,
    "stale_critical_state_passed": stale_critical_state_passed,
    "error_state_passed": error_state_passed,
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
  fail "dashboard shell determinism matrix lane failed closed: ${reason_codes_csv}"
fi

echo "dashboard shell determinism matrix lane tests passed."
