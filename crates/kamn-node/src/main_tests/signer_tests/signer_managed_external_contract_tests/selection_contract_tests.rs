use super::super::support::managed_external_core_signer_env_guards;
use super::super::*;

#[test]
fn integration_kolme_live_managed_external_adapter_provenance_consumed_by_signer_selection() {
    // Regression: #2323
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
        "op-node-live-2323",
        "state:node-live-2323",
        "kamn:did:agent:node-live-2323",
        1,
        "payload:node-live-2323",
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
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 41)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte(),
        managed_pubkey,
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );

    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":41,"account_id":"acct-2323"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let (signed_wire_payload, signer_selection) = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        Some("ops-primary"),
        Some("managed-external"),
    )
    .expect("managed-external signing should succeed through secure backend route");
    assert_eq!(signer_selection.profile, "ops-primary");
    assert_eq!(signer_selection.key_source, "managed-external");
    assert_eq!(
        signer_selection.private_key_env,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
    );
    let signature = extract_json_string_field(signed_wire_payload.as_str(), "signature")
        .expect("direct signed payload must include signature field");
    assert_eq!(signature.len(), 128);
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        1,
        "managed-external signing should issue one nonce lookup before payload emission"
    );
}
