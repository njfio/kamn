#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_runtime_commit_adapter_contract_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected runtime commit adapter contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "functional_adapter_maps_transport_provider_and_finality_failures_to_typed_errors" "$CONTRACT_LANE"; then
  echo "expected runtime commit adapter lane to execute adapter failure mapping coverage" >&2
  exit 1
fi

if ! grep -q "check_runtime_commit_replay_policy.py" "$CONTRACT_LANE"; then
  echo "expected runtime commit adapter lane to execute replay/finality policy checks" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "Kolme runtime commit adapter contract lane tests passed."; then
  echo "expected runtime commit adapter contract lane success marker" >&2
  exit 1
fi

echo "runtime commit adapter contract lane script tests passed."
