use super::*;

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_markers(
) {
    assert!(DOC.contains(
        "### Parallel Lane Topology Host-Mode-Host-Pair-Lane-Set-Lane-Id-Bundle Coherence Contracts (Issue #5392)"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-id-bundle-coherence.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_rows_csv=same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->listener_approver_parallel_applied+listener_approver_parallel_deferred+processor_listener_parallel_applied+processor_listener_parallel_deferred,distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->listener_approver_asymmetric_parallel_applied+listener_approver_asymmetric_parallel_deferred+processor_listener_asymmetric_parallel_applied+processor_listener_asymmetric_parallel_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_contract=topology_id_to_host_mode_host_pair_lane_set_lane_id_bundle_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5392"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_bundle_coherence_markers(
) {
    assert!(DOC.contains(
        "### Parallel Lane Topology Host-Mode-Host-Pair-Lane-Set-Lane-Fingerprint-Bundle Coherence Contracts (Issue #5394)"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_bundle_coherence_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-bundle-coherence.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_bundle_coherence_rows_csv=same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->listener_approver_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+listener_approver_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1,distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->listener_approver_asymmetric_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+listener_approver_asymmetric_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_asymmetric_parallel_applied|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_applied|kamn.runtime.daemon.phase6.reason-taxonomy.v1+processor_listener_asymmetric_parallel_deferred|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1|m10_phase6_scheduler_cycle_deferred|kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_bundle_coherence_contract=topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_bundle_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_bundle_coherence_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_bundle_coherence_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_bundle_coherence_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5394"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_coherence_markers(
) {
    assert!(DOC.contains(
        "### Parallel Lane Topology Host-Mode-Host-Pair-Lane-Set-Lane-Fingerprint-Hash Coherence Contracts (Issue #5396)"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_coherence_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-fingerprint-hash-coherence.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_coherence_rows_csv=distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->18ce08940c67c38e,same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->37e351d41d1e30ea"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_coherence_contract=topology_id_to_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_coherence_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_coherence_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_coherence_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5396"));
}
