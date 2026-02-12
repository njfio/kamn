#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST_PATH="$ROOT_DIR/scripts/framework/manifests/kolme_signature_parity_contract_lane.json"

if [ "${1:-}" = "--resolve-manifest-path" ]; then
  echo "$MANIFEST_PATH"
  exit 0
fi

exec bash "$ROOT_DIR/scripts/framework/run_manifest_lane.sh" \
  --manifest "$MANIFEST_PATH" \
  --phase contract \
  -- \
  "$@"
