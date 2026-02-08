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
