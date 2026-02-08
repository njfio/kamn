const DOC: &str = include_str!("../../../docs/foundation/rust-sdk-alpha.md");

#[test]
fn doc_contains_live_transport_sdk_scope() {
    assert!(DOC.contains("LiveTransportKamnClient"));
    assert!(DOC.contains("LiveTransportConfig"));
    assert!(DOC.contains("TransportMode"));
    assert!(DOC.contains("KamnTransport"));
}

#[test]
fn doc_contains_live_transport_validation_commands() {
    assert!(DOC.contains("bash scripts/sdk/run_rust_live_transport_contract_lane.sh"));
    assert!(DOC.contains("bash scripts/sdk/run_rust_live_transport_deep_lane.sh"));
}

#[test]
fn regression_requires_transport_mode_mismatch_guard_contract() {
    // Regression: #620
    assert!(DOC.contains("mismatch rejection (`Regression: #620`)"));
}
