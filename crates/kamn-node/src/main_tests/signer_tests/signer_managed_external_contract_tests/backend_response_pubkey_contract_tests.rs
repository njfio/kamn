use super::super::support::managed_external_core_signer_env_guards;
use super::super::*;

#[test]
fn regression_kolme_live_managed_external_backend_response_requires_signer_public_key_marker() {
    // Regression: #2509
    let _lock = lock_signer_env_guard();
    let (_core_signer_key_guard, _core_service_key_guard) =
        managed_external_core_signer_env_guards();
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2509-provenance-required",
        "state:node-live-2509-provenance-required",
        "kamn:did:agent:node-live-2509-provenance-required",
        1,
        "payload:node-live-2509-provenance-required",
    )
    .expect("request should build");
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    let managed_pubkey = encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let _managed_signer_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_pubkey.as_str()),
    );
    let canonical_message =
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 45)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte()
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":45,"account_id":"acct-2509-provenance-required"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed-external backend response must include signer public key marker");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_response_provenance_missing")),
        "missing managed-external signer provenance marker must fail closed"
    );
}
