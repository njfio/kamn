use kamn_sdk::{SdkError, TcpSignedEnvelope, TcpTransportAdapter, TcpTransportConfig};
#[path = "support/tcp_handshake_constant_time_support.rs"]
mod support;

use std::thread;
use std::time::Duration;
use support::{
    did, free_addr, handshake_payload, handshake_signature, send_raw_payload,
    signer_public_key_hex_for_private_key, TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
    TEST_TCP_SIGNING_PRIVATE_KEY_HEX_ALT,
};

const SOURCE: &str = include_str!("../src/tcp.rs");

#[test]
fn regression_requires_constant_time_tcp_handshake_compares() {
    let function_start = SOURCE
        .find("fn verify_matches_envelope(&self, envelope: &TcpSignedEnvelope) -> Result<(), SdkError> {")
        .unwrap_or_else(|| panic!("verify_matches_envelope function must exist"));
    let function_source = &SOURCE[function_start..];

    assert!(
        function_source.contains("constant_time_eq_bytes("),
        "tcp handshake verification must use a constant-time compare helper for signer key and signature"
    );
    assert!(
        !function_source.contains("if self.signer_public_key != envelope.signer_public_key"),
        "plain signer public key inequality must not remain in verify_matches_envelope"
    );
    assert!(
        !function_source.contains("if self.signature != envelope.signature"),
        "plain signature inequality must not remain in verify_matches_envelope"
    );
}

#[test]
fn integration_tcp_listener_accepts_matching_handshake() {
    let addr = free_addr();
    let listener_config = TcpTransportConfig::new(addr.as_str())
        .unwrap_or_else(|error| panic!("listener config failed: {error}"));
    let listener_adapter = TcpTransportAdapter::new(listener_config);

    let envelope = TcpSignedEnvelope::new(
        did("sender-match", TEST_TCP_SIGNING_PRIVATE_KEY_HEX),
        did("listener-match", TEST_TCP_SIGNING_PRIVATE_KEY_HEX),
        7,
        "state:match",
        "matching-handshake",
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
    )
    .unwrap_or_else(|error| panic!("envelope build failed: {error}"));

    let payload = handshake_payload(
        &envelope,
        envelope.signer_public_key.as_str(),
        handshake_signature(&envelope, TEST_TCP_SIGNING_PRIVATE_KEY_HEX).as_str(),
    );

    let listener_thread = thread::spawn(move || listener_adapter.listen_once());
    thread::sleep(Duration::from_millis(30));
    send_raw_payload(addr.as_str(), payload.as_str());

    let received = listener_thread
        .join()
        .unwrap_or_else(|_| panic!("listener thread panicked"))
        .unwrap_or_else(|error| panic!("listener failed: {error}"));

    assert_eq!(received.envelope, envelope);
    assert!(received.peer_addr.starts_with("127.0.0.1:"));
}

#[test]
fn integration_tcp_listener_rejects_mismatched_handshake_signer_key() {
    let addr = free_addr();
    let listener_config = TcpTransportConfig::new(addr.as_str())
        .unwrap_or_else(|error| panic!("listener config failed: {error}"));
    let listener_adapter = TcpTransportAdapter::new(listener_config);

    let envelope = TcpSignedEnvelope::new(
        did("sender-signer-mismatch", TEST_TCP_SIGNING_PRIVATE_KEY_HEX),
        did("listener-signer-mismatch", TEST_TCP_SIGNING_PRIVATE_KEY_HEX),
        8,
        "state:signer-mismatch",
        "mismatched-handshake-signer",
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
    )
    .unwrap_or_else(|error| panic!("envelope build failed: {error}"));

    let forged_signer_public_key =
        signer_public_key_hex_for_private_key(TEST_TCP_SIGNING_PRIVATE_KEY_HEX_ALT);
    let payload = handshake_payload(
        &envelope,
        forged_signer_public_key.as_str(),
        handshake_signature(&envelope, TEST_TCP_SIGNING_PRIVATE_KEY_HEX).as_str(),
    );

    let listener_thread = thread::spawn(move || listener_adapter.listen_once());
    thread::sleep(Duration::from_millis(30));
    send_raw_payload(addr.as_str(), payload.as_str());

    let listener_result = listener_thread
        .join()
        .unwrap_or_else(|_| panic!("listener thread panicked"));

    assert_eq!(
        listener_result,
        Err(SdkError::InvalidInput {
            field: "handshake.signer_public_key",
            reason: "does not match envelope signer public key",
        })
    );
}

#[test]
fn integration_tcp_listener_rejects_mismatched_handshake_signature() {
    let addr = free_addr();
    let listener_config = TcpTransportConfig::new(addr.as_str())
        .unwrap_or_else(|error| panic!("listener config failed: {error}"));
    let listener_adapter = TcpTransportAdapter::new(listener_config);

    let envelope = TcpSignedEnvelope::new(
        did(
            "sender-signature-mismatch",
            TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
        ),
        did(
            "listener-signature-mismatch",
            TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
        ),
        9,
        "state:signature-mismatch",
        "mismatched-handshake-signature",
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
    )
    .unwrap_or_else(|error| panic!("envelope build failed: {error}"));

    let payload = handshake_payload(
        &envelope,
        envelope.signer_public_key.as_str(),
        "sig:secp256k1:baseline-v2:0:00",
    );

    let listener_thread = thread::spawn(move || listener_adapter.listen_once());
    thread::sleep(Duration::from_millis(30));
    send_raw_payload(addr.as_str(), payload.as_str());

    let listener_result = listener_thread
        .join()
        .unwrap_or_else(|_| panic!("listener thread panicked"));

    assert_eq!(
        listener_result,
        Err(SdkError::InvalidInput {
            field: "handshake.signature",
            reason: "does not match envelope signature",
        })
    );
}
