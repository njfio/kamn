const DOC: &str = include_str!("../../../docs/research/kolme-fork-api-contract-inventory.md");

#[test]
fn inventory_captures_kolme_fork_base_api_routes_and_kamn_expectations() {
    assert!(DOC.contains("`/broadcast`"));
    assert!(DOC.contains("`/get-next-nonce`"));
    assert!(DOC.contains("`/block/{height}`"));
    assert!(DOC.contains("`/notifications`"));
    assert!(DOC.contains("`/fork-info`"));
    assert!(DOC.contains("`/healthz`"));
    assert!(DOC.contains("`/broadcast/runtime-commit`"));
    assert!(DOC.contains("`/runtime-commit/status`"));
}

#[test]
fn inventory_lists_integration_gaps_and_follow_up_issue_links() {
    assert!(DOC.contains("Gap: runtime_commit_submit_endpoint_mismatch"));
    assert!(DOC.contains("Gap: runtime_commit_payload_shape_mismatch"));
    assert!(DOC.contains("Gap: runtime_commit_finality_endpoint_missing"));
    assert!(DOC.contains("Gap: block_fallback_schema_mismatch"));
    assert!(DOC.contains("Follow-up Tasks"));
    assert!(DOC.contains("- #1502"));
    assert!(DOC.contains("- #1503"));
    assert!(DOC.contains("- #1504"));
}

#[test]
fn regression_marker_documents_fork_contract_inventory_baseline() {
    // Regression: #1501
    assert!(
        DOC.contains(
            "KAMN-to-kolme_fork endpoint/method/payload contract inventory remains synchronized with code-level integration assumptions (`Regression: #1501`)."
        )
    );
}
