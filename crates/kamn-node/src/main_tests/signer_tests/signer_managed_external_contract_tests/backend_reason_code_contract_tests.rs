use super::super::support::managed_external_core_signer_env_guards;
use super::super::*;

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
    let _lock = lock_signer_env_guard();
    let (_core_signer_key_guard, _core_service_key_guard) =
        managed_external_core_signer_env_guards();
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
    let _lock = lock_signer_env_guard();
    let (_core_signer_key_guard, _core_service_key_guard) =
        managed_external_core_signer_env_guards();
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
    let _lock = lock_signer_env_guard();
    let (_core_signer_key_guard, _core_service_key_guard) =
        managed_external_core_signer_env_guards();
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
    let _lock = lock_signer_env_guard();
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
