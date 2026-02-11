#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_native_api_parity_live_proof_lane.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_native_api_parity_live_proof_policy.py"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_local_native_api_parity_live_proof_contract_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
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
  echo "expected local native API parity live proof lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/assert_local_heavy_opt_in.sh" "$RUNNER"; then
  echo "expected local native API parity runner to invoke shared local-heavy opt-in guard helper" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local native API parity live proof policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local native API parity live proof contract lane to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_native_api_parity_live_proof_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local native API parity live proof lane runner" >&2
  exit 1
fi

if ! grep -q "check_local_native_api_parity_live_proof_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local native API parity live proof policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_native_api_parity_live_proof_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local native API parity live proof contract lane" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected native parity dry-run to pass"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected native parity dry-run mode marker"

checker_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_output" "final_decision")" "GO" "expected checker GO decision for dry-run report"

set +e
bash "$RUNNER" \
  --mode run \
  --nonce-command "printf 'nonce_ok\n'" \
  --broadcast-command "printf 'broadcast_ok\n'" \
  --finality-command "printf 'finality_ok\n'" \
  --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without local opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected local opt-in failure message for native parity run mode" >&2
  exit 1
fi

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --nonce-command "sleep 5" \
    --broadcast-command "printf 'broadcast_ok\n'" \
    --finality-command "printf 'finality_ok\n'" \
    --max-seconds 3 \
    --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
timeout_code=$?
set -e

if [ "$timeout_code" -eq 0 ]; then
  echo "expected timeout path in native parity run mode to fail closed" >&2
  exit 1
fi

if ! grep -q "reason_code=nonce_command_timeout" "$TMP_ERR"; then
  echo "expected nonce timeout reason marker from native parity run mode" >&2
  exit 1
fi

checker_no_go_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision NO-GO \
    --ci-fast-gate PASS \
    --require-reason-code nonce_command_timeout \
    --output-json "$TMP_POLICY"
)"
assert_eq "$(extract_value "$checker_no_go_output" "status")" "ok" "expected checker to accept timeout report when NO-GO is expected"
assert_eq "$(extract_value "$checker_no_go_output" "final_decision")" "GO" "expected checker GO policy decision when timeout NO-GO report matches policy"

contract_output="$(
  bash "$CONTRACT_LANE" \
    --output-json "$TMP_REPORT" \
    --policy-output-json "$TMP_POLICY"
)"
if ! printf '%s\n' "$contract_output" | grep -q "local native API parity live proof contract lane tests passed."; then
  echo "expected local native API parity live proof contract lane success marker" >&2
  exit 1
fi

echo "local native API parity live proof contract lane tests passed."
