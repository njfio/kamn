#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
RUNNER_PATH="$ROOT_DIR/scripts/ci/check_non_kolme_wave_wrapper_family_budget_trend_impl.sh"

test_harness_require_executable "$RUNNER_PATH" "expected shared non-Kolme wave budget trend checker runner: $RUNNER_PATH"

for wave in {1..19}; do
  checker_path="$ROOT_DIR/scripts/ci/check_non_kolme_wave${wave}_wrapper_family_budget_trend.sh"
  expected_target="check_non_kolme_wave_wrapper_family_budget_trend_impl.sh"
  threshold_fixture="$ROOT_DIR/fixtures/ci/non_kolme_wave${wave}_wrapper_family_trend_thresholds.json"

  if [ ! -L "$checker_path" ]; then
    echo "expected checker entrypoint to be a symlink to shared runner: $checker_path" >&2
    exit 1
  fi

  target_path="$(readlink "$checker_path")"
  if [ "$target_path" != "$expected_target" ]; then
    echo "expected $checker_path to target $expected_target but found $target_path" >&2
    exit 1
  fi

  if [ ! -f "$threshold_fixture" ]; then
    echo "expected non-Kolme wave threshold fixture: $threshold_fixture" >&2
    exit 1
  fi
done

echo "non-Kolme wave budget trend checker runner contract tests passed."
