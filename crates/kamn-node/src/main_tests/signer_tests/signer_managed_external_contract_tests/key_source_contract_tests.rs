use super::super::*;

#[test]
fn regression_kolme_live_managed_external_requires_key_reference_env_marker() {
    // Regression: #2322
    let _lock = lock_signer_env_guard();
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
fn regression_kolme_live_signer_preflight_rejects_missing_managed_key_reference() {
    // Regression: #3539
    let _lock = lock_signer_env_guard();
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _key_ref_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_KEY_REF", None);
    let _managed_public_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE).as_str()),
    );
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some("printf 'signature_hex=00\\nrecovery_id=0\\nsigner_public_key_hex=02\\n'"),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _fallback_key_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);

    let error = enforce_kolme_live_signer_preflight(Some("ops-primary"), Some("managed-external"))
        .expect_err("managed-external preflight must require key reference marker");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_key_reference_missing")),
        "preflight should fail closed with deterministic key-reference marker reason"
    );
}

#[test]
fn regression_kolme_live_managed_external_rejects_invalid_key_reference_schema() {
    // Regression: #2322
    let _lock = lock_signer_env_guard();
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
    let _lock = lock_signer_env_guard();
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
            enforce_kolme_live_signer_preflight(Some("ops-primary"), Some("managed-external")),
            Err(ConfigError::RuntimeKolmeLive(message))
            if message.contains("managed_signer_raw_private_key_forbidden")
        ),
        "managed-external strict signer selection must reject raw private key env path"
    );
}
