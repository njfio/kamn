#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/non_kolme_wave3_wrapper_family_trend_thresholds.json"

exec python3 "$ROOT_DIR/scripts/ci/kolme_wrapper_inventory_baseline.py" check \
  --trend-mode \
  --threshold-file "$THRESHOLD_FILE" \
  "$@"
