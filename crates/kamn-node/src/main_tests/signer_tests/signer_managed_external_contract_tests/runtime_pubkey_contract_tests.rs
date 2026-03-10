use super::super::*;
use super::support::{assert_reason, backend_command_guard, managed_env, managed_request};

#[test]
fn regression_kolme_live_managed_external_requires_runtime_signer_public_key_marker() {
    let _lock = lock_signer_env_guard();
    let error = runtime_pubkey_error(
        "2512-pubkey-marker-missing",
        47,
        "acct-2512-pubkey-marker-missing",
        None,
    );
    assert_reason(
        error,
        "managed_signer_public_key_marker_missing",
        "missing managed-external signer public key marker must fail closed",
    );
}

#[test]
fn regression_kolme_live_managed_external_rejects_invalid_runtime_signer_public_key_marker() {
    let _lock = lock_signer_env_guard();
    let error = runtime_pubkey_error(
        "2512-pubkey-marker-invalid",
        48,
        "acct-2512-pubkey-marker-invalid",
        Some("invalid-pubkey-marker"),
    );
    assert_reason(
        error,
        "managed_signer_public_key_marker_invalid",
        "invalid managed-external signer public key marker must fail closed with deterministic reason code",
    );
}

fn runtime_pubkey_error(
    suffix: &str,
    nonce: u64,
    account_id: &str,
    runtime_pubkey: Option<&str>,
) -> ConfigError {
    let _env = managed_env(
        "ops-primary",
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE,
    );
    let _runtime_pubkey_guard =
        EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX", runtime_pubkey);
    let request = managed_request(suffix);
    let expected_pubkey = managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE);
    let _backend_command_guard =
        backend_command_guard(&request, nonce, Some(expected_pubkey.as_str()));
    super::support::managed_build_error(&request, nonce, account_id, None, Some("managed-external"))
}
