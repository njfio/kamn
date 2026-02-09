#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/token/run_token_launch_handoff_contract_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected token launch handoff contract lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "token launch handoff contract lane tests passed."; then
  echo "expected token launch handoff contract lane success output" >&2
  exit 1
fi

echo "token launch handoff contract lane script tests passed."
