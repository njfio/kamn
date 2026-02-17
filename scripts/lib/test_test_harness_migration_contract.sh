#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS_PATH="$ROOT_DIR/scripts/lib/test_harness.sh"

if [ ! -f "$HARNESS_PATH" ]; then
  echo "expected shared shell test harness library: $HARNESS_PATH" >&2
  exit 1
fi

required_scripts=(
  "$ROOT_DIR/scripts/ci/test_wave_wrapper_family_baseline_contract_impl.sh"
  "$ROOT_DIR/scripts/ci/test_wave_wrapper_family_budget_trend_impl.sh"
  "$ROOT_DIR/scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh"
)

for script_path in "${required_scripts[@]}"; do
  if [ ! -f "$script_path" ]; then
    echo "expected migrated test script to exist: $script_path" >&2
    exit 1
  fi
  if ! grep -Fq "/scripts/lib/test_harness.sh" "$script_path"; then
    echo "expected migrated test script to source shared test harness: $script_path" >&2
    exit 1
  fi
done

echo "shared test harness migration contract tests passed."
