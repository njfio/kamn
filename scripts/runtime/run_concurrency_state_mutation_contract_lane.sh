#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/runtime/run_concurrency_state_mutation_contract_lane.sh \
    [--output-json <path>]
USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

output_json=""
max_seconds="${KAMN_RUNTIME_CONCURRENCY_MUTATION_MAX_SECONDS:-120}"

while [[ $# -gt 0 ]]; do
  case "$1" in
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
  echo "KAMN_RUNTIME_CONCURRENCY_MUTATION_MAX_SECONDS must be a positive integer" >&2
  exit 1
fi

declare -a concurrency_cases=(
  "unit_concurrency_replay_fixture_entries_are_valid"
  "task_accept_concurrency_has_single_winner_and_consistent_state"
  "task_submit_concurrency_rejects_duplicate_task_id_deterministically"
  "peer_lifecycle_concurrency_preserves_transition_contract_across_phases"
  "functional_task_accept_concurrency_replay_fixture_preserves_invariants"
  "integration_peer_lifecycle_concurrency_replay_is_deterministic_across_rounds"
  "functional_escrow_dispute_refund_concurrency_replay_fixture_preserves_terminal_snapshot"
  "integration_escrow_dispute_refund_concurrency_replay_is_deterministic_across_rounds"
  "regression_concurrency_accept_race_never_allows_multiple_winners"
  "regression_escrow_refund_race_never_allows_multiple_refund_winners"
  "performance_concurrency_state_mutation_contract_lane_stays_within_budget"
  "performance_escrow_dispute_refund_concurrency_lane_stays_within_budget"
)

start_epoch="$(date +%s)"

for case_name in "${concurrency_cases[@]}"; do
  cargo test -p kamn-core --test concurrency_state_mutation "$case_name" -- --exact >/dev/null
done

cargo test -p kamn-core --test runtime_network_docs doc_contains_concurrency_harness_contract_rules -- --exact >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "runtime concurrency state mutation lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

if [[ -n "$output_json" ]]; then
  mkdir -p "$(dirname "$output_json")"
  concurrency_cases_file="$(mktemp)"
  trap 'rm -f "$concurrency_cases_file"' EXIT
  printf '%s\n' "${concurrency_cases[@]}" >"$concurrency_cases_file"

  python3 - \
    "$output_json" \
    "$concurrency_cases_file" \
    "$elapsed_seconds" \
    "$max_seconds" <<'PY'
import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1])
cases_path = pathlib.Path(sys.argv[2])
elapsed_seconds = int(sys.argv[3])
max_seconds = int(sys.argv[4])

cases = [
    line.strip()
    for line in cases_path.read_text(encoding="utf-8").splitlines()
    if line.strip()
]

payload = {
    "schema_version": "kamn.runtime.concurrency-mutation-contract-report.v1",
    "status": "pass",
    "suite": "concurrency_state_mutation_contract_lane",
    "replay_artifact_key": "concurrency_mutation_replay:v1",
    "executed_tests": cases,
    "test_count": len(cases),
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "reason_codes": ["none"],
}
output_path.write_text(json.dumps(payload, separators=(",", ":")), encoding="utf-8")
PY

  echo "runtime_concurrency_mutation_contract_report=$output_json"
fi

echo "runtime concurrency state mutation contract lane tests passed."
