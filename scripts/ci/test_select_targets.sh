#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/select_targets.sh"

output="$(GITHUB_BASE_REF=__missing__ bash "$SCRIPT")"

run_rust="$(printf '%s\n' "$output" | awk -F= '/^run_rust=/{print $2}')"
if [ "$run_rust" != "true" ]; then
  echo "expected run_rust=true for current CI/workflow changes" >&2
  exit 1
fi

test_cmd="$(printf '%s\n' "$output" | awk -F= '/^test_cmd=/{sub(/^test_cmd=/,""); print}')"
if ! printf '%s\n' "$test_cmd" | grep -q "run_cargo_test_with_quarantine.sh"; then
  echo "expected select_targets test_cmd to use quarantine wrapper" >&2
  exit 1
fi

echo "select_targets tests passed."
