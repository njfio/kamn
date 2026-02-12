#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/kolme_wave10_wrapper_family_trend_thresholds.json"

if [ ! -f "$THRESHOLD_FILE" ]; then
  echo "expected wave-10 trend threshold file to exist: $THRESHOLD_FILE" >&2
  exit 1
fi

exec python3 "$ROOT_DIR/scripts/ci/kolme_wrapper_inventory_baseline.py" check \
  --trend-mode \
  --threshold-file "$THRESHOLD_FILE" \
  "$@"
