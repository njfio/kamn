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
max_seconds="${KAMN_RUNTIME_LIFECYCLE_PROPERTY_MAX_SECONDS:-360}"
target_dir="${KAMN_RUNTIME_LIFECYCLE_PROPERTY_TARGET_DIR:-$ROOT_DIR/target/contract-lanes/lifecycle-property}"

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

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

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

cargo_artifacts="$TMP_DIR/lifecycle-property-cargo-artifacts.json"
mkdir -p "$target_dir"
CARGO_TARGET_DIR="$target_dir" timeout "$max_seconds" cargo test -p kamn-core \
  --test task_state_machine \
  --test escrow_lifecycle \
  --test dispute_refund_transition_contracts \
  --test runtime_peer_lifecycle \
  --no-run \
  --message-format=json >"$cargo_artifacts"

target_executable() {
  local target_name="$1"
  python3 - "$cargo_artifacts" "$target_name" <<'PY'
import json
import pathlib
import sys

artifact_file = pathlib.Path(sys.argv[1])
target_name = sys.argv[2]
for line in artifact_file.read_text(encoding="utf-8").splitlines():
    if not line.strip().startswith("{"):
        continue
    try:
        artifact = json.loads(line)
    except json.JSONDecodeError:
        continue
    if artifact.get("reason") != "compiler-artifact":
        continue
    if artifact.get("target", {}).get("name") != target_name:
        continue
    executable = artifact.get("executable")
    if executable and pathlib.Path(executable).is_file():
        print(executable)
        raise SystemExit(0)
raise SystemExit(f"expected Cargo to report lifecycle property executable: {target_name}")
PY
}

start_epoch="$(date +%s)"

executed_tests=()
executed_cases=()
for property_case in "${property_cases[@]}"; do
  test_target="${property_case%%:*}"
  test_name="${property_case#*:}"
  test_executable="$(target_executable "$test_target")"
  if ! timeout "$max_seconds" "$test_executable" "$test_name" --exact >/dev/null; then
    echo "runtime lifecycle property contract test failed or timed out: $property_case" >&2
    exit 1
  fi
  executed_tests+=("$test_name")
  executed_cases+=("$test_target:$test_name")
done

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "runtime lifecycle property contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

if [[ -n "$output_json" ]]; then
  mkdir -p "$(dirname "$output_json")"
  tests_file="$TMP_DIR/lifecycle-property-tests.txt"
  cases_file="$TMP_DIR/lifecycle-property-cases.txt"
  printf '%s\n' "${executed_tests[@]}" >"$tests_file"
  printf '%s\n' "${executed_cases[@]}" >"$cases_file"

  python3 - "$output_json" "$tests_file" "$cases_file" "$elapsed_seconds" "$max_seconds" <<'PY'
import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1])
tests_path = pathlib.Path(sys.argv[2])
cases_path = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])

tests = [line.strip() for line in tests_path.read_text(encoding="utf-8").splitlines() if line.strip()]
cases = [line.strip() for line in cases_path.read_text(encoding="utf-8").splitlines() if line.strip()]
executed_cases = []
for case in cases:
    target, name = case.split(":", 1)
    executed_cases.append({"target": target, "name": name})

payload = {
    "schema_version": "kamn.runtime.lifecycle-property-contract-report.v1",
    "status": "pass",
    "suite": "lifecycle_property_contract_lane",
    "replay_schema_version": "kamn.runtime.lifecycle-property-replay-metadata.v1",
    "replay_artifact_key": "lifecycle_property_replay:v1",
    "executed_tests": tests,
    "executed_cases": executed_cases,
    "generated_sequence_bounds": {
        "task": {"alphabet_size": 8, "max_sequence_length": 4},
        "escrow": {"alphabet_size": 5, "max_sequence_length": 4},
        "peer": {"alphabet_size": 6, "max_sequence_length": 4},
    },
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "reason_codes": ["none"],
}
output_path.write_text(json.dumps(payload, separators=(",", ":")), encoding="utf-8")
PY

  echo "runtime_lifecycle_property_contract_report=$output_json"
fi

echo "runtime lifecycle property contract lane tests passed."
