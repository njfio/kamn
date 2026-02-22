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
    assert!(envelope.signature.starts_with("sig:ed25519:baseline-v1:"));
}
