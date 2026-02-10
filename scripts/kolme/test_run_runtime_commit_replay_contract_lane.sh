#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_runtime_commit_replay_contract_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected runtime commit replay contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "run_runtime_commit_replay_tamper_matrix.py" "$CONTRACT_LANE"; then
  echo "expected runtime commit replay contract lane to include replay matrix coverage" >&2
  exit 1
fi

if ! grep -q "run_runtime_commit_adapter_contract_lane.sh" "$CONTRACT_LANE"; then
  echo "expected runtime commit replay contract lane to include adapter replay/finality coverage" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "Kolme runtime commit replay contract lane tests passed."; then
  echo "expected runtime commit replay contract lane success marker" >&2
  exit 1
fi

echo "runtime commit replay contract lane script tests passed."
