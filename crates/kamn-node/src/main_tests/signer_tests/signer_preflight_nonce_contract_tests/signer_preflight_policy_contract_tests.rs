use super::super::*;

#[test]
fn integration_kolme_live_signer_preflight_rejects_non_failover_rotation_regression() {
    // Regression: #3956
    let _lock = lock_signer_env_guard();
    let _profile_env_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _primary_key_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    );
    let _fallback_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let _previous_profile_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
        Some("ops-primary"),
    );
    let _rotation_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", Some("1"));
    let _previous_rotation_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", Some("2"));
    let _required_approvals_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
        Some("1"),
    );
    let _approved_signers_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
        Some("ops-primary"),
    );

    let error = enforce_kolme_live_signer_preflight(Some("ops-primary"), Some("env-local"))
        .expect_err("non-failover rotation epoch regression must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_rotation_epoch_regressed")),
        "non-failover stale metadata must preserve deterministic regression reason code"
    );
}
#[test]
fn regression_kolme_live_signer_requires_primary_key_env_value() {
    // Regression: #2222
    let _lock = lock_signer_env_guard();
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
    let _lock = lock_signer_env_guard();
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
