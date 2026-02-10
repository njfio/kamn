#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_fast_gate_native_api_parity_contract_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/kolme/check_fast_gate_native_api_parity_policy.py"
DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_ERR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$RUNNER" ]; then
  echo "expected fast-gate native API parity contract lane to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected fast-gate native API parity policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "run_fast_gate_native_api_parity_contract_lane.sh" "$DOC_FILE"; then
  echo "expected CI strategy doc to reference fast-gate native API parity contract lane command" >&2
  exit 1
fi

if ! grep -q "check_fast_gate_native_api_parity_policy.py" "$DOC_FILE"; then
  echo "expected CI strategy doc to reference fast-gate native API parity policy checker command" >&2
  exit 1
fi

if ! grep -q "KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS=120" "$DOC_FILE"; then
  echo "expected CI strategy doc to include fast-gate native parity runtime budget marker" >&2
  exit 1
fi

if ! grep -q "test_run_fast_gate_native_api_parity_contract_lane.sh" "$CI_TOOLS_SCRIPT"; then
  echo "expected ci-tools regression suite to execute fast-gate native API parity contract lane test" >&2
  exit 1
fi

runner_output="$(
  bash "$RUNNER" \
    --output-json "$TMP_REPORT"
)"
assert_eq "$(extract_value "$runner_output" "status")" "ok" "expected fast-gate native parity lane to pass"
assert_eq "$(extract_value "$runner_output" "reason_code")" "fast_gate_native_api_parity_passed" "expected fast-gate native parity pass reason marker"

checker_go_output="$(
  python3 "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --require-reason-code fast_gate_native_api_parity_passed \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_go_output" "status")" "ok" "expected policy checker to accept fast-gate native parity pass report"
assert_eq "$(extract_value "$checker_go_output" "final_decision")" "GO" "expected GO policy decision for fast-gate native parity pass report"

set +e
bash "$RUNNER" \
  --nonce-broadcast-command "false" \
  --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
runner_fail_code=$?
set -e
if [ "$runner_fail_code" -eq 0 ]; then
  echo "expected fast-gate native parity lane to fail closed for forced nonce/broadcast command failure" >&2
  exit 1
fi
if ! grep -q "reason_code=nonce_broadcast_contract_failed" "$TMP_ERR"; then
  echo "expected forced nonce/broadcast command failure reason marker from fast-gate native parity lane" >&2
  exit 1
fi

checker_no_go_output="$(
  python3 "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision NO-GO \
    --ci-fast-gate PASS \
    --require-reason-code nonce_broadcast_contract_failed \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_no_go_output" "status")" "ok" "expected policy checker to accept expected NO-GO fast-gate native parity report"
assert_eq "$(extract_value "$checker_no_go_output" "final_decision")" "GO" "expected GO policy decision when expected NO-GO fast-gate native parity report matches policy"

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if report.get("schema_version") != "kamn.kolme.fast-gate-native-api-parity-summary.v1":
    raise SystemExit("unexpected fast-gate native parity summary schema")
if report.get("status") != "fail":
    raise SystemExit("expected forced fast-gate native parity report status to be fail")
if report.get("reason_code") != "nonce_broadcast_contract_failed":
    raise SystemExit("expected forced fast-gate native parity report reason code")
PY

echo "fast-gate native API parity contract lane tests passed."
