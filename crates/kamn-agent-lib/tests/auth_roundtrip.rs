use kamn_agent_lib::auth::KamnAuthHeaders;
use kamn_agent_lib::identity::AgentIdentity;

#[test]
fn spec_c04_auth_roundtrip_builds_kamn_headers() {
    let identity = AgentIdentity::from_agent_name("alice").expect("identity");
    let headers = KamnAuthHeaders::build(
        identity.did().as_str(),
        identity.signing_key(),
        7,
        "state-hash-123",
        br#"{"message":"hello"}"#,
        Some("messages:send"),
    )
    .expect("headers");

    assert_eq!(headers.sender_did_header, identity.did().as_str());
    assert_eq!(headers.nonce_header, "7");
    assert_eq!(headers.authz_scope_header.as_deref(), Some("messages:send"));
    assert!(headers.signature_header.starts_with("sig:ed25519:baseline-v1:"));
}
