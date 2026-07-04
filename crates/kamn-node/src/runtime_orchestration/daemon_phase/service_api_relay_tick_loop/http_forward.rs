mod io;

use io::send_relay_request;
use kamn_core::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
};
use std::collections::BTreeMap;

const SERVICE_API_RELAY_FORWARD_PATH: &str = "/v1/messages/relay";
const SERVICE_API_RELAY_FORWARD_SCOPE: &str = "messages:write";
const SERVICE_API_RELAY_FORWARD_DEFAULT_SENDER_DID: &str = "kamn:did:agent:relay-daemon";

struct RelayRequestSignatureInput {
    sender_did: String,
    relay_nonce: u64,
    signature: String,
    signer_public_key_hex: String,
    relay_payload_body: String,
}

pub(super) fn forward_service_api_relay_entry(
    relay_route_map: &BTreeMap<String, String>,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
    service_api_signature_state_hash: &str,
    signing_private_key_hex: &str,
    relay_nonce_counter: &mut u64,
) -> Result<(), String> {
    let relay_addr = relay_recipient_address(relay_route_map, relay_entry)?;
    let request = build_signed_relay_request(
        relay_addr,
        relay_entry,
        service_api_signature_state_hash,
        signing_private_key_hex,
        relay_nonce_counter,
    )?;
    send_relay_request(relay_addr, &request)
}

fn build_signed_relay_request(
    relay_addr: &str,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
    service_api_signature_state_hash: &str,
    signing_private_key_hex: &str,
    relay_nonce_counter: &mut u64,
) -> Result<String, String> {
    let signed_input = build_relay_request_signature_input(
        relay_entry,
        service_api_signature_state_hash,
        signing_private_key_hex,
        relay_nonce_counter,
    )?;
    Ok(build_relay_request(
        relay_addr,
        signed_input.sender_did.as_str(),
        signed_input.signer_public_key_hex.as_str(),
        signed_input.relay_nonce,
        signed_input.signature.as_str(),
        signed_input.relay_payload_body.as_str(),
    ))
}

fn build_relay_request_signature_input(
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
    service_api_signature_state_hash: &str,
    signing_private_key_hex: &str,
    relay_nonce_counter: &mut u64,
) -> Result<RelayRequestSignatureInput, String> {
    let relay_payload_body = serialize_relay_payload(relay_entry)?;
    let sender_did = resolve_sender_did(relay_entry).to_owned();
    let relay_nonce = next_relay_nonce(relay_nonce_counter);
    let signature = relay_request_signature(
        sender_did.as_str(),
        relay_nonce,
        service_api_signature_state_hash,
        relay_payload_body.as_str(),
        signing_private_key_hex,
    )?;
    Ok(RelayRequestSignatureInput {
        sender_did,
        relay_nonce,
        signature,
        signer_public_key_hex: signer_public_key_hex(signing_private_key_hex)?,
        relay_payload_body,
    })
}

fn relay_recipient_address<'a>(
    relay_route_map: &'a BTreeMap<String, String>,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> Result<&'a str, String> {
    relay_route_map
        .get(relay_entry.recipient_did.as_str())
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "relay recipient route missing for recipient_did={}",
                relay_entry.recipient_did
            )
        })
}

fn serialize_relay_payload(
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> Result<String, String> {
    serde_json::to_string(&serde_json::json!({
        "message_id": relay_entry.message_id.as_str(),
        "sender_did": relay_entry.sender_did.as_deref(),
        "recipient_did": relay_entry.recipient_did.as_str(),
        "body": relay_entry.body.as_str(),
        "queued_at_unix": relay_entry.queued_at_unix,
    }))
    .map_err(|error| format!("relay payload serialization failed: {error}"))
}

fn resolve_sender_did(
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> &str {
    relay_entry
        .sender_did
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(SERVICE_API_RELAY_FORWARD_DEFAULT_SENDER_DID)
}

fn next_relay_nonce(relay_nonce_counter: &mut u64) -> u64 {
    *relay_nonce_counter = relay_nonce_counter.saturating_add(1);
    (*relay_nonce_counter).max(1)
}

fn relay_request_signature(
    sender_did: &str,
    relay_nonce: u64,
    service_api_signature_state_hash: &str,
    relay_payload_body: &str,
    signing_private_key_hex: &str,
) -> Result<String, String> {
    service_auth_sign_with_private_key_hex(
        sender_did,
        relay_nonce,
        service_api_signature_state_hash,
        relay_payload_body,
        signing_private_key_hex,
    )
    .map_err(|error| format!("relay request signature generation failed: {error}"))
}

fn signer_public_key_hex(signing_private_key_hex: &str) -> Result<String, String> {
    service_auth_public_key_hex_from_private_key_hex(signing_private_key_hex)
        .map_err(|error| format!("relay signer public key derivation failed: {error}"))
}

fn build_relay_request(
    relay_addr: &str,
    sender_did: &str,
    signer_public_key_hex: &str,
    relay_nonce: u64,
    signature: &str,
    relay_payload_body: &str,
) -> String {
    format!(
        "POST {SERVICE_API_RELAY_FORWARD_PATH} HTTP/1.1\r\nHost: {relay_addr}\r\nConnection: close\r\nContent-Type: application/json\r\nX-KAMN-Sender-DID: {sender_did}\r\nX-KAMN-Signer-Public-Key: {signer_public_key_hex}\r\nX-KAMN-Request-Nonce: {relay_nonce}\r\nX-KAMN-Request-Signature: {signature}\r\nX-KAMN-Authz-Scope: {SERVICE_API_RELAY_FORWARD_SCOPE}\r\nContent-Length: {}\r\n\r\n{}",
        relay_payload_body.len(),
        relay_payload_body
    )
}
