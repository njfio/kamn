use super::super::*;
use super::support::{
    assert_reason, backend_command_guard, managed_payload_and_request_count, managed_request,
    primary_managed_core_env, primary_managed_pubkey_guard,
};
use crate::signer::KolmeLiveSignerSelection;

#[test]
fn regression_kolme_live_managed_external_backend_response_accepts_case_variant_signer_public_key()
{
    let _lock = lock_signer_env_guard();
    let (_payload, selection, _requests) = case_variant_selection();
    assert_eq!(selection.key_source, "managed-external");
}

#[test]
fn regression_kolme_live_managed_external_backend_response_rejects_malformed_signer_public_key() {
    let _lock = lock_signer_env_guard();
    let error = malformed_provenance_error();
    assert_reason(
        error,
        "managed_signer_backend_response_provenance_malformed",
        "managed-external signer malformed provenance marker must preserve deterministic reason code",
    );
}

fn case_variant_selection() -> (String, KolmeLiveSignerSelection, usize) {
    let (_env, _core_env) = primary_managed_core_env();
    let request = managed_request("6584-provenance-case-match");
    let uppercase_pubkey =
        managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE).to_uppercase();
    let _pubkey_guard = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(uppercase_pubkey.as_str()),
    );
    let _backend_command_guard =
        backend_command_guard(&request, 58, Some(uppercase_pubkey.as_str()));
    managed_payload_and_request_count(
        &request,
        58,
        "acct-6584-provenance-case-match",
        None,
        Some("managed-external"),
    )
}

fn malformed_provenance_error() -> ConfigError {
    let (_env, _core_env) = primary_managed_core_env();
    let request = managed_request("6584-provenance-malformed");
    let _pubkey_guard = primary_managed_pubkey_guard();
    let _backend_command_guard = backend_command_guard(&request, 59, Some("not-hex"));
    super::support::managed_build_error(
        &request,
        59,
        "acct-6584-provenance-malformed",
        None,
        Some("managed-external"),
    )
}
