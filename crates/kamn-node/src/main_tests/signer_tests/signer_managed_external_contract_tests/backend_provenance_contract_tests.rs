use super::super::*;
use super::support::{
    assert_reason, backend_command_guard, managed_request, primary_managed_core_env,
    primary_managed_pubkey_guard,
};

#[test]
fn regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch() {
    let _lock = lock_signer_env_guard();
    let error = mismatch_backend_error();
    assert_reason(
        error,
        "managed_signer_backend_response_provenance_mismatch",
        "managed-external signer provenance mismatch must fail closed",
    );
}

fn mismatch_backend_error() -> ConfigError {
    let (_env, _core_env) = primary_managed_core_env();
    let request = managed_request("2509-provenance-mismatch");
    let _pubkey_guard = primary_managed_pubkey_guard();
    let mismatch_pubkey =
        managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE_SECONDARY);
    let _backend_command_guard =
        backend_command_guard(&request, 46, Some(mismatch_pubkey.as_str()));
    super::support::managed_build_error(
        &request,
        46,
        "acct-2509-provenance-mismatch",
        None,
        Some("managed-external"),
    )
}
