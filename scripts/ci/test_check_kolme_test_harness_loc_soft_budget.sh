#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_SCRIPT="$ROOT_DIR/scripts/ci/generate_kolme_test_harness_loc_report.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_kolme_test_harness_loc_soft_budget.sh"
BUDGET_FILE="$ROOT_DIR/.ci/kolme-test-harness-loc-soft-budget.env"
BASELINE_FILE="$ROOT_DIR/.ci/kolme-test-harness-loc-baseline.env"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$REPORT_SCRIPT" ]; then
  echo "expected Kolme test harness LOC report script to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECK_SCRIPT" ]; then
  echo "expected Kolme test harness LOC soft budget checker to be executable" >&2
  exit 1
fi

if [ ! -f "$BUDGET_FILE" ]; then
  echo "expected Kolme test harness soft budget file to exist" >&2
  exit 1
fi

if [ ! -f "$BASELINE_FILE" ]; then
  echo "expected Kolme test harness baseline file to exist" >&2
  exit 1
fi

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

POLICY_WITHIN="$TMP_DIR/kolme-policy-within.json"
within_output="$(bash "$CHECK_SCRIPT" --report-file "$REPORT_FILE" --output-json "$POLICY_WITHIN")"

if ! printf '%s\n' "$within_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for Kolme within-budget checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^soft_budget_status=within$'; then
  echo "expected soft_budget_status=within for Kolme within-budget checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^review_required=false$'; then
  echo "expected review_required=false for Kolme within-budget checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$within_output" | grep -q '^reason_codes=none$'; then
  echo "expected reason_codes=none for Kolme within-budget checker path" >&2
  exit 1
fi

REPORT_EXCEEDED="$TMP_DIR/kolme-test-harness-report-exceeded.json"
cat >"$REPORT_EXCEEDED" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 500,
  "harness_shell_line_total": 50000
}
EOF_REPORT

POLICY_EXCEEDED="$TMP_DIR/kolme-policy-exceeded.json"
exceeded_output="$(bash "$CHECK_SCRIPT" --report-file "$REPORT_EXCEEDED" --output-json "$POLICY_EXCEEDED")"

if ! printf '%s\n' "$exceeded_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for Kolme soft-budget exceed advisory path" >&2
  exit 1
fi

if ! printf '%s\n' "$exceeded_output" | grep -q '^soft_budget_status=exceeded$'; then
  echo "expected soft_budget_status=exceeded for Kolme soft-budget exceed advisory path" >&2
  exit 1
fi

if ! printf '%s\n' "$exceeded_output" | grep -q '^review_required=true$'; then
  echo "expected review_required=true for Kolme soft-budget exceed advisory path" >&2
  exit 1
fi

if ! printf '%s\n' "$exceeded_output" | grep -q '^reason_codes=harness_script_count_soft_max_exceeded,harness_shell_line_total_soft_max_exceeded$'; then
  echo "expected deterministic reason_codes for Kolme soft-budget exceed advisory path" >&2
  exit 1
fi

BROKEN_REPORT="$TMP_DIR/kolme-broken-report.json"
cat >"$BROKEN_REPORT" <<'EOF_REPORT'
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
