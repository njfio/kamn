//! Typed codec contracts for Kolme nonce and broadcast APIs.

use crate::flat_json_policy::{
    parse_flat_json_value_fields, required_json_string_field, required_positive_u64_json_field,
    KolmeFlatJsonPolicyError, KolmeFlatJsonValue,
};
use crate::json_scalar_policy::{
    parse_json_string_token as parse_json_string, percent_encode_component as percent_encode,
    skip_ascii_whitespace,
};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Error raised while building or parsing Kolme API codec payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeApiCodecError {
    /// Request payload failed validation.
    InvalidRequest {
        /// Field that failed validation.
        field: &'static str,
        /// Validation reason.
        reason: &'static str,
    },
    /// Response payload failed deterministic parse/validation.
    MalformedResponse {
        /// Parse/validation failure reason.
        reason: String,
    },
}

impl fmt::Display for KolmeApiCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest { field, reason } => {
                write!(f, "invalid request {field}: {reason}")
            }
            Self::MalformedResponse { reason } => write!(f, "malformed response: {reason}"),
        }
    }
}

impl Error for KolmeApiCodecError {}

/// Typed nonce lookup request for Kolme `/get-next-nonce`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiNextNonceRequest {
    /// Public key used to resolve next nonce and account identity.
    pub pubkey: String,
}

impl KolmeApiNextNonceRequest {
    /// Builds a deterministic nonce lookup request.
    pub fn new(pubkey: &str) -> Result<Self, KolmeApiCodecError> {
        let trimmed = pubkey.trim();
        if trimmed.is_empty() {
            return Err(KolmeApiCodecError::InvalidRequest {
                field: "pubkey",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            pubkey: trimmed.to_owned(),
        })
    }

    /// Returns encoded request path for the configured nonce endpoint.
    pub fn query_path(&self, nonce_path: &str) -> String {
        let base_path = if nonce_path.trim().is_empty() {
            "/get-next-nonce".to_owned()
        } else {
            nonce_path.trim().to_owned()
        };
        let separator = if base_path.contains('?') { "&" } else { "?" };
        format!(
            "{base_path}{separator}pubkey={}",
            percent_encode(self.pubkey.as_str())
        )
    }
}

/// Typed nonce lookup response for Kolme `/get-next-nonce`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiNextNonceResponse {
    /// Monotonic next nonce for the provided public key.
    pub next_nonce: u64,
    /// Optional account identifier mapped to the provided public key.
    pub account_id: Option<String>,
}

impl KolmeApiNextNonceResponse {
    /// Parses one nonce lookup response JSON payload.
    pub fn parse_json(response: &str) -> Result<Self, KolmeApiCodecError> {
        let fields = parse_flat_json_value_fields(response).map_err(map_flat_json_error)?;
        let next_nonce =
            required_positive_u64_json_field(&fields, "next_nonce").map_err(map_flat_json_error)?;
        let account_id = optional_nullable_json_string_field(&fields, "account_id")
            .map_err(map_flat_json_error)?;
        Ok(Self {
            next_nonce,
            account_id,
        })
    }
}

/// Typed broadcast request payload for Kolme `/broadcast`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiBroadcastRequest {
    /// Tagged transaction message payload.
    pub message: String,
    /// Chain signature for the transaction message payload.
    pub signature: String,
    /// Signature recovery identifier.
    pub recovery_id: u8,
}

impl KolmeApiBroadcastRequest {
    /// Builds a deterministic broadcast request payload.
    pub fn new(
        message: &str,
        signature: &str,
        recovery_id: u8,
    ) -> Result<Self, KolmeApiCodecError> {
        let message = message.trim();
        if message.is_empty() {
            return Err(KolmeApiCodecError::InvalidRequest {
                field: "message",
                reason: "must not be empty",
            });
        }
        let signature = signature.trim();
        if signature.is_empty() {
            return Err(KolmeApiCodecError::InvalidRequest {
                field: "signature",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            message: message.to_owned(),
            signature: signature.to_owned(),
            recovery_id,
        })
    }

    /// Returns deterministic JSON payload in canonical field order.
    pub fn to_json_payload(&self) -> String {
        format!(
            "{{\"message\":\"{}\",\"signature\":\"{}\",\"recovery_id\":{}}}",
            escape_json_string(self.message.as_str()),
            escape_json_string(self.signature.as_str()),
            self.recovery_id
        )
    }
}

/// Typed broadcast response payload for Kolme `/broadcast`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiBroadcastResponse {
    /// Transaction hash identifier from broadcast response.
    pub txhash: String,
}

