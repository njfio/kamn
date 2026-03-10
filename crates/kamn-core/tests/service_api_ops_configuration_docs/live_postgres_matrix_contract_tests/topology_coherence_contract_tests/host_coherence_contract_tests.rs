use super::*;

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_coherence_markers(
) {
    assert!(DOC.contains(
        "### Parallel Lane Topology Host-Mode-Host-Pair Coherence Contracts (Issue #5386)"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_coherence_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-coherence.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_coherence_rows_csv=same_host_parallel->same_host->node_alpha->node_alpha,distributed_label_parallel->distributed_label->node_alpha->node_beta"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_coherence_contract=topology_id_to_host_mode_host_pair_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_coherence_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_coherence_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_coherence_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5386"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_cardinality_coherence_markers(
) {
    assert!(DOC.contains(
        "### Parallel Lane Topology Host-Mode-Host-Pair-Cardinality Coherence Contracts (Issue #5388)"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_cardinality_coherence_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-cardinality-coherence.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_cardinality_coherence_rows_csv=same_host_parallel->same_host->node_alpha->node_alpha->1,distributed_label_parallel->distributed_label->node_alpha->node_beta->2"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_cardinality_coherence_contract=topology_id_to_host_mode_host_pair_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_cardinality_coherence_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_cardinality_coherence_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_cardinality_coherence_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5388"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_count_coherence_markers(
) {
    assert!(DOC.contains(
        "### Parallel Lane Topology Host-Mode-Host-Pair-Lane-Set-Lane-Count Coherence Contracts (Issue #5390)"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_count_coherence_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-count-coherence.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_count_coherence_rows_csv=same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->4,distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->4"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_count_coherence_contract=topology_id_to_host_mode_host_pair_lane_set_lane_count_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_count_coherence_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_count_coherence_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_count_coherence_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5390"));
}
