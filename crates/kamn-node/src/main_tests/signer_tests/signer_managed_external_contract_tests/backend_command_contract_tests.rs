use super::super::*;

#[test]
fn regression_kolme_live_managed_external_strict_contracts_require_backend_command_marker() {
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
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2432-missing-backend-command",
        "state:node-live-2432-missing-backend-command",
        "kamn:did:agent:node-live-2432-missing-backend-command",
        1,
        "payload:node-live-2432-missing-backend-command",
    )
    .expect("request should build");

    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":42,"account_id":"acct-2432"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        Some("ops-primary"),
        Some("managed-external"),
    )
    .expect_err("strict managed-external signer contracts must require backend command marker");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_required_missing")),
        "strict managed-external runtime path must fail closed with deterministic missing backend reason code"
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        0,
        "managed-external missing backend command must fail before nonce lookup"
    );
}

#[test]
fn regression_kolme_live_managed_external_requires_backend_command_without_required_marker() {
    // Regression: #2505
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
    let _required_marker_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED", None);
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2505-command-required",
        "state:node-live-2505-command-required",
        "kamn:did:agent:node-live-2505-command-required",
        1,
        "payload:node-live-2505-command-required",
    )
    .expect("request should build");
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":44,"account_id":"acct-2505-required"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed-external signer mode must require backend command marker");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_required_missing")),
        "managed-external signer mode without backend command must fail closed"
    );
}
