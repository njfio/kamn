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
    assert!(DOC.contains("bash scripts/sdk/run_local_e2e_demo.sh"));
    assert!(DOC.contains("bash scripts/sdk/run_localhost_signed_demo.sh"));
    assert!(DOC.contains("bash scripts/sdk/run_tcp_signed_relay_demo.sh"));
    assert!(DOC.contains("bash scripts/sdk/run_tcp_failover_reconnect_matrix.sh --lane fast"));
    assert!(DOC.contains("bash scripts/sdk/run_rust_live_transport_contract_lane.sh"));
    assert!(DOC.contains("bash scripts/sdk/run_rust_live_transport_deep_lane.sh"));
}

#[test]
fn regression_requires_transport_mode_mismatch_guard_contract() {
    // Regression: #620
    assert!(DOC.contains("mismatch rejection (`Regression: #620`)"));
}

#[test]
fn regression_requires_local_e2e_demo_marker_contract() {
    // Regression: #770
    assert!(DOC.contains("`Regression: #770`"));
    assert!(DOC.contains("status=ok"));
    assert!(DOC.contains("escrow_id=<id>"));
}

#[test]
fn regression_requires_localhost_signed_demo_marker_contract() {
    // Regression: #807
    assert!(DOC.contains("`Regression: #807`"));
    assert!(DOC.contains("verified=true"));
    assert!(DOC.contains("signature=sig:ed25519:baseline-v1:..."));
}

#[test]
fn regression_requires_tcp_signed_relay_demo_marker_contract() {
    // Regression: #822
    assert!(DOC.contains("`Regression: #822`"));
    assert!(DOC.contains("adapter=tcp"));
    assert!(DOC.contains("tcp_signed_relay_listener"));
    assert!(DOC.contains("tcp_signed_relay_sender"));
}

#[test]
fn regression_requires_tcp_handshake_replay_guard_contract() {
    // Regression: #823
    assert!(DOC.contains("`Regression: #823`"));
    assert!(DOC.contains("Forged handshake frames are rejected"));
    assert!(DOC.contains("conflict: tcp handshake replay detected"));
}

#[test]
fn regression_requires_tcp_failover_reconnect_matrix_contract() {
    // Regression: #824
    assert!(DOC.contains("`Regression: #824`"));
    assert!(DOC.contains("kamn.sdk.tcp-failover-reconnect.matrix.v1"));
    assert!(DOC.contains("fixtures/sdk_failover_reconnect/reconnect_drift_signatures.txt"));
    assert!(DOC.contains("KAMN_TCP_FAILOVER_DEEP_CADENCE=scheduled"));
}
