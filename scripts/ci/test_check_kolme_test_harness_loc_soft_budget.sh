#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
REPORT_SCRIPT="$ROOT_DIR/scripts/ci/generate_kolme_test_harness_loc_report.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_kolme_test_harness_loc_soft_budget.sh"
BUDGET_FILE="$ROOT_DIR/.ci/kolme-test-harness-loc-soft-budget.env"
BASELINE_FILE="$ROOT_DIR/.ci/kolme-test-harness-loc-baseline.env"
TREND_THRESHOLD_FILE="$ROOT_DIR/.ci/kolme-test-harness-loc-trend-thresholds.env"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$REPORT_SCRIPT" "expected Kolme test harness LOC report script to be executable"

test_harness_require_executable "$CHECK_SCRIPT" "expected Kolme test harness LOC soft budget checker to be executable"

test_harness_require_file "$BUDGET_FILE" "expected Kolme test harness soft budget file to exist"

test_harness_require_file "$BASELINE_FILE" "expected Kolme test harness baseline file to exist"

test_harness_require_file "$TREND_THRESHOLD_FILE" "expected Kolme test harness trend threshold file to exist"

REPORT_FILE="$TMP_DIR/kolme-test-harness-report.json"
report_output="$(bash "$REPORT_SCRIPT" --output-json "$REPORT_FILE")"

if ! printf '%s\n' "$report_output" | grep -q '^status=ok$'; then
  echo "expected status=ok from Kolme harness report generator" >&2
  exit 1
fi

if ! printf '%s\n' "$report_output" | grep -q '^harness_script_count='; then
  echo "expected harness_script_count marker from Kolme harness report generator" >&2
  exit 1
fi

if ! printf '%s\n' "$report_output" | grep -q '^harness_shell_line_total='; then
  echo "expected harness_shell_line_total marker from Kolme harness report generator" >&2
  exit 1
fi

POLICY_LIVE="$TMP_DIR/kolme-policy-live.json"
live_output="$(bash "$CHECK_SCRIPT" --report-file "$REPORT_FILE" --output-json "$POLICY_LIVE")"

if ! printf '%s\n' "$live_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for live Kolme policy checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$live_output" | grep -q '^soft_budget_status='; then
  echo "expected soft_budget_status marker for live Kolme policy checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$live_output" | grep -q '^trend_status='; then
  echo "expected trend_status marker for live Kolme policy checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$live_output" | grep -q '^policy_decision='; then
  echo "expected policy_decision marker for live Kolme policy checker path" >&2
  exit 1
fi

REPORT_WITHIN="$TMP_DIR/kolme-test-harness-report-within.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$REPORT_WITHIN" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 62,
  "harness_shell_line_total": 8691
}
EOF_REPORT

POLICY_WITHIN="$TMP_DIR/kolme-policy-within.json"
within_output="$(bash "$CHECK_SCRIPT" --report-file "$REPORT_WITHIN" --output-json "$POLICY_WITHIN")"

if ! printf '%s\n' "$within_output" | grep -q '^soft_budget_status=within$'; then
  echo "expected soft_budget_status=within for deterministic within-threshold path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^trend_status=within$'; then
  echo "expected trend_status=within for deterministic within-threshold path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^policy_decision=GO$'; then
  echo "expected policy_decision=GO for deterministic within-threshold path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^review_required=false$'; then
  echo "expected review_required=false for deterministic within-threshold path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^reason_codes=none$'; then
  echo "expected reason_codes=none for deterministic within-threshold path" >&2
  exit 1
fi

REPORT_WARN="$TMP_DIR/kolme-test-harness-report-warn.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$REPORT_WARN" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 70,
  "harness_shell_line_total": 10350
}
EOF_REPORT

POLICY_WARN="$TMP_DIR/kolme-policy-warn.json"
warn_output="$(bash "$CHECK_SCRIPT" --report-file "$REPORT_WARN" --output-json "$POLICY_WARN")"

