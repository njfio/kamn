use super::super::support::managed_external_core_signer_env_guards;
use super::super::*;
use super::support::{
    assert_provider_reason, assert_reason, managed_pubkey_guard, managed_request,
};

#[test]
fn regression_kolme_live_managed_external_maps_provider_unavailable_reason_code() {
    let request = managed_request("2323-provider");
    let expected_pubkey = managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE);
    assert_provider_reason(
        &request,
        false,
        expected_pubkey.as_str(),
        "managed_signer_provider_unavailable",
        "managed-external provider unavailability must fail closed",
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_timeout_maps_reason_code() {
    let _lock = lock_signer_env_guard();
    let _core_env = managed_external_core_signer_env_guards();
    let _backend_command_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND", Some("sleep 2"));
    let _backend_timeout_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("1"));
    let request = managed_request("2423-timeout");
    let error = timeout_or_unavailable_error(&request, 1, "acct-2423-timeout", "ops-primary");
    assert_reason(
        error,
        "managed_signer_backend_timeout",
        "managed-external backend timeout must map to deterministic reason code",
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_malformed_response_maps_reason_code() {
    let _lock = lock_signer_env_guard();
    let _core_env = managed_external_core_signer_env_guards();
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some("printf 'signature_hex=zzzz\\nrecovery_id=9\\nsigner_public_key_hex=03af446f76cf36092a4e45864210a1dbf03e872756eec21de61910859f8a607dd2\\n'"),
    );
    let _backend_timeout_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("5"));
    let request = managed_request("2423-malformed");
    let error = timeout_or_unavailable_error(&request, 1, "acct-2423-malformed", "ops-primary");
    assert_reason(
        error,
        "managed_signer_backend_response_malformed",
        "managed-external backend malformed response must map to deterministic reason code",
    );
}

#[test]
fn regression_kolme_live_managed_external_backend_unavailable_maps_reason_code() {
    let _lock = lock_signer_env_guard();
    let _core_env = managed_external_core_signer_env_guards();
    let _backend_command_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some("this-command-should-not-exist-2423"),
    );
    let _backend_timeout_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS", Some("5"));
    let request = managed_request("2423-unavailable");
    let error = timeout_or_unavailable_error(&request, 1, "acct-2423-unavailable", "ops-primary");
    assert_reason(
        error,
        "managed_signer_backend_unavailable",
        "managed-external backend unavailability must map to deterministic reason code",
    );
}

#[test]
fn regression_kolme_live_managed_external_adapter_retired_not_integrated_marker() {
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

fn timeout_or_unavailable_error(
    request: &KolmeRuntimeCommitRequest,
    nonce: u64,
    account_id: &str,
    profile: &str,
) -> ConfigError {
    let _profile_env_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some(profile));
    let _key_ref_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let _primary_key_guard = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let _pubkey_guard = managed_pubkey_guard(
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
    );
    super::support::managed_build_error(request, nonce, account_id, None, Some("managed-external"))
}
