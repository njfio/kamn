use super::*;

#[test]
fn unit_kolme_live_signer_builds_direct_signed_wire_payload() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2197",
        "state:node-live-2197",
        "kamn:did:agent:node-live-2197",
        1,
        "payload:node-live-2197",
    )
    .expect("request should build");

    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":22,"account_id":"acct-2197"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let (signed_wire_payload, signer_selection) = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        None,
    )
    .expect("signed payload should be produced");

    assert_eq!(signer_selection.profile, "ops-primary");
    assert_eq!(
        signer_selection.private_key_env,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
    );
    assert_eq!(signer_selection.key_source, "env-local");
    assert!(signed_wire_payload.contains("\"message\":\"{\\\"pubkey\\\":"));
    let signature = extract_json_string_field(signed_wire_payload.as_str(), "signature")
        .expect("direct signed payload must include signature field");
    assert_eq!(
        signature.len(),
        128,
        "secp256k1 signature must be 64 bytes hex"
    );
    assert!(
        signature
            .as_bytes()
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "signature must be lowercase hex"
    );
}

#[test]
fn unit_kolme_live_signer_adapter_signs_and_verifies_runtime_message() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let message = "{\"pubkey\":\"pk-adapter\",\"nonce\":7,\"created\":\"2026-02-12T00:00:00Z\",\"messages\":[]}";
    let (adapter, selection) =
        build_kolme_live_signer_adapter(None, None).expect("adapter should build");
    assert_eq!(selection.profile, "ops-primary");
    let (signature_hex, recovery_id) = adapter
        .sign_message(message)
        .expect("adapter signing should succeed");
    assert_eq!(signature_hex.len(), 128);
    assert!(
        signature_hex
            .as_bytes()
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "adapter signature must be lowercase hex"
    );
    adapter
        .verify_message(message, signature_hex.as_str(), recovery_id)
        .expect("adapter signature verification should succeed");
}

#[test]
fn regression_kolme_live_signer_adapter_rejects_malformed_signature_hex() {
    // Regression: #2297
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _env_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let (adapter, _selection) =
        build_kolme_live_signer_adapter(None, None).expect("adapter should build");
    assert!(
        matches!(
            adapter.verify_message(
                "{\"pubkey\":\"pk-adapter\",\"nonce\":7,\"created\":\"2026-02-12T00:00:00Z\",\"messages\":[]}",
                "zz",
                0,
            ),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("runtime_commit_signature_hex contains invalid hex character")
        ),
        "malformed signature hex must fail closed in adapter verification"
    );
}

#[test]
fn regression_kolme_live_signer_adapter_rejects_recovered_key_mismatch() {
    // Regression: #2297
    let primary = super::KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    )
    .expect("primary adapter should build");
    let secondary = super::KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
    )
    .expect("secondary adapter should build");
    let message = "{\"pubkey\":\"pk-adapter\",\"nonce\":9,\"created\":\"2026-02-12T00:00:00Z\",\"messages\":[]}";
    let (signature_hex, recovery_id) = primary
        .sign_message(message)
        .expect("primary adapter signature should succeed");
    assert!(
        matches!(
            secondary.verify_message(message, signature_hex.as_str(), recovery_id),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("recovered public key does not match signer selection")
        ),
        "signature verification must fail closed when recovered key mismatches signer adapter key"
    );
}

#[test]
fn regression_signer_private_key_parse_path_requires_zeroize_markers() {
    // Regression: #2672
    const SIGNER_SOURCE: &str = include_str!("../signer.rs");
    assert!(
        SIGNER_SOURCE.contains("private_key_hex.zeroize()"),
        "signer private key hex buffers must be explicitly zeroized after parsing"
    );
    assert!(
        SIGNER_SOURCE.contains("private_key_bytes.zeroize()"),
        "decoded signer private key byte buffers must be explicitly zeroized after key setup"
    );
}

#[test]
fn regression_live_signer_vector_probe_must_not_be_ignored() {
    const SOURCE: &str = include_str!("signer_tests.rs");
    let lines: Vec<&str> = SOURCE.lines().collect();
    let fn_line = lines
        .iter()
        .position(|line| {
            line.trim() == "fn integration_kolme_live_signer_vector_probe_contract() {"
        })
        .expect("live signer vector probe function must exist");
    let mut attributes = Vec::new();
    let mut cursor = fn_line;
    while cursor > 0 {
        cursor -= 1;
        let trimmed = lines[cursor].trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("#[") {
            attributes.push(trimmed);
            continue;
        }
        break;
    }

    assert!(
        attributes.iter().all(|line| !line.contains("ignore")),
        "live signer vector probe must stay active; local-heavy gating belongs in runtime preflight, not #[ignore]"
    );
}

