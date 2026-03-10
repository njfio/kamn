use super::super::*;
use super::support::{
    backend_command_guard, managed_payload_and_request_count, managed_request,
    primary_managed_core_env, primary_managed_pubkey_guard,
};
use crate::signer::KolmeLiveSignerSelection;

#[test]
fn integration_kolme_live_managed_external_adapter_provenance_consumed_by_signer_selection() {
    let _lock = lock_signer_env_guard();
    let (payload, selection, request_count) = managed_selection_result();
    assert_selection_payload(payload.as_str(), &selection, request_count);
}

fn managed_selection_result() -> (String, KolmeLiveSignerSelection, usize) {
    let (_env, _core_env) = primary_managed_core_env();
    let request = managed_request("2323");
    let expected_pubkey = managed_signer_public_key_hex(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE);
    let _pubkey_guard = primary_managed_pubkey_guard();
    let _backend_command_guard =
        backend_command_guard(&request, 41, Some(expected_pubkey.as_str()));
    managed_payload_and_request_count(
        &request,
        41,
        "acct-2323",
        Some("ops-primary"),
        Some("managed-external"),
    )
}

fn assert_selection_payload(
    payload: &str,
    selection: &KolmeLiveSignerSelection,
    request_count: usize,
) {
    assert_eq!(selection.profile, "ops-primary");
    assert_eq!(selection.key_source, "managed-external");
    assert_eq!(
        selection.private_key_env,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
    );
    let signature = extract_json_string_field(payload, "signature")
        .expect("direct signed payload must include signature field");
    assert_eq!(signature.len(), 128);
    assert_eq!(
        request_count, 1,
        "managed-external signing should issue one nonce lookup before payload emission"
    );
}
