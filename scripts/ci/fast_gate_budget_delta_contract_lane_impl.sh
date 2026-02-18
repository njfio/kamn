#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_fast_gate_budget_delta_contract_lane.sh \
  --output-json <path> \
  [--max-runtime-seconds <int>]
USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATE_SCRIPT="$ROOT_DIR/scripts/ci/generate_fast_gate_budget_delta_report.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_fast_gate_budget_delta_threshold.sh"

OUTPUT_JSON=""
MAX_RUNTIME_SECONDS="${KAMN_FAST_GATE_BUDGET_DELTA_CONTRACT_MAX_SECONDS:-120}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-json)
      OUTPUT_JSON="${2:-}"
      shift 2
      ;;
    --max-runtime-seconds)
      MAX_RUNTIME_SECONDS="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$OUTPUT_JSON" ]; then
  usage >&2
  exit 2
fi

case "$MAX_RUNTIME_SECONDS" in
  ''|*[!0-9]*)
    echo "--max-runtime-seconds must be a non-negative integer" >&2
    exit 2
    ;;
esac

if [ ! -x "$GENERATE_SCRIPT" ]; then
  echo "expected fast-gate budget-delta generator to be executable: $GENERATE_SCRIPT" >&2
  exit 1
fi

if [ ! -x "$CHECK_SCRIPT" ]; then
  echo "expected fast-gate budget-delta checker to be executable: $CHECK_SCRIPT" >&2
  exit 1
fi

mkdir -p "$(dirname "$OUTPUT_JSON")"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

start_epoch="$(date +%s)"

