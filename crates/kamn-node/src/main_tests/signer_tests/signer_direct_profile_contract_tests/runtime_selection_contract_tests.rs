use super::super::support::managed_external_core_signer_env_guards;
use super::super::*;

#[test]
fn integration_runtime_kolme_live_renders_secondary_signer_selection_markers() {
    // Regression: #2241
    let _lock = lock_signer_env_guard();
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _secondary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":37,"account_id":"acct-live-secondary"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ef56ab78","finality":"final"}"#,
        ),
    ]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("kolme-live args should parse");
    let report = execute(parsed).expect("kolme-live execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"kolme_live_signer_profile\":\"ops-secondary\""));
    assert!(rendered.contains(
        "\"kolme_live_signer_private_key_env\":\"KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY\""
    ));
    assert!(rendered.contains("\"kolme_live_signer_key_source\":\"env-local\""));
}

#[test]
fn integration_runtime_kolme_live_renders_managed_external_signer_selection_markers() {
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
    let request = build_kolme_live_request(
        &bootstrap(NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "./data".to_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        })
        .expect("bootstrap plan should build"),
    )
    .expect("runtime commit request should build");
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
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 43)
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
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":43,"account_id":"acct-live-managed"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:aa11bb22","finality":"final"}"#,
        ),
    ]);
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-strict-signer-contracts".to_owned(),
        "--kolme-live-signer-profile".to_owned(),
        "ops-primary".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "managed-external".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];

    let parsed = parse_args(args).expect("kolme-live args should parse");
    let report = execute(parsed).expect("kolme-live execution should succeed");
    let rendered = render_bootstrap_report(&report, OutputMode::json());
    assert!(rendered.contains("\"kolme_live_signer_profile\":\"ops-primary\""));
    assert!(rendered.contains("\"kolme_live_signer_key_source\":\"managed-external\""));
    assert!(rendered.contains(
        "\"kolme_live_signer_private_key_env\":\"KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX\""
    ));
}
