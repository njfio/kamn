#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"

CHECKER="$ROOT_DIR/scripts/ci/check_shell_surface_threshold_ratchet.sh"
PY_CHECKER="$ROOT_DIR/scripts/ci/check_shell_surface_threshold_ratchet.py"
HARD_CEILING_FILE="$ROOT_DIR/.ci/shell-loc-hard-ceiling.env"
RATIO_THRESHOLD_FILE="$ROOT_DIR/.ci/shell-rust-ratio-guardrail.env"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECKER" "expected shell-surface threshold ratchet checker wrapper to be executable"
test_harness_require_executable "$PY_CHECKER" "expected shell-surface threshold ratchet python checker to be executable"
test_harness_require_file "$HARD_CEILING_FILE" "expected shell LOC hard-ceiling env fixture to exist"
test_harness_require_file "$RATIO_THRESHOLD_FILE" "expected shell-rust ratio guardrail env fixture to exist"

write_hard_env() {
  local output_file="$1"
  local hard_max="$2"
  cat >"$output_file" <<EOF
HARD_SHELL_LOC_MAX=$hard_max
EOF
}

write_ratio_env() {
  local output_file="$1"
  local warn_max="$2"
  local fail_max="$3"
  cat >"$output_file" <<EOF
WARN_SHELL_RUST_RATIO_MAX=$warn_max
FAIL_SHELL_RUST_RATIO_MAX=$fail_max
EOF
}

PASS_DEFAULT_REPORT="$TMP_DIR/pass-default-report.json"
bash "$CHECKER" \
  --repo-root "$ROOT_DIR" \
  --output-json "$PASS_DEFAULT_REPORT" >"$TMP_DIR/pass-default.out"
grep -q '^status=pass$' "$TMP_DIR/pass-default.out"
grep -q '^reason_codes=none$' "$TMP_DIR/pass-default.out"
grep -q '^threshold_ratchet_status=within$' "$TMP_DIR/pass-default.out"
grep -q '^threshold_ratchet_violations=none$' "$TMP_DIR/pass-default.out"
grep -q '^review_required=false$' "$TMP_DIR/pass-default.out"

BASELINE_HARD="$TMP_DIR/baseline-hard.env"
BASELINE_RATIO="$TMP_DIR/baseline-ratio.env"
CURRENT_HARD="$TMP_DIR/current-hard.env"
CURRENT_RATIO="$TMP_DIR/current-ratio.env"
write_hard_env "$BASELINE_HARD" "130000"
write_ratio_env "$BASELINE_RATIO" "0.95" "1.00"
write_hard_env "$CURRENT_HARD" "130000"
write_ratio_env "$CURRENT_RATIO" "0.95" "1.00"

PASS_FILE_REPORT="$TMP_DIR/pass-file-report.json"
bash "$CHECKER" \
  --repo-root "$ROOT_DIR" \
  --hard-ceiling-file "$CURRENT_HARD" \
  --ratio-threshold-file "$CURRENT_RATIO" \
  --baseline-hard-ceiling-file "$BASELINE_HARD" \
  --baseline-ratio-threshold-file "$BASELINE_RATIO" \
  --output-json "$PASS_FILE_REPORT" >"$TMP_DIR/pass-file.out"
grep -q '^status=pass$' "$TMP_DIR/pass-file.out"
grep -q '^reason_codes=none$' "$TMP_DIR/pass-file.out"

write_hard_env "$CURRENT_HARD" "130500"
FAIL_UNWAIVED_REPORT="$TMP_DIR/fail-unwaived-report.json"
if bash "$CHECKER" \
  --repo-root "$ROOT_DIR" \
  --hard-ceiling-file "$CURRENT_HARD" \
  --ratio-threshold-file "$CURRENT_RATIO" \
  --baseline-hard-ceiling-file "$BASELINE_HARD" \
  --baseline-ratio-threshold-file "$BASELINE_RATIO" \
  --ratchet-exception-file "$TMP_DIR/missing-exception.json" \
  --output-json "$FAIL_UNWAIVED_REPORT" >"$TMP_DIR/fail-unwaived.out" 2>&1; then
  echo "expected checker to fail for ratchet regression without exception" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/fail-unwaived.out"
grep -q '^reason_codes=shell_surface_threshold_ratchet_regression_unwaived$' "$TMP_DIR/fail-unwaived.out"
grep -q '^threshold_ratchet_status=regressed$' "$TMP_DIR/fail-unwaived.out"
grep -q '^threshold_ratchet_violations=HARD_SHELL_LOC_MAX$' "$TMP_DIR/fail-unwaived.out"

