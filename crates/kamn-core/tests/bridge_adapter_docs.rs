const DOC: &str = include_str!("../../../docs/foundation/bridge-adapter-abstraction.md");

#[test]
fn doc_contains_bridge_adapter_core_contracts() {
    assert!(DOC.contains("# Bridge Adapter Abstraction"));
    assert!(DOC.contains("BridgeAdapterEngine"));
    assert!(DOC.contains("process_inbound_to_envelope(...)"));
    assert!(DOC.contains("run_bridge_replay_harness"));
    assert!(DOC.contains("bridge_replay_suites"));
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
fn regression_requires_stale_inbound_rejection_rule() {
    // Regression: #546
    assert!(DOC.contains("StaleInboundMessage"));
    assert!(DOC
        .contains("stale inbound event beyond freshness window is rejected (`Regression: #546`)"));
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

#[test]
fn regression_requires_bridge_fixture_matrix_guard() {
    // Regression: #587
    assert!(DOC.contains("fixtures/bridge_replay/replay_validation_cases.json"));
    assert!(DOC.contains("scripts/bridge/run_bridge_replay_matrix.sh"));
    assert!(DOC.contains("signature-failure"));
    assert!(DOC.contains("adapter subset execution"));
    assert!(DOC.contains("Regression: #587"));
}

#[test]
fn doc_contains_credential_redaction_contract_lane() {
    assert!(DOC.contains("## Credentialed Staging + Redaction Contract"));
    assert!(DOC.contains("run_bridge_credential_redaction_check.py"));
    assert!(DOC.contains("run_bridge_credentialed_contract_lane.sh"));
    assert!(DOC.contains("run_bridge_credentialed_deep_lane.sh"));
}

#[test]
fn regression_requires_credential_leakage_guard() {
    // Regression: #621
    assert!(DOC.contains("credential leakage and replay gaps remain blocked (`Regression: #621`)"));
    assert!(DOC.contains(
        "staged credentialed bridge lane blocks raw secret exposure in logs/artifacts while retaining replay safety (`Regression: #621`)."
    ));
}
