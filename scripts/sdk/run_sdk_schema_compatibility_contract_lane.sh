#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_RUNNER="$ROOT_DIR/scripts/sdk/run_sdk_parity_matrix.sh"
GENERATOR="$ROOT_DIR/scripts/sdk/generate_sdk_schema_compatibility_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/sdk/check_sdk_schema_compatibility_policy.sh"
FIXTURE="$ROOT_DIR/fixtures/sdk_parity/register_validation_cases.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'EOF'
Usage:
  bash scripts/sdk/run_sdk_schema_compatibility_contract_lane.sh \
    [--output-file <path>] \
    [--lane contract|deep]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

output_file=""
lane="contract"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --lane)
      lane="${2:-}"
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

if [[ "$lane" != "contract" && "$lane" != "deep" ]]; then
  fail "--lane must be contract or deep"
fi

if [ ! -x "$MATRIX_RUNNER" ]; then
  fail "expected sdk parity matrix runner to be executable"
fi

if [ ! -x "$GENERATOR" ]; then
  fail "expected sdk schema compatibility evidence generator to be executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "expected sdk schema compatibility policy checker to be executable"
fi

if [ ! -f "$FIXTURE" ]; then
  fail "sdk parity fixture not found: $FIXTURE"
fi

if [[ -z "$output_file" ]]; then
  output_file="$TMP_DIR/sdk-schema-compatibility-contract.json"
fi

matrix_report="$TMP_DIR/sdk-parity-matrix-report.json"
start_epoch="$(date +%s)"

matrix_output="$(
  bash "$MATRIX_RUNNER" \
    --fixture "$FIXTURE" \
    --output-json "$matrix_report"
)"

if ! printf '%s\n' "$matrix_output" | grep -q "status=pass"; then
  fail "expected sdk parity matrix contract run to pass"
fi

generation_output="$(
  bash "$GENERATOR" \
    --output-file "$output_file" \
    --lane "$lane" \
    --matrix-report-file "$matrix_report" \
    --compatibility-suite-status pass \
    --runtime-budget-status within \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generation_output" | grep -q "^final_decision=GO$"; then
  fail "expected sdk schema compatibility bundle decision to be GO"
fi

if ! printf '%s\n' "$generation_output" | grep -q "^reason_key=sdk_schema_compatibility_reason_codes:GO:v1$"; then
  fail "expected sdk schema compatibility GO reason key"
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$output_file")"

if ! printf '%s\n' "$policy_output" | grep -q "^status=ok$"; then
  fail "expected sdk schema compatibility policy status marker"
fi

if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected sdk schema compatibility policy decision to be GO"
fi

if ! printf '%s\n' "$policy_output" | grep -q "^failed_checks=none$"; then
  fail "expected sdk schema compatibility policy failed checks to be none"
fi

max_seconds="${KAMN_SDK_SCHEMA_COMPATIBILITY_MAX_SECONDS:-60}"
if [[ ! "$max_seconds" =~ ^[1-9][0-9]*$ ]]; then
  fail "KAMN_SDK_SCHEMA_COMPATIBILITY_MAX_SECONDS must be a positive integer"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "sdk schema compatibility contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'lane=%s\n' "$lane"
printf 'bundle_file=%s\n' "$output_file"
printf 'matrix_report=%s\n' "$matrix_report"
printf 'final_decision=GO\n'
echo "sdk schema compatibility contract lane tests passed."
