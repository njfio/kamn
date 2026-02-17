#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/non_kolme_wave19_wrapper_family_trend_thresholds.json"
MAX_RUNTIME_SECONDS="${KAMN_NON_KOLME_WAVE19_TREND_MAX_SECONDS:-45}"

exec python3 "$ROOT_DIR/scripts/ci/kolme_wrapper_inventory_baseline.py" check \
  --trend-mode \
  --threshold-file "$THRESHOLD_FILE" \
  --max-runtime-seconds "$MAX_RUNTIME_SECONDS" \
  "$@"
