#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/runtime/run_lifecycle_property_contract_lane.sh \
    [--output-json <path>]
USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

output_json=""
max_seconds="${KAMN_RUNTIME_LIFECYCLE_PROPERTY_MAX_SECONDS:-120}"

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
  echo "KAMN_RUNTIME_LIFECYCLE_PROPERTY_MAX_SECONDS must be a positive integer" >&2
  exit 1
fi

start_epoch="$(date +%s)"

declare -a property_cases=(
  "task_state_machine:task_lifecycle_property_generated_sequences_preserve_transition_contracts"
  "task_state_machine:task_lifecycle_property_restore_roundtrip_preserves_state_and_history"
  "task_state_machine:task_lifecycle_property_terminal_states_are_absorbing"
  "escrow_lifecycle:escrow_property_generated_action_sequences_preserve_amount_and_status_invariants"
  "escrow_lifecycle:escrow_property_terminal_statuses_reject_all_mutating_actions"
  "dispute_refund_transition_contracts:functional_property_dispute_refund_sequences_preserve_contracts"
  "dispute_refund_transition_contracts:integration_dispute_refund_replay_traces_are_deterministic"
  "dispute_refund_transition_contracts:regression_replay_dispute_after_refund_fails_closed_with_reason_code"
  "dispute_refund_transition_contracts:performance_dispute_refund_property_contract_lane_stays_within_budget"
  "runtime_peer_lifecycle:peer_lifecycle_property_generated_event_sequences_match_transition_contract"
  "runtime_peer_lifecycle:peer_lifecycle_property_sequence_replay_is_deterministic"
  "runtime_peer_lifecycle:peer_lifecycle_property_roundtrip_disconnect_recovers_connection_path"
)

executed_tests=()
for property_case in "${property_cases[@]}"; do
  test_target="${property_case%%:*}"
  test_name="${property_case#*:}"
  cargo test -p kamn-core --test "$test_target" "$test_name" -- --exact >/dev/null
  executed_tests+=("$test_name")
done

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "runtime lifecycle property contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

if [[ -n "$output_json" ]]; then
  mkdir -p "$(dirname "$output_json")"
  tests_file="$(mktemp)"
  trap 'rm -f "$tests_file"' EXIT
  printf '%s\n' "${executed_tests[@]}" >"$tests_file"

  python3 - "$output_json" "$tests_file" "$elapsed_seconds" "$max_seconds" <<'PY'
import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1])
tests_path = pathlib.Path(sys.argv[2])
elapsed_seconds = int(sys.argv[3])
max_seconds = int(sys.argv[4])

tests = [line.strip() for line in tests_path.read_text(encoding="utf-8").splitlines() if line.strip()]
payload = {
    "schema_version": "kamn.runtime.lifecycle-property-contract-report.v1",
    "status": "pass",
    "suite": "lifecycle_property_contract_lane",
    "replay_artifact_key": "lifecycle_property_replay:v1",
    "executed_tests": tests,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "reason_codes": ["none"],
}
output_path.write_text(json.dumps(payload, separators=(",", ":")), encoding="utf-8")
PY

  echo "runtime_lifecycle_property_contract_report=$output_json"
fi

echo "runtime lifecycle property contract lane tests passed."
