#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_live_network_pilot_deep_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_live_network_pilot_deep_lane.sh"

if [[ ! -x "$CONTRACT_LANE" ]]; then
  echo "expected live-network pilot deep contract lane script to be executable" >&2
  exit 1
fi

if [[ ! -x "$DEEP_LANE" ]]; then
  echo "expected live-network pilot deep lane script to be executable" >&2
  exit 1
fi

if ! grep -q "run_live_network_pilot_deep_lane.sh" "$CONTRACT_LANE"; then
  echo "expected live-network pilot deep contract lane to execute deep lane runner" >&2
  exit 1
fi

contract_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$contract_output" | grep -q "live-network pilot deep contract lane tests passed."; then
  echo "expected live-network pilot deep contract lane success marker" >&2
  exit 1
fi

echo "live-network pilot deep contract lane script tests passed."
