use super::*;

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_permutation_invariance_markers(
) {
    assert!(DOC.contains("### Parallel Lane Permutation-Invariance Matrix (Issue #5360)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_permutation_ids_csv=baseline,reverse,rotate_left_1,interleaved_even_then_odd"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_permutation_invariance_contract=deterministic_permutations_must_preserve_sorted_lane_reason_taxonomy_fingerprints"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_permutation_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_permutations_are_invariant -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_permutation_invariance_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5360"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_fingerprint_schema_markers(
) {
    assert!(DOC.contains("### Parallel Lane Fingerprint Schema Contracts (Issue #5362)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_fingerprint_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-fingerprint.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_fingerprint_field_order_csv=lane_id,leg_a_reason,leg_a_taxonomy,leg_b_reason,leg_b_taxonomy"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_fingerprint_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_fingerprint_schema_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_fingerprint_schema_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5362"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_scope_markers(
) {
    assert!(DOC.contains("### Parallel Lane Topology-Scope Contracts (Issue #5364)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_ids_csv=same_host_parallel,distributed_label_parallel"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_contract=topology_labels_must_preserve_sorted_lane_reason_taxonomy_fingerprints_under_repeated_runs"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_scope_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_scope_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_scope_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5364"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_permutation_markers(
) {
    assert!(
        DOC.contains("### Parallel Lane Topology Permutation-Invariance Contracts (Issue #5366)")
    );
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_permutation_ids_csv=baseline,reverse,rotate_left_1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_permutation_contract=deterministic_topology_profile_permutations_must_preserve_sorted_topology_fingerprint_bundles"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_permutation_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_permutations_are_invariant -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_permutation_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5366"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_markers(
) {
    assert!(DOC.contains("### Parallel Lane Topology Host-Pair Contracts (Issue #5368)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_required_host_pair_ids_csv=node_alpha->node_alpha,node_alpha->node_beta"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_contract=host_pair_ids_must_remain_stable_under_repeated_runs_and_topology_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pairs_are_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5368"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_directionality_markers(
) {
    assert!(
        DOC.contains("### Parallel Lane Topology Host-Pair Directionality Contracts (Issue #5370)")
    );
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_directionality_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-directionality.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_directionality_extraction_rule=host_a_to_host_b_arrow_notation_non_commutative"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_directionality_forbidden_reverse_pairs_csv=node_beta->node_alpha"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_directionality_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5370"));
}
