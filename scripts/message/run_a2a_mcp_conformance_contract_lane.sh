#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/message/run_a2a_mcp_conformance_harness.py"
POLICY_CHECKER="$ROOT_DIR/scripts/message/check_a2a_mcp_conformance_policy.sh"
FIXTURE="$ROOT_DIR/fixtures/a2a_mcp_conformance/replay_cases.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/message/run_a2a_mcp_conformance_contract_lane.sh \
    [--output-json <path>] \
    [--max-cases <n>] \
    [--skip-tests]
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

output_json=""
max_cases=""
skip_tests=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --max-cases)
      max_cases="${2:-}"
      shift 2
      ;;
    --skip-tests)
      skip_tests=true
      shift
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

if [ ! -x "$RUNNER" ]; then
  fail "A2A/MCP conformance harness runner is not executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "A2A/MCP conformance policy checker is not executable"
fi

if [ ! -f "$FIXTURE" ]; then
  fail "A2A/MCP conformance fixture file not found: $FIXTURE"
fi

if [[ -n "$max_cases" && ! "$max_cases" =~ ^[0-9]+$ ]]; then
  fail "--max-cases must be a positive integer"
fi

if [[ -n "$max_cases" && "$max_cases" -eq 0 ]]; then
  fail "--max-cases must be greater than zero"
fi

cd "$ROOT_DIR"

if [ "$skip_tests" != true ]; then
  cargo test -p kamn-core --test a2a_mcp_interop_docs >/dev/null
  cargo test -p kamn-core --test message_envelope_schema >/dev/null
fi

if [[ -z "$output_json" ]]; then
  output_json="$TMP_DIR/a2a-mcp-conformance-report.json"
fi

start_epoch="$(date +%s)"

runner_args=(--fixture "$FIXTURE" --output-json "$output_json")
if [[ -n "$max_cases" ]]; then
  runner_args+=(--max-cases "$max_cases")
fi

runner_output="$(python3 "$RUNNER" "${runner_args[@]}")"
if ! printf '%s\n' "$runner_output" | grep -q '^status=pass;'; then
  fail "expected A2A/MCP conformance harness status=pass"
fi

policy_output="$(bash "$POLICY_CHECKER" --report-file "$output_json")"
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  fail "expected A2A/MCP conformance policy checker final_decision=GO"
fi

if ! printf '%s\n' "$policy_output" | grep -q '^failed_cases=none$'; then
  fail "expected A2A/MCP conformance policy checker failed_cases=none"
fi

max_seconds="${A2A_MCP_CONFORMANCE_MAX_SECONDS:-90}"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  fail "A2A/MCP conformance contract lane exceeded runtime budget: ${elapsed_seconds}s"
fi

printf 'status=ok\n'
printf 'report_file=%s\n' "$output_json"
printf 'reason_key=%s\n' "$(extract_value "$runner_output" "reason_key")"
printf 'final_decision=%s\n' "$(extract_value "$policy_output" "final_decision")"
echo "A2A/MCP conformance contract lane tests passed."