#[test]
fn integration_kolme_live_signer_vector_probe_contract() {
    if env::var("KAMN_KOLME_LOCAL_HEAVY").ok().as_deref() != Some("1") {
        eprintln!(
            "skipping signer vector probe; set KAMN_KOLME_LOCAL_HEAVY=1 to run local-heavy parity probe"
        );
        return;
    }

    let private_key_hex = env::var("KAMN_KOLME_SIGNATURE_VECTOR_PRIVATE_KEY_HEX")
        .expect("KAMN_KOLME_SIGNATURE_VECTOR_PRIVATE_KEY_HEX must be set");
    let message = env::var("KAMN_KOLME_SIGNATURE_VECTOR_MESSAGE")
        .expect("KAMN_KOLME_SIGNATURE_VECTOR_MESSAGE must be set");

    let adapter = super::KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        private_key_hex.as_str(),
        "KAMN_KOLME_SIGNATURE_VECTOR_PRIVATE_KEY_HEX",
    )
    .expect("signature parity adapter should build");
    let (signature_hex, recovery_id) = adapter
        .sign_message(message.as_str())
        .expect("signature parity adapter signing should succeed");
    let pubkey_hex = adapter.public_key_compressed_hex();

    println!("signature_hex={signature_hex}");
    println!("recovery_id={recovery_id}");
    println!("pubkey_hex={pubkey_hex}");

    if let Ok(expected_signature_hex) =
        env::var("KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_SIGNATURE_HEX")
    {
        assert_eq!(
            signature_hex, expected_signature_hex,
            "signature parity probe must match expected signature vector"
        );
    }
    if let Ok(expected_recovery_id) = env::var("KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_RECOVERY_ID") {
        let expected_recovery_id = expected_recovery_id
            .parse::<u8>()
            .expect("expected recovery id must parse as u8");
        assert_eq!(
            recovery_id, expected_recovery_id,
            "signature parity probe must match expected recovery id vector"
        );
    }
    if let Ok(expected_pubkey_hex) = env::var("KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_PUBKEY_HEX") {
        assert_eq!(
            pubkey_hex, expected_pubkey_hex,
            "signature parity probe must match expected pubkey vector"
        );
    }
}

#[test]
fn unit_kolme_live_signer_profile_defaults_to_primary_key_env() {
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", None);

    let (profile, env_name) = resolve_kolme_live_signer_private_key_env_name(None)
        .expect("default profile selection should succeed");
    assert_eq!(profile, "ops-primary");
    assert_eq!(env_name, "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX");
}

#[test]
fn regression_kolme_live_signer_profile_rejects_unsupported_value() {
    // Regression: #2222
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
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
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
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

#[test]
fn integration_runtime_kolme_live_renders_secondary_signer_selection_markers() {
    // Regression: #2241
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
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
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
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

#[test]
fn unit_kolme_live_native_direct_message_contains_required_fields() {
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2207",
        "state:node-live-2207",
        "kamn:did:agent:node-live-2207",
        1,
        "payload:node-live-2207",
    )
    .expect("request should build");

    let message = render_kolme_live_native_direct_message(
        &request,
        "02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344",
        19,
    )
    .expect("native direct message should render");

    assert!(message.contains(
        "\"pubkey\":\"02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344\""
    ));
    assert!(message.contains("\"nonce\":19"));
    assert!(message.contains("\"created\":\""));
    assert!(message.contains("\"messages\":["));
}

#[test]
fn integration_kolme_live_nonce_resolver_fetches_next_nonce() {
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":27,"account_id":"acct-2207"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let signer_adapter = super::KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    )
    .expect("deterministic signer adapter should build");
    let pubkey = signer_adapter.public_key_compressed_hex();

    let nonce = resolve_kolme_live_nonce(base_url.as_str(), &mut transport, pubkey.as_str())
        .expect("nonce should resolve");
    assert_eq!(nonce, 27);

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(recorded_requests.len(), 1);
    assert!(recorded_requests[0].contains("GET /get-next-nonce?pubkey="));
}

#[test]
fn integration_kolme_live_nonce_resolver_retries_unavailable_then_succeeds() {
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"nonce unavailable\"}".to_owned(),
        },
        MockHttpReply::ok(r#"{"next_nonce":29,"account_id":"acct-2207"}"#),
    ]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let pubkey = "03c9e9fd7028a8b17f4fbe0f6f7d38af2ec527f6bb2af04d4d2e2b0eb4f1f01b8a";

    let nonce = resolve_kolme_live_nonce(base_url.as_str(), &mut transport, pubkey)
        .expect("nonce resolver should recover from transient unavailable response");
    assert_eq!(nonce, 29);

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        2,
        "nonce resolver should retry once after unavailable response"
    );
}

