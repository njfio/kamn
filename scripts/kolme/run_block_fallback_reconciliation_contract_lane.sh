#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNTIME_NETWORK_DOC="$ROOT_DIR/docs/foundation/runtime-network.md"
DEVNET_DOC="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
ROADMAP_DOC="$ROOT_DIR/docs/planning/kolme-integration-roadmap.md"
TEST_TARGET="kolme_runtime_commit_block_fallback"
MAX_SECONDS="${KAMN_KOLME_BLOCK_FALLBACK_MAX_SECONDS:-75}"

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$MAX_SECONDS" -le 0 ]; then
  echo "KAMN_KOLME_BLOCK_FALLBACK_MAX_SECONDS must be a positive integer" >&2
  exit 1
fi

if [ ! -f "$RUNTIME_NETWORK_DOC" ]; then
  echo "expected runtime network documentation to exist" >&2
  exit 1
fi

if [ ! -f "$DEVNET_DOC" ]; then
  echo "expected Kolme devnet ops documentation to exist" >&2
  exit 1
fi

if [ ! -f "$ROADMAP_DOC" ]; then
  echo "expected Kolme integration roadmap documentation to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"

cargo test -p kamn-core --test "$TEST_TARGET" >/dev/null

if ! grep -q "run_block_fallback_reconciliation_contract_lane.sh" "$RUNTIME_NETWORK_DOC"; then
  echo "expected runtime network documentation to reference block fallback reconciliation lane command" >&2
  exit 1
fi

if ! grep -q "run_block_fallback_reconciliation_contract_lane.sh" "$DEVNET_DOC"; then
  echo "expected Kolme devnet ops documentation to reference block fallback reconciliation lane command" >&2
  exit 1
fi

if ! grep -q "run_block_fallback_reconciliation_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected Kolme integration roadmap to reference block fallback reconciliation lane command" >&2
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

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  echo "Kolme block fallback reconciliation contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "Kolme block fallback reconciliation contract lane tests passed."
