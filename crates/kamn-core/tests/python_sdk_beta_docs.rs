const DOC: &str = include_str!("../../../docs/foundation/python-sdk-beta.md");

#[test]
fn doc_contains_python_live_transport_scope() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("LiveTransportConfig"));
    assert!(DOC.contains("LiveKAMNClient"));
    assert!(DOC.contains("TransportModeMismatchError"));
}

#[test]
fn doc_contains_python_live_transport_validation_commands() {
    assert!(DOC.contains("bash scripts/sdk/run_live_transport_parity_contract_lane.sh"));
    assert!(DOC.contains("bash scripts/sdk/run_live_transport_parity_deep_lane.sh"));
    assert!(DOC.contains("python3 -m unittest tests/python/test_sdk.py"));
}

#[test]
fn regression_requires_transport_mode_mismatch_guard_rule() {
    // Regression: #620
    assert!(DOC.contains("contract drift (`Regression: #620`)"));
}
