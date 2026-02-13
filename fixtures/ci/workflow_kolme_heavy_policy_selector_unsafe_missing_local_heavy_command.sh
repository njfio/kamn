case "${CI_ENABLE_KOLME_LOCAL_HEAVY_CONTRACT_TESTS:-false}" in
  1|true|TRUE|yes|YES|on|ON)
    KOLME_LOCAL_HEAVY_SELECTOR_OPT_IN=true
    ;;
esac

case "$file" in
  scripts/framework/test_assert_local_heavy_opt_in.sh|scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_lane.sh|scripts/kolme/test_check_local_kolme_fork_rust_test_matrix_policy.sh|scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_contract_lane.sh|scripts/kolme/test_run_local_bootstrap_health_checks.sh|scripts/kolme/test_run_local_e2e_integration_lane.sh|scripts/kolme/test_run_local_heavy_validation_matrix.sh|scripts/kolme/test_run_local_runtime_commit_live_lane.sh|scripts/kolme/test_run_local_runtime_commit_live_finality_evidence_contract_lane.sh|scripts/kolme/test_run_local_native_api_parity_live_proof_contract_lane.sh|scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh|scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh)
    KOLME_LOCAL_HEAVY_CONTRACT_CHANGED=true
    ;;
esac

write_output "run_kolme_local_heavy_contract_tests" "$RUN_KOLME_LOCAL_HEAVY_CONTRACT_TESTS"
write_output "kolme_local_heavy_selector_opt_in" "$KOLME_LOCAL_HEAVY_SELECTOR_OPT_IN"
