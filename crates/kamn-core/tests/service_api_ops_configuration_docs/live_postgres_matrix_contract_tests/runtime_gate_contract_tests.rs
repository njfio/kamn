use super::*;

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_validation_slice_markers() {
    assert!(DOC.contains(
        "## PostgreSQL Live Integration + Daemon Runtime Validation Slice (Issue #5338)"
    ));
    assert!(DOC.contains("phase6_live_postgres_daemon_runtime_slice_status=verified"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_slice_env_gate=KAMN_TEST_POSTGRES_URL|DATABASE_URL"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_slice_reason_taxonomy_version=kamn.runtime.daemon.phase6-live-postgres.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_slice_reason_codes_csv=live_postgres_env_unset,live_postgres_adapter_connected,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_slice_contract=live_postgres_env_gate->adapter_connect_and_migrate->daemon_phase6_runtime_projection"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_postgres_execution_adapter spec_c01_and_c03_live_adapter_executes_insert_and_lookup_with_session_context -- --exact"
    ));
    assert!(DOC.contains("Regression: #5338"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_gate_and_deferred_markers() {
    assert!(DOC.contains("### Gate and Deferred Path Hardening (Issue #5340)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_gate_reason_contract=env_unset->skip_with_reason;env_set->adapter_connect_and_migrate"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_deferred_contract=live_postgres_adapter_connected+shutdown_signal->m10_phase6_scheduler_cycle_deferred"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_runtime_daemon_live_postgres_validation_slice_reports_unset_env_gate_reason -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::unit_runtime_daemon_live_postgres_validation_slice_prefers_kamn_test_postgres_url -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_deferred_path -- --exact"
    ));
    assert!(DOC.contains("Regression: #5340"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_stability_markers() {
    assert!(DOC.contains("### Scenario Matrix Stability (Issue #5342)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_contract=env_unset->live_postgres_env_unset;env_set_no_shutdown->m10_phase6_scheduler_cycle_applied;env_set_shutdown->m10_phase6_scheduler_cycle_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_stability_contract=repeated_runs_preserve_reason_code_per_matrix_scenario"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_env_matrix_contract_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs -- --exact"
    ));
    assert!(DOC.contains("Regression: #5342"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_ordering_markers(
) {
    assert!(DOC.contains("### Matrix Taxonomy and Canonical Ordering (Issue #5344)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_reason_taxonomy_version=kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_reason_codes_csv=live_postgres_env_unset,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_scenarios_csv=env_unset,env_set_no_shutdown,env_set_shutdown"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_order_contract=matrix_rows_order=env_unset->env_set_no_shutdown->env_set_shutdown;reason_codes_align_with_scenarios"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_matrix_projection_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_ordering_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5344"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_bridge_markers(
) {
    assert!(DOC.contains("### Runtime-to-Matrix Taxonomy Bridge (Issue #5346)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_reason_taxonomy_version=kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_reason_taxonomy_version=kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_taxonomy_bridge_contract=runtime_reason_taxonomy_v1->matrix_scenario_taxonomy_v1;applied_and_deferred_reasons_must_align"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_matrix_taxonomy_bridge_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_taxonomy_versions_are_stable_across_repeated_runs -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_bridge_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5346"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_load_profile_markers()
{
    assert!(DOC.contains("### Bounded Load-Profile Matrix (Issue #5348)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_load_profile_ids_csv=applied_t3_i10,applied_t5_i25,applied_t9_i40,deferred_t5_i25_s3_d2_to4,deferred_t7_i25_s3_d2_to4,deferred_t9_i40_s3_d2_to4"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_load_profile_contract=applied_profiles->m10_phase6_scheduler_cycle_applied;deferred_profiles->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_load_profile_matrix_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_load_profile_matrix_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_load_profile_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5348"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_profile_markers()
{
    assert!(DOC.contains("### Role-Profile Matrix Determinism (Issue #5350)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_role_profile_ids_csv=processor_applied,processor_deferred,listener_applied,listener_deferred,approver_applied,approver_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_role_profile_contract=processor|listener|approver_applied->m10_phase6_scheduler_cycle_applied;processor|listener|approver_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_role_profile_matrix_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_role_profile_matrix_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_profile_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5350"));
}
