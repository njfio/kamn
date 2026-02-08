const DOC: &str = include_str!("../../../docs/foundation/discord-bridge-approver-gating.md");

#[test]
fn doc_contains_discord_bridge_scope_and_quorum_contracts() {
    assert!(DOC.contains("# Discord Bridge Approver-Gated Outbound Flow"));
    assert!(DOC.contains("DiscordBridgeConfig"));
    assert!(DOC.contains("process_outbound_with_approvals(...)"));
    assert!(DOC.contains("approver quorum"));
}

#[test]
fn doc_contains_bridge_replay_subset_validation_lane() {
    assert!(DOC.contains("scripts/bridge/run_bridge_replay_matrix.sh"));
    assert!(DOC.contains("--suites bridge_adapter,discord_bridge"));
    assert!(DOC.contains("bridge_replay_suites"));
}

#[test]
fn regression_requires_signature_failure_fixture_reference() {
    // Regression: #587
    assert!(DOC.contains("signature-failure"));
    assert!(DOC.contains("Regression: #587"));
}
