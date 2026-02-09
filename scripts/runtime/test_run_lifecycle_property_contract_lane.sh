#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_lifecycle_property_contract_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected runtime lifecycle property contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "task_lifecycle_property_generated_sequences_preserve_transition_contracts" "$CONTRACT_LANE"; then
  echo "expected lifecycle property contract lane to cover task lifecycle generated sequence invariants" >&2
  exit 1
fi

if ! grep -q "escrow_property_generated_action_sequences_preserve_amount_and_status_invariants" "$CONTRACT_LANE"; then
  echo "expected lifecycle property contract lane to cover escrow lifecycle invariants" >&2
  exit 1
fi

if ! grep -q "peer_lifecycle_property_generated_event_sequences_match_transition_contract" "$CONTRACT_LANE"; then
  echo "expected lifecycle property contract lane to cover peer lifecycle invariants" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "runtime lifecycle property contract lane tests passed."; then
  echo "expected runtime lifecycle property contract lane success marker" >&2
  exit 1
fi

echo "runtime lifecycle property contract lane script tests passed."
