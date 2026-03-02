use kamn_agent_lib::envelope::{build_and_sign_envelope, CanonicalMessageEnvelope};
use kamn_agent_lib::identity::AgentIdentity;
use kamn_sdk::{
    service_public_key_for_private_key, service_signature_for_state_hash_with_private_key, AgentDid,
};

const TEST_ENVELOPE_SIGNING_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
const TEST_ENVELOPE_SIGNING_PRIVATE_KEY_HEX_ALT: &str =
    "094cf4e1f3d974bbf3e72233e2c2937e8fdb094740e0f017e010aa47ac1201ac";

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

    assert_eq!(envelope.from, identity.did().as_str());
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

#[test]
fn regression_envelope_verify_integrity_rejects_missing_did_key_binding_fingerprint() {
    // Regression: #6299
    let from = AgentDid::parse("kamn:did:agent:sender-unbound").expect("from did");
    let signer_public_key =
        service_public_key_for_private_key(TEST_ENVELOPE_SIGNING_PRIVATE_KEY_HEX)
            .expect("signer key");
    let signature = service_signature_for_state_hash_with_private_key(
        &from,
        17,
        "state:binding",
        "binding-check",
        TEST_ENVELOPE_SIGNING_PRIVATE_KEY_HEX,
    )
    .expect("signature");
    let envelope = CanonicalMessageEnvelope {
        from: from.to_string(),
        to: "kamn:did:agent:listener-bound".to_owned(),
        nonce: 17,
        state_hash: "state:binding".to_owned(),
        body: "binding-check".to_owned(),
        signer_public_key,
        signature,
    };

    let error = envelope
        .verify_integrity()
        .expect_err("missing did binding must fail closed");
    assert_eq!(
        error.to_string(),
        "invalid input for from: must include key-binding fingerprint matching signer_public_key"
    );
}

#[test]
fn regression_envelope_verify_integrity_rejects_mismatched_did_key_binding_fingerprint() {
    // Regression: #6299
    let signer_public_key =
        service_public_key_for_private_key(TEST_ENVELOPE_SIGNING_PRIVATE_KEY_HEX)
            .expect("signer key");
    let mismatched_binding_public_key =
        service_public_key_for_private_key(TEST_ENVELOPE_SIGNING_PRIVATE_KEY_HEX_ALT)
            .expect("alternate signer key");
    let from = AgentDid::with_public_key_hex_binding(
        "sender-bound-mismatch",
        mismatched_binding_public_key.as_str(),
    )
    .expect("from did");
    let signature = service_signature_for_state_hash_with_private_key(
        &from,
        18,
        "state:binding-mismatch",
        "binding-mismatch-check",
        TEST_ENVELOPE_SIGNING_PRIVATE_KEY_HEX,
    )
    .expect("signature");
    let envelope = CanonicalMessageEnvelope {
        from: from.to_string(),
        to: "kamn:did:agent:listener-bound".to_owned(),
        nonce: 18,
        state_hash: "state:binding-mismatch".to_owned(),
        body: "binding-mismatch-check".to_owned(),
        signer_public_key,
        signature,
    };

    let error = envelope
        .verify_integrity()
        .expect_err("mismatched did binding must fail closed");
    assert_eq!(
        error.to_string(),
        "invalid input for from: must include key-binding fingerprint matching signer_public_key"
    );
}

#[test]
fn integration_build_and_sign_envelope_emits_bound_sender_did() {
    // Integration: #6299
    let identity = AgentIdentity::from_agent_name("alice").expect("identity");
    let envelope = build_and_sign_envelope(
        &identity,
        "kamn:did:agent:listener-bound",
        "state:integration-bound",
        20,
        "integration-bound-check",
    )
    .expect("envelope");
    let from = AgentDid::parse(envelope.from.as_str()).expect("from did");

    assert!(
        from.key_binding_fingerprint().is_some(),
        "sender did must include key-binding fingerprint"
    );
    from.ensure_public_key_hex_binding(envelope.signer_public_key.as_str())
        .expect("sender did binding must match signer public key");
}

#[test]
fn integration_build_and_sign_envelope_rejects_unbound_sender_identity() {
    // Integration: #6299
    let identity = AgentIdentity::from_did_and_signing_key(
        "kamn:did:agent:sender-unbound",
        TEST_ENVELOPE_SIGNING_PRIVATE_KEY_HEX,
    )
    .expect("identity");
    let error = build_and_sign_envelope(
        &identity,
        "kamn:did:agent:listener-bound",
        "state:integration-unbound",
        21,
        "integration-unbound-check",
    )
    .expect_err("unbound sender identity must fail closed");

    assert_eq!(
        error.to_string(),
        "invalid input for from: must include key-binding fingerprint matching signer_public_key"
    );
}
