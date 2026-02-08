#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
DEEP_WORKFLOW="$ROOT_DIR/.github/workflows/ci-deep-validate.yml"

for workflow in "$FAST_WORKFLOW" "$DEEP_WORKFLOW"; do
  if ! grep -q "shared-key: kamn-rust-ci-v1" "$workflow"; then
    echo "expected rust-cache shared key in $workflow" >&2
    exit 1
  fi

  if ! grep -Fq 'save-if: ${{ github.ref == '\''refs/heads/main'\'' }}' "$workflow"; then
    echo "expected rust-cache save-if guard in $workflow" >&2
    exit 1
  fi
done

if ! grep -q "run_invariant_harness.sh --mode deep --parallelism 2" "$DEEP_WORKFLOW"; then
  echo "expected deep invariant harness to use bounded parallelism in ci-deep-validate.yml" >&2
  exit 1
fi

# Regression: #568
if ! grep -Fq "if: steps.scope.outputs.run_ci_tool_checks == 'true'" "$FAST_WORKFLOW"; then
  echo "expected ci-fast-gate CI tool checks to be gated by run_ci_tool_checks output" >&2
  exit 1
fi

echo "workflow cache policy tests passed."
