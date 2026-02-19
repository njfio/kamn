#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"

POLICY_SCRIPT="$ROOT_DIR/scripts/ci/check_daemon_os_signal_stress_policy.py"
THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/daemon_os_signal_stress_policy_thresholds.env"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$POLICY_SCRIPT" "expected overload dry-run policy checker to be executable"
test_harness_require_file "$THRESHOLD_FILE" "expected overload dry-run threshold fixture to exist"

REPORT_FILE="$TMP_DIR/daemon-stress-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$REPORT_FILE" <<'JSON'
{
  "schema_version": "kamn.ci.daemon-os-signal-stress-matrix-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "reason_code": "stable_success",
  "runtime_seconds": 42
}
JSON

python3 "$POLICY_SCRIPT" \
  --report-file "$REPORT_FILE" \
  --threshold-file "$THRESHOLD_FILE" \
  --ci-tools-script "$ROOT_DIR/scripts/ci/test_ci_tools.sh" \
  --expected-final-decision GO \
  --output-json "$TMP_DIR/policy-pass.json" >"$TMP_DIR/pass.out"

grep -q '^status=pass$' "$TMP_DIR/pass.out"
grep -q '^final_decision=GO$' "$TMP_DIR/pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/pass.out"

RUNTIME_FAIL_REPORT="$TMP_DIR/daemon-stress-runtime-fail-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$RUNTIME_FAIL_REPORT" <<'JSON'
{
  "schema_version": "kamn.ci.daemon-os-signal-stress-matrix-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "reason_code": "stable_success",
  "runtime_seconds": 601
}
JSON

if python3 "$POLICY_SCRIPT" \
  --report-file "$RUNTIME_FAIL_REPORT" \
  --threshold-file "$THRESHOLD_FILE" \
  --ci-tools-script "$ROOT_DIR/scripts/ci/test_ci_tools.sh" \
  --expected-final-decision GO \
  --output-json "$TMP_DIR/policy-runtime-fail.json" >"$TMP_DIR/runtime-fail.out" 2>&1; then
  echo "expected overload dry-run checker to fail on runtime threshold exceedance" >&2
  exit 1
fi
grep -q '^reason_codes=overload_policy_runtime_budget_exceeded$' "$TMP_DIR/runtime-fail.out"

BAD_REASON_REPORT="$TMP_DIR/daemon-stress-bad-reason-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$BAD_REASON_REPORT" <<'JSON'
{
  "schema_version": "kamn.ci.daemon-os-signal-stress-matrix-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "reason_code": "unexpected_reason_code",
  "runtime_seconds": 42
}
JSON

if python3 "$POLICY_SCRIPT" \
  --report-file "$BAD_REASON_REPORT" \
  --threshold-file "$THRESHOLD_FILE" \
  --ci-tools-script "$ROOT_DIR/scripts/ci/test_ci_tools.sh" \
  --expected-final-decision GO \
  --output-json "$TMP_DIR/policy-reason-fail.json" >"$TMP_DIR/reason-fail.out" 2>&1; then
  echo "expected overload dry-run checker to fail on unknown reason code" >&2
  exit 1
fi
grep -q '^reason_codes=overload_policy_reason_code_unknown$' "$TMP_DIR/reason-fail.out"

FAKE_CI_TOOLS="$TMP_DIR/test_ci_tools_fast_mode_violation.sh"
cat >"$FAKE_CI_TOOLS" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [ "${KAMN_CI_TOOLS_FAST_MODE:-false}" = "true" ]; then
  bash "$ROOT_DIR/scripts/ci/test_run_daemon_os_signal_stress_matrix.sh"
  bash "$ROOT_DIR/scripts/ci/run_daemon_os_signal_stress_matrix.sh"
  exit 0
fi
EOF
chmod +x "$FAKE_CI_TOOLS"

if python3 "$POLICY_SCRIPT" \
  --report-file "$REPORT_FILE" \
  --threshold-file "$THRESHOLD_FILE" \
  --ci-tools-script "$FAKE_CI_TOOLS" \
  --expected-final-decision GO \
  --output-json "$TMP_DIR/policy-selector-fail.json" >"$TMP_DIR/selector-fail.out" 2>&1; then
  echo "expected overload dry-run checker to fail when heavy run leaks into fast mode" >&2
  exit 1
fi
grep -q '^reason_codes=overload_policy_ci_tools_fast_mode_heavy_run_leaked$' "$TMP_DIR/selector-fail.out"

echo "daemon os-signal overload dry-run policy checker tests passed."
