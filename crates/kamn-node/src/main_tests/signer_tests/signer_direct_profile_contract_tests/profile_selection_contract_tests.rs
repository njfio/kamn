use super::super::*;

#[test]
fn unit_kolme_live_signer_profile_defaults_to_primary_key_env() {
    let _lock = lock_signer_env_guard();
    let _profile_env_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", None);

    let (profile, env_name) = resolve_kolme_live_signer_private_key_env_name(None)
        .expect("default profile selection should succeed");
    assert_eq!(profile, "ops-primary");
    assert_eq!(env_name, "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX");
}
#[test]
fn regression_kolme_live_signer_profile_rejects_unsupported_value() {
    // Regression: #2222
    let _lock = lock_signer_env_guard();
    let _profile_env_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("legacy"));
    assert!(
        matches!(
            resolve_kolme_live_signer_private_key_env_name(None),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("KAMN_KOLME_LIVE_SIGNER_PROFILE has unsupported profile")
        ),
        "unsupported signer profile must fail closed"
    );
}
#[test]
fn integration_kolme_live_signer_profile_secondary_uses_secondary_key_env() {
    let _lock = lock_signer_env_guard();
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _secondary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
    );
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2222",
        "state:node-live-2222",
        "kamn:did:agent:node-live-2222",
        1,
        "payload:node-live-2222",
    )
    .expect("request should build");

    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":31,"account_id":"acct-2222"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let (signed_wire_payload, signer_selection) = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        None,
    )
    .expect("secondary profile signing should succeed");
    assert_eq!(signer_selection.profile, "ops-secondary");
    assert_eq!(
        signer_selection.private_key_env,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
    );
    assert_eq!(signer_selection.key_source, "env-local");
    let signature = extract_json_string_field(signed_wire_payload.as_str(), "signature")
        .expect("direct signed payload must include signature field");
    assert_eq!(signature.len(), 128);
}
