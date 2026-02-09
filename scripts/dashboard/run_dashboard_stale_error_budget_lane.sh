#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/dashboard/run_dashboard_stale_error_budget_lane.sh \
    --output-json <path>
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/canary/generate_post_cutover_slo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/canary/check_post_cutover_slo_policy.sh"
OBSERVABILITY_DOC="$ROOT_DIR/docs/foundation/observability-slo-dashboards.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
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

if [ ! -x "$GENERATOR" ]; then
  fail "expected post-cutover SLO evidence generator to be executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "expected post-cutover SLO policy checker to be executable"
fi

if [ ! -f "$OBSERVABILITY_DOC" ]; then
  fail "expected observability SLO dashboard doc to exist"
fi

max_seconds="${KAMN_DASHBOARD_STALE_ERROR_MAX_SECONDS:-180}"
skip_commands="${KAMN_DASHBOARD_STALE_ERROR_SKIP_COMMANDS:-false}"
force_stale_data_missing="${KAMN_DASHBOARD_STALE_ERROR_FORCE_STALE_DATA_MISSING:-false}"
force_error_budget_missing="${KAMN_DASHBOARD_STALE_ERROR_FORCE_ERROR_BUDGET_MISSING:-false}"
force_docs_contract_missing="${KAMN_DASHBOARD_STALE_ERROR_FORCE_DOCS_CONTRACT_MISSING:-false}"
force_lane_failure="${KAMN_DASHBOARD_STALE_ERROR_FORCE_LANE_FAILURE:-false}"

if [[ ! "$max_seconds" =~ ^[0-9]+$ ]]; then
  fail "KAMN_DASHBOARD_STALE_ERROR_MAX_SECONDS must be a non-negative integer"
fi

for value_name in \
  skip_commands \
  force_stale_data_missing \
  force_error_budget_missing \
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
bundle_file="$TMP_DIR/dashboard-stale-error-evidence.json"
generator_output=""
policy_output=""
generator_exit_code=0
policy_exit_code=0
dashboard_lane_passed=true

if [[ "$force_lane_failure" == "true" ]]; then
  dashboard_lane_passed=false
  generator_exit_code=1
  policy_exit_code=1
elif [[ "$skip_commands" != "true" ]]; then
  commands+=("bash scripts/canary/generate_post_cutover_slo_evidence_bundle.sh --output-file ${bundle_file} --window-minutes 15 --p95-latency-ms 140 --max-p95-latency-ms 200 --error-rate-bps 18 --max-error-rate-bps 25 --delivery-success-bps 9992 --min-delivery-success-bps 9950 --snapshot-age-seconds 30 --max-snapshot-age-seconds 120 --evidence-complete true --ci-fast-gate PASS")
  set +e
  generator_output="$(
    bash "$GENERATOR" \
      --output-file "$bundle_file" \
      --window-minutes 15 \
      --p95-latency-ms 140 \
      --max-p95-latency-ms 200 \
      --error-rate-bps 18 \
      --max-error-rate-bps 25 \
      --delivery-success-bps 9992 \
      --min-delivery-success-bps 9950 \
      --snapshot-age-seconds 30 \
      --max-snapshot-age-seconds 120 \
      --evidence-complete true \
      --ci-fast-gate PASS 2>&1
  )"
  generator_exit_code=$?
  set -e

  if [ "$generator_exit_code" -eq 0 ]; then
    commands+=("bash scripts/canary/check_post_cutover_slo_policy.sh --bundle-file ${bundle_file}")
    set +e
    policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$bundle_file" 2>&1)"
    policy_exit_code=$?
    set -e
  else
    policy_exit_code=1
  fi

  if [ "$generator_exit_code" -ne 0 ] || [ "$policy_exit_code" -ne 0 ]; then
    dashboard_lane_passed=false
  fi
fi

canary_bundle_final_decision="unknown"
if [[ "$skip_commands" != "true" && -n "$generator_output" ]]; then
  maybe_bundle_decision="$(extract_value "$generator_output" "final_decision")"
  if [[ -n "$maybe_bundle_decision" ]]; then
    canary_bundle_final_decision="$maybe_bundle_decision"
  fi
fi

canary_policy_final_decision="unknown"
if [[ "$skip_commands" != "true" && -n "$policy_output" ]]; then
  maybe_policy_decision="$(extract_value "$policy_output" "final_decision")"
  if [[ -n "$maybe_policy_decision" ]]; then
    canary_policy_final_decision="$maybe_policy_decision"
  fi
fi

stale_data_passed=true
error_budget_passed=true
if [[ "$skip_commands" != "true" ]]; then
  if [ "$dashboard_lane_passed" != "true" ]; then
    stale_data_passed=false
    error_budget_passed=false
  else
    set +e
    python3 - "$bundle_file" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
