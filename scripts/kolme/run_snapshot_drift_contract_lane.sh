#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_snapshot_drift_contract_lane.json"

exec bash "$ROOT_DIR/scripts/framework/run_manifest_lane.sh" \
  --manifest "$MANIFEST" \
  --phase contract \
  --cwd "$ROOT_DIR" \
  -- "$@"
