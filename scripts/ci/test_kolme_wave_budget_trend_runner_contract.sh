#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER_PATH="$ROOT_DIR/scripts/ci/check_kolme_wave_wrapper_family_budget_trend_impl.sh"

if [ ! -x "$RUNNER_PATH" ]; then
  echo "expected shared Kolme wave budget trend checker runner: $RUNNER_PATH" >&2
  exit 1
fi

for wave in 8 10 11; do
  checker_path="$ROOT_DIR/scripts/ci/check_kolme_wave${wave}_wrapper_family_budget_trend.sh"
  expected_target="check_kolme_wave_wrapper_family_budget_trend_impl.sh"
  threshold_fixture="$ROOT_DIR/fixtures/ci/kolme_wave${wave}_wrapper_family_trend_thresholds.json"

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
    echo "expected Kolme wave threshold fixture: $threshold_fixture" >&2
    exit 1
  fi
done

echo "Kolme wave budget trend checker runner contract tests passed."
