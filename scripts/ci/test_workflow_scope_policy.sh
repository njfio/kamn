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

if ! grep -Fq "if: steps.scope.outputs.run_frontend_dashboard_tests == 'true'" "$FAST_WORKFLOW"; then
  echo "expected frontend dashboard scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/frontend/test_dashboard_package.sh" "$FAST_WORKFLOW"; then
  echo "expected frontend dashboard test command in ci-fast-gate.yml" >&2
  exit 1
fi

# Regression: #587
if ! grep -Fq "if: steps.scope.outputs.run_bridge_replay_harness == 'true'" "$FAST_WORKFLOW"; then
  echo "expected bridge replay harness scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/bridge/run_bridge_replay_matrix.sh --fixture fixtures/bridge_replay/replay_validation_cases.json --suites \"\${{ steps.scope.outputs.bridge_replay_suites }}\" --output-json bridge-replay-report.json" "$FAST_WORKFLOW"; then
  echo "expected bridge replay harness command in ci-fast-gate.yml" >&2
  exit 1
fi

# Regression: #583
if ! grep -Fq "if: steps.scope.outputs.run_sdk_parity_matrix == 'true'" "$FAST_WORKFLOW"; then
  echo "expected sdk parity matrix scope condition in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -Fq "bash scripts/sdk/run_sdk_parity_matrix.sh --fixture fixtures/sdk_parity/register_validation_cases.json --output-json sdk-parity-report.json" "$FAST_WORKFLOW"; then
  echo "expected sdk parity matrix command in ci-fast-gate.yml" >&2
  exit 1
fi

echo "workflow scope policy tests passed."
