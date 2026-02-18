#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CHECKER="$ROOT_DIR/scripts/ci/check_kamn_node_main_rs_extraction_threshold.sh"
THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/kamn_node_main_rs_extraction_thresholds.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECKER" "expected kamn-node main.rs extraction-threshold checker wrapper to be executable"

test_harness_require_file "$THRESHOLD_FILE" "expected kamn-node main.rs extraction-threshold fixture to exist"

PASS_REPORT="$TMP_DIR/pass-report.json"
bash "$CHECKER" \
  --threshold-file "$THRESHOLD_FILE" \
  --line-count-override 900 \
  --output-json "$PASS_REPORT" >"$TMP_DIR/pass.out"

grep -q '^status=pass$' "$TMP_DIR/pass.out"
grep -q '^policy_decision=GO$' "$TMP_DIR/pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/pass.out"
grep -q '^exception_status=not-required$' "$TMP_DIR/pass.out"

WARN_THRESHOLD_FILE="$TMP_DIR/warn-threshold.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$WARN_THRESHOLD_FILE" <<'JSON'
{
  "schema_version": "kamn.ci.kamn-node-main-rs-extraction-thresholds.v1",
  "warn_line_count": 1000,
  "fail_line_count": 1200
}
JSON

bash "$CHECKER" \
  --threshold-file "$WARN_THRESHOLD_FILE" \
  --line-count-override 1100 >"$TMP_DIR/warn.out"

grep -q '^status=warn$' "$TMP_DIR/warn.out"
grep -q '^policy_decision=WARN$' "$TMP_DIR/warn.out"
grep -q 'main_rs_line_count_warn_threshold_exceeded' "$TMP_DIR/warn.out"

set +e
bash "$CHECKER" \
  --threshold-file "$WARN_THRESHOLD_FILE" \
  --line-count-override 1300 >"$TMP_DIR/fail.out" 2>&1
fail_code=$?
set -e

if [ "$fail_code" -eq 0 ]; then
  echo "expected checker to fail when main.rs line count exceeds fail threshold without tracked exception" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail.out"
grep -q '^policy_decision=NO-GO$' "$TMP_DIR/fail.out"
grep -q 'main_rs_line_count_fail_threshold_exceeded' "$TMP_DIR/fail.out"

EXCEPTION_FILE="$TMP_DIR/exception.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$EXCEPTION_FILE" <<'JSON'
{
  "schema_version": "kamn.ci.kamn-node-main-rs-extraction-threshold-exception.v1",
  "reason": "temporary extraction bridge",
  "tracking_issue": "#3261",
  "expires_on": "2099-12-31",
  "max_line_count": 1500
}
JSON

bash "$CHECKER" \
  --threshold-file "$WARN_THRESHOLD_FILE" \
  --line-count-override 1300 \
  --exception-file "$EXCEPTION_FILE" >"$TMP_DIR/exception-pass.out"

grep -q '^status=warn$' "$TMP_DIR/exception-pass.out"
grep -q '^policy_decision=WARN$' "$TMP_DIR/exception-pass.out"
grep -q '^exception_status=applied$' "$TMP_DIR/exception-pass.out"
grep -q '^reason_codes=main_rs_threshold_exception_applied$' "$TMP_DIR/exception-pass.out"

EXPIRED_EXCEPTION_FILE="$TMP_DIR/exception-expired.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$EXPIRED_EXCEPTION_FILE" <<'JSON'
{
  "schema_version": "kamn.ci.kamn-node-main-rs-extraction-threshold-exception.v1",
  "reason": "expired exception",
  "tracking_issue": "#3261",
  "expires_on": "2000-01-01",
  "max_line_count": 1500
}
JSON

set +e
bash "$CHECKER" \
  --threshold-file "$WARN_THRESHOLD_FILE" \
  --line-count-override 1300 \
  --exception-file "$EXPIRED_EXCEPTION_FILE" >"$TMP_DIR/exception-expired.out" 2>&1
expired_code=$?
set -e

if [ "$expired_code" -eq 0 ]; then
  echo "expected checker to fail when tracked exception has expired" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/exception-expired.out"
grep -q 'main_rs_threshold_exception_expired' "$TMP_DIR/exception-expired.out"

INVALID_THRESHOLD_FILE="$TMP_DIR/invalid-threshold.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$INVALID_THRESHOLD_FILE" <<'JSON'
{
  "schema_version": "kamn.ci.kamn-node-main-rs-extraction-thresholds.v1",
  "warn_line_count": 1400,
  "fail_line_count": 1200
}
JSON

set +e
bash "$CHECKER" \
  --threshold-file "$INVALID_THRESHOLD_FILE" \
  --line-count-override 1000 >"$TMP_DIR/invalid-threshold.out" 2>&1
invalid_threshold_code=$?
set -e

if [ "$invalid_threshold_code" -eq 0 ]; then
  echo "expected checker to fail when threshold ordering is invalid" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/invalid-threshold.out"
grep -q 'threshold_order_invalid' "$TMP_DIR/invalid-threshold.out"

echo "kamn-node main.rs extraction-threshold checker tests passed."