impl KolmeApiBroadcastResponse {
    /// Parses one broadcast response JSON payload.
    pub fn parse_json(response: &str) -> Result<Self, KolmeApiCodecError> {
        let fields = parse_flat_json_value_fields(response).map_err(map_flat_json_error)?;
        let txhash = required_json_string_field(&fields, "txhash").map_err(map_flat_json_error)?;
        Ok(Self { txhash })
    }
}

/// Validates required direct-signed transaction message fields.
pub fn validate_direct_signed_transaction_message(message: &str) -> Result<(), KolmeApiCodecError> {
    let required_string_fields = ["pubkey", "created"];
    for field in required_string_fields {
        let value = find_json_string_field(message, field).map_err(|_| {
            KolmeApiCodecError::MalformedResponse {
                reason: format!("direct signed payload message field is invalid: {field}"),
            }
        })?;
        if value.is_none() {
            return Err(KolmeApiCodecError::MalformedResponse {
                reason: format!("direct signed payload message missing required field: {field}"),
            });
        }
    }

    let nonce = find_json_u64_field(message, "nonce").map_err(|_| {
        KolmeApiCodecError::MalformedResponse {
            reason: "direct signed payload message field is invalid: nonce".to_owned(),
        }
    })?;
    if nonce.is_none() {
        return Err(KolmeApiCodecError::MalformedResponse {
            reason: "direct signed payload message missing required field: nonce".to_owned(),
        });
    }

    let has_messages = has_json_array_field(message, "messages").map_err(|_| {
        KolmeApiCodecError::MalformedResponse {
            reason: "direct signed payload message field is invalid: messages".to_owned(),
        }
    })?;
    if !has_messages {
        return Err(KolmeApiCodecError::MalformedResponse {
            reason: "direct signed payload message missing required field: messages".to_owned(),
        });
    }

    Ok(())
}

fn optional_nullable_json_string_field(
    fields: &HashMap<String, KolmeFlatJsonValue>,
    field: &'static str,
) -> Result<Option<String>, KolmeFlatJsonPolicyError> {
    let Some(value) = fields.get(field) else {
        return Ok(None);
    };
    match value {
        KolmeFlatJsonValue::Null => Ok(None),
        KolmeFlatJsonValue::String(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(KolmeFlatJsonPolicyError::MalformedResponse {
                    reason: format!("field must not be empty: {field}"),
                });
            }
            Ok(Some(trimmed.to_owned()))
        }
        _ => Err(KolmeFlatJsonPolicyError::MalformedResponse {
            reason: format!("field must be string|null: {field}"),
        }),
    }
}

fn find_json_string_field(payload: &str, field: &str) -> Result<Option<String>, &'static str> {
    let pattern = format!("\"{field}\"");
    for (index, _) in payload.match_indices(pattern.as_str()) {
        let mut cursor = index + pattern.len();
        cursor = skip_ascii_whitespace(payload, cursor);
        if payload.as_bytes().get(cursor).copied() != Some(b':') {
            continue;
        }
        cursor += 1;
        cursor = skip_ascii_whitespace(payload, cursor);
        if payload.as_bytes().get(cursor).copied() != Some(b'"') {
            continue;
        }
        let mut end = cursor + 1;
        let mut escape = false;
        while let Some(byte) = payload.as_bytes().get(end).copied() {
            if escape {
                escape = false;
                end += 1;
                continue;
            }
            if byte == b'\\' {
                escape = true;
                end += 1;
                continue;
            }
            if byte == b'"' {
                let token = &payload[cursor..=end];
                let parsed = parse_json_string(token)?;
                if parsed.trim().is_empty() {
                    return Err("field must not be empty");
                }
                return Ok(Some(parsed));
            }
            end += 1;
        }
        return Err("field value is unterminated");
    }
    Ok(None)
}

