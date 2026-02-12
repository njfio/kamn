#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEFAULT_THRESHOLD_FILE="$ROOT_DIR/.ci/kolme-wrapper-budget-trend-thresholds.json"

if [ ! -f "$DEFAULT_THRESHOLD_FILE" ]; then
  echo "expected trend threshold file to exist: $DEFAULT_THRESHOLD_FILE" >&2
  exit 1
fi

exec python3 "$ROOT_DIR/scripts/ci/kolme_wrapper_inventory_baseline.py" check \
  --trend-mode \
  --threshold-file "$DEFAULT_THRESHOLD_FILE" \
  "$@"
