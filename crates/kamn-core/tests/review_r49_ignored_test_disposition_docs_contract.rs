const DOC: &str =
    include_str!("../../../docs/planning/2026-02-21-r49-ignored-test-periodic-reevaluation.md");

#[test]
fn functional_r49_ignored_test_re_evaluation_markers_present() {
    assert!(DOC.contains(
        "ignored_test_disposition_schema_version=kamn.review.ignored-test-disposition.v1"
    ));
    assert!(DOC.contains("ignored_test_periodic_review_cycle=R49"));
    assert!(DOC.contains("ignored_test_inventory_count=12"));
    assert!(DOC.contains("ignored_test_inventory_evidence_command=bash scripts/ci/check_ignored_test_inventory_drift.sh"));
    assert!(DOC.contains("ignored_test_disposition_decision_set=retain|promote|deprecate"));
}

#[test]
fn integration_r49_ignored_test_re_evaluation_covers_all_baseline_entries() {
    let required_test_names = [
        "performance_channel_snapshot_deep_lane_stress",
        "performance_message_lifecycle_snapshot_deep_lane_stress",
        "performance_network_fault_simulation_chaos_lane_stress",
        "performance_file_snapshot_store_recovery_deep_lane_large_payload",
        "performance_task_operation_snapshot_store_deep_lane_stress",
        "performance_concurrency_state_mutation_deep_lane_stress",
        "performance_durable_guard_recovery_matrix_deep_lane",
        "performance_bundle_store_deep_lane_stress",
        "performance_signer_emulator_bulk_signing_deep_lane",
        "performance_zk_witness_mutation_deep_lane_stress",
        "performance_live_transport_multi_client_deep_lane",
        "performance_tcp_failover_reconnect_matrix_deep_lane",
    ];

    for test_name in required_test_names {
        assert!(
            DOC.contains(test_name),
            "missing ignored-test disposition row for {test_name}"
        );
    }
}