#[test]
fn regression_kolme_live_nonce_resolver_rejects_malformed_response() {
    // Regression: #2207
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":0,"account_id":"acct-2207"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = resolve_kolme_live_nonce(
        base_url.as_str(),
        &mut transport,
        "02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344",
    )
    .expect_err("invalid nonce payload must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("nonce response malformed")),
        "expected fail-closed nonce parser error"
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        1,
        "malformed nonce responses must fail fast without retry"
    );
}

#[test]
fn regression_kolme_live_signer_requires_primary_key_env_value() {
    // Regression: #2222
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    assert!(
        matches!(
            build_kolme_live_signer_adapter(None, None),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX must be set")
        ),
        "missing primary signer private key env must fail closed"
    );
}

#[test]
fn regression_issue_2279_kolme_live_signer_rejects_fallback_private_key_env_path() {
    // Regression: #2279
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _primary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let _fallback_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
    );
    assert!(
        matches!(
            build_kolme_live_signer_adapter(Some("ops-primary"), Some("env-local")),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("fallback_signer_secret_present_violation")
        ),
        "fallback signer private key env path must fail closed"
    );
}

#[test]
fn regression_kolme_live_managed_external_requires_key_reference_env_marker() {
    // Regression: #2322
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_KEY_REF", None);
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    assert!(
        matches!(
            build_kolme_live_signer_adapter(Some("ops-primary"), Some("managed-external")),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("managed_signer_key_reference_missing")
        ),
        "managed-external strict signer selection must require key reference env marker"
    );
}

#[test]
fn regression_kolme_live_managed_external_rejects_invalid_key_reference_schema() {
    // Regression: #2322
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_KEY_REF", Some("invalid:key-ref"));
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    assert!(
        matches!(
            build_kolme_live_signer_adapter(Some("ops-primary"), Some("managed-external")),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("managed_signer_key_reference_invalid")
        ),
        "invalid managed-external key reference schema must fail closed"
    );
}

#[test]
fn regression_kolme_live_managed_external_rejects_raw_private_key_env_path() {
    // Regression: #2322
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    assert!(
        matches!(
            build_kolme_live_signer_adapter(Some("ops-primary"), Some("managed-external")),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("managed_signer_raw_private_key_forbidden")
        ),
        "managed-external strict signer selection must reject raw private key env path"
    );
}

#[test]
fn regression_kolme_live_managed_external_strict_contracts_require_backend_command_marker() {
    // Regression: #2432
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
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
fn regression_kolme_live_managed_external_required_marker_rejects_invalid_boolean() {
    // Regression: #2432
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
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
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
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

#[test]
fn regression_kolme_live_managed_external_requires_backend_command_without_required_marker() {
    // Regression: #2505
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
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

#[test]
fn integration_kolme_live_managed_external_builds_direct_signed_wire_payload() {
    // Regression: #2323
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
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

#[test]
fn regression_kolme_live_managed_external_backend_response_requires_signer_public_key_marker() {
    // Regression: #2509
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
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

#[test]
fn regression_kolme_live_managed_external_requires_runtime_signer_public_key_marker() {
    // Regression: #2512
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _signer_pubkey_marker_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX", None);
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2512-pubkey-marker-missing",
        "state:node-live-2512-pubkey-marker-missing",
        "kamn:did:agent:node-live-2512-pubkey-marker-missing",
        1,
        "payload:node-live-2512-pubkey-marker-missing",
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
    let canonical_message =
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 47)
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
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":47,"account_id":"acct-2512-pubkey-marker-missing"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed-external runtime path must require signer public key marker");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_public_key_marker_missing")),
        "missing managed-external signer public key marker must fail closed"
    );
}

#[test]
fn regression_kolme_live_managed_external_rejects_invalid_runtime_signer_public_key_marker() {
    // Regression: #2512
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _signer_pubkey_marker_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some("invalid-pubkey-marker"),
    );
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2512-pubkey-marker-invalid",
        "state:node-live-2512-pubkey-marker-invalid",
        "kamn:did:agent:node-live-2512-pubkey-marker-invalid",
        1,
        "payload:node-live-2512-pubkey-marker-invalid",
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
    let canonical_message =
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 48)
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
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":48,"account_id":"acct-2512-pubkey-marker-invalid"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("invalid managed-external signer public key marker must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_public_key_marker_invalid")),
        "invalid managed-external signer public key marker must fail closed with deterministic reason code"
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch() {
    // Regression: #2509
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
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
        "op-node-live-2509-provenance-mismatch",
        "state:node-live-2509-provenance-mismatch",
        "kamn:did:agent:node-live-2509-provenance-mismatch",
        1,
        "payload:node-live-2509-provenance-mismatch",
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
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 46)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    let secondary_key =
        build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE_SECONDARY)
            .expect("secondary managed signing key should derive");
    let secondary_pubkey = encode_kolme_hex_lower(
        secondary_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let backend_command = format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte(),
        secondary_pubkey,
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(backend_command.as_str()),
    );
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":46,"account_id":"acct-2509-provenance-mismatch"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        None,
        Some("managed-external"),
    )
    .expect_err("managed-external backend response must reject signer public key mismatch");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_response_provenance_mismatch")),
        "managed-external signer provenance mismatch must fail closed"
    );
}

