use super::super::*;

#[test]
fn unit_kolme_live_signer_builds_direct_signed_wire_payload() {
    let _lock = lock_signer_env_guard();
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
    let _lock = lock_signer_env_guard();
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

    let adapter = KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
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