CURRENT_PASS_JSON="$tmp_dir/current-pass.json"
CURRENT_FAIL_JSON="$tmp_dir/current-fail.json"
FRESH_THRESHOLD_ENV="$tmp_dir/threshold-fresh.env"
RATCHET_BASELINE_ENV="$tmp_dir/ratchet-baseline.env"
RATCHET_REGRESSION_BASELINE_ENV="$tmp_dir/ratchet-regression-baseline.env"
STALE_THRESHOLD_ENV="$tmp_dir/threshold-stale.env"
CORRUPT_THRESHOLD_ENV="$tmp_dir/threshold-corrupt.env"
WAIVER_JSON="$tmp_dir/waiver.json"
RATCHET_EXCEPTION_JSON="$tmp_dir/ratchet-exception.json"
PASS_REPORT="$tmp_dir/pass-report.json"
FAIL_REPORT="$tmp_dir/fail-report.json"
PASS_OUT="$tmp_dir/pass.out"
UNWAIVED_OUT="$tmp_dir/unwaived.out"
WAIVED_OUT="$tmp_dir/waived.out"
RATCHET_UNWAIVED_OUT="$tmp_dir/ratchet-unwaived.out"
RATCHET_WAIVED_OUT="$tmp_dir/ratchet-waived.out"
STALE_OUT="$tmp_dir/stale.out"
CORRUPT_OUT="$tmp_dir/corrupt.out"

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$CURRENT_PASS_JSON" <<'JSON'
{
  "lane": "fast-gate",
  "status": "pass",
  "test_scope": "targeted",
  "elapsed_seconds": 240,
  "runner_minutes": 4
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$CURRENT_FAIL_JSON" <<'JSON'
{
  "lane": "fast-gate",
  "status": "pass",
  "test_scope": "targeted",
  "elapsed_seconds": 400,
  "runner_minutes": 8
}
JSON

cat >"$FRESH_THRESHOLD_ENV" <<'ENV'
FAST_GATE_DELTA_BASELINE_ELAPSED_SECONDS=230
FAST_GATE_DELTA_BASELINE_RUNNER_MINUTES=4
FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT=20
FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT=20
FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON=2026-01-01
FAST_GATE_DELTA_THRESHOLD_MAX_AGE_DAYS=36500
ENV

cat >"$RATCHET_BASELINE_ENV" <<'ENV'
FAST_GATE_DELTA_BASELINE_ELAPSED_SECONDS=230
FAST_GATE_DELTA_BASELINE_RUNNER_MINUTES=4
FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT=20
FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT=20
FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON=2026-01-01
FAST_GATE_DELTA_THRESHOLD_MAX_AGE_DAYS=36500
ENV

cat >"$RATCHET_REGRESSION_BASELINE_ENV" <<'ENV'
FAST_GATE_DELTA_BASELINE_ELAPSED_SECONDS=230
FAST_GATE_DELTA_BASELINE_RUNNER_MINUTES=4
FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT=10
FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT=10
FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON=2026-01-01
FAST_GATE_DELTA_THRESHOLD_MAX_AGE_DAYS=36500
ENV

cat >"$STALE_THRESHOLD_ENV" <<'ENV'
FAST_GATE_DELTA_BASELINE_ELAPSED_SECONDS=230
FAST_GATE_DELTA_BASELINE_RUNNER_MINUTES=4
FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT=20
FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT=20
FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON=2000-01-01
FAST_GATE_DELTA_THRESHOLD_MAX_AGE_DAYS=30
ENV

cat >"$CORRUPT_THRESHOLD_ENV" <<'ENV'
FAST_GATE_DELTA_BASELINE_ELAPSED_SECONDS=230
FAST_GATE_DELTA_BASELINE_RUNNER_MINUTES=4
FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT=invalid
FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT=20
FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON=2026-01-01
FAST_GATE_DELTA_THRESHOLD_MAX_AGE_DAYS=36500
ENV

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$WAIVER_JSON" <<'JSON'
{
  "reason": "Temporary migration overhead while lane framework settles",
  "expires_on": "2099-12-31",
  "allow_metrics": [
    "elapsed_seconds_delta_pct",
    "runner_minutes_delta_pct"
  ]
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$RATCHET_EXCEPTION_JSON" <<'JSON'
{
  "reason": "Temporary ratchet exception while governance updates settle",
  "expires_on": "2099-12-31",
  "mitigation_issue": "#4859",
  "allow_threshold_keys": [
    "FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT",
    "FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT"
  ]
}
JSON

bash "$GENERATE_SCRIPT" \
  --current-json "$CURRENT_PASS_JSON" \
  --baseline-file "$FRESH_THRESHOLD_ENV" \
  --output-json "$PASS_REPORT" >/dev/null

bash "$GENERATE_SCRIPT" \
  --current-json "$CURRENT_FAIL_JSON" \
  --baseline-file "$FRESH_THRESHOLD_ENV" \
  --output-json "$FAIL_REPORT" >/dev/null

pass_status="fail"
if bash "$CHECK_SCRIPT" \
  --report-json "$PASS_REPORT" \
  --threshold-file "$FRESH_THRESHOLD_ENV" \
  --waiver-file "$WAIVER_JSON" \
  --ratchet-baseline-file "$RATCHET_BASELINE_ENV" \
  --ratchet-exception-file "$RATCHET_EXCEPTION_JSON" >"$PASS_OUT"; then
  if grep -q '^status=pass$' "$PASS_OUT" && grep -q '^reason_codes=none$' "$PASS_OUT"; then
    pass_status="pass"
  fi
fi

unwaived_status="pass"
if bash "$CHECK_SCRIPT" \
  --report-json "$FAIL_REPORT" \
  --threshold-file "$FRESH_THRESHOLD_ENV" \
  --waiver-file "$tmp_dir/missing-waiver.json" \
  --ratchet-baseline-file "$RATCHET_BASELINE_ENV" \
  --ratchet-exception-file "$RATCHET_EXCEPTION_JSON" >"$UNWAIVED_OUT" 2>&1; then
  unwaived_status="pass"
elif grep -q '^status=fail$' "$UNWAIVED_OUT" \
  && grep -q '^reason_codes=delta_threshold_violation_unwaived$' "$UNWAIVED_OUT"; then
  unwaived_status="fail"
fi

waived_status="fail"
if bash "$CHECK_SCRIPT" \
  --report-json "$FAIL_REPORT" \
  --threshold-file "$FRESH_THRESHOLD_ENV" \
  --waiver-file "$WAIVER_JSON" \
  --ratchet-baseline-file "$RATCHET_BASELINE_ENV" \
  --ratchet-exception-file "$RATCHET_EXCEPTION_JSON" >"$WAIVED_OUT"; then
  if grep -q '^status=pass$' "$WAIVED_OUT" \
    && grep -q '^reason_codes=delta_threshold_waiver_applied$' "$WAIVED_OUT"; then
    waived_status="pass"
  fi
fi

stale_threshold_status="pass"
if bash "$CHECK_SCRIPT" \
  --report-json "$PASS_REPORT" \
  --threshold-file "$STALE_THRESHOLD_ENV" \
  --waiver-file "$WAIVER_JSON" \
  --ratchet-baseline-file "$RATCHET_BASELINE_ENV" \
  --ratchet-exception-file "$RATCHET_EXCEPTION_JSON" >"$STALE_OUT" 2>&1; then
  stale_threshold_status="pass"
elif grep -q 'threshold file stale' "$STALE_OUT"; then
  stale_threshold_status="fail"
fi

corrupt_threshold_status="pass"
if bash "$CHECK_SCRIPT" \
  --report-json "$PASS_REPORT" \
  --threshold-file "$CORRUPT_THRESHOLD_ENV" \
  --waiver-file "$WAIVER_JSON" \
  --ratchet-baseline-file "$RATCHET_BASELINE_ENV" \
  --ratchet-exception-file "$RATCHET_EXCEPTION_JSON" >"$CORRUPT_OUT" 2>&1; then
  corrupt_threshold_status="pass"
elif grep -q 'FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT must be a numeric value' "$CORRUPT_OUT"; then
  corrupt_threshold_status="fail"
fi

trend_contract_status="pass"
if [ "$pass_status" != "pass" ] || [ "$unwaived_status" != "fail" ] || [ "$waived_status" != "pass" ]; then
  trend_contract_status="fail"
fi

ratchet_unwaived_status="pass"
if bash "$CHECK_SCRIPT" \
  --report-json "$PASS_REPORT" \
  --threshold-file "$FRESH_THRESHOLD_ENV" \
  --waiver-file "$WAIVER_JSON" \
  --ratchet-baseline-file "$RATCHET_REGRESSION_BASELINE_ENV" \
  --ratchet-exception-file "$tmp_dir/missing-ratchet-exception.json" >"$RATCHET_UNWAIVED_OUT" 2>&1; then
  ratchet_unwaived_status="pass"
elif grep -q '^reason_codes=fast_gate_delta_threshold_ratchet_regression_unwaived$' "$RATCHET_UNWAIVED_OUT"; then
  ratchet_unwaived_status="fail"
fi

ratchet_waived_status="fail"
if bash "$CHECK_SCRIPT" \
  --report-json "$PASS_REPORT" \
  --threshold-file "$FRESH_THRESHOLD_ENV" \
  --waiver-file "$WAIVER_JSON" \
  --ratchet-baseline-file "$RATCHET_REGRESSION_BASELINE_ENV" \
  --ratchet-exception-file "$RATCHET_EXCEPTION_JSON" >"$RATCHET_WAIVED_OUT"; then
  if grep -q '^reason_codes=fast_gate_delta_threshold_ratchet_exception_applied$' "$RATCHET_WAIVED_OUT"; then
    ratchet_waived_status="pass"
  fi
fi

if [ "$ratchet_unwaived_status" != "fail" ] || [ "$ratchet_waived_status" != "pass" ]; then
  trend_contract_status="fail"
fi

stale_threshold_guard_status="pass"
if [ "$stale_threshold_status" != "fail" ]; then
  stale_threshold_guard_status="fail"
fi

corrupt_threshold_guard_status="pass"
if [ "$corrupt_threshold_status" != "fail" ]; then
  corrupt_threshold_guard_status="fail"
fi

reason_code_contract_status="pass"
if ! grep -q '^reason_codes=delta_threshold_violation_unwaived$' "$UNWAIVED_OUT"; then
  reason_code_contract_status="fail"
fi
if ! grep -q '^reason_codes=delta_threshold_waiver_applied$' "$WAIVED_OUT"; then
  reason_code_contract_status="fail"
fi
if ! grep -q '^reason_codes=fast_gate_delta_threshold_ratchet_regression_unwaived$' "$RATCHET_UNWAIVED_OUT"; then
  reason_code_contract_status="fail"
fi
if ! grep -q '^reason_codes=fast_gate_delta_threshold_ratchet_exception_applied$' "$RATCHET_WAIVED_OUT"; then
  reason_code_contract_status="fail"
fi

status="pass"
reason_key="fast_gate_budget_delta_contract_ok"
if [ "$trend_contract_status" != "pass" ]; then
  status="fail"
  reason_key="fast_gate_budget_delta_contract_trend_status_mismatch"
fi
if [ "$status" = "pass" ] && [ "$stale_threshold_guard_status" != "pass" ]; then
  status="fail"
  reason_key="fast_gate_delta_threshold_file_stale"
fi
if [ "$status" = "pass" ] && [ "$corrupt_threshold_guard_status" != "pass" ]; then
  status="fail"
  reason_key="fast_gate_delta_threshold_file_corrupt"
fi
if [ "$status" = "pass" ] && [ "$reason_code_contract_status" != "pass" ]; then
  status="fail"
  reason_key="fast_gate_budget_delta_reason_code_contract_failed"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$status" = "pass" ] && [ "$elapsed_seconds" -gt "$MAX_RUNTIME_SECONDS" ]; then
  status="fail"
  reason_key="fast_gate_budget_delta_contract_runtime_budget_exceeded"
fi

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$OUTPUT_JSON" <<JSON
{
  "schema_version": "kamn.ci.fast-gate-budget-delta-contract-report.v1",
  "status": "$status",
  "trend_contract_status": "$trend_contract_status",
  "stale_threshold_guard_status": "$stale_threshold_guard_status",
  "corrupt_threshold_guard_status": "$corrupt_threshold_guard_status",
  "reason_code_contract_status": "$reason_code_contract_status",
  "pass_status": "$pass_status",
  "unwaived_status": "$unwaived_status",
  "waived_status": "$waived_status",
  "ratchet_unwaived_status": "$ratchet_unwaived_status",
  "ratchet_waived_status": "$ratchet_waived_status",
  "stale_threshold_status": "$stale_threshold_status",
  "corrupt_threshold_status": "$corrupt_threshold_status",
  "runtime_seconds": $elapsed_seconds,
  "max_runtime_seconds": $MAX_RUNTIME_SECONDS,
  "reason_key": "$reason_key"
}
JSON

echo "fast_gate_budget_delta_contract_status=$status"
echo "fast_gate_budget_delta_contract_pass_status=$pass_status"
echo "fast_gate_budget_delta_contract_unwaived_status=$unwaived_status"
echo "fast_gate_budget_delta_contract_waived_status=$waived_status"
echo "fast_gate_budget_delta_contract_ratchet_unwaived_status=$ratchet_unwaived_status"
echo "fast_gate_budget_delta_contract_ratchet_waived_status=$ratchet_waived_status"
echo "fast_gate_budget_delta_contract_stale_threshold_status=$stale_threshold_status"
echo "fast_gate_budget_delta_contract_corrupt_threshold_status=$corrupt_threshold_status"
echo "fast_gate_budget_delta_contract_report=$(realpath "$OUTPUT_JSON")"

if [ "$status" != "pass" ]; then
  echo "Fast-gate budget-delta contract lane failed: $reason_key" >&2
  exit 1
fi

echo "Fast-gate budget-delta contract lane tests passed."
