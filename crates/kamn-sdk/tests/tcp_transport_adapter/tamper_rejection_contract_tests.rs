use super::support::*;

#[test]
fn regression_tampered_tcp_envelope_signature_is_rejected() {
    let envelope = build_envelope(
        did("sender-regression"),
        did("listener-regression"),
        1,
        "state:regression",
        "expected-body",
    );
    let tampered_payload = format!(
        "from={}\n\
to={}\n\
nonce=1\n\
state_hash=state:regression\n\
body=tampered-body-extended\n\
signer_public_key={}\n\
signature={}\n",
        envelope.from.as_str(),
        envelope.to.as_str(),
        envelope.signer_public_key,
        envelope.signature
    );
    assert_eq!(
        TcpSignedEnvelope::parse_wire_payload(tampered_payload.as_str()),
        Err(SdkError::InvalidInput {
            field: "signature",
            reason: "failed cryptographic envelope verification",
        })
    );
}

#[test]
fn regression_forged_handshake_frame_is_rejected() {
    let addr = free_addr();
    let listener_adapter = TcpTransportAdapter::new(
        TcpTransportConfig::new(addr.as_str())
            .unwrap_or_else(|error| panic!("listener config failed: {error}")),
    );
    let forged_payload = forged_handshake_payload();
    let listener_thread = listen_once_in_thread(listener_adapter);
    wait_for_listener();
    send_raw_payload(addr.as_str(), forged_payload.as_str());
    assert_eq!(
        join_listener(listener_thread),
        Err(SdkError::InvalidInput {
            field: "handshake.signature",
            reason: "does not match envelope signature",
        })
    );
}

fn forged_handshake_payload() -> String {
    let envelope = build_envelope(
        did("sender-forged"),
        did("listener-forged"),
        5,
        "state:forged",
        "forged-handshake-frame",
    );
    format!(
        "frame=handshake\n\
version=1\n\
profile=secp256k1:baseline-v2\n\
from={}\n\
to={}\n\
nonce={}\n\
signer_public_key={}\n\
signature=sig:secp256k1:baseline-v2:0:00\n\n{}",
        envelope.from,
        envelope.to,
        envelope.nonce,
        envelope.signer_public_key,
        envelope.to_wire_payload()
    )
}
