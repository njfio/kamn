#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_outbound_quorum_contract_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected bridge outbound quorum contract lane script to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" --skip-intent-lane >"$TMP_OUT"
if ! grep -q "bridge outbound quorum contract lane tests passed." "$TMP_OUT"; then
  echo "expected bridge outbound quorum contract lane success marker" >&2
  exit 1
fi

if ! grep -q "run_cross_chain_outbound_intent_contract_lane.sh" "$FAST_SCRIPT"; then
  echo "expected outbound quorum contract lane to include cross-chain intent contract baseline" >&2
  exit 1
fi

if ! grep -q "bridge_outbound_quorum_execution" "$FAST_SCRIPT"; then
  echo "expected outbound quorum contract lane to execute bridge outbound quorum execution test binary" >&2
  exit 1
fi

echo "bridge outbound quorum contract lane script tests passed."
