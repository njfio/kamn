use super::*;

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_mapping_markers(
) {
    assert!(DOC.contains("### Parallel Lane Topology Host-Pair Mapping Contracts (Issue #5372)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_mapping_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-mapping.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_mapping_rows_csv=same_host_parallel->node_alpha->node_alpha,distributed_label_parallel->node_alpha->node_beta"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_mapping_contract=topology_id_to_host_pair_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_mapping_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_mapping_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_mapping_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5372"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_lane_set_mapping_markers(
) {
    assert!(DOC.contains("### Parallel Lane Topology Lane-Set Mapping Contracts (Issue #5374)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_set_mapping_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-lane-set-mapping.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_set_mapping_rows_csv=same_host_parallel->symmetric_parallel,distributed_label_parallel->asymmetric_parallel"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_set_mapping_contract=topology_id_to_lane_set_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_lane_set_mapping_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_lane_set_mapping_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_lane_set_mapping_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5374"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_lane_count_mapping_markers(
) {
    assert!(DOC.contains("### Parallel Lane Topology Lane-Count Mapping Contracts (Issue #5376)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_count_mapping_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-lane-count-mapping.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_count_mapping_rows_csv=same_host_parallel->4,distributed_label_parallel->4"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_lane_count_mapping_contract=topology_id_to_lane_count_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_lane_count_mapping_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_lane_count_mapping_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_lane_count_mapping_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5376"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_mapping_markers(
) {
    assert!(DOC.contains("### Parallel Lane Topology Host-Mode Mapping Contracts (Issue #5378)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_mapping_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-mapping.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_mapping_rows_csv=same_host_parallel->same_host,distributed_label_parallel->distributed_label"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_mapping_contract=topology_id_to_host_mode_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_mapping_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_mapping_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_mapping_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5378"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_cardinality_mapping_markers(
) {
    assert!(
        DOC.contains("### Parallel Lane Topology Host-Cardinality Mapping Contracts (Issue #5380)")
    );
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_cardinality_mapping_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-cardinality-mapping.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_cardinality_mapping_rows_csv=same_host_parallel->1,distributed_label_parallel->2"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_cardinality_mapping_contract=topology_id_to_unique_host_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_cardinality_mapping_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_cardinality_mapping_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_cardinality_mapping_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5380"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_cardinality_coherence_markers(
) {
    assert!(DOC.contains(
        "### Parallel Lane Topology Host-Mode-Cardinality Coherence Contracts (Issue #5382)"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_cardinality_coherence_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-cardinality-coherence.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_cardinality_coherence_rows_csv=same_host_parallel->same_host->1,distributed_label_parallel->distributed_label->2"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_cardinality_coherence_contract=topology_id_to_host_mode_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_cardinality_coherence_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_cardinality_coherence_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_cardinality_coherence_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5382"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_cardinality_coherence_markers(
) {
    assert!(DOC.contains(
        "### Parallel Lane Topology Host-Pair-Cardinality Coherence Contracts (Issue #5384)"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_cardinality_coherence_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-cardinality-coherence.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_cardinality_coherence_rows_csv=same_host_parallel->node_alpha->node_alpha->1,distributed_label_parallel->node_alpha->node_beta->2"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_cardinality_coherence_contract=topology_id_to_host_pair_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_cardinality_coherence_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_cardinality_coherence_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_cardinality_coherence_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5384"));
}
