use super::super::*;

#[derive(Debug, Deserialize)]
pub(crate) struct ServiceApiErrorEnvelope {
    pub(crate) error: String,
    pub(crate) reason_code: String,
    pub(crate) message: String,
}

pub(crate) const SERVICE_API_AUTH_MISSING_HEADER_REASON_CODE: &str =
    "service_api_auth_sender_did_header_missing";
pub(crate) const SERVICE_API_AUTH_SCOPE_HEADER_MISSING_REASON_CODE: &str =
    "service_api_auth_scope_header_missing";
pub(crate) const SERVICE_API_AUTH_SCOPE_INVALID_REASON_CODE: &str =
    "service_api_auth_scope_invalid";
pub(crate) const SERVICE_API_AGENT_DID_PATH_INVALID_REASON_CODE: &str =
    "service_api_agent_did_path_invalid";
pub(crate) const SERVICE_API_MESSAGE_RECIPIENT_DID_INVALID_REASON_CODE: &str =
    "service_api_message_recipient_did_invalid";
pub(crate) const SERVICE_API_RELAY_DID_INVALID_REASON_CODE: &str = "service_api_relay_did_invalid";
pub(crate) const SERVICE_API_AUTH_SCOPE_ROUTE_MISMATCH_REASON_CODE: &str =
    "service_api_auth_scope_route_mismatch";
pub(crate) const TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

pub(crate) fn signed_header_present(headers: &[(&str, &str)], name: &str) -> bool {
    headers
        .iter()
        .any(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
}

pub(crate) fn test_service_api_auth_public_key_hex() -> String {
    service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX)
        .expect("service-auth public key should derive")
}

pub(crate) fn test_service_api_sender_did(sender: &str) -> String {
    let public_key_hex = test_service_api_auth_public_key_hex();
    let Ok(parsed_sender_did) = AgentDid::parse(sender) else {
        return sender.to_owned();
    };
    if parsed_sender_did.method_specific_id().starts_with("pkh-") {
        return sender.to_owned();
    }
    if parsed_sender_did
        .ensure_public_key_hex_binding(public_key_hex.as_str())
        .is_ok()
    {
        return sender.to_owned();
    }
    AgentDid::with_public_key_hex_binding(
        parsed_sender_did.method_specific_id(),
        public_key_hex.as_str(),
    )
    .expect("test sender did should bind to fixture signer key")
    .as_str()
    .to_owned()
}

pub(crate) fn service_api_request_signature_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    service_auth_sign_with_private_key_hex(
        test_service_api_sender_did(sender).as_str(),
        nonce,
        state_hash,
        payload,
        TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX,
    )
    .expect("service-auth signature should render for test fixture key")
}
