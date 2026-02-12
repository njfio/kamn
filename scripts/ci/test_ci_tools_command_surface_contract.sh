#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"

required_commands=(
  'bash "$ROOT_DIR/scripts/ci/test_kolme_command_surface_coverage_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_command_surface_asymmetry_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_tranche1_manifest_migration_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_runtime_nonce_manifest_migration_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_version_matrix_manifest_migration_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_profile_selftest_portability_manifest_migration_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_runtime_triadic_bootstrap_e2e_manifest_migration_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_bootstrap_conformance_runtime_process_manifest_migration_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_parity_demo_real_process_manifest_migration_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_manifest_migration_contract_dispatch_wrapper_matrix.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_run_kolme_manifest_migration_contract_dispatch.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_tranche1_dispatch_execution_parity_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_wrapper_inventory_baseline_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_kolme_wrapper_budget_trend.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_lane_migration_matrix_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_contract_lane_dispatch_wrapper_matrix.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_generate_fork_compatibility_evidence.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_fork_compatibility_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_runtime_commit_adapter_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_kolme_fork_rust_test_matrix_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kamn_live_runtime_integration_real_node_profile.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kamn_live_runtime_real_node_profile_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_profile_preflight_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_profile_preflight_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_self_test_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_self_test_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_portability_preflight_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_kolme_fork_portability_preflight_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_portability_preflight_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_checkout_bootstrap_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_kolme_fork_checkout_bootstrap_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_checkout_bootstrap_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_kolme_fork_real_process_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_runtime_commit_decomposition_parity_matrix.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_bootstrap_health_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_bootstrap_health_checks_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_e2e_integration_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_live_node_validation_bundle_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_live_node_validation_bundle_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_e2e_integration_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_heavy_validation_matrix_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_heavy_validation_matrix_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_runtime_commit_live_finality_evidence_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_generate_test_harness_loc_report.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_test_harness_loc_soft_budget.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_missing_docs_velocity_guard_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_missing_docs_graduation_batch_report_contract.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_assert_local_heavy_opt_in.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_generate_local_lane_summary.sh"'
)

for command in "${required_commands[@]}"; do
  if ! grep -Fq "$command" "$CI_TOOLS_SCRIPT"; then
    echo "expected ci tools regression lane to include command: $command" >&2
    exit 1
  fi
done

echo "ci tools command surface contract tests passed."
