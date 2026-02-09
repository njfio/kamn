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
  "make smoke-live-network"
  "make deep-live-network"
  "make demo"
  "make demo-localhost-transport"
  "make ci-tools"
  "run_runtime_snapshot_contract_tests=true"
  "test_scope=runtime-contract"
  "run_localhost_signed_integration_contract_lane_tests"
  "sdk-live-localhost-integration"
  "run_live_transport_parity_contract_tests=true"
  "live_transport_parity_languages=rust,python,typescript"
  "run_dashboard_contract_tests=true"
  "test_scope=frontend-contract"
  "run_localhost_signed_integration_contract_lane.sh"
  "run_backend_session_auth_freshness_contract_lane.sh"
  "scripts/ci/select_targets.sh"
  "Regression: #900"
  "Regression: #939"
  "Regression: #941"
)

for snippet in "${required_snippets[@]}"; do
  if ! grep -Fq -- "$snippet" "$STRATEGY_DOC"; then
    echo "CI strategy contract failed: missing snippet '$snippet'." >&2
    exit 1
  fi
done

echo "CI strategy contract tests passed."
