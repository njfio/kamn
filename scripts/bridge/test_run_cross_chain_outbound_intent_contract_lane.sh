#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_deep_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected outbound intent contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected outbound intent deep lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "cross-chain outbound intent contract lane tests passed."; then
  echo "expected outbound intent contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_cross_chain_outbound_intent_contract_lane.sh" "$DEEP_LANE"; then
  echo "expected deep lane script to invoke outbound intent contract lane baseline checks first" >&2
  exit 1
fi

if ! grep -q "bridge-outbound-intent-deep-report.json" "$DEEP_LANE"; then
  echo "expected deep lane script to emit outbound intent deep report artifact" >&2
  exit 1
fi

echo "cross-chain outbound intent contract lane script tests passed."
