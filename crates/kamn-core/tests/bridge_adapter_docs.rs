const DOC: &str = include_str!("../../../docs/foundation/bridge-adapter-abstraction.md");

#[test]
fn doc_contains_bridge_adapter_core_contracts() {
    assert!(DOC.contains("# Bridge Adapter Abstraction"));
    assert!(DOC.contains("BridgeAdapterEngine"));
    assert!(DOC.contains("process_inbound_to_envelope(...)"));
}

#[test]
fn regression_requires_duplicate_inbound_replay_rejection_rule() {
    // Regression: #423
    assert!(DOC.contains("DuplicateInboundMessageId"));
    assert!(DOC.contains("duplicate inbound event is rejected (`Regression: #423`)"));
}

#[test]
fn regression_requires_duplicate_outbound_replay_rejection_rule() {
    // Regression: #433
    assert!(DOC.contains("DuplicateOutboundRequestId"));
    assert!(DOC.contains("duplicate outbound request is rejected (`Regression: #433`)"));
}

#[test]
fn regression_requires_single_pass_projection_rule() {
    // Regression: #438
    assert!(DOC.contains("single-pass inbound projection"));
    assert!(DOC.contains(
        "first inbound-to-envelope projection does not self-trigger duplicate replay rejection (`Regression: #438`)"
    ));
}

#[test]
fn regression_requires_cross_chain_single_pass_projection_rule() {
    // Regression: #443
    assert!(DOC.contains(
        "cross-chain inbound projection also preserves single-pass replay safety (`Regression: #443`)"
    ));
}
