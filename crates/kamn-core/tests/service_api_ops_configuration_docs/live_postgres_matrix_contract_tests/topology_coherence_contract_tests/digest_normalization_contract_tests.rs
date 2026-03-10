use super::*;

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_markers(
) {
    assert!(DOC.contains(
        "### Parallel Lane Topology Host-Mode-Host-Pair-Lane-Set-Lane-Fingerprint-Hash Order-Normalization Contracts (Issue #5398)"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-hash-order-normalization.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_rows_csv=distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e,same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_contract=topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows_must_remain_canonically_sorted_after_order_normalization"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5398"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_markers(
) {
    assert_doc_contains_all(&["### Parallel Lane Topology Host-Mode-Host-Pair-Lane-Set-Lane-Fingerprint-Hash Order-Normalization-Digest Contracts (Issue #5400)", "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-hash-order-normalization-digest.v1", "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_rows_csv=distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e,same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea", "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_rows_fnv1a64_hex=25b9729eaeb44fe9", "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_contract=topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows_digest_must_remain_stable_under_order_normalization_and_permutations", "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_contract_is_canonical -- --exact", "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_is_stable -- --exact", "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_markers -- --exact", "Regression: #5400"]);
}