fn find_json_u64_field(payload: &str, field: &str) -> Result<Option<u64>, &'static str> {
    let pattern = format!("\"{field}\"");
    for (index, _) in payload.match_indices(pattern.as_str()) {
        let mut cursor = index + pattern.len();
        cursor = skip_ascii_whitespace(payload, cursor);
        if payload.as_bytes().get(cursor).copied() != Some(b':') {
            continue;
        }
        cursor += 1;
        cursor = skip_ascii_whitespace(payload, cursor);
        let Some(first) = payload.as_bytes().get(cursor).copied() else {
            return Err("field value is missing");
        };
        if first == b'"' {
            let mut end = cursor + 1;
            let mut escape = false;
            while let Some(byte) = payload.as_bytes().get(end).copied() {
                if escape {
                    escape = false;
                    end += 1;
                    continue;
                }
                if byte == b'\\' {
                    escape = true;
                    end += 1;
                    continue;
                }
                if byte == b'"' {
                    let token = &payload[cursor..=end];
                    let parsed = parse_json_string(token)?;
                    return parse_positive_u64(parsed.as_str()).map(Some);
                }
                end += 1;
            }
            return Err("field value is unterminated");
        }
        let mut end = cursor;
        while let Some(byte) = payload.as_bytes().get(end).copied() {
            if byte.is_ascii_digit() {
                end += 1;
                continue;
            }
            break;
        }
        if end == cursor {
            return Err("field must be a positive integer");
        }
        let token = &payload[cursor..end];
        return parse_positive_u64(token).map(Some);
    }
    Ok(None)
}

fn parse_positive_u64(token: &str) -> Result<u64, &'static str> {
    let trimmed = token.trim();
    let parsed = trimmed
        .parse::<u64>()
        .map_err(|_| "field must be a positive integer")?;
    if parsed == 0 {
        return Err("field must be a positive integer");
    }
    Ok(parsed)
}

fn has_json_array_field(payload: &str, field: &str) -> Result<bool, &'static str> {
    let pattern = format!("\"{field}\"");
    for (index, _) in payload.match_indices(pattern.as_str()) {
        let mut cursor = index + pattern.len();
        cursor = skip_ascii_whitespace(payload, cursor);
        if payload.as_bytes().get(cursor).copied() != Some(b':') {
            continue;
        }
        cursor += 1;
        cursor = skip_ascii_whitespace(payload, cursor);
        let Some(first) = payload.as_bytes().get(cursor).copied() else {
            return Err("field must be array");
        };
        if first == b'[' {
            return Ok(true);
        }
        return Err("field must be array");
    }
    Ok(false)
}

fn map_flat_json_error(error: KolmeFlatJsonPolicyError) -> KolmeApiCodecError {
    KolmeApiCodecError::MalformedResponse {
        reason: error.to_string(),
    }
}

/// Escapes one UTF-8 string for deterministic JSON string rendering.
pub fn escape_json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{
        validate_direct_signed_transaction_message, KolmeApiBroadcastRequest,
        KolmeApiBroadcastResponse, KolmeApiCodecError, KolmeApiNextNonceRequest,
        KolmeApiNextNonceResponse,
    };

    #[test]
    fn unit_next_nonce_request_requires_pubkey() {
        assert_eq!(
            KolmeApiNextNonceRequest::new(""),
            Err(KolmeApiCodecError::InvalidRequest {
                field: "pubkey",
                reason: "must not be empty",
            })
        );
    }

    #[test]
    fn unit_broadcast_response_requires_txhash() {
        assert_eq!(
            KolmeApiBroadcastResponse::parse_json("{\"txhash\":\"\"}"),
            Err(KolmeApiCodecError::MalformedResponse {
                reason: "field must not be empty: txhash".to_owned(),
            })
        );
    }

    #[test]
    fn unit_next_nonce_response_supports_nullable_account_id() {
        let response =
            KolmeApiNextNonceResponse::parse_json("{\"next_nonce\":7,\"account_id\":null}")
                .expect("response should parse");
        assert_eq!(response.next_nonce, 7);
        assert_eq!(response.account_id, None);
    }

    #[test]
    fn unit_broadcast_request_serializes_canonical_json_order() {
        let request =
            KolmeApiBroadcastRequest::new("{\"nonce\":42}", "sig-42", 0).expect("request");
        assert_eq!(
            request.to_json_payload(),
            "{\"message\":\"{\\\"nonce\\\":42}\",\"signature\":\"sig-42\",\"recovery_id\":0}"
        );
    }

    #[test]
    fn unit_validate_direct_signed_transaction_message_requires_messages_array() {
        assert_eq!(
            validate_direct_signed_transaction_message(
                "{\"pubkey\":\"p\",\"nonce\":1,\"created\":\"c\",\"messages\":\"x\"}"
            ),
            Err(KolmeApiCodecError::MalformedResponse {
                reason: "direct signed payload message field is invalid: messages".to_owned(),
            })
        );
    }
}
