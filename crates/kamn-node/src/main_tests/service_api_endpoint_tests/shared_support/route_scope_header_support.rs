use super::super::auth_fixture_support::{
    signed_header_present, test_service_api_auth_public_key_hex, test_service_api_sender_did,
};
use super::required_scope_for_test_route;

pub(super) fn clone_headers(headers: &[(&str, &str)]) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect()
}

pub(super) fn request_is_signed(headers: &[(&str, &str)]) -> bool {
    [
        "X-KAMN-Sender-DID",
        "X-KAMN-Request-Nonce",
        "X-KAMN-Request-Signature",
    ]
    .iter()
    .all(|name| signed_header_present(headers, name))
}

pub(super) fn add_missing_scope(
    method: &str,
    path: &str,
    enriched: &mut Vec<(String, String)>,
    headers: &[(&str, &str)],
) {
    if signed_header_present(headers, "X-KAMN-Authz-Scope") {
        return;
    }
    if let Some(scope) = required_scope_for_test_route(method, path) {
        enriched.push(("X-KAMN-Authz-Scope".to_owned(), scope.to_owned()));
    }
}

pub(super) fn normalize_sender_did(enriched: &mut [(String, String)]) {
    for (name, value) in enriched {
        if name.eq_ignore_ascii_case("X-KAMN-Sender-DID") {
            *value = test_service_api_sender_did(value.as_str());
        }
    }
}

pub(super) fn add_missing_signer_public_key(
    enriched: &mut Vec<(String, String)>,
    headers: &[(&str, &str)],
) {
    if signed_header_present(headers, "X-KAMN-Signer-Public-Key") {
        return;
    }
    enriched.push((
        "X-KAMN-Signer-Public-Key".to_owned(),
        test_service_api_auth_public_key_hex(),
    ));
}
