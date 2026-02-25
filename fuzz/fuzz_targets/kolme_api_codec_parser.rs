#![no_main]

use kamn_core::{
    KolmeApiBroadcastRequest, KolmeApiBroadcastResponse, KolmeApiNextNonceRequest,
    KolmeApiNextNonceResponse, KolmeRuntimeCommitRequest,
};
use libfuzzer_sys::fuzz_target;

const DEFAULT_NONCE_RESPONSE: &str = "{\"next_nonce\":1,\"account_id\":null}";
const DEFAULT_DID: &str = "kamn:did:agent:fuzz-actor-1";

fn bounded_utf8(data: &[u8], max_len: usize) -> String {
    let mut value = String::from_utf8_lossy(data).to_string();
    if value.len() > max_len {
        value.truncate(max_len);
    }
    value
}

fn first_non_empty_line(input: &str, fallback: &str, max_len: usize) -> String {
    let value = input
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(fallback);
    value.chars().take(max_len).collect()
}

fuzz_target!(|data: &[u8]| {
    let input = bounded_utf8(data, 4096);
    let response_payload = if input.trim().is_empty() {
        DEFAULT_NONCE_RESPONSE
    } else {
        input.as_str()
    };

    let pubkey = first_non_empty_line(input.as_str(), "02fuzzpubkey", 128);
    let signature = first_non_empty_line(input.as_str(), "sig-fuzz", 256);
    let recovery_id = data.first().copied().unwrap_or(0);

    let _ = KolmeApiNextNonceRequest::new(pubkey.as_str())
        .map(|request| request.query_path("/get-next-nonce"));
    let _ = KolmeApiBroadcastRequest::new(response_payload, signature.as_str(), recovery_id)
        .map(|request| request.to_json_payload());
    let _ = KolmeApiNextNonceResponse::parse_json(response_payload);
    let _ = KolmeApiBroadcastResponse::parse_json(response_payload);

    if let Ok(request) = KolmeRuntimeCommitRequest::deterministic(
        "op-fuzz",
        "state:fuzz",
        DEFAULT_DID,
        1,
        "payload:fuzz",
    ) {
        let canonical = request.to_wire_payload();
        let message_candidate = if input.trim().is_empty() {
            canonical.as_str()
        } else {
            input.as_str()
        };

        let _ = request
            .translate_to_signed_broadcast_envelope(
                "kamn:key:signer:fuzz",
                message_candidate,
                "signature-fuzz",
                recovery_id,
            )
            .and_then(|envelope| envelope.to_broadcast_request());
    }
});
