use super::super::request_auth::{
    authorize_service_api_request, authorize_service_api_request_with_legacy_policy,
    require_valid_sender_did_header, resolve_signer_public_key_for_request,
    sender_did_matches_signer_public_key,
};
use super::super::*;
use super::support::{legacy_sender_request, test_service_api_runtime_state};

#[test]
fn regression_public_auth_entrypoint_rejects_legacy_sender_binding_without_signer_header() {
    let state = test_service_api_runtime_state();
    let request = legacy_sender_request(&state.snapshot);
    let mut replay_guard = ServiceApiReplayGuard::new(8, Duration::from_secs(60));

    let error = authorize_service_api_request(&state, &request, &mut replay_guard)
        .expect_err("public auth entrypoint should fail closed without signer header");
    let RequestAuthFailure::Unauthorized(error) = error else {
        panic!("expected unauthorized auth failure");
    };
    assert_eq!(
        error.reason_code,
        REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED
    );
    assert!(
        error
            .message
            .contains(REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER)
    );
}

#[test]
fn regression_explicit_legacy_auth_policy_still_allows_test_only_fallback_binding() {
    let state = test_service_api_runtime_state();
    let request = legacy_sender_request(&state.snapshot);
    let mut replay_guard = ServiceApiReplayGuard::new(8, Duration::from_secs(60));

    authorize_service_api_request_with_legacy_policy(&state, &request, &mut replay_guard, true)
        .expect("explicit legacy policy should remain available for targeted tests");
}

#[test]
fn unit_sender_did_binding_accepts_self_certifying_public_key_suffix() {
    let signer_public_key_hex =
        "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11";
    let sender_did = format!("kamn:did:agent:pkh-{signer_public_key_hex}");
    assert!(sender_did_matches_signer_public_key(
        sender_did.as_str(),
        signer_public_key_hex,
        false
    ));
}

#[test]
fn regression_sender_did_binding_accepts_case_variant_signer_public_key_header() {
    let signer_public_key_hex =
        "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11";
    let sender_did = format!("kamn:did:agent:pkh-{signer_public_key_hex}");
    assert!(sender_did_matches_signer_public_key(
        sender_did.as_str(),
        signer_public_key_hex.to_uppercase().as_str(),
        false
    ));
}

#[test]
fn regression_sender_did_binding_accepts_keyh_bound_pkh_did() {
    let signer_public_key_hex =
        "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11";
    let sender_did = AgentDid::with_public_key_hex_binding(
        format!("pkh-{signer_public_key_hex}").as_str(),
        signer_public_key_hex,
    )
    .expect("key-bound sender did should build");
    assert!(sender_did_matches_signer_public_key(
        sender_did.as_str(),
        signer_public_key_hex,
        false
    ));
}

#[test]
fn unit_sender_did_binding_rejects_self_certifying_key_mismatch() {
    let sender_did =
        "kamn:did:agent:pkh-02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11";
    assert!(!sender_did_matches_signer_public_key(
        sender_did,
        "03f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
        false
    ));
}

#[test]
fn regression_sender_did_binding_rejects_keyh_bound_pkh_did_mismatch() {
    let sender_did = AgentDid::with_public_key_hex_binding(
        "pkh-02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
        "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
    )
    .expect("key-bound sender did should build");
    assert!(!sender_did_matches_signer_public_key(
        sender_did.as_str(),
        "03f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
        false
    ));
}

#[test]
fn unit_sender_did_binding_rejects_legacy_did_without_legacy_policy() {
    assert!(!sender_did_matches_signer_public_key(
        "kamn:did:agent:alice",
        "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
        false
    ));
    assert!(sender_did_matches_signer_public_key(
        "kamn:did:agent:alice",
        "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
        true
    ));
}

#[test]
fn regression_signer_public_key_resolution_requires_header_without_legacy_policy() {
    let headers = BTreeMap::new();
    let error = resolve_signer_public_key_for_request(
        &headers,
        Some("02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11"),
        false,
    )
    .expect_err("production policy should require explicit signer public key header");
    let RequestAuthFailure::Unauthorized(error) = error else {
        panic!("expected unauthorized auth failure");
    };
    assert_eq!(
        error.reason_code,
        REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED
    );
    assert!(
        error
            .message
            .contains(REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER)
    );
}

#[test]
fn unit_signer_public_key_resolution_allows_legacy_fallback_when_enabled() {
    let headers = BTreeMap::new();
    let resolved = resolve_signer_public_key_for_request(
        &headers,
        Some("02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11"),
        true,
    )
    .expect("legacy policy should allow shared fallback key");
    assert_eq!(
        resolved,
        "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11"
    );
}

#[test]
fn regression_sender_did_header_rejects_legacy_did_shape() {
    let request = ParsedRequest {
        method: "POST".to_owned(),
        path: ROUTE_MESSAGES_SEND.to_owned(),
        body: "{}".to_owned(),
        headers: BTreeMap::from([(
            REQUEST_AUTH_SENDER_DID_HEADER.to_owned(),
            "did:kamn:agent:legacy-alpha".to_owned(),
        )]),
    };
    let error = require_valid_sender_did_header(&request)
        .expect_err("legacy did shape should fail closed at auth ingress");
    let RequestAuthFailure::Unauthorized(error) = error else {
        panic!("expected unauthorized auth failure");
    };
    assert_eq!(error.reason_code, REASON_CODE_AUTH_SENDER_DID_INVALID);
    assert!(error.message.contains("invalid sender did"));
}
