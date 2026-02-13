#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh \
    [--target <all|envelope|did>] \
    [--output-json <path>]
USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

target_selector="all"
output_json=""
max_seconds="${KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_MAX_SECONDS:-120}"

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
  echo "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_MAX_SECONDS must be a positive integer" >&2
  exit 1
fi

if [[ "$target_selector" != "all" && "$target_selector" != "envelope" && "$target_selector" != "did" ]]; then
  echo "--target must be one of: all, envelope, did" >&2
  exit 1
fi

declare -a envelope_cases=(
  "unit_input_mutation_coverage_guided_envelope_seed_corpus_covers_boundary_classes"
)

declare -a did_cases=(
  "unit_input_mutation_coverage_guided_did_seed_corpus_covers_boundary_classes"
)

declare -a shared_cases=(
  "functional_input_mutation_coverage_guided_targets_are_deterministic"
  "integration_input_mutation_coverage_guided_reason_taxonomy_stable"
  "regression_input_mutation_coverage_guided_seed_corpus_contains_known_malformed_classes"
  "performance_input_mutation_coverage_guided_contract_lane_stays_within_budget"
)

start_epoch="$(date +%s)"
executed_envelope_cases=()
executed_did_cases=()
executed_shared_cases=()

if [[ "$target_selector" != "did" ]]; then
  for case_name in "${envelope_cases[@]}"; do
    cargo test -p kamn-core --test input_mutation_coverage_guided "$case_name" -- --exact >/dev/null
    executed_envelope_cases+=("$case_name")
  done
fi

if [[ "$target_selector" != "envelope" ]]; then
  for case_name in "${did_cases[@]}"; do
    cargo test -p kamn-core --test input_mutation_coverage_guided "$case_name" -- --exact >/dev/null
    executed_did_cases+=("$case_name")
  done
fi

if [[ "$target_selector" == "all" ]]; then
  for case_name in "${shared_cases[@]}"; do
    cargo test -p kamn-core --test input_mutation_coverage_guided "$case_name" -- --exact >/dev/null
    executed_shared_cases+=("$case_name")
  done
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "runtime input mutation coverage-guided lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

if [[ -n "$output_json" ]]; then
  mkdir -p "$(dirname "$output_json")"
  envelope_cases_file="$(mktemp)"
  did_cases_file="$(mktemp)"
  shared_cases_file="$(mktemp)"
  trap 'rm -f "$envelope_cases_file" "$did_cases_file" "$shared_cases_file"' EXIT
  printf '%s\n' "${executed_envelope_cases[@]}" >"$envelope_cases_file"
  printf '%s\n' "${executed_did_cases[@]}" >"$did_cases_file"
  printf '%s\n' "${executed_shared_cases[@]}" >"$shared_cases_file"

  python3 - \
    "$output_json" \
    "$envelope_cases_file" \
    "$did_cases_file" \
    "$shared_cases_file" \
    "$target_selector" \
    "$elapsed_seconds" \
    "$max_seconds" <<'PY'
import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1])
envelope_cases_path = pathlib.Path(sys.argv[2])
did_cases_path = pathlib.Path(sys.argv[3])
shared_cases_path = pathlib.Path(sys.argv[4])
target = sys.argv[5]
elapsed_seconds = int(sys.argv[6])
max_seconds = int(sys.argv[7])

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
shared_cases = [
    line.strip()
    for line in shared_cases_path.read_text(encoding="utf-8").splitlines()
    if line.strip()
]

payload = {
    "schema_version": "kamn.runtime.input-mutation-coverage-guided-contract-report.v1",
    "status": "pass",
    "suite": "input_mutation_coverage_guided_contract_lane",
    "target": target,
    "replay_schema_version": "kamn.runtime.input-mutation-coverage-guided-replay-metadata.v1",
    "replay_artifact_key": "input_mutation_coverage_guided_replay:v1",
    "seed_corpus_keys": {
        "envelope": "input_mutation_coverage_guided_envelope_seed:v1",
        "did": "input_mutation_coverage_guided_did_seed:v1",
    },
    "minimizer": "minimal_failing_seed_prefix",
    "envelope_tests": envelope_cases,
    "did_tests": did_cases,
    "shared_tests": shared_cases,
    "envelope_test_count": len(envelope_cases),
    "did_test_count": len(did_cases),
    "shared_test_count": len(shared_cases),
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "reason_codes": ["none"],
}
output_path.write_text(json.dumps(payload, separators=(",", ":")), encoding="utf-8")
PY

  echo "runtime_input_mutation_coverage_guided_contract_report=$output_json"
fi

echo "runtime input mutation coverage-guided contract lane tests passed."
