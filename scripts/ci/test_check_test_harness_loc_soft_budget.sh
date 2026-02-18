#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
SCRIPT="$ROOT_DIR/scripts/ci/check_test_harness_loc_soft_budget.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$SCRIPT" "expected test harness LOC soft budget checker to be executable"

REPORT_FILE="$TMP_DIR/test-harness-loc-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$REPORT_FILE" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 5,
  "harness_shell_line_total": 100
}
EOF_REPORT

BUDGET_FILE="$TMP_DIR/soft-budget.env"
cat >"$BUDGET_FILE" <<'EOF_BUDGET'
TEST_HARNESS_SCRIPT_COUNT_SOFT_MAX=10
TEST_HARNESS_SHELL_LINE_TOTAL_SOFT_MAX=120
EOF_BUDGET

BASELINE_FILE="$TMP_DIR/baseline.env"
cat >"$BASELINE_FILE" <<'EOF_BASELINE'
TEST_HARNESS_SCRIPT_COUNT_BASELINE=3
TEST_HARNESS_SHELL_LINE_TOTAL_BASELINE=90
EOF_BASELINE

EXPECTED_REASON_TAXONOMY_VERSION='kamn.ci.test-harness-loc-soft-budget-reason-taxonomy.v1'
EXPECTED_REASON_CODES_CSV='report_file_not_found,budget_file_not_found,baseline_file_not_found,trend_threshold_file_not_found,report_json_invalid,report_schema_mismatch,report_harness_script_count_invalid,report_harness_shell_line_total_invalid,budget_key_missing,budget_value_invalid,baseline_key_missing,baseline_value_invalid,trend_threshold_key_missing,trend_threshold_value_invalid,trend_threshold_order_invalid,harness_script_count_soft_max_exceeded,harness_shell_line_total_soft_max_exceeded,harness_script_count_trend_warn_delta_exceeded,harness_shell_line_total_trend_warn_delta_exceeded,harness_script_count_trend_fail_delta_exceeded,harness_shell_line_total_trend_fail_delta_exceeded,trend_fail_enforcement_triggered'

POLICY_JSON="$TMP_DIR/policy-within.json"
within_output="$(
  bash "$SCRIPT" \
    --report-file "$REPORT_FILE" \
    --budget-file "$BUDGET_FILE" \
    --baseline-file "$BASELINE_FILE" \
    --output-json "$POLICY_JSON"
)"

if ! printf '%s\n' "$within_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for within-budget soft checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^soft_budget_status=within$'; then
  echo "expected soft_budget_status=within for within-budget soft checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^review_required=false$'; then
  echo "expected review_required=false for within-budget soft checker path" >&2
  exit 1
fi
if ! printf '%s\n' "$within_output" | grep -q '^reason_codes=none$'; then
  echo "expected reason_codes=none for within-budget soft checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q "^reason_taxonomy_version=${EXPECTED_REASON_TAXONOMY_VERSION}$"; then
  echo "expected deterministic reason taxonomy version marker for within-budget soft checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q "^reason_codes_csv=${EXPECTED_REASON_CODES_CSV}$"; then
  echo "expected deterministic reason taxonomy csv marker for within-budget soft checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected normalized reason_codes_value=none for within-budget soft checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^reason_class=stable$'; then
  echo "expected reason_class=stable for within-budget soft checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^delta_harness_script_count=2$'; then
  echo "expected deterministic script-count delta for within-budget soft checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^delta_harness_shell_line_total=10$'; then
  echo "expected deterministic shell-line delta for within-budget soft checker path" >&2
  exit 1
fi

BUDGET_FILE_EXCEEDED="$TMP_DIR/soft-budget-exceeded.env"
cat >"$BUDGET_FILE_EXCEEDED" <<'EOF_BUDGET'
TEST_HARNESS_SCRIPT_COUNT_SOFT_MAX=4
TEST_HARNESS_SHELL_LINE_TOTAL_SOFT_MAX=80
EOF_BUDGET

POLICY_EXCEEDED_JSON="$TMP_DIR/policy-exceeded.json"
exceeded_output="$(
  bash "$SCRIPT" \
    --report-file "$REPORT_FILE" \
    --budget-file "$BUDGET_FILE_EXCEEDED" \
    --baseline-file "$BASELINE_FILE" \
    --output-json "$POLICY_EXCEEDED_JSON"
)"

if ! printf '%s\n' "$exceeded_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for soft-budget exceed advisory path" >&2
  exit 1
fi

if ! printf '%s\n' "$exceeded_output" | grep -q '^soft_budget_status=exceeded$'; then
  echo "expected soft_budget_status=exceeded for soft-budget exceed advisory path" >&2
  exit 1
fi

if ! printf '%s\n' "$exceeded_output" | grep -q '^review_required=true$'; then
  echo "expected review_required=true for soft-budget exceed advisory path" >&2
  exit 1
fi

if ! printf '%s\n' "$exceeded_output" | grep -q '^exceeded_metrics=harness_script_count,harness_shell_line_total$'; then
  echo "expected exceeded metrics marker for soft-budget exceed advisory path" >&2
  exit 1
fi
if ! printf '%s\n' "$exceeded_output" | grep -q '^reason_codes=harness_script_count_soft_max_exceeded,harness_shell_line_total_soft_max_exceeded$'; then
  echo "expected deterministic reason_codes marker for soft-budget exceed advisory path" >&2
  exit 1
fi

if ! printf '%s\n' "$exceeded_output" | grep -q '^reason_codes_value=harness_script_count_soft_max_exceeded,harness_shell_line_total_soft_max_exceeded$'; then
  echo "expected deterministic normalized reason_codes_value marker for soft-budget exceed advisory path" >&2
  exit 1
fi

if ! printf '%s\n' "$exceeded_output" | grep -q '^reason_class=budgeted$'; then
  echo "expected reason_class=budgeted for soft-budget exceed advisory path" >&2
  exit 1
fi

BROKEN_REPORT="$TMP_DIR/broken-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$BROKEN_REPORT" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.unexpected.v1",
  "harness_script_count": 5,
  "harness_shell_line_total": 100
}
EOF_REPORT

set +e
broken_output="$(
  bash "$SCRIPT" \
    --report-file "$BROKEN_REPORT" \
    --budget-file "$BUDGET_FILE" \
    --baseline-file "$BASELINE_FILE" 2>&1
)"
broken_code=$?
set -e

if [ "$broken_code" -eq 0 ]; then
  echo "expected invalid report schema to fail checker contract" >&2
  exit 1
fi

if ! printf '%s\n' "$broken_output" | grep -q '^error=unexpected report schema:'; then
  echo "expected explicit schema error marker for invalid report path" >&2
  exit 1
fi
if ! printf '%s\n' "$broken_output" | grep -q '^reason_codes=report_schema_mismatch$'; then
  echo "expected deterministic reason_codes marker for invalid report schema path" >&2
  exit 1
fi

if ! printf '%s\n' "$broken_output" | grep -q '^reason_codes_value=report_schema_mismatch$'; then
  echo "expected normalized reason_codes_value marker for invalid report schema path" >&2
  exit 1
fi

if ! printf '%s\n' "$broken_output" | grep -q '^reason_class=violation$'; then
  echo "expected reason_class=violation marker for invalid report schema path" >&2
  exit 1
fi

echo "test harness LOC soft budget checker tests passed."
