#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/sdk/run_live_transport_smoke_parity_contract_lane.sh \
    [--output-file <path>]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SMOKE_RUNNER="$ROOT_DIR/scripts/sdk/run_live_transport_smoke_parity_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/sdk/check_live_transport_smoke_parity_policy.sh"
RUST_SDK_DOC="$ROOT_DIR/docs/foundation/rust-sdk-alpha.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

output_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
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

if [ ! -x "$SMOKE_RUNNER" ]; then
  fail "expected sdk live transport smoke parity runner to be executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "expected sdk live transport smoke parity policy checker to be executable"
fi

if [ ! -f "$RUST_SDK_DOC" ]; then
  fail "expected rust sdk alpha doc to exist"
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/sdk-live-transport-smoke-go.json"
fi

max_contract_seconds="${KAMN_SDK_SMOKE_PARITY_CONTRACT_MAX_SECONDS:-240}"
if [[ ! "$max_contract_seconds" =~ ^[1-9][0-9]*$ ]]; then
  fail "KAMN_SDK_SMOKE_PARITY_CONTRACT_MAX_SECONDS must be a positive integer"
fi

start_epoch="$(date +%s)"

go_output="$(
  KAMN_SDK_SMOKE_PARITY_MAX_SECONDS="$max_contract_seconds" \
    bash "$SMOKE_RUNNER" --output-json "$output_file"
)"

if ! printf '%s\n' "$go_output" | grep -q '^status=pass$'; then
  fail "expected sdk live transport smoke parity lane to report pass status"
fi

if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  fail "expected sdk live transport smoke parity lane to report GO decision"
fi

go_policy_output="$(bash "$POLICY_CHECKER" --report-file "$output_file")"
if ! printf '%s\n' "$go_policy_output" | grep -q '^final_decision=GO$'; then
  fail "expected sdk live transport smoke parity policy check decision to be GO"
fi
if ! printf '%s\n' "$go_policy_output" | grep -q '^failed_checks=none$'; then
  fail "expected sdk live transport smoke parity GO policy check to have no failed checks"
fi

runtime_budget_report="$TMP_DIR/sdk-live-transport-smoke-runtime-budget-no-go.json"
set +e
runtime_budget_output="$(
  KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS=true \
  KAMN_SDK_SMOKE_PARITY_FAKE_DELAY_SECONDS=1 \
  KAMN_SDK_SMOKE_PARITY_MAX_SECONDS=0 \
  bash "$SMOKE_RUNNER" --output-json "$runtime_budget_report" 2>&1
)"
runtime_budget_code=$?
set -e

if [ "$runtime_budget_code" -eq 0 ]; then
  fail "expected runtime-budget failure run to fail closed"
fi

if ! printf '%s\n' "$runtime_budget_output" | grep -q 'runtime_budget_exceeded'; then
  fail "expected runtime-budget failure run to emit runtime_budget_exceeded"
fi

runtime_budget_policy_output="$(bash "$POLICY_CHECKER" --report-file "$runtime_budget_report")"
if ! printf '%s\n' "$runtime_budget_policy_output" | grep -q '^final_decision=NO-GO$'; then
  fail "expected runtime-budget policy check to return NO-GO"
fi
if ! printf '%s\n' "$runtime_budget_policy_output" | grep -q 'runtime_budget_exceeded'; then
  fail "expected runtime-budget policy check failed checks to include runtime_budget_exceeded"
fi

retry_budget_report="$TMP_DIR/sdk-live-transport-smoke-retry-budget-no-go.json"
set +e
retry_budget_output="$(
  KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS=true \
  KAMN_SDK_SMOKE_PARITY_FORCE_FAILURE=true \
  KAMN_SDK_SMOKE_PARITY_MAX_RETRIES=1 \
  bash "$SMOKE_RUNNER" --output-json "$retry_budget_report" 2>&1
)"
retry_budget_code=$?
set -e

if [ "$retry_budget_code" -eq 0 ]; then
  fail "expected retry-budget failure run to fail closed"
fi

if ! printf '%s\n' "$retry_budget_output" | grep -q 'retry_budget_exceeded'; then
  fail "expected retry-budget failure run to emit retry_budget_exceeded"
fi

retry_budget_policy_output="$(bash "$POLICY_CHECKER" --report-file "$retry_budget_report")"
if ! printf '%s\n' "$retry_budget_policy_output" | grep -q '^final_decision=NO-GO$'; then
  fail "expected retry-budget policy check to return NO-GO"
fi
if ! printf '%s\n' "$retry_budget_policy_output" | grep -q 'retry_budget_exceeded'; then
  fail "expected retry-budget policy check failed checks to include retry_budget_exceeded"
fi

if ! grep -q 'run_live_transport_smoke_parity_lane.sh' "$RUST_SDK_DOC"; then
  fail "expected rust sdk alpha doc to reference sdk smoke parity lane runner"
fi
if ! grep -q 'check_live_transport_smoke_parity_policy.sh' "$RUST_SDK_DOC"; then
  fail "expected rust sdk alpha doc to reference sdk smoke parity policy checker"
fi
if ! grep -q 'run_live_transport_smoke_parity_contract_lane.sh' "$RUST_SDK_DOC"; then
  fail "expected rust sdk alpha doc to reference sdk smoke parity contract lane runner"
fi
if ! grep -q 'kamn.sdk.live-transport-smoke-parity-report.v1' "$RUST_SDK_DOC"; then
  fail "expected rust sdk alpha doc to reference sdk smoke parity report schema marker"
fi
if ! grep -q 'KAMN_SDK_SMOKE_PARITY_MAX_SECONDS' "$RUST_SDK_DOC"; then
  fail "expected rust sdk alpha doc to reference sdk smoke parity runtime budget marker"
fi
if ! grep -q 'KAMN_SDK_SMOKE_PARITY_MAX_RETRIES' "$RUST_SDK_DOC"; then
  fail "expected rust sdk alpha doc to reference sdk smoke parity retry budget marker"
fi
if ! grep -q 'Regression: #938' "$RUST_SDK_DOC"; then
  fail "expected rust sdk alpha doc to include Regression: #938 marker"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_contract_seconds" ]; then
  fail "sdk live transport smoke parity contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'final_decision=GO\n'
echo "sdk live transport smoke parity contract lane tests passed."
