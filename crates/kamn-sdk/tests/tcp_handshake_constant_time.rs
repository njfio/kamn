use kamn_core::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
};
use kamn_sdk::{AgentDid, SdkError, TcpSignedEnvelope, TcpTransportAdapter, TcpTransportConfig};
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

const SOURCE: &str = include_str!("../src/tcp.rs");
const TEST_TCP_SIGNING_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
const TEST_TCP_SIGNING_PRIVATE_KEY_HEX_ALT: &str =
    "094cf4e1f3d974bbf3e72233e2c2937e8fdb094740e0f017e010aa47ac1201ac";

fn did(value: &str, private_key_hex: &str) -> AgentDid {
    let signer_public_key = signer_public_key_hex_for_private_key(private_key_hex);
    match AgentDid::with_public_key_hex_binding(value, signer_public_key.as_str()) {
        Ok(bound) => bound,
        Err(error) => panic!("bound did parse failed: {error}"),
    }
}

fn free_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|error| {
        panic!("failed to allocate free tcp address: {error}");
    });
    listener
        .local_addr()
        .unwrap_or_else(|error| panic!("failed to read allocated local address: {error}"))
        .to_string()
}

fn send_raw_payload(addr: &str, payload: &str) {
    for _attempt in 0..40 {
        match TcpStream::connect(addr) {
            Ok(mut stream) => {
                stream
                    .write_all(payload.as_bytes())
                    .unwrap_or_else(|error| panic!("failed to write raw payload: {error}"));
                stream
                    .flush()
                    .unwrap_or_else(|error| panic!("failed to flush raw payload: {error}"));
                stream.shutdown(Shutdown::Write).unwrap_or_else(|error| {
                    panic!("failed to shutdown raw payload stream: {error}");
                });
                return;
            }
            Err(_) => thread::sleep(Duration::from_millis(10)),
        }
    }
    panic!("failed to connect raw payload sender to {addr}");
}

fn signer_public_key_hex_for_private_key(private_key_hex: &str) -> String {
    service_auth_public_key_hex_from_private_key_hex(private_key_hex).unwrap_or_else(|error| {
        panic!("failed to derive tcp signer public key for {private_key_hex}: {error}")
    })
}

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

    let handshake_signature = service_auth_sign_with_private_key_hex(
        envelope.from.as_str(),
        envelope.nonce,
        envelope.state_hash.as_str(),
        envelope.body.as_str(),
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
    )
    .unwrap_or_else(|error| panic!("handshake signature failed: {error}"));

    let payload = format!(
        "frame=handshake\nversion=1\nprofile=secp256k1:baseline-v2\nfrom={}\nto={}\nnonce={}\nsigner_public_key={}\nsignature={}\n\n{}",
        envelope.from,
        envelope.to,
        envelope.nonce,
        envelope.signer_public_key,
        handshake_signature,
        envelope.to_wire_payload()
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

    let forged_signer_public_key = signer_public_key_hex_for_private_key(TEST_TCP_SIGNING_PRIVATE_KEY_HEX_ALT);
    let handshake_signature = service_auth_sign_with_private_key_hex(
        envelope.from.as_str(),
        envelope.nonce,
        envelope.state_hash.as_str(),
        envelope.body.as_str(),
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
    )
    .unwrap_or_else(|error| panic!("handshake signature failed: {error}"));

    let payload = format!(
        "frame=handshake\nversion=1\nprofile=secp256k1:baseline-v2\nfrom={}\nto={}\nnonce={}\nsigner_public_key={}\nsignature={}\n\n{}",
        envelope.from,
        envelope.to,
        envelope.nonce,
        forged_signer_public_key,
        handshake_signature,
        envelope.to_wire_payload()
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
        did("sender-signature-mismatch", TEST_TCP_SIGNING_PRIVATE_KEY_HEX),
        did("listener-signature-mismatch", TEST_TCP_SIGNING_PRIVATE_KEY_HEX),
        9,
        "state:signature-mismatch",
        "mismatched-handshake-signature",
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
    )
    .unwrap_or_else(|error| panic!("envelope build failed: {error}"));

    let payload = format!(
        "frame=handshake\nversion=1\nprofile=secp256k1:baseline-v2\nfrom={}\nto={}\nnonce={}\nsigner_public_key={}\nsignature=sig:secp256k1:baseline-v2:0:00\n\n{}",
        envelope.from,
        envelope.to,
        envelope.nonce,
        envelope.signer_public_key,
        envelope.to_wire_payload()
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
