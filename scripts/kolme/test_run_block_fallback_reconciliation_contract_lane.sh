#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/kolme/run_block_fallback_reconciliation_contract_lane.sh"
RUNTIME_NETWORK_DOC="$ROOT_DIR/docs/foundation/runtime-network.md"
DEVNET_DOC="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
ROADMAP_DOC="$ROOT_DIR/docs/planning/kolme-integration-roadmap.md"

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected block fallback reconciliation contract lane script to be executable" >&2
  exit 1
fi

if [ ! -f "$RUNTIME_NETWORK_DOC" ] || [ ! -f "$DEVNET_DOC" ] || [ ! -f "$ROADMAP_DOC" ]; then
  echo "expected runtime/kolme documentation to exist" >&2
  exit 1
fi

if ! grep -q "run_block_fallback_reconciliation_contract_lane.sh" "$RUNTIME_NETWORK_DOC"; then
  echo "expected runtime network documentation to reference block fallback reconciliation lane command" >&2
  exit 1
fi

if ! grep -q "run_block_fallback_reconciliation_contract_lane.sh" "$DEVNET_DOC"; then
  echo "expected Kolme devnet ops documentation to reference block fallback reconciliation lane command" >&2
  exit 1
fi

if ! grep -q "run_block_fallback_reconciliation_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected Kolme integration roadmap documentation to reference block fallback reconciliation lane command" >&2
  exit 1
fi

if ! grep -q 'Regression: #1464' "$RUNTIME_NETWORK_DOC"; then
  echo "expected runtime network documentation to include block fallback regression marker" >&2
  exit 1
fi

if ! grep -q 'Regression: #1464' "$ROADMAP_DOC"; then
  echo "expected Kolme integration roadmap to include block fallback regression marker" >&2
  exit 1
fi

KAMN_KOLME_BLOCK_FALLBACK_MAX_SECONDS=75 bash "$LANE_SCRIPT" >/tmp/kolme-block-fallback-contract-lane.log

if ! grep -q "Kolme block fallback reconciliation contract lane tests passed." /tmp/kolme-block-fallback-contract-lane.log; then
  echo "expected block fallback reconciliation contract lane success output" >&2
  exit 1
fi

echo "block fallback reconciliation contract lane test passed."
