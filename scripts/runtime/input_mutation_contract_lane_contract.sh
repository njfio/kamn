#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/runtime/run_input_mutation_contract_lane.sh \
    [--target <all|envelope|did>] \
    [--output-json <path>]
USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ZK_WITNESS_MUTATION_LANE="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_contract_lane.sh"
ZK_WITNESS_MUTATION_DEEP_LANE="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_deep_lane.sh"
COVERAGE_GUIDED_LANE="$ROOT_DIR/scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh"
COVERAGE_GUIDED_DEEP_LANE="$ROOT_DIR/scripts/runtime/run_input_mutation_coverage_guided_deep_lane.sh"
cd "$ROOT_DIR"

for required_exec in "$ZK_WITNESS_MUTATION_LANE" "$ZK_WITNESS_MUTATION_DEEP_LANE" "$COVERAGE_GUIDED_LANE" "$COVERAGE_GUIDED_DEEP_LANE"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected executable script '$required_exec'" >&2
    exit 1
  fi
done

output_json=""
target_selector="all"
max_seconds="${KAMN_RUNTIME_INPUT_MUTATION_MAX_SECONDS:-120}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      target_selector="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ ! "$max_seconds" =~ ^[1-9][0-9]*$ ]]; then
  echo "KAMN_RUNTIME_INPUT_MUTATION_MAX_SECONDS must be a positive integer" >&2
  exit 1
fi

if [[ "$target_selector" != "all" && "$target_selector" != "envelope" && "$target_selector" != "did" ]]; then
  echo "--target must be one of: all, envelope, did" >&2
  exit 1
fi

declare -a envelope_cases=(
  "fuzz_smoke_envelope_mutation_lane_is_panic_free_and_deterministic"
  "functional_envelope_mutation_suite_covers_malformed_truncated_and_tampered_classes"
  "integration_envelope_mutation_fail_closed_reasons_are_explicit_and_deterministic"
  "regression_envelope_mutation_reason_signatures_remain_stable"
  "performance_envelope_mutation_contract_lane_stays_within_budget"
)

declare -a did_cases=(
  "fuzz_smoke_did_parse_mutations_are_panic_free_and_deterministic"
  "functional_did_mutation_suite_covers_normalization_encoding_and_method_mismatch_classes"
  "integration_did_mutation_fail_closed_reasons_are_explicit_and_deterministic"
  "regression_did_mutation_reason_signatures_remain_stable"
  "performance_did_mutation_contract_lane_stays_within_budget"
)

start_epoch="$(date +%s)"

executed_envelope_cases=()
executed_did_cases=()
coverage_guided_lane_mode="not-run"
coverage_guided_deep_mode="not-run"

if [[ "$target_selector" != "did" ]]; then
  for case_name in "${envelope_cases[@]}"; do
    cargo test -p kamn-core --test message_envelope_fuzz_smoke "$case_name" -- --exact >/dev/null
    executed_envelope_cases+=("$case_name")
  done
fi

if [[ "$target_selector" != "envelope" ]]; then
  for case_name in "${did_cases[@]}"; do
    cargo test -p kamn-core --test did_fuzz_smoke "$case_name" -- --exact >/dev/null
    executed_did_cases+=("$case_name")
  done
fi

zk_lane_mode="not-run"
if [[ "$target_selector" == "all" ]]; then
  zk_lane_mode="fast"
  if [ "${KAMN_RUNTIME_ZK_WITNESS_MUTATION_DEEP:-false}" = "true" ]; then
    zk_lane_mode="deep"
    bash "$ZK_WITNESS_MUTATION_DEEP_LANE" >/dev/null
  else
    bash "$ZK_WITNESS_MUTATION_LANE" >/dev/null
  fi

  cargo test -p kamn-core --test runtime_network_docs doc_contains_mutation_fail_closed_contract_rules -- --exact >/dev/null

  bash "$COVERAGE_GUIDED_LANE" >/dev/null
  coverage_guided_lane_mode="fast"
  if [ "${KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_LOCAL_ONLY:-false}" = "true" ]; then
    bash "$COVERAGE_GUIDED_DEEP_LANE" >/dev/null
    coverage_guided_deep_mode="deep"
  else
    coverage_guided_deep_mode="skipped_local_only"
    echo "runtime_input_mutation_coverage_guided_deep=skipped_local_only"
  fi
else
  :
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "runtime input mutation contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

if [[ -n "$output_json" ]]; then
  mkdir -p "$(dirname "$output_json")"
  envelope_cases_file="$(mktemp)"
  did_cases_file="$(mktemp)"
  trap 'rm -f "$envelope_cases_file" "$did_cases_file"' EXIT
  printf '%s\n' "${executed_envelope_cases[@]}" >"$envelope_cases_file"
  printf '%s\n' "${executed_did_cases[@]}" >"$did_cases_file"

  python3 - \
    "$output_json" \
    "$envelope_cases_file" \
    "$did_cases_file" \
    "$target_selector" \
    "$zk_lane_mode" \
    "$coverage_guided_lane_mode" \
    "$coverage_guided_deep_mode" \
    "$elapsed_seconds" \
    "$max_seconds" <<'PY'
import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1])
envelope_cases_path = pathlib.Path(sys.argv[2])
did_cases_path = pathlib.Path(sys.argv[3])
target = sys.argv[4]
zk_lane_mode = sys.argv[5]
coverage_guided_lane_mode = sys.argv[6]
coverage_guided_deep_mode = sys.argv[7]
elapsed_seconds = int(sys.argv[8])
max_seconds = int(sys.argv[9])

envelope_cases = [
    line.strip()
    for line in envelope_cases_path.read_text(encoding="utf-8").splitlines()
    if line.strip()
]
did_cases = [
    line.strip()
    for line in did_cases_path.read_text(encoding="utf-8").splitlines()
    if line.strip()
]

payload = {
    "schema_version": "kamn.runtime.input-mutation-contract-report.v1",
    "status": "pass",
    "suite": "input_mutation_contract_lane",
    "target": target,
    "replay_schema_version": "kamn.runtime.input-mutation-replay-metadata.v1",
    "replay_artifact_key": "input_mutation_replay:v1",
    "seed_corpus_keys": {
        "envelope": "input_mutation_envelope_seed:v1",
        "did": "input_mutation_did_seed:v1",
    },
    "envelope_tests": envelope_cases,
    "did_tests": did_cases,
    "envelope_test_count": len(envelope_cases),
    "did_test_count": len(did_cases),
    "zk_witness_lane_mode": zk_lane_mode,
    "coverage_guided_lane_mode": coverage_guided_lane_mode,
    "coverage_guided_deep_mode": coverage_guided_deep_mode,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "reason_codes": ["none"],
}
output_path.write_text(json.dumps(payload, separators=(",", ":")), encoding="utf-8")
PY

  echo "runtime_input_mutation_contract_report=$output_json"
fi

echo "runtime input mutation contract lane tests passed."
