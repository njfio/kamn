use super::support::{
    is_zeroized_hex_buffer, lock_signer_env_guard, test_primary_selection, EnvVarGuard,
    TEST_PRIVATE_KEY_HEX, TEST_PRIVATE_KEY_HEX_SECONDARY,
};
use super::ConfigError;

#[test]
fn regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer() {
    let _lock = lock_signer_env_guard();
    let _secondary_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
        Some(TEST_PRIVATE_KEY_HEX_SECONDARY),
    );

    let mut private_key_hex = TEST_PRIVATE_KEY_HEX.to_owned();
    let error = super::super::ensure_kolme_live_strict_signer_secret_source_precedence_and_zeroize(
        Some("ops-primary"),
        &test_primary_selection(),
        &mut private_key_hex,
    )
    .expect_err("strict signer precedence violation must fail closed");

    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("signer_secret_source_precedence_violation"))
    );
    assert!(is_zeroized_hex_buffer(private_key_hex.as_str()));
}

#[test]
fn regression_strict_signer_secret_source_precedence_rejects_dual_private_key_envs() {
    let _lock = lock_signer_env_guard();
    let _profile_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let _primary_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_PRIVATE_KEY_HEX),
    );
    let _secondary_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
        Some(TEST_PRIVATE_KEY_HEX_SECONDARY),
    );
    let _fallback_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);

    let error = super::super::read_kolme_live_signer_private_key_hex(
        Some("ops-primary"),
        Some("env-local"),
    )
    .expect_err("strict signer contracts must reject dual private key env sources");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("signer_secret_source_precedence_violation"))
    );
}

#[test]
fn regression_strict_secondary_profile_requires_secondary_secret_even_with_primary_present() {
    let _lock = lock_signer_env_guard();
    let _profile_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let _primary_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_PRIVATE_KEY_HEX),
    );
    let _secondary_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY", None);
    let _fallback_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);

    let error = super::super::read_kolme_live_signer_private_key_hex(
        Some("ops-secondary"),
        Some("env-local"),
    )
    .expect_err("strict secondary signer contracts must require secondary private key env");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY must be set"))
    );
}
