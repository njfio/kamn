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
  "run_frontend_dashboard_tests=true"
  "test_scope=frontend"
  "run_deploy_preflight_tests=true"
  "test_scope=deploy"
  "run_settlement_reconciliation_contract_tests=true"
  "test_scope=escrow-contract"
  "run_bridge_replay_harness=true"
  "test_scope=bridge"
  "run_localhost_signed_integration_contract_lane.sh"
  "run_backend_session_auth_freshness_contract_lane.sh"
  "run_dashboard_stale_error_budget_contract_lane.sh"
  "run_dashboard_shell_determinism_matrix_contract_lane.sh"
  "run_deployment_slo_rollback_contract_lane.sh"
  "run_settlement_reconciliation_contract_lane.sh"
  "run_bridge_adapter_conformance_contract_lane.sh"
  "run_post_cutover_slo_contract_lane.sh"
  "run_classification_redaction_contract_lane.sh"
  "run_quorum_attestation_replay_contract_lane.sh"
  "test_run_kamn_core_rustdoc_artifact_contract_lane.sh"
  "test_check_kamn_core_rustdoc_artifact_policy.sh"
  "scripts/ci/select_targets.sh"
  "Regression: #900"
  "Regression: #939"
  "Regression: #906"
  "Regression: #913"
  "Regression: #914"
  "Regression: #911"
  "Regression: #942"
  "Regression: #943"
  "Regression: #941"
  "Regression: #944"
  "Regression: #907"
)

for snippet in "${required_snippets[@]}"; do
  if ! grep -Fq -- "$snippet" "$STRATEGY_DOC"; then
    echo "CI strategy contract failed: missing snippet '$snippet'." >&2
    exit 1
  fi
done

echo "CI strategy contract tests passed."
