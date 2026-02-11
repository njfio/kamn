//! Typed codec contracts for Kolme nonce and broadcast APIs.

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
        let fields = parse_flat_json_value_fields(response)?;
        let next_nonce = required_positive_u64_json_field(&fields, "next_nonce")?;
        let account_id = optional_nullable_json_string_field(&fields, "account_id")?;
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
            json_escape(self.message.as_str()),
            json_escape(self.signature.as_str()),
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
        let fields = parse_flat_json_value_fields(response)?;
        let txhash = required_json_string_field(&fields, "txhash")?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum FlatJsonValue {
    String(String),
    Number(String),
    Boolean,
    Null,
}

fn parse_flat_json_value_fields(
    response: &str,
) -> Result<HashMap<String, FlatJsonValue>, KolmeApiCodecError> {
    let body = response.trim();
    if !(body.starts_with('{') && body.ends_with('}')) {
        return Err(KolmeApiCodecError::MalformedResponse {
            reason: "json response must be an object".to_owned(),
        });
    }
    let inner = &body[1..body.len() - 1];
    if inner.trim().is_empty() {
        return Ok(HashMap::new());
    }

    let entries =
        split_unquoted(inner, ',').map_err(|reason| KolmeApiCodecError::MalformedResponse {
            reason: format!("invalid json response: {reason}"),
        })?;

    let mut fields = HashMap::new();
    for entry in entries {
        let pair = split_unquoted(entry.as_str(), ':').map_err(|reason| {
            KolmeApiCodecError::MalformedResponse {
                reason: format!("invalid json response pair: {reason}"),
            }
        })?;
        if pair.len() != 2 {
            return Err(KolmeApiCodecError::MalformedResponse {
                reason: "json response pair must contain exactly one ':'".to_owned(),
            });
        }

        let key = parse_json_string(pair[0].trim()).map_err(|reason| {
            KolmeApiCodecError::MalformedResponse {
                reason: format!("invalid json key: {reason}"),
            }
        })?;
        let value = parse_json_value(pair[1].trim())?;
        fields.insert(key, value);
    }
    Ok(fields)
}

fn parse_json_value(token: &str) -> Result<FlatJsonValue, KolmeApiCodecError> {
    let trimmed = token.trim();
    if trimmed.starts_with('"') {
        let value =
            parse_json_string(trimmed).map_err(|reason| KolmeApiCodecError::MalformedResponse {
                reason: format!("invalid json value: {reason}"),
            })?;
        return Ok(FlatJsonValue::String(value));
    }
    if trimmed == "null" {
        return Ok(FlatJsonValue::Null);
    }
    if trimmed == "true" {
        return Ok(FlatJsonValue::Boolean);
    }
    if trimmed == "false" {
        return Ok(FlatJsonValue::Boolean);
    }
    if is_json_number_literal(trimmed) {
        return Ok(FlatJsonValue::Number(trimmed.to_owned()));
    }
    Err(KolmeApiCodecError::MalformedResponse {
        reason: "invalid json value token".to_owned(),
    })
}

fn is_json_number_literal(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let mut chars = token.chars();
    let first = chars.next().unwrap_or_default();
    if first == '-' {
        let remainder = chars.as_str();
        return !remainder.is_empty() && remainder.chars().all(|ch| ch.is_ascii_digit());
    }
    first.is_ascii_digit() && chars.as_str().chars().all(|ch| ch.is_ascii_digit())
}

fn required_json_string_field(
    fields: &HashMap<String, FlatJsonValue>,
    field: &'static str,
) -> Result<String, KolmeApiCodecError> {
    let value = fields
        .get(field)
        .ok_or_else(|| KolmeApiCodecError::MalformedResponse {
            reason: format!("missing required field: {field}"),
        })?;
    let FlatJsonValue::String(raw) = value else {
        return Err(KolmeApiCodecError::MalformedResponse {
            reason: format!("field must be a string: {field}"),
        });
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(KolmeApiCodecError::MalformedResponse {
            reason: format!("field must not be empty: {field}"),
        });
    }
    Ok(trimmed.to_owned())
}

