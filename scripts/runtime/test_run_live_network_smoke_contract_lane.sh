#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_live_network_smoke_contract_lane.sh"
SMOKE_SCRIPT="$ROOT_DIR/scripts/runtime/run_live_network_smoke_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected live-network smoke contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$SMOKE_SCRIPT" ]; then
  echo "expected live-network smoke runner script to be executable" >&2
  exit 1
fi

if ! grep -q "run_live_network_smoke_lane.sh" "$CONTRACT_LANE"; then
  echo "expected live-network smoke contract lane to execute smoke runner script" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "live-network smoke contract lane tests passed."; then
  echo "expected live-network smoke contract lane success marker" >&2
  exit 1
fi

echo "live-network smoke contract lane script tests passed."