metrics = payload.get("metrics", {})
if metrics.get("snapshot_age_seconds", 10**9) > metrics.get("max_snapshot_age_seconds", -1):
    raise SystemExit(1)
if metrics.get("error_rate_bps", 10**9) > metrics.get("max_error_rate_bps", -1):
    raise SystemExit(2)
PY
    metrics_check_code=$?
    set -e
    if [ "$metrics_check_code" -eq 1 ]; then
      stale_data_passed=false
    elif [ "$metrics_check_code" -eq 2 ]; then
      error_budget_passed=false
    elif [ "$metrics_check_code" -ne 0 ]; then
      stale_data_passed=false
      error_budget_passed=false
    fi

    if [ "$canary_policy_final_decision" != "GO" ]; then
      if [ "$stale_data_passed" = true ] && [ "$error_budget_passed" = true ]; then
        stale_data_passed=false
      fi
    fi
  fi
fi

if [[ "$force_stale_data_missing" == "true" ]]; then
  stale_data_passed=false
fi
if [[ "$force_error_budget_missing" == "true" ]]; then
  error_budget_passed=false
fi

docs_contract_passed=true
required_doc_snippets=(
  "## Dashboard Stale/Error Budget Policy Checker Contract"
  "run_dashboard_stale_error_budget_lane.sh"
  "check_dashboard_stale_error_budget_policy.sh"
  "run_dashboard_stale_error_budget_contract_lane.sh"
  "kamn.dashboard.stale-error-budget-report.v1"
  "KAMN_DASHBOARD_STALE_ERROR_MAX_SECONDS"
  "KAMN_DASHBOARD_STALE_ERROR_CONTRACT_MAX_SECONDS"
  "Regression: #942"
)

for snippet in "${required_doc_snippets[@]}"; do
  if ! grep -Fq "$snippet" "$OBSERVABILITY_DOC"; then
    docs_contract_passed=false
    break
  fi
done

if [[ "$force_docs_contract_missing" == "true" ]]; then
  docs_contract_passed=false
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"

reason_codes=()
if [ "$dashboard_lane_passed" != "true" ]; then
  reason_codes+=("dashboard_lane_failed")
fi
if [ "$stale_data_passed" != "true" ]; then
  reason_codes+=("stale_data_threshold_missing")
fi
if [ "$error_budget_passed" != "true" ]; then
  reason_codes+=("error_budget_threshold_missing")
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
reason_key="dashboard_stale_error_budget_reason_codes:${final_decision}:v1"

reason_codes_csv="none"
if [ "${#reason_codes[@]}" -gt 0 ]; then
  reason_codes_csv="$(printf '%s,' "${reason_codes[@]}" | sed 's/,$//')"
fi

python3 - "$output_json" "$status" "$final_decision" "$reason_key" "$elapsed_seconds" "$max_seconds" "$skip_commands" "$bundle_file" "$generator_exit_code" "$policy_exit_code" "$canary_bundle_final_decision" "$canary_policy_final_decision" "$dashboard_lane_passed" "$stale_data_passed" "$error_budget_passed" "$docs_contract_passed" "$reason_codes_csv" "${commands[@]}" <<'PY'
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
bundle_file = sys.argv[8]
generator_exit_code = int(sys.argv[9])
policy_exit_code = int(sys.argv[10])
canary_bundle_final_decision = sys.argv[11]
canary_policy_final_decision = sys.argv[12]
dashboard_lane_passed = sys.argv[13] == "true"
stale_data_passed = sys.argv[14] == "true"
error_budget_passed = sys.argv[15] == "true"
docs_contract_passed = sys.argv[16] == "true"
reason_codes_csv = sys.argv[17]
commands = sys.argv[18:]

payload = {
    "schema_version": "kamn.dashboard.stale-error-budget-report.v1",
    "evidence_key": "dashboard_stale_error_budget:v1",
    "status": status,
    "final_decision": final_decision,
    "reason_key": reason_key,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "skip_commands": skip_commands,
    "bundle_file": bundle_file,
    "generator_exit_code": generator_exit_code,
    "policy_exit_code": policy_exit_code,
    "canary_bundle_final_decision": canary_bundle_final_decision,
    "canary_policy_final_decision": canary_policy_final_decision,
    "dashboard_lane_passed": dashboard_lane_passed,
    "stale_data_passed": stale_data_passed,
    "error_budget_passed": error_budget_passed,
    "docs_contract_passed": docs_contract_passed,
    "command_count": len(commands),
    "commands": commands,
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
  fail "dashboard stale/error budget lane failed closed: ${reason_codes_csv}"
fi

echo "dashboard stale/error budget lane tests passed."