fn optional_nullable_json_string_field(
    fields: &HashMap<String, FlatJsonValue>,
    field: &'static str,
) -> Result<Option<String>, KolmeApiCodecError> {
    let Some(value) = fields.get(field) else {
        return Ok(None);
    };
    match value {
        FlatJsonValue::Null => Ok(None),
        FlatJsonValue::String(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(KolmeApiCodecError::MalformedResponse {
                    reason: format!("field must not be empty: {field}"),
                });
            }
            Ok(Some(trimmed.to_owned()))
        }
        _ => Err(KolmeApiCodecError::MalformedResponse {
            reason: format!("field must be string|null: {field}"),
        }),
    }
}

fn required_positive_u64_json_field(
    fields: &HashMap<String, FlatJsonValue>,
    field: &'static str,
) -> Result<u64, KolmeApiCodecError> {
    let value = fields
        .get(field)
        .ok_or_else(|| KolmeApiCodecError::MalformedResponse {
            reason: format!("missing required field: {field}"),
        })?;

    let raw = match value {
        FlatJsonValue::Number(value) => value.as_str(),
        FlatJsonValue::String(value) => value.trim(),
        _ => {
            return Err(KolmeApiCodecError::MalformedResponse {
                reason: format!("field must be numeric: {field}"),
            })
        }
    };

    let parsed = raw
        .parse::<i128>()
        .map_err(|_| KolmeApiCodecError::MalformedResponse {
            reason: format!("invalid numeric field: {field}"),
        })?;
    if parsed <= 0 {
        return Err(KolmeApiCodecError::MalformedResponse {
            reason: format!("{field} must be positive"),
        });
    }
    u64::try_from(parsed).map_err(|_| KolmeApiCodecError::MalformedResponse {
        reason: format!("invalid numeric field: {field}"),
    })
}

fn split_unquoted(input: &str, delimiter: char) -> Result<Vec<String>, &'static str> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' && in_quotes {
            current.push(ch);
            escape = true;
            continue;
        }

        if ch == '"' {
            in_quotes = !in_quotes;
            current.push(ch);
            continue;
        }

        if ch == delimiter && !in_quotes {
            if current.trim().is_empty() {
                return Err("empty segment");
            }
            parts.push(current.trim().to_owned());
            current.clear();
            continue;
        }

        current.push(ch);
    }

    if in_quotes {
        return Err("unterminated quoted string");
    }
    if current.trim().is_empty() {
        return Err("empty trailing segment");
    }
    parts.push(current.trim().to_owned());
    Ok(parts)
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

fn skip_ascii_whitespace(value: &str, mut cursor: usize) -> usize {
    while let Some(byte) = value.as_bytes().get(cursor).copied() {
        if byte.is_ascii_whitespace() {
            cursor += 1;
            continue;
        }
        break;
    }
    cursor
}

fn parse_json_string(token: &str) -> Result<String, &'static str> {
    let trimmed = token.trim();
    if !(trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2) {
        return Err("token must be a quoted string");
    }
    let mut output = String::new();
    let mut escape = false;
    for ch in trimmed[1..trimmed.len() - 1].chars() {
        if escape {
            let mapped = match ch {
                '\\' => '\\',
                '"' => '"',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => return Err("unsupported escape sequence"),
            };
            output.push(mapped);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        output.push(ch);
    }
    if escape {
        return Err("unterminated escape sequence");
    }
    Ok(output)
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        let ch = byte as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~') {
            encoded.push(ch);
        } else {
            encoded.push('%');
            encoded.push_str(format!("{byte:02X}").as_str());
        }
    }
    encoded
}

fn json_escape(value: &str) -> String {
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
