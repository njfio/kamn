use super::support::*;

#[test]
fn unit_tcp_envelope_roundtrip_is_cryptographic() {
    let envelope = build_envelope(
        did("sender-1"),
        did("listener-1"),
        7,
        "state:runtime-7",
        "hello-runtime",
    );
    let wire = envelope.to_wire_payload();
    let parsed = TcpSignedEnvelope::parse_wire_payload(&wire)
        .unwrap_or_else(|error| panic!("envelope parse failed: {error}"));
    assert_eq!(parsed, envelope);
}

#[test]
fn unit_tcp_envelope_rejects_duplicate_keys() {
    let payload = "from=kamn:did:agent:sender-1\nfrom=kamn:did:agent:sender-1\nto=kamn:did:agent:listener-1\nnonce=1\nstate_hash=state:dup\nbody=dup\nsignature=sig:deterministic-v1:baseline-v1:kamn:did:agent:sender-1:1:state:dup:3\n";
    assert_eq!(
        TcpSignedEnvelope::parse_wire_payload(payload),
        Err(SdkError::InvalidInput {
            field: "wire_payload",
            reason: "duplicate key: from",
        })
    );
}

#[test]
fn regression_tcp_envelope_rejects_baseline_v1_deterministic_signature() {
    let from = did("sender-legacy");
    let to = did("listener-legacy");
    let signer_public_key = signer_public_key_hex();
    let signature = signature_for_fields(from.as_str(), 2, "state:legacy", "legacy-payload");
    let payload = format!(
        "from={}\n\
to={}\n\
nonce=2\n\
state_hash=state:legacy\n\
body=legacy-payload\n\
signer_public_key={signer_public_key}\n\
signature={signature}\n",
        from.as_str(),
        to.as_str()
    );
    assert_eq!(
        TcpSignedEnvelope::parse_wire_payload(payload.as_str()),
        Err(SdkError::InvalidInput {
            field: "signature",
            reason: "failed cryptographic envelope verification",
        })
    );
}

#[test]
fn regression_tcp_envelope_rejects_missing_did_key_binding_fingerprint() {
    let envelope = manual_binding_envelope(
        did_unbound("sender-unbound"),
        3,
        "state:binding",
        "binding-check",
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
    );
    assert_binding_failure(envelope);
}

#[test]
fn regression_tcp_envelope_rejects_mismatched_did_key_binding_fingerprint() {
    let from = AgentDid::with_public_key_hex_binding(
        "sender-bound-mismatch",
        signer_public_key_hex_for_private_key(TEST_TCP_SIGNING_PRIVATE_KEY_HEX_ALT).as_str(),
    )
    .expect("from did");
    let envelope = manual_binding_envelope(
        from,
        4,
        "state:binding-mismatch",
        "binding-mismatch-check",
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
    );
    assert_binding_failure(envelope);
}

#[test]
fn integration_tcp_parse_rejects_unbound_sender_did_key_binding() {
    let payload = unbound_sender_payload();
    assert_eq!(
        TcpSignedEnvelope::parse_wire_payload(payload.as_str()),
        Err(binding_failure_error())
    );
}

fn assert_binding_failure(envelope: TcpSignedEnvelope) {
    assert_eq!(envelope.verify_integrity(), Err(binding_failure_error()));
}

fn binding_failure_error() -> SdkError {
    SdkError::InvalidInput {
        field: "from",
        reason: "must include key-binding fingerprint matching signer_public_key",
    }
}

fn manual_binding_envelope(
    from: AgentDid,
    nonce: u64,
    state_hash: &str,
    body: &str,
    signature_key: &str,
    signer_key: &str,
) -> TcpSignedEnvelope {
    TcpSignedEnvelope {
        signature: service_auth_sign_with_private_key_hex(
            from.as_str(),
            nonce,
            state_hash,
            body,
            signature_key,
        )
        .expect("signature"),
        signer_public_key: signer_public_key_hex_for_private_key(signer_key),
        from,
        to: did("listener-bound"),
        nonce,
        state_hash: state_hash.to_owned(),
        body: body.to_owned(),
    }
}

fn unbound_sender_payload() -> String {
    let from = did_unbound("sender-wire-unbound");
    let to = did("listener-wire-bound");
    let from_value = from.as_str();
    let to_value = to.as_str();
    let signature = service_auth_sign_with_private_key_hex(
        from_value,
        5,
        "state:wire-binding",
        "wire-binding-check",
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
    )
    .expect("signature");
    let signer_public_key = signer_public_key_hex_for_private_key(TEST_TCP_SIGNING_PRIVATE_KEY_HEX);
    format!(
        "from={from_value}\nto={to_value}\nnonce=5\nstate_hash=state:wire-binding\nbody=wire-binding-check\nsigner_public_key={signer_public_key}\nsignature={signature}\n"
    )
}
