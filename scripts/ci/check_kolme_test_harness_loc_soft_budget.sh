#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEFAULT_BUDGET_FILE="$ROOT_DIR/.ci/kolme-test-harness-loc-soft-budget.env"
DEFAULT_BASELINE_FILE="$ROOT_DIR/.ci/kolme-test-harness-loc-baseline.env"
DEFAULT_TREND_THRESHOLD_FILE="$ROOT_DIR/.ci/kolme-test-harness-loc-trend-thresholds.env"

exec bash "$ROOT_DIR/scripts/ci/check_test_harness_loc_soft_budget.sh" \
  --budget-file "$DEFAULT_BUDGET_FILE" \
  --baseline-file "$DEFAULT_BASELINE_FILE" \
  --trend-threshold-file "$DEFAULT_TREND_THRESHOLD_FILE" \
  "$@"
