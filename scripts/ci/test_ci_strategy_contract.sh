#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"

if [ ! -f "$STRATEGY_DOC" ]; then
  echo "CI strategy contract failed: docs/ci/strategy.md is missing." >&2
  exit 1
fi

required_snippets=(
  "make check"
  "make test"
  "make demo"
  "run_localhost_signed_integration_contract_lane_tests"
  "sdk-live-localhost-integration"
  "run_localhost_signed_integration_contract_lane.sh"
  "scripts/ci/select_targets.sh"
  "Regression: #900"
)

for snippet in "${required_snippets[@]}"; do
  if ! grep -Fq -- "$snippet" "$STRATEGY_DOC"; then
    echo "CI strategy contract failed: missing snippet '$snippet'." >&2
    exit 1
  fi
done

echo "CI strategy contract tests passed."
