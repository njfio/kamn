const DOC: &str = include_str!("../../../docs/foundation/signer-backend-abstraction.md");

#[test]
fn doc_contains_signer_backend_contract_and_router_rules() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("SigningRequest"));
    assert!(DOC.contains("SignerBackend"));
    assert!(DOC.contains("LocalSignerBackend"));
    assert!(DOC.contains("SecureSignerBackend"));
    assert!(DOC.contains("sign_with_secure_fallback"));
}

#[test]
fn doc_contains_fallback_semantics_and_transaction_integration() {
    assert!(DOC.contains("## Backend Compatibility Rules"));
    assert!(DOC.contains("provider handshake matrix statuses"));
    assert!(DOC.contains("ProviderHandshakeRejected"));
    assert!(DOC.contains(
        "falls back from secure to local only for `ProviderUnavailable` and `operator` role keys."
    ));
    assert!(DOC.contains(
        "fallback is denied for handshake policy blocks (`ProviderHandshakeRejected`) (`Regression: #677`)."
    ));
    assert!(DOC.contains("does not fallback on hard request errors"));
    assert!(DOC.contains("## Transaction Path Integration"));
    assert!(DOC.contains("SigningRequest::for_transaction(...)"));
    assert!(DOC.contains("baseline_signature_for_fields(...)"));
    assert!(DOC.contains("signature_profile_compatibility_fixtures_for_fields(...)"));
    assert!(DOC.contains("legacy-unversioned"));
    assert!(DOC.contains("baseline-v0"));
    assert!(DOC.contains("secp256k1+baseline-v1"));
    assert!(DOC.contains("baseline signature algorithm: `ed25519`."));
    assert!(DOC.contains("baseline signature profile id: `baseline-v1`"));
    assert!(DOC.contains("parse_signature_profile_metadata(...)"));
}

#[test]
fn doc_contains_signer_emulator_contract_lane_policy() {
    assert!(DOC.contains("## Signer Emulator Contract Lanes"));
    assert!(DOC.contains("bash scripts/signer/run_signer_emulator_contract_lane.sh"));
    assert!(DOC.contains("bash scripts/signer/run_signer_provider_deep_lane.sh"));
    assert!(DOC.contains(
        "functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider"
    ));
    assert!(DOC.contains("regression_provider_handshake_policy_block_rejects_without_fallback"));
    assert!(DOC.contains(
        "integration_signature_profile_fixture_matrix_remains_consistent_with_transaction_guards"
    ));
}

#[test]
fn doc_contains_production_style_secure_provider_adapter_rules() {
    assert!(DOC.contains("secure-aws-kms-emulator"));
    assert!(DOC.contains("secure:aws-kms:<key-ref>"));
    assert!(DOC.contains("role-scoped production keys"));
    assert!(DOC.contains("KeyRoleMismatch"));
    assert!(DOC.contains("FallbackDeniedByRolePolicy"));
    assert!(DOC.contains("UnsupportedSecureProvider"));
    assert!(DOC.contains("UnsupportedSignerKeyRole"));
    assert!(DOC.contains("MalformedSecureKeyReference"));
}

#[test]
fn regression_requires_no_fallback_on_unsupported_secure_key_reference() {
    // Regression: #160
    assert!(DOC.contains("does not fallback on hard request errors"));
    assert!(DOC.contains("canonical signature-profile helper consumed by both paths"));
    assert!(DOC.contains("non-versioned signature profile is rejected (`Regression: #404`)"));
    assert!(DOC.contains("algorithm/profile drift is rejected (`Regression: #677`)."));
    assert!(DOC.contains(
        "signer and transaction compatibility fixture matrix decisions stay aligned (`Regression: #677`)."
    ));
    assert!(DOC.contains("Contract lane guards remain required for signer provider compatibility (`Regression: #619`)."));
    assert!(DOC.contains(
        "fallback is denied for handshake policy blocks (`ProviderHandshakeRejected`) (`Regression: #677`)."
    ));
    assert!(DOC.contains("SecureProviderBackendMismatch"));
}
