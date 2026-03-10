use super::super::*;

#[test]
fn regression_kolme_live_managed_external_required_marker_rejects_invalid_boolean() {
    // Regression: #2432
    let _lock = lock_signer_env_guard();
    let _required_marker_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED",
        Some("invalid-bool"),
    );
    assert!(
        matches!(
            resolve_kolme_live_managed_signer_required_marker(),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("managed_signer_backend_required_invalid")
        ),
        "managed signer required marker must reject non-boolean values"
    );
}

#[test]
fn regression_kolme_live_managed_external_required_marker_forces_backend_command() {
    // Regression: #2432
    let _lock = lock_signer_env_guard();
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _signer_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE).as_str()),
    );
    let _backend_command_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND", None);
    let _required_marker_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED", Some("true"));
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2432-required-marker",
        "state:node-live-2432-required-marker",
        "kamn:did:agent:node-live-2432-required-marker",
        1,
        "payload:node-live-2432-required-marker",
    )
    .expect("request should build");

    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":43,"account_id":"acct-2432-required"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed signer required marker must force backend command contract");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_required_missing")),
        "required marker should fail closed when backend command is absent"
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        0,
        "required-marker managed-external path must fail before nonce lookup"
    );
}