VALID_EXCEPTION="$TMP_DIR/valid-exception.json"
cat >"$VALID_EXCEPTION" <<'EOF'
{
  "schema_version": "kamn.ci.shell-surface-threshold-ratchet-exception.v1",
  "reason": "Temporary threshold exception while mitigation lands",
  "expires_on": "2099-12-31",
  "mitigation_issue": "#4859",
  "allow_threshold_keys": [
    "HARD_SHELL_LOC_MAX"
  ]
}
EOF

PASS_EXCEPTION_REPORT="$TMP_DIR/pass-exception-report.json"
bash "$CHECKER" \
  --repo-root "$ROOT_DIR" \
  --hard-ceiling-file "$CURRENT_HARD" \
  --ratio-threshold-file "$CURRENT_RATIO" \
  --baseline-hard-ceiling-file "$BASELINE_HARD" \
  --baseline-ratio-threshold-file "$BASELINE_RATIO" \
  --ratchet-exception-file "$VALID_EXCEPTION" \
  --output-json "$PASS_EXCEPTION_REPORT" >"$TMP_DIR/pass-exception.out"
grep -q '^status=pass$' "$TMP_DIR/pass-exception.out"
grep -q '^reason_codes=shell_surface_threshold_ratchet_exception_applied$' "$TMP_DIR/pass-exception.out"
grep -q '^threshold_ratchet_status=exception-applied$' "$TMP_DIR/pass-exception.out"
grep -q '^threshold_ratchet_mitigation_issue=#4859$' "$TMP_DIR/pass-exception.out"
grep -q '^review_required=true$' "$TMP_DIR/pass-exception.out"

INVALID_EXCEPTION="$TMP_DIR/invalid-exception.json"
cat >"$INVALID_EXCEPTION" <<'EOF'
{
  "schema_version": "kamn.ci.shell-surface-threshold-ratchet-exception.v1",
  "reason": "Invalid mitigation issue linkage",
  "expires_on": "2099-12-31",
  "mitigation_issue": "4859",
  "allow_threshold_keys": [
    "HARD_SHELL_LOC_MAX"
  ]
}
EOF

if bash "$CHECKER" \
  --repo-root "$ROOT_DIR" \
  --hard-ceiling-file "$CURRENT_HARD" \
  --ratio-threshold-file "$CURRENT_RATIO" \
  --baseline-hard-ceiling-file "$BASELINE_HARD" \
  --baseline-ratio-threshold-file "$BASELINE_RATIO" \
  --ratchet-exception-file "$INVALID_EXCEPTION" \
  --output-json "$TMP_DIR/invalid-exception-report.json" >"$TMP_DIR/invalid-exception.out" 2>&1; then
  echo "expected checker to fail when exception mitigation issue marker is invalid" >&2
  exit 1
fi
grep -q '^reason_codes=shell_surface_threshold_ratchet_exception_file_invalid$' "$TMP_DIR/invalid-exception.out"
grep -q 'exception mitigation_issue must be #<issue-id>' "$TMP_DIR/invalid-exception.out"

write_hard_env "$CURRENT_HARD" "130000"
write_ratio_env "$CURRENT_RATIO" "0.97" "1.02"
if bash "$CHECKER" \
  --repo-root "$ROOT_DIR" \
  --hard-ceiling-file "$CURRENT_HARD" \
  --ratio-threshold-file "$CURRENT_RATIO" \
  --baseline-hard-ceiling-file "$BASELINE_HARD" \
  --baseline-ratio-threshold-file "$BASELINE_RATIO" \
  --ratchet-exception-file "$TMP_DIR/missing-exception.json" \
  --output-json "$TMP_DIR/ratio-regression-report.json" >"$TMP_DIR/ratio-regression.out" 2>&1; then
  echo "expected checker to fail for warn/fail ratio ratchet regressions" >&2
  exit 1
fi
grep -q '^reason_codes=shell_surface_threshold_ratchet_regression_unwaived$' "$TMP_DIR/ratio-regression.out"
grep -q '^threshold_ratchet_violations=WARN_SHELL_RUST_RATIO_MAX,FAIL_SHELL_RUST_RATIO_MAX$' "$TMP_DIR/ratio-regression.out"

write_ratio_env "$CURRENT_RATIO" "1.20" "1.00"
if bash "$CHECKER" \
  --repo-root "$ROOT_DIR" \
  --hard-ceiling-file "$CURRENT_HARD" \
  --ratio-threshold-file "$CURRENT_RATIO" \
  --baseline-hard-ceiling-file "$BASELINE_HARD" \
  --baseline-ratio-threshold-file "$BASELINE_RATIO" \
  --output-json "$TMP_DIR/order-invalid-report.json" >"$TMP_DIR/order-invalid.out" 2>&1; then
  echo "expected checker to fail when warn threshold exceeds fail threshold" >&2
  exit 1
fi
grep -q '^reason_codes=shell_surface_threshold_ratchet_threshold_order_invalid$' "$TMP_DIR/order-invalid.out"

echo "shell-surface threshold ratchet checker tests passed."
