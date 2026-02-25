use kamn_agent_lib::envelope::build_and_sign_envelope;
use kamn_agent_lib::identity::AgentIdentity;

#[test]
fn spec_c05_envelope_construction_stable_signature_and_nonce() {
    let identity = AgentIdentity::from_agent_name("alice").expect("identity");
    let envelope = build_and_sign_envelope(
        &identity,
        "kamn:did:agent:bob",
        "state-hash-123",
        9,
        "hello world",
    )
    .expect("envelope");

    assert_eq!(envelope.from, "kamn:did:agent:alice");
    assert_eq!(envelope.to, "kamn:did:agent:bob");
    assert_eq!(envelope.nonce, 9);
    assert_eq!(envelope.state_hash, "state-hash-123");
    assert_eq!(envelope.body, "hello world");
    assert_eq!(envelope.signer_public_key.len(), 66);
    assert!(envelope.signature.starts_with("sig:secp256k1:baseline-v2:"));
}

#[test]
fn spec_c05_envelope_construction_rejects_tampered_signature_payload() {
    let identity = AgentIdentity::from_agent_name("alice").expect("identity");
    let mut envelope = build_and_sign_envelope(
        &identity,
        "kamn:did:agent:bob",
        "state-hash-123",
        10,
        "hello world",
    )
    .expect("envelope");
    envelope.signature.push('f');

    let error = envelope
        .verify_integrity()
        .expect_err("tampered signature must fail closed");
    assert_eq!(
        error.to_string(),
        "invalid input for signature: does not match canonical envelope fields"
    );
}

#[test]
fn spec_c05_envelope_construction_rejects_invalid_signer_public_key() {
    let identity = AgentIdentity::from_agent_name("alice").expect("identity");
    let mut envelope = build_and_sign_envelope(
        &identity,
        "kamn:did:agent:bob",
        "state-hash-123",
        10,
        "hello world",
    )
    .expect("envelope");
    envelope.signer_public_key = "not-a-valid-public-key".to_owned();
    let error = envelope
        .verify_integrity()
        .expect_err("invalid signer public key must fail closed");
    assert_eq!(
        error.to_string(),
        "invalid input for signer_public_key: must be valid compressed secp256k1 public key hex"
    );
}
