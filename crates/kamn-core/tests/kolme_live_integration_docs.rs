const DOC: &str = include_str!("../../../docs/architecture/kolme-live-integration.md");

#[test]
fn doc_contains_process_isolation_marker_contracts() {
    assert!(DOC.contains("transport_convergence_status"));
    assert!(DOC.contains("libp2p_process_isolation_status"));
    assert!(DOC.contains("libp2p_two_node_process_isolated_status"));
    assert!(DOC.contains("libp2p_three_node_process_isolated_status"));
    assert!(DOC.contains("runtime_provider_client_contract=KolmeRuntimeCommitLiveProvider"));
}

#[test]
fn doc_contains_process_isolation_fail_closed_reasons() {
    assert!(DOC.contains("local_full_stack_integration_policy_reason_taxonomy_version_mismatch"));
    assert!(DOC
        .contains("local_full_stack_integration_policy_libp2p_process_isolation_status_mismatch"));
    assert!(DOC.contains(
        "local_full_stack_integration_policy_libp2p_two_node_process_isolated_status_mismatch"
    ));
    assert!(DOC.contains(
        "local_full_stack_integration_policy_libp2p_three_node_process_isolated_status_mismatch"
    ));
    assert!(DOC.contains(
        "local_full_stack_integration_policy_libp2p_summary_three_node_partition_rejoin_status_mismatch"
    ));
    assert!(DOC.contains(
        "local_full_stack_integration_policy_libp2p_summary_three_node_publish_drop_status_mismatch"
    ));
}
