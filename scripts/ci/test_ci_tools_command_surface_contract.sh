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
  'bash "$ROOT_DIR/scripts/ci/test_kolme_wave8_wrapper_family_baseline_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_kolme_wave8_wrapper_family_budget_trend.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_wave10_wrapper_family_baseline_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_kolme_wave10_wrapper_family_budget_trend.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_wave11_wrapper_family_baseline_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_kolme_wave11_wrapper_family_budget_trend.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_non_kolme_wave_trend_test_loc_soft_budget.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_non_kolme_compliance_contract_lane_dispatch_wrapper_matrix.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_non_kolme_manifest_backed_contract_lane_dispatch_wrapper_matrix.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_non_kolme_bridge_contract_lane_dispatch_wrapper_matrix.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_non_kolme_sdk_contract_lane_dispatch_wrapper_matrix.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_non_kolme_lightweight_contract_lane_dispatch_wrapper_matrix.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_lane_migration_matrix_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_contract_lane_dispatch_wrapper_matrix.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_generate_fork_compatibility_evidence.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_fork_compatibility_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_runtime_commit_adapter_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_continuous_runtime_commit_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_did_lifecycle_chain_adapter_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_message_proof_anchoring_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_kolme_fork_rust_test_matrix_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kamn_live_runtime_integration_real_node_profile.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kamn_live_runtime_real_node_profile_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_live_deployment_preflight_contract_lane.sh"'
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
  'bash "$ROOT_DIR/scripts/kolme/test_run_managed_signer_startup_live_validation_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_validate_continuous_runtime_commit_live.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_validate_did_lifecycle_chain_adapter_live.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_validate_message_proof_anchoring_live.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_signature_parity_matrix.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_signature_parity_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_signature_parity_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_generate_test_harness_loc_report.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_generate_kolme_test_harness_loc_trend_report.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_test_harness_loc_soft_budget.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_kolme_test_harness_loc_soft_budget.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_ignored_test_inventory_drift.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_ignored_test_inventory_metadata_policy.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_ignored_test_inventory_parser_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_local_retry_diagnostics_ci_exclusion_policy.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_run_kolme_test_harness_loc_soft_budget_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_run_fast_gate_budget_delta_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/runtime/test_validate_local_retry_diagnostics_live.sh"'
  'bash "$ROOT_DIR/scripts/runtime/test_check_local_retry_diagnostics_live_policy.sh"'
  'bash "$ROOT_DIR/scripts/runtime/test_validate_local_retry_diagnostics_live_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_missing_docs_velocity_guard_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_missing_docs_graduation_batch_report_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_ci_strategy_wave_range_marker_contract.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_assert_local_heavy_opt_in.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_generate_local_lane_summary.sh"'
)

for command in "${required_commands[@]}"; do
  if ! grep -Fq "$command" "$CI_TOOLS_SCRIPT"; then
    echo "expected ci tools regression lane to include command: $command" >&2
    exit 1
  fi
done

required_wave_loop_snippets=(
  'run_non_kolme_wave_wrapper_family_contracts()'
  'for wave in {1..19}; do'
  'bash "$ROOT_DIR/scripts/ci/test_non_kolme_wave${wave}_wrapper_family_baseline_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_check_non_kolme_wave${wave}_wrapper_family_budget_trend.sh"'
  'run_non_kolme_lightweight_wave_wrapper_matrix_contracts()'
  'for lightweight_wave in {10..18}; do'
  'bash "$ROOT_DIR/scripts/framework/test_non_kolme_wave${lightweight_wave}_lightweight_contract_lane_dispatch_wrapper_matrix.sh"'
)

for snippet in "${required_wave_loop_snippets[@]}"; do
  if ! grep -Fq "$snippet" "$CI_TOOLS_SCRIPT"; then
    echo "expected ci tools regression lane to include non-Kolme wave helper snippet: $snippet" >&2
    exit 1
  fi
done

wave_helper_invocation_count="$(grep -Ec '^[[:space:]]*run_non_kolme_wave_wrapper_family_contracts$' "$CI_TOOLS_SCRIPT")"
if [ "$wave_helper_invocation_count" -ne 2 ]; then
  echo "expected ci tools regression lane to invoke run_non_kolme_wave_wrapper_family_contracts exactly twice; found $wave_helper_invocation_count" >&2
  exit 1
fi

lightweight_wave_helper_invocation_count="$(grep -Ec '^[[:space:]]*run_non_kolme_lightweight_wave_wrapper_matrix_contracts$' "$CI_TOOLS_SCRIPT")"
if [ "$lightweight_wave_helper_invocation_count" -ne 2 ]; then
  echo "expected ci tools regression lane to invoke run_non_kolme_lightweight_wave_wrapper_matrix_contracts exactly twice; found $lightweight_wave_helper_invocation_count" >&2
  exit 1
fi

echo "ci tools command surface contract tests passed."
