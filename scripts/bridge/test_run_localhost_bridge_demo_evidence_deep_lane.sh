#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_SCRIPT="$ROOT_DIR/scripts/bridge/run_localhost_bridge_demo_evidence_deep_lane.sh"

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected localhost bridge demo evidence deep lane script to be executable" >&2
  exit 1
fi

if ! grep -Fq "run_localhost_bridge_demo_evidence_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected localhost bridge demo evidence deep lane to run contract lane baseline checks first" >&2
  exit 1
fi

if ! grep -Fq "bridge_adapter,telegram_bridge,discord_bridge,cross_chain_bridge" "$DEEP_SCRIPT"; then
  echo "expected localhost bridge demo evidence deep lane to run full bridge replay suite" >&2
  exit 1
fi

if ! grep -Fq -- "--output-json" "$DEEP_SCRIPT"; then
  echo "expected localhost bridge demo evidence deep lane to support output-json artifacts" >&2
  exit 1
fi

echo "localhost bridge demo evidence deep lane script tests passed."
