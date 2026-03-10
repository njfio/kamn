use super::super::*;
use super::support::{
    assert_reason, backend_command_guard, managed_build_error, managed_request,
    primary_managed_core_env, primary_managed_pubkey_guard,
};

#[test]
fn regression_kolme_live_managed_external_backend_response_requires_signer_public_key_marker() {
    let _lock = lock_signer_env_guard();
    let error = missing_provenance_error();
    assert_reason(
        error,
        "managed_signer_backend_response_provenance_missing",
        "missing managed-external signer provenance marker must fail closed",
    );
}

fn missing_provenance_error() -> ConfigError {
    let (_env, _core_env) = primary_managed_core_env();
    let request = managed_request("2509-provenance-required");
    let _pubkey_guard = primary_managed_pubkey_guard();
    let _backend_command_guard = backend_command_guard(&request, 45, None);
    managed_build_error(
        &request,
        45,
        "acct-2509-provenance-required",
        None,
        Some("managed-external"),
    )
}
