#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"

if ! grep -Fq "steps.scope.outputs.run_deploy_preflight_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected deploy preflight scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/deploy/test_preflight_topology.sh" "$FAST_WORKFLOW"; then
  echo "expected deploy preflight topology tests in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/deploy/test_generate_bundle.sh" "$FAST_WORKFLOW"; then
  echo "expected deploy bundle generator tests in ci-fast-gate.yml" >&2
  exit 1
fi

echo "workflow scope policy tests passed."
