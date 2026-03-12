use super::super::super::support::service_api_signature_state_hash;
use super::super::super::*;
use super::fixtures::TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX;
use kamn_core::service_auth_sign_with_private_key_hex;

pub(crate) fn legacy_sender_request(snapshot: &ServiceApiSnapshot) -> ParsedRequest {
    let sender_did = "kamn:did:agent:alice";
    let nonce = 41_u64;
    let body = "{}";
    let signature = service_auth_sign_with_private_key_hex(
        sender_did,
        nonce,
        service_api_signature_state_hash(snapshot).as_str(),
        body,
        TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX,
    )
    .expect("service-auth signature should render for test fixture key");
    ParsedRequest {
        method: "POST".to_owned(),
        path: ROUTE_MESSAGES_SEND.to_owned(),
        body: body.to_owned(),
        headers: BTreeMap::from([
            (
                REQUEST_AUTH_SENDER_DID_HEADER.to_owned(),
                sender_did.to_owned(),
            ),
            (REQUEST_AUTH_NONCE_HEADER.to_owned(), nonce.to_string()),
            (REQUEST_AUTH_SIGNATURE_HEADER.to_owned(), signature),
        ]),
    }
}
