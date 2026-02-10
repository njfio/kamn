#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PROPERTY_LANE="$ROOT_DIR/scripts/runtime/run_lifecycle_property_contract_lane.sh"
FUZZ_LANE="$ROOT_DIR/scripts/runtime/run_input_mutation_contract_lane.sh"
CONCURRENCY_LANE="$ROOT_DIR/scripts/runtime/run_concurrency_state_mutation_contract_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_invariant_fuzz_concurrency_policy.sh"
RUNTIME_NETWORK_DOC="$ROOT_DIR/docs/foundation/runtime-network.md"
INVARIANTS_DOC="$ROOT_DIR/docs/foundation/invariants.md"
PERFORMANCE_DOC="$ROOT_DIR/docs/foundation/performance-target-benchmarking.md"
TESTING_STRATEGY_DOC="$ROOT_DIR/docs/testing/invariant-and-fuzz-strategy.md"

output_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

for required_exec in "$PROPERTY_LANE" "$FUZZ_LANE" "$CONCURRENCY_LANE" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected executable script '$required_exec'" >&2
    exit 1
  fi
done

for required_doc in "$RUNTIME_NETWORK_DOC" "$INVARIANTS_DOC" "$PERFORMANCE_DOC" "$TESTING_STRATEGY_DOC"; do
  if [ ! -f "$required_doc" ]; then
    echo "expected required documentation file '$required_doc'" >&2
    exit 1
  fi
done

max_seconds="${KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS:-180}"
if [[ ! "$max_seconds" =~ ^[1-9][0-9]*$ ]]; then
  echo "contract max seconds must be a positive integer" >&2
  exit 1
fi

start_epoch="$(date +%s)"

property_output="$(bash "$PROPERTY_LANE")"
if ! printf '%s\n' "$property_output" | grep -Fq "runtime lifecycle property contract lane tests passed."; then
  echo "expected lifecycle property lane success marker" >&2
  exit 1
fi

fuzz_output="$(bash "$FUZZ_LANE")"
if ! printf '%s\n' "$fuzz_output" | grep -Fq "runtime input mutation contract lane tests passed."; then
  echo "expected input mutation lane success marker" >&2
  exit 1
fi

concurrency_output="$(bash "$CONCURRENCY_LANE")"
if ! printf '%s\n' "$concurrency_output" | grep -Fq "runtime concurrency state mutation contract lane tests passed."; then
  echo "expected concurrency state mutation lane success marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "runtime invariant/fuzz/concurrency contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
summary_report="$TMP_DIR/invariant-fuzz-concurrency-contract-report.json"

python3 - "$summary_report" "$elapsed_seconds" "$max_seconds" <<'PY'
import json
import pathlib
import sys

report_file = pathlib.Path(sys.argv[1])
elapsed_seconds = int(sys.argv[2])
max_seconds = int(sys.argv[3])

summary = {
    "schema_version": "kamn.runtime.invariant-fuzz-concurrency-contract-report.v1",
    "status": "pass",
    "property_lane_status": "pass",
    "fuzz_lane_status": "pass",
    "concurrency_lane_status": "pass",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "reason_codes": ["none"],
}
report_file.write_text(json.dumps(summary, separators=(",", ":")), encoding="utf-8")
PY

policy_output="$(bash "$POLICY_CHECKER" --report-file "$summary_report")"
if ! printf '%s\n' "$policy_output" | grep -Fq "status=ok"; then
  echo "expected invariant/fuzz/concurrency policy checker success marker" >&2
  exit 1
fi

if ! grep -Fq "run_invariant_fuzz_concurrency_contract_lane.sh" "$RUNTIME_NETWORK_DOC"; then
  echo "expected runtime-network docs to reference invariant/fuzz/concurrency lane command" >&2
  exit 1
fi

if ! grep -Fq "check_invariant_fuzz_concurrency_policy.sh" "$RUNTIME_NETWORK_DOC"; then
  echo "expected runtime-network docs to reference invariant/fuzz/concurrency policy checker command" >&2
  exit 1
fi

if ! grep -Fq "kamn.runtime.invariant-fuzz-concurrency-contract-report.v1" "$RUNTIME_NETWORK_DOC"; then
  echo "expected runtime-network docs to reference invariant/fuzz/concurrency report schema" >&2
  exit 1
fi

if ! grep -Fq "run_invariant_fuzz_concurrency_contract_lane.sh" "$INVARIANTS_DOC"; then
  echo "expected invariants docs to reference invariant/fuzz/concurrency lane command" >&2
  exit 1
fi

if ! grep -Fq "KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS" "$PERFORMANCE_DOC"; then
  echo "expected performance benchmarking docs to reference invariant/fuzz/concurrency runtime budget env" >&2
  exit 1
fi

if ! grep -Fq "run_lifecycle_property_contract_lane.sh" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference lifecycle property lane command" >&2
  exit 1
fi

if ! grep -Fq "run_input_mutation_contract_lane.sh" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference input mutation lane command" >&2
  exit 1
fi

if ! grep -Fq "run_concurrency_state_mutation_contract_lane.sh" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference concurrency state mutation lane command" >&2
  exit 1
fi

if ! grep -Fq "run_invariant_fuzz_concurrency_contract_lane.sh" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference combined contract lane command" >&2
  exit 1
fi

if ! grep -Fq "check_invariant_fuzz_concurrency_policy.sh" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference combined policy checker command" >&2
  exit 1
fi

if ! grep -Fq "kamn.runtime.invariant-fuzz-concurrency-contract-report.v1" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference combined report schema" >&2
  exit 1
fi

if ! grep -Fq "KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference combined lane runtime budget env" >&2
  exit 1
fi

if [[ -n "$output_json" ]]; then
  cp "$summary_report" "$output_json"
fi

echo "runtime_invariant_fuzz_concurrency_property=pass"
echo "runtime_invariant_fuzz_concurrency_fuzz=pass"
echo "runtime_invariant_fuzz_concurrency_concurrency=pass"
echo "runtime_invariant_fuzz_concurrency_policy=ok"
echo "runtime invariant/fuzz/concurrency contract lane tests passed."
