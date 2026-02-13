use kamn_core::{ConfigError, KolmeApiNextNonceRequest, KolmeRuntimeCommitRequest};

use super::KOLME_LIVE_NATIVE_CREATED_AT;

pub(crate) fn escape_kolme_json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub(crate) fn render_kolme_live_native_direct_message(
    request: &KolmeRuntimeCommitRequest,
    pubkey: &str,
    nonce: u64,
) -> Result<String, ConfigError> {
    if nonce == 0 {
        return Err(ConfigError::RuntimeKolmeLive(
            "native direct-signed message nonce must be positive".to_owned(),
        ));
    }
    let pubkey = KolmeApiNextNonceRequest::new(pubkey)
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?
        .pubkey;
    request
        .validate()
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    let metadata_message = format!(
        "{{\"type\":\"kamn-runtime-commit\",\"operation_id\":\"{}\",\"state_root\":\"{}\",\"actor_did\":\"{}\",\"payload_hash\":\"{}\",\"idempotency_key\":\"{}\",\"wire_payload\":\"{}\"}}",
        escape_kolme_json_string(request.operation_id.as_str()),
        escape_kolme_json_string(request.state_root.as_str()),
        escape_kolme_json_string(request.actor_did.as_str()),
        escape_kolme_json_string(request.payload_hash.as_str()),
        escape_kolme_json_string(request.idempotency_key()),
        escape_kolme_json_string(request.to_wire_payload().as_str()),
    );
    Ok(format!(
        "{{\"pubkey\":\"{}\",\"nonce\":{},\"created\":\"{}\",\"messages\":[{}],\"max_height\":null}}",
        escape_kolme_json_string(pubkey.as_str()),
        nonce,
        KOLME_LIVE_NATIVE_CREATED_AT,
        metadata_message
    ))
}
