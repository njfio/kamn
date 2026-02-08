#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
DEEP_WORKFLOW="$ROOT_DIR/.github/workflows/ci-deep-validate.yml"

for workflow in "$FAST_WORKFLOW" "$DEEP_WORKFLOW"; do
  if ! grep -q -- '--max-attempts 2' "$workflow"; then
    echo "expected bounded retry of 2 attempts in $workflow" >&2
    exit 1
  fi

  if ! grep -q -- '--max-attempts 1' "$workflow"; then
    echo "expected non-retried invariant lane setting in $workflow" >&2
    exit 1
  fi

  if grep -q -- '--max-attempts 3' "$workflow"; then
    echo "unexpected max-attempts 3 detected in $workflow" >&2
    exit 1
  fi
done

echo "workflow retry policy tests passed."
