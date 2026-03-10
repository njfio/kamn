use super::super::support::managed_external_core_signer_env_guards;
use super::super::*;

#[test]
fn regression_kolme_live_managed_external_backend_response_accepts_case_variant_signer_public_key()
{
    // Regression: #6584
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
        "op-node-live-6584-provenance-case-match",
        "state:node-live-6584-provenance-case-match",
        "kamn:did:agent:node-live-6584-provenance-case-match",
        1,
        "payload:node-live-6584-provenance-case-match",
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
        Some(managed_pubkey.to_uppercase().as_str()),
    );
    let canonical_message =
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 58)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte(),
        managed_pubkey.to_uppercase(),
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":58,"account_id":"acct-6584-provenance-case-match"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let (_signed_wire_payload, signer_selection) = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect("case-equivalent managed-external signer provenance should verify");
    assert_eq!(signer_selection.key_source, "managed-external");
}

#[test]
fn regression_kolme_live_managed_external_backend_response_rejects_malformed_signer_public_key() {
    // Regression: #6584
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
        "op-node-live-6584-provenance-malformed",
        "state:node-live-6584-provenance-malformed",
        "kamn:did:agent:node-live-6584-provenance-malformed",
        1,
        "payload:node-live-6584-provenance-malformed",
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
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 59)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex=not-hex\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte(),
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":59,"account_id":"acct-6584-provenance-malformed"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("malformed backend signer public key must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_response_provenance_malformed")),
        "managed-external signer malformed provenance marker must preserve deterministic reason code"
    );
}