if ! printf '%s\n' "$warn_output" | grep -q '^soft_budget_status=exceeded$'; then
  echo "expected soft_budget_status=exceeded for deterministic warn-threshold path" >&2
  exit 1
fi

if ! printf '%s\n' "$warn_output" | grep -q '^trend_status=warn$'; then
  echo "expected trend_status=warn for deterministic warn-threshold path" >&2
  exit 1
fi

if ! printf '%s\n' "$warn_output" | grep -q '^policy_decision=WARN$'; then
  echo "expected policy_decision=WARN for deterministic warn-threshold path" >&2
  exit 1
fi

if ! printf '%s\n' "$warn_output" | grep -q '^review_required=true$'; then
  echo "expected review_required=true for deterministic warn-threshold path" >&2
  exit 1
fi

if ! printf '%s\n' "$warn_output" | grep -q 'harness_shell_line_total_soft_max_exceeded'; then
  echo "expected warn path reason_codes to include soft-budget exceed marker" >&2
  exit 1
fi

if ! printf '%s\n' "$warn_output" | grep -q 'harness_shell_line_total_trend_warn_delta_exceeded'; then
  echo "expected warn path reason_codes to include trend warn marker" >&2
  exit 1
fi

REPORT_FAIL="$TMP_DIR/kolme-test-harness-report-fail.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$REPORT_FAIL" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 90,
  "harness_shell_line_total": 13000
}
EOF_REPORT

POLICY_FAIL="$TMP_DIR/kolme-policy-fail.json"
fail_output="$(bash "$CHECK_SCRIPT" --report-file "$REPORT_FAIL" --output-json "$POLICY_FAIL")"

if ! printf '%s\n' "$fail_output" | grep -q '^trend_status=fail$'; then
  echo "expected trend_status=fail for deterministic fail-threshold path" >&2
  exit 1
fi

if ! printf '%s\n' "$fail_output" | grep -q '^policy_decision=NO-GO$'; then
  echo "expected policy_decision=NO-GO for deterministic fail-threshold path" >&2
  exit 1
fi

if ! printf '%s\n' "$fail_output" | grep -q 'harness_script_count_trend_fail_delta_exceeded'; then
  echo "expected fail path reason_codes to include script-count fail trend marker" >&2
  exit 1
fi

set +e
enforced_fail_output="$(bash "$CHECK_SCRIPT" --report-file "$REPORT_FAIL" --enforce-trend-fail 2>&1)"
enforced_fail_code=$?
set -e

if [ "$enforced_fail_code" -eq 0 ]; then
  echo "expected enforce-trend-fail path to return non-zero for fail trend status" >&2
  exit 1
fi

if ! printf '%s\n' "$enforced_fail_output" | grep -q '^status=fail$'; then
  echo "expected status=fail marker for enforce-trend-fail path" >&2
  exit 1
fi

if ! printf '%s\n' "$enforced_fail_output" | grep -q '^trend_status=fail$'; then
  echo "expected trend_status=fail marker for enforce-trend-fail path" >&2
  exit 1
fi

BROKEN_REPORT="$TMP_DIR/kolme-broken-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$BROKEN_REPORT" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.unexpected.v1",
  "harness_script_count": 1,
  "harness_shell_line_total": 1
}
EOF_REPORT

set +e
broken_output="$(bash "$CHECK_SCRIPT" --report-file "$BROKEN_REPORT" 2>&1)"
broken_code=$?
set -e

if [ "$broken_code" -eq 0 ]; then
  echo "expected Kolme checker to fail closed for invalid schema" >&2
  exit 1
fi

if ! printf '%s\n' "$broken_output" | grep -q '^reason_codes=report_schema_mismatch$'; then
  echo "expected deterministic reason code for invalid Kolme report schema" >&2
  exit 1
fi

echo "Kolme test harness LOC soft budget checker tests passed."
