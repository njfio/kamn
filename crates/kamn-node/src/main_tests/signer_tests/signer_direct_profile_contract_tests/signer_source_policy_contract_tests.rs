use super::super::*;

#[test]
fn regression_kolme_live_signer_adapter_rejects_malformed_signature_hex() {
    // Regression: #2297
    let _lock = lock_signer_env_guard();
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
    let primary = KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    )
    .expect("primary adapter should build");
    let secondary = KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
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
    const SIGNER_ADAPTER_SOURCE: &str = include_str!("../../../signer/signer_adapter.rs");
    assert!(
        SIGNER_ADAPTER_SOURCE.contains("private_key_hex.zeroize()"),
        "signer private key hex buffers must be explicitly zeroized after parsing"
    );
    assert!(
        SIGNER_ADAPTER_SOURCE.contains("private_key_bytes.zeroize()"),
        "decoded signer private key byte buffers must be explicitly zeroized after key setup"
    );
    assert!(
        SIGNER_ADAPTER_SOURCE.contains("key_material.zeroize()"),
        "managed signer key material buffers must be explicitly zeroized after key setup"
    );
}

#[test]
fn regression_signer_secret_source_precedence_path_requires_zeroize_markers() {
    // Regression: #4165
    const SIGNER_SOURCE: &str = include_str!("../../../signer.rs");
    assert!(
        SIGNER_SOURCE
            .contains("ensure_kolme_live_strict_signer_secret_source_precedence_and_zeroize"),
        "signer source precedence path must route through explicit zeroization helper"
    );
    assert!(
        SIGNER_SOURCE.contains("private_key_hex.zeroize()"),
        "signer source precedence helper must explicitly zeroize env-secret buffers"
    );
}

#[test]
fn regression_live_signer_vector_probe_must_not_be_ignored() {
    const SOURCE: &str = include_str!("direct_signature_contract_tests.rs");
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