#[test]
fn regression_kolme_live_managed_external_maps_provider_unavailable_reason_code() {
    // Regression: #2323
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2323-provider",
        "state:node-live-2323-provider",
        "kamn:did:agent:node-live-2323-provider",
        1,
        "payload:node-live-2323-provider",
    )
    .expect("request should build");
    let expected_signer_public_key_hex = encode_kolme_hex_lower(
        build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
            .expect("managed signing key should derive")
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let error = sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &request,
        1,
        "payload:managed-signature",
        SignerProviderHandshakeMatrix::with_uniform_availability(false),
        expected_signer_public_key_hex.as_str(),
    )
    .expect_err("managed-external provider unavailability must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_provider_unavailable")),
        "managed-external provider unavailability must map to deterministic reason code"
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_timeout_maps_reason_code() {
    // Regression: #2423
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _backend_command_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND", Some("sleep 2"));
    let _backend_timeout_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("1"));
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2423-timeout",
        "state:node-live-2423-timeout",
        "kamn:did:agent:node-live-2423-timeout",
        1,
        "payload:node-live-2423-timeout",
    )
    .expect("request should build");
    let expected_signer_public_key_hex = encode_kolme_hex_lower(
        build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
            .expect("managed signing key should derive")
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let error = sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &request,
        1,
        "payload:managed-signature",
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        expected_signer_public_key_hex.as_str(),
    )
    .expect_err("managed-external backend timeout must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_timeout")),
        "managed-external backend timeout must map to deterministic reason code"
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_malformed_response_maps_reason_code() {
    // Regression: #2423
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some("printf 'signature_hex=zzzz\\nrecovery_id=9\\nsigner_public_key_hex=03af446f76cf36092a4e45864210a1dbf03e872756eec21de61910859f8a607dd2\\n'"),
    );
    let _backend_timeout_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("5"));
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2423-malformed",
        "state:node-live-2423-malformed",
        "kamn:did:agent:node-live-2423-malformed",
        1,
        "payload:node-live-2423-malformed",
    )
    .expect("request should build");
    let expected_signer_public_key_hex = encode_kolme_hex_lower(
        build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
            .expect("managed signing key should derive")
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let error = sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &request,
        1,
        "payload:managed-signature",
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        expected_signer_public_key_hex.as_str(),
    )
    .expect_err("managed-external backend malformed response must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_response_malformed")),
        "managed-external backend malformed response must map to deterministic reason code"
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_unavailable_maps_reason_code() {
    // Regression: #2423
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some("this-command-should-not-exist-2423"),
    );
    let _backend_timeout_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("5"));
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2423-unavailable",
        "state:node-live-2423-unavailable",
        "kamn:did:agent:node-live-2423-unavailable",
        1,
        "payload:node-live-2423-unavailable",
    )
    .expect("request should build");
    let expected_signer_public_key_hex = encode_kolme_hex_lower(
        build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
            .expect("managed signing key should derive")
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    );
    let error = sign_kolme_live_managed_external_message(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        &request,
        1,
        "payload:managed-signature",
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        expected_signer_public_key_hex.as_str(),
    )
    .expect_err("managed-external backend unavailability must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_backend_unavailable")),
        "managed-external backend unavailability must map to deterministic reason code"
    );
}

#[test]
fn regression_kolme_live_managed_external_adapter_retired_not_integrated_marker() {
    // Regression: #2423
    let _lock = signer_env_lock()
        .lock()
        .expect("signer env lock should guard test mutation");
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let error = build_kolme_live_signer_adapter(Some("ops-primary"), Some("managed-external"))
        .expect_err("managed-external private-key adapter path must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if !message.contains("managed_signer_backend_path_not_integrated")),
        "managed-external signer adapter path must retire not-integrated marker"
    );
}
