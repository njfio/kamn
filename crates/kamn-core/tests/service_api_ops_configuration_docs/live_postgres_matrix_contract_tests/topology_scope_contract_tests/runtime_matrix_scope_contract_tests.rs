use super::*;

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_pair_markers() {
    assert!(DOC.contains("### Two-Node Role-Pair Matrix (Issue #5352)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_role_pair_ids_csv=processor_to_listener_applied,processor_to_listener_deferred,listener_to_approver_applied,listener_to_approver_deferred,approver_to_processor_applied,approver_to_processor_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_role_pair_contract=role_pair_leg_a_applied->m10_phase6_scheduler_cycle_applied;role_pair_leg_b_applied->m10_phase6_scheduler_cycle_applied;role_pair_leg_a_deferred->m10_phase6_scheduler_cycle_deferred;role_pair_leg_b_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_role_pair_matrix_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_role_pair_matrix_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_pair_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5352"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_role_pair_markers(
) {
    assert!(DOC.contains("### Bounded Parallel Role-Pair Lane Matrix (Issue #5354)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_parallel_role_pair_lane_ids_csv=processor_listener_parallel_applied,processor_listener_parallel_deferred,listener_approver_parallel_applied,listener_approver_parallel_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_parallel_role_pair_contract=parallel_lane_leg_a_applied->m10_phase6_scheduler_cycle_applied;parallel_lane_leg_b_applied->m10_phase6_scheduler_cycle_applied;parallel_lane_leg_a_deferred->m10_phase6_scheduler_cycle_deferred;parallel_lane_leg_b_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_role_pair_lane_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_role_pair_lane_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_role_pair_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5354"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_asymmetric_parallel_lane_markers(
) {
    assert!(DOC.contains("### Asymmetric Parallel Lane Matrix (Issue #5356)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_asymmetric_parallel_lane_ids_csv=processor_listener_asymmetric_parallel_applied,processor_listener_asymmetric_parallel_deferred,listener_approver_asymmetric_parallel_applied,listener_approver_asymmetric_parallel_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_asymmetric_parallel_contract=asymmetric_parallel_leg_a_applied->m10_phase6_scheduler_cycle_applied;asymmetric_parallel_leg_b_applied->m10_phase6_scheduler_cycle_applied;asymmetric_parallel_leg_a_deferred->m10_phase6_scheduler_cycle_deferred;asymmetric_parallel_leg_b_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_asymmetric_parallel_lane_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_asymmetric_parallel_lane_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_asymmetric_parallel_lane_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5356"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_order_invariance_markers(
) {
    assert!(DOC.contains("### Parallel Lane Order-Invariance Matrix (Issue #5358)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_order_invariance_contract=baseline_and_permuted_lane_orders_must_produce_equivalent_sorted_reason_taxonomy_fingerprints"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_order_invariance_lane_sets_csv=symmetric_parallel,asymmetric_parallel"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_order_invariance_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_order_is_invariant -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_order_invariance_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5358"));
}
