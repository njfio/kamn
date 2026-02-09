#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/runtime/run_concurrency_state_mutation_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/runtime/run_concurrency_state_mutation_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected runtime concurrency state mutation contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected runtime concurrency state mutation deep lane script to be executable" >&2
  exit 1
fi

if ! grep -q "functional_task_accept_concurrency_replay_fixture_preserves_invariants" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include functional replay fixture coverage" >&2
  exit 1
fi

if ! grep -q "integration_peer_lifecycle_concurrency_replay_is_deterministic_across_rounds" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include integration replay determinism coverage" >&2
  exit 1
fi

if ! grep -q "regression_concurrency_accept_race_never_allows_multiple_winners" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include regression winner exclusivity coverage" >&2
  exit 1
fi

if ! grep -q "performance_concurrency_state_mutation_contract_lane_stays_within_budget" "$FAST_SCRIPT"; then
  echo "expected concurrency contract lane to include performance budget coverage" >&2
  exit 1
fi

lane_output="$(bash "$FAST_SCRIPT")"
if ! printf '%s\n' "$lane_output" | grep -q "runtime concurrency state mutation contract lane tests passed."; then
  echo "expected runtime concurrency state mutation contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_concurrency_state_mutation_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep lane to execute concurrency contract lane baseline first" >&2
  exit 1
fi

if ! grep -q "performance_concurrency_state_mutation_deep_lane_stress -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep lane to execute ignored concurrency stress test" >&2
  exit 1
fi

echo "runtime concurrency state mutation contract lane script tests passed."
